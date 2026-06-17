pub mod manager;

pub use manager::{AttestationManager, PolicyDescriptor};

use async_trait::async_trait;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExecutionAttestation {
    pub id: String,
    pub policy_compliance: bool,
}

impl ExecutionAttestation {
    pub fn is_policy_compliant(&self) -> bool {
        self.policy_compliance
    }

    pub fn policy_attestation_id(&self) -> String {
        "policy_id_123".to_string()
    }
}

#[async_trait]
pub trait AttestationProvider: Send + Sync {
    async fn run_authorized(&self, workload: &str, cost_cap: Option<f64>, identity: &crate::identity_attestation::IdentityAttestation) -> Result<ExecutionAttestation, String>;
}

pub trait AttestationVerifier: Send + Sync {
    fn verify(&self, data: &[u8], signature: &[u8]) -> Result<bool, String>;
}

pub struct CathedralComputeProvider;

impl CathedralComputeProvider {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl AttestationProvider for CathedralComputeProvider {
    async fn run_authorized(&self, workload: &str, _cost_cap: Option<f64>, _identity: &crate::identity_attestation::IdentityAttestation) -> Result<ExecutionAttestation, String> {
        Ok(ExecutionAttestation {
            id: format!("exec_{}", uuid::Uuid::new_v4()),
            policy_compliance: true,
        })
    }
}
