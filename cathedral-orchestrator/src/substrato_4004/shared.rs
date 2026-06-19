use ethers::types::{Address, U256};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub id: String,
    pub action_type: String,
    pub payload: serde_json::Value,
    pub metadata: HashMap<String, String>,
}

impl Action {
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(&self).unwrap_or_default()
    }
}

pub enum FilterVerdict {
    Passed,
    Failed(Vec<String>),
}

pub struct EthicalFilter;

impl EthicalFilter {
    pub async fn evaluate(&self, _action: &Action) -> FilterVerdict {
        FilterVerdict::Passed
    }
}

pub fn extract_address(action: &Action, field: &str) -> Result<Address, String> {
    let s = action.payload.get(field)
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("Missing field {}", field))?;
    s.parse().map_err(|e| format!("Invalid address: {}", e))
}

pub fn extract_u256(action: &Action, field: &str) -> Result<U256, String> {
    let s = action.payload.get(field)
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("Missing field {}", field))?;
    U256::from_dec_str(s).map_err(|e| format!("Invalid U256: {}", e))
}

pub fn extract_optional_memo(_action: &Action) -> Result<Option<[u8; 32]>, String> {
    Ok(None)
}

pub fn extract_policy_scope(_action: &Action) -> Result<crate::substrato_4004::b20_mapper::PolicyScope, String> {
    Ok(crate::substrato_4004::b20_mapper::PolicyScope::TransferSender)
}

pub fn extract_u64(_action: &Action, _field: &str) -> Result<u64, String> {
    Ok(0)
}

pub fn extract_pausable_features(_action: &Action) -> Result<Vec<crate::substrato_4004::b20_mapper::PausableFeature>, String> {
    Ok(vec![])
}

pub fn hash_memo(_prefix: &str, _action: &Action) -> [u8; 32] {
    [0; 32]
}
