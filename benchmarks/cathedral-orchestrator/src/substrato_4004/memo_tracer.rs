use std::sync::Arc;
use ethers::types::Address;
use sha2::{Digest, Sha256};

use crate::substrato_4004::shared::Action;
use crate::substrato_4004::event_store::{EventStore, OrchestratorEvent};
use crate::substrato_4004::settlement_engine::CrossChainEmitterV2;
use crate::substrato_4004::metrics::B20_MEMO_INDEXED_TOTAL;

#[derive(Debug, Clone)]
pub struct TracerError(pub String);

impl std::fmt::Display for TracerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TracerError({})", self.0)
    }
}
impl std::error::Error for TracerError {}

impl From<String> for TracerError {
    fn from(err: String) -> Self {
        TracerError(err)
    }
}

pub struct MemoTracer {
    pub event_store: Arc<EventStore>,
    pub cross_chain_emitter: Arc<CrossChainEmitterV2>,
}

impl MemoTracer {
    pub fn generate_memo(&self, action: &Action) -> [u8; 32] {
        let action_hash = Sha256::digest(action.canonical_bytes());
        let mut memo = [0u8; 32];
        memo.copy_from_slice(&action_hash[..32]);
        memo
    }

    pub async fn index_memo_event(
        &self,
        tx_hash: &str,
        log_index: u64,
        caller: Address,
        memo: [u8; 32],
    ) -> Result<(), TracerError> {
        let event = OrchestratorEvent::B20Memo {
            tx_hash: tx_hash.to_string(),
            log_index,
            caller: format!("{:?}", caller),
            memo: hex::encode(memo),
            timestamp: chrono::Utc::now().timestamp(),
        };

        self.event_store.store(event.clone()).await.map_err(|e| TracerError(e))?;
        B20_MEMO_INDEXED_TOTAL.inc();
        self.cross_chain_emitter.emit_cross_chain(event).await.map_err(|e| TracerError(e))?;

        Ok(())
    }

    pub async fn resolve_memo(&self, memo: [u8; 32]) -> Result<Option<Action>, TracerError> {
        let events = self.event_store
            .query_by_memo(&hex::encode(memo))
            .await
            .map_err(|e| TracerError(e))?;

        if let Some(event) = events.first() {
            if let OrchestratorEvent::ActionProposed { action, .. } = &event.payload {
                return Ok(Some(action.clone()));
            }
        }

        Ok(None)
    }
}
