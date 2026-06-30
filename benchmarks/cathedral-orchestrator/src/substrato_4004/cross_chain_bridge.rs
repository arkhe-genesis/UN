use std::sync::Arc;
use ethers::types::{Address, U256};

use crate::substrato_4004::settlement_engine::{B20SettlementEngine, B20Payment, CrossChainEmitterV2};
use crate::substrato_4004::event_store::{OrchestratorEvent};
use crate::substrato_4004::memo_tracer::MemoTracer;
use crate::substrato_4004::b20_mapper::{B20Operation, PolicyScope};
use crate::substrato_4004::shared::{Action, hash_memo};
use crate::substrato_4004::metrics::B20_XRPL_BRIDGE_TRANSFERS_TOTAL;

#[derive(Debug, Clone)]
pub enum BridgeError {
    ComplianceFailed(crate::substrato_4004::compliance_engine::ComplianceVerdict),
    EscrowNotReleased(String),
    Settlement(crate::substrato_4004::settlement_engine::SettlementError),
    CrossChain(String),
}

impl std::fmt::Display for BridgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl std::error::Error for BridgeError {}

impl From<crate::substrato_4004::settlement_engine::SettlementError> for BridgeError {
    fn from(err: crate::substrato_4004::settlement_engine::SettlementError) -> Self {
        BridgeError::Settlement(err)
    }
}

pub struct EscrowState {
    pub token: Address,
    pub amount: U256,
    pub released: bool,
}

pub struct EscrowManager;

impl EscrowManager {
    pub async fn get_state(&self, _escrow_id: &str) -> Result<EscrowState, BridgeError> {
        Ok(EscrowState {
            token: Address::zero(),
            amount: U256::from(0),
            released: true,
        })
    }
}

pub struct X402XrplBridge {
    pub escrow_manager: EscrowManager,
}

impl X402XrplBridge {
    pub async fn create_settlement_escrow(&self, _payment: &str) -> Result<String, BridgeError> {
        Ok("xrpl_escrow_123".to_string())
    }
}

pub struct B20XrplBridge {
    pub b20_settlement: Arc<B20SettlementEngine>,
    pub xrpl_bridge: Arc<X402XrplBridge>,
    pub cross_chain_emitter: Arc<CrossChainEmitterV2>,
    pub memo_tracer: Arc<MemoTracer>,
}

impl B20XrplBridge {
    pub async fn get_bridge_escrow_address(&self) -> Result<Address, BridgeError> {
        Ok(Address::zero())
    }

    pub async fn b20_to_xrpl_escrow(
        &self,
        payment: &B20Payment,
    ) -> Result<String, BridgeError> {
        let action = payment.to_action();
        let compliance = self.b20_settlement.compliance_engine.evaluate_compliance(&action).await.map_err(|e| BridgeError::Settlement(crate::substrato_4004::settlement_engine::SettlementError::Compliance(e)))?;

        if !compliance.overall {
            return Err(BridgeError::ComplianceFailed(compliance));
        }

        let escrow_address = self.get_bridge_escrow_address().await?;
        let freeze_tx = self.b20_settlement.execute_b20_operation(&B20Operation::Transfer {
            token: payment.token,
            from: payment.from,
            to: escrow_address,
            amount: payment.amount,
            memo: Some(self.memo_tracer.generate_memo(&action)),
            policy_scope: PolicyScope::TransferSender,
        }).await?;

        let xrpl_escrow_id = self.xrpl_bridge.create_settlement_escrow(
            &payment.to_x402_payment()
        ).await?;

        self.cross_chain_emitter.emit_cross_chain(OrchestratorEvent::B20ToXrplBridge {
            b20_tx_hash: freeze_tx,
            xrpl_escrow_id: xrpl_escrow_id.clone(),
            amount: payment.amount.to_string(),
            token: format!("{:?}", payment.token),
            timestamp: chrono::Utc::now().timestamp(),
        }).await.map_err(|e| BridgeError::CrossChain(e))?;

        B20_XRPL_BRIDGE_TRANSFERS_TOTAL.inc();
        Ok(xrpl_escrow_id)
    }

    pub async fn xrpl_to_b20_release(
        &self,
        xrpl_escrow_id: &str,
        b20_recipient: Address,
    ) -> Result<String, BridgeError> {
        let escrow_state = self.xrpl_bridge.escrow_manager.get_state(xrpl_escrow_id).await?;

        if !escrow_state.released {
            return Err(BridgeError::EscrowNotReleased(xrpl_escrow_id.to_string()));
        }

        let escrow_address = self.get_bridge_escrow_address().await?;

        let mock_action = Action {
            id: xrpl_escrow_id.to_string(),
            action_type: "xrpl-release".to_string(),
            payload: serde_json::Value::Null,
            metadata: std::collections::HashMap::new(),
        };

        let release_tx: String = self.b20_settlement.execute_b20_operation(&B20Operation::Transfer {
            token: escrow_state.token,
            from: escrow_address,
            to: b20_recipient,
            amount: escrow_state.amount,
            memo: Some(hash_memo("xrpl-release", &mock_action)),
            policy_scope: PolicyScope::TransferSender,
        }).await?;

        self.cross_chain_emitter.emit_cross_chain(OrchestratorEvent::XrplToB20Release {
            xrpl_escrow_id: xrpl_escrow_id.to_string(),
            b20_tx_hash: release_tx.clone(),
            recipient: format!("{:?}", b20_recipient),
            timestamp: chrono::Utc::now().timestamp(),
        }).await.map_err(|e| BridgeError::CrossChain(e))?;

        B20_XRPL_BRIDGE_TRANSFERS_TOTAL.inc();
        Ok(release_tx)
    }
}
