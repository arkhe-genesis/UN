use serde::{Deserialize, Serialize};

use crate::simulation::trajectory_store::TrajectoryStore;
use crate::testing::test_agent::TestResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestAttestation {
    pub attestation_id: String,
    pub test_result: TestResult,
    pub signature: Option<String>,
    pub verified: bool,
}

impl TestAttestation {
    pub fn new(test_result: TestResult) -> Self {
        Self {
            attestation_id: uuid::Uuid::new_v4().to_string(),
            test_result,
            signature: None,
            verified: false,
        }
    }

    pub async fn persist(&self, store: &TrajectoryStore) -> Result<String, String> {
        let json =
            serde_json::to_string(self).map_err(|e| format!("Serialization error: {}", e))?;
        let goal = format!(
            "test_attestation:{}:{:?}",
            self.test_result.test_name, self.test_result.test_type
        );
        store
            .record_trajectory("test_orchestrator", &goal, vec![], &json, vec![], vec![])
            .await
    }
}

pub trait TestAttestationExt {
    async fn store_test_result_as_attestation(
        &self,
        store: &TrajectoryStore,
    ) -> Result<String, String>;
}

impl TestAttestationExt for TestResult {
    async fn store_test_result_as_attestation(
        &self,
        store: &TrajectoryStore,
    ) -> Result<String, String> {
        let att = TestAttestation::new(self.clone());
        att.persist(store).await
    }
}
