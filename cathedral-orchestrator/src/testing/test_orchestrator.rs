use futures::future::join_all;
use serde_json::json;
use std::sync::Arc;
use tracing::{error, info};

use crate::simulation::trajectory_store::TrajectoryStore;
use crate::testing::test_agent::{TestAgent, TestResult, TestType};
use crate::testing::test_attestation::TestAttestationExt;

pub struct TestOrchestrator {
    store: Arc<TrajectoryStore>,
    pub test_agents: Vec<Arc<dyn TestAgent>>,
}

impl TestOrchestrator {
    pub fn new(store: Arc<TrajectoryStore>) -> Self {
        Self {
            store,
            test_agents: Vec::new(),
        }
    }

    pub fn register_test_agent(&mut self, agent: Arc<dyn TestAgent>) {
        info!("📋 Agente de teste registado: {}", agent.test_name());
        self.test_agents.push(agent);
    }


    pub async fn run_all_tests(&self) -> Vec<TestResult> {
        info!(
            "🚀 Executando todos os {} testes...",
            self.test_agents.len()
        );

        let context = crate::testing::test_agent::TestContext::new("orchestrator");

        let handles: Vec<_> = self
            .test_agents
            .iter()
            .map(|agent| {
                let ctx = context.clone();
                let agent_clone = agent.clone();
                tokio::spawn(async move {
                    use crate::testing::otel_integration::TraceableTestAgent;
                    agent_clone.run_test_with_tracing_and_metrics(&ctx).await
                })
            })
            .collect();

        let results: Vec<Result<Result<TestResult, String>, _>> = join_all(handles).await;
        let mut test_results = Vec::new();

        for result in results {
            match result {
                Ok(Ok(test_result)) => {
                    if let Err(e) = test_result
                        .store_test_result_as_attestation(self.store.as_ref())
                        .await
                    {
                        error!("Falha ao persistir atestado de teste: {}", e);
                    }
                    test_results.push(test_result);
                }
                Ok(Err(e)) => error!("Erro no teste: {}", e),
                Err(e) => error!("Panic no teste: {}", e),
            }
        }

        self.generate_report(&test_results).await;
        info!("✅ Testes concluídos: {} resultados", test_results.len());
        test_results
    }

    async fn generate_report(&self, results: &[TestResult]) {
        let total = results.len();
        let passed = results.iter().filter(|r| r.passed).count();
        let failed = total - passed;

        let report = json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "total_tests": total,
            "passed": passed,
            "failed": failed,
            "success_rate": if total > 0 { passed as f64 / total as f64 } else { 0.0 },
            "results": results.iter().map(|r| json!({
                "name": r.test_name,
                "type": format!("{:?}", r.test_type),
                "passed": r.passed,
                "duration_ms": r.duration_ms,
                "details": r.details,
            })).collect::<Vec<_>>(),
        });

        let report_json = serde_json::to_string_pretty(&report).unwrap_or_default();
        info!("📊 Relatório de testes:\n{}", report_json);

        let report_result = TestResult {
            test_id: uuid::Uuid::new_v4().to_string(),
            test_name: "test_report".to_string(),
            test_type: TestType::Integration,
            passed: true,
            duration_ms: 0,
            details: report,
            attestation_id: None,
            timestamp: chrono::Utc::now(),
        };

        if let Err(e) = report_result
            .store_test_result_as_attestation(self.store.as_ref())
            .await
        {
            error!("Falha ao persistir relatório como TestAttestation: {}", e);
        }
    }
}
