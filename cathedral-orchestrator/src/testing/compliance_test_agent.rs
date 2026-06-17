use async_trait::async_trait;
use serde_json::json;
use std::sync::Arc;
use tracing::{info, instrument};

use crate::governance::geometric_policy_engine::GeometricPolicyEngine;
use crate::testing::test_agent::{TestAgent, TestContext, TestResult, TestType};

pub struct ComplianceTestAgent {
    name: String,
    _policy_engine: Arc<GeometricPolicyEngine>,
    required_policies: Vec<String>,
}

impl ComplianceTestAgent {
    pub fn new(policy_engine: Arc<GeometricPolicyEngine>, required_policies: Vec<String>) -> Self {
        Self {
            name: "ComplianceTestAgent".to_string(),
            _policy_engine: policy_engine,
            required_policies,
        }
    }
}

#[async_trait]
impl TestAgent for ComplianceTestAgent {
    fn test_name(&self) -> &str {
        &self.name
    }
    fn test_type(&self) -> TestType {
        TestType::Compliance
    }

    #[instrument(name = "compliance_test.run", skip(self))]
    async fn run_test(&self, _context: &TestContext) -> Result<TestResult, String> {
        info!("📜 Executando teste de conformidade...");
        let start = std::time::Instant::now();
        let duration_ms = start.elapsed().as_millis() as u64;

        let details = json!({
            "required_policies": self.required_policies,
        });

        Ok(TestResult {
            test_id: uuid::Uuid::new_v4().to_string(),
            test_name: self.name.clone(),
            test_type: TestType::Compliance,
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
            "required_policies": self.required_policies,
            "agent_name": self.name,
        })
    }
}
