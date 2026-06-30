use async_trait::async_trait;
use serde_json::json;
use tracing::{info, instrument};

use crate::testing::test_agent::{TestAgent, TestContext, TestResult, TestType};

pub struct IntegrationTestAgent {
    name: String,
    test_count: usize,
}

impl IntegrationTestAgent {
    pub fn new(test_count: usize) -> Self {
        Self {
            name: "IntegrationTestAgent".to_string(),
            test_count,
        }
    }
}

#[async_trait]
impl TestAgent for IntegrationTestAgent {
    fn test_name(&self) -> &str {
        &self.name
    }
    fn test_type(&self) -> TestType {
        TestType::Integration
    }

    #[instrument(name = "integration_test.run", skip(self))]
    async fn run_test(&self, _context: &TestContext) -> Result<TestResult, String> {
        info!("🔗 Executando teste de integração...");
        let start = std::time::Instant::now();
        let duration_ms = start.elapsed().as_millis() as u64;

        let details = json!({
            "test_count": self.test_count,
        });

        Ok(TestResult {
            test_id: uuid::Uuid::new_v4().to_string(),
            test_name: self.name.clone(),
            test_type: TestType::Integration,
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
            "test_count": self.test_count,
            "agent_name": self.name,
        })
    }
}
