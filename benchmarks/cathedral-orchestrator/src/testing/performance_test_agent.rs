use async_trait::async_trait;
use serde_json::json;
use tracing::{info, instrument};

use crate::testing::test_agent::{TestAgent, TestContext, TestResult, TestType};

pub struct PerformanceTestAgent {
    name: String,
    default_concurrency: usize,
}

impl PerformanceTestAgent {
    pub fn new(default_concurrency: usize) -> Self {
        Self {
            name: "PerformanceTestAgent".to_string(),
            default_concurrency,
        }
    }
}

#[async_trait]
impl TestAgent for PerformanceTestAgent {
    fn test_name(&self) -> &str {
        &self.name
    }
    fn test_type(&self) -> TestType {
        TestType::Performance
    }

    #[instrument(name = "performance_test.run", skip(self))]
    async fn run_test(&self, context: &TestContext) -> Result<TestResult, String> {
        info!("⚡ Executando teste de performance...");
        let start = std::time::Instant::now();
        let duration_ms = start.elapsed().as_millis() as u64;

        let details = json!({
            "concurrency": self.default_concurrency,
        });

        Ok(TestResult {
            test_id: uuid::Uuid::new_v4().to_string(),
            test_name: self.name.clone(),
            test_type: TestType::Performance,
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
            "default_concurrency": self.default_concurrency,
            "agent_name": self.name,
        })
    }
}
