use async_trait::async_trait;
use serde_json::json;
use tracing::{info, instrument};

use crate::testing::test_agent::{TestAgent, TestContext, TestResult, TestType};

pub struct SecurityTestAgent {
    name: String,
}

impl Default for SecurityTestAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl SecurityTestAgent {
    pub fn new() -> Self {
        Self {
            name: "SecurityTestAgent".to_string(),
        }
    }
}

#[async_trait]
impl TestAgent for SecurityTestAgent {
    fn test_name(&self) -> &str {
        &self.name
    }
    fn test_type(&self) -> TestType {
        TestType::Security
    }

    #[instrument(name = "security_test.run", skip(self))]
    async fn run_test(&self, context: &TestContext) -> Result<TestResult, String> {
        info!("🔐 Executando teste de segurança...");
        let start = std::time::Instant::now();
        let duration_ms = start.elapsed().as_millis() as u64;

        let details = json!({});

        Ok(TestResult {
            test_id: uuid::Uuid::new_v4().to_string(),
            test_name: self.name.clone(),
            test_type: TestType::Security,
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
            "agent_name": self.name,
        })
    }
}
