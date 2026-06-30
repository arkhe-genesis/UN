use safe_core_dyn_signature::{DynSignature, DynPublicKey, verify_dyn_signature};
use crate::error::{GovernanceError, GovernanceResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GovernanceAction {
    RollbackCurriculum {
        target_sth: Vec<u8>,
        reason: String,
    },
    AdjustTeacherReward {
        teacher_id: String,
        environment_hash: String,
        reward_delta: f64,
        reason: String,
    },
    BanRoutingPath {
        router_id: String,
        from_module: String,
        to_module: String,
        reason: String,
    },
    EmergencyFreeze {
        reason: String,
        duration_seconds: u64,
    },
    Unfreeze {
        reason: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceEntry {
    pub action: GovernanceAction,
    pub issued_by: String,
    pub timestamp: i64,
    pub signature: DynSignature,
    pub verifying_key: DynPublicKey,
}

impl GovernanceEntry {
    pub fn verify(&self) -> GovernanceResult<()> {
        // Use canonical JSON representation before signing
        let value = serde_json::to_value(&self.action)
            .map_err(|e| GovernanceError::Serialization(e.to_string()))?;

        let payload = serde_json::to_vec(&value)
             .map_err(|e| GovernanceError::Serialization(e.to_string()))?;

        verify_dyn_signature(&self.signature, &self.verifying_key, &payload)
            .map_err(|e| GovernanceError::InvalidSignature(e.to_string()))
    }
}
