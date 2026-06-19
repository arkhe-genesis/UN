use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrchestratorEvent {
    ComplianceChecked {
        action_id: String,
        verdict: crate::substrato_4004::compliance_engine::ComplianceVerdict,
        timestamp: i64,
    },
    B20Memo {
        tx_hash: String,
        log_index: u64,
        caller: String,
        memo: String,
        timestamp: i64,
    },
    ActionProposed {
        action: crate::substrato_4004::shared::Action,
    },
    B20BatchSettled {
        batch_id: String,
        receipt: crate::substrato_4004::settlement_engine::SettlementReceipt,
        timestamp: i64,
    },
    B20ToXrplBridge {
        b20_tx_hash: String,
        xrpl_escrow_id: String,
        amount: String,
        token: String,
        timestamp: i64,
    },
    XrplToB20Release {
        xrpl_escrow_id: String,
        b20_tx_hash: String,
        recipient: String,
        timestamp: i64,
    },
}

#[derive(Debug, Clone)]
pub struct StoreEvent {
    pub payload: OrchestratorEvent,
}

pub struct EventStore;

impl EventStore {
    pub async fn emit(&self, _event: OrchestratorEvent) -> Result<(), String> {
        Ok(())
    }

    pub async fn store(&self, _event: OrchestratorEvent) -> Result<(), String> {
        Ok(())
    }

    pub async fn query_by_memo(&self, _memo: &str) -> Result<Vec<StoreEvent>, String> {
        Ok(vec![])
    }
}
