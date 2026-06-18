
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Critical => write!(f, "critical"),
            Severity::High => write!(f, "high"),
            Severity::Medium => write!(f, "medium"),
            Severity::Low => write!(f, "low"),
            Severity::Info => write!(f, "info"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vulnerability {
    pub id: String,
    pub title: String,
    pub description: String,
    pub severity: Severity,
    pub location: String,
    pub cwe_id: Option<String>,
    pub verified: bool,
    pub exploitation_details: Option<String>,
    pub remediation: Option<String>,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnerabilityProof {
    pub result_hash: String,
    pub signature: String,
    pub attestor_public_key: String,
    pub timestamp: u64,
    pub openant_version: String,
}

pub struct OpenAntClient {}

impl OpenAntClient {
    pub fn new() -> Self {
        Self {}
    }
}
