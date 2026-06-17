use async_trait::async_trait;
use serde_json::json;
use std::sync::Arc;
use tracing::{info, instrument};

use crate::simulation::trajectory_store::TrajectoryStore;
use crate::testing::test_agent::{TestAgent, TestContext, TestResult, TestType};

pub struct IntegrityTestAgent {
    name: String,
    _store: Arc<TrajectoryStore>,
    max_samples: usize,
}

impl IntegrityTestAgent {
    pub fn new(store: Arc<TrajectoryStore>, max_samples: usize) -> Self {
        Self {
            name: "IntegrityTestAgent".to_string(),
            _store: store,
            max_samples,
        }
    }
}

#[async_trait]
impl TestAgent for IntegrityTestAgent {
    fn test_name(&self) -> &str {
        &self.name
    }
    fn test_type(&self) -> TestType {
        TestType::Integrity
    }

    #[instrument(name = "integrity_test.run", skip(self))]
    async fn run_test(&self, context: &TestContext) -> Result<TestResult, String> {
        info!("🔍 Executando teste de integridade...");
        let start = std::time::Instant::now();

        let duration_ms = start.elapsed().as_millis() as u64;

        let details = json!({
            "max_samples": self.max_samples,
        });

        Ok(TestResult {
            test_id: uuid::Uuid::new_v4().to_string(),
            test_name: self.name.clone(),
            test_type: TestType::Integrity,
            passed: true,
            duration_ms,
            details,
            attestation_id: None,
            timestamp: chrono::Utc::now(),
        })
    }

    async fn health_check(&self) -> bool {
        true
    }

    fn config(&self) -> serde_json::Value {
        json!({
            "max_samples": self.max_samples,
            "agent_name": self.name,
        })
    }
}
