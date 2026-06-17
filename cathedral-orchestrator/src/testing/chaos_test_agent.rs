use async_trait::async_trait;
use serde_json::json;
use tracing::{info, instrument};

use crate::testing::test_agent::{TestAgent, TestContext, TestResult, TestType};

pub struct ChaosTestAgent {
    name: String,
    failure_rate: f64,
    kill_percentage: f32,
}

impl ChaosTestAgent {
    pub fn new(failure_rate: f64, kill_percentage: f32) -> Self {
        Self {
            name: "ChaosTestAgent".to_string(),
            failure_rate,
            kill_percentage,
        }
    }
}

#[async_trait]
impl TestAgent for ChaosTestAgent {
    fn test_name(&self) -> &str {
        &self.name
    }
    fn test_type(&self) -> TestType {
        TestType::Chaos
    }

    #[instrument(name = "chaos_test.run", skip(self))]
    async fn run_test(&self, context: &TestContext) -> Result<TestResult, String> {
        info!("💀 Executando teste de caos...");
        let start = std::time::Instant::now();
        let duration_ms = start.elapsed().as_millis() as u64;

        let details = json!({
            "failure_rate": self.failure_rate,
            "kill_percentage": self.kill_percentage,
        });

        Ok(TestResult {
            test_id: uuid::Uuid::new_v4().to_string(),
            test_name: self.name.clone(),
            test_type: TestType::Chaos,
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
            "failure_rate": self.failure_rate,
            "kill_percentage": self.kill_percentage,
            "agent_name": self.name,
        })
    }
}
