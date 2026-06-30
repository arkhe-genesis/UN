use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthicsVerdict {
    pub verdict: crate::rule::Severity,
    pub reason: String,
    pub rule_id: Option<String>,
}
