use cathedral_orchestrator::geometry::service::CausalGeometryService;
use cathedral_orchestrator::governance::geometric_policy_engine::GeometricPolicyEngine;
use cathedral_orchestrator::simulation::trajectory_store::TrajectoryStore;
use cathedral_orchestrator::testing::{
    ChaosTestAgent, ComplianceTestAgent, IntegrationTestAgent, IntegrityTestAgent,
    PerformanceTestAgent, SecurityTestAgent, TestAgent, TestContext, TestOrchestrator, TestType,
    TraceableTestAgent,
};
use cathedral_orchestrator::{PrivacyGuard, SemanticCache, SimpleEmbedder};
use std::sync::Arc;
use tokio;

#[tokio::main]
async fn main() -> Result<(), String> {
    println!("Testing setup...");
    let cache = Arc::new(SemanticCache::new(()).await?);
    let guard = Arc::new(PrivacyGuard::load("test", None)?);
    let store = Arc::new(TrajectoryStore::new(cache, guard, 10));

    let mut orchestrator = TestOrchestrator::new(store.clone());

    orchestrator.register_test_agent(Arc::new(IntegrityTestAgent::new(store.clone(), 10)));
    orchestrator.register_test_agent(Arc::new(PerformanceTestAgent::new(10)));
    orchestrator.register_test_agent(Arc::new(ChaosTestAgent::new(0.1, 0.2)));
    orchestrator.register_test_agent(Arc::new(SecurityTestAgent::new()));

    let engine = Arc::new(GeometricPolicyEngine::new(Arc::new(
        CausalGeometryService::new(Arc::new(SimpleEmbedder::new(384)), 384),
    )));
    orchestrator.register_test_agent(Arc::new(ComplianceTestAgent::new(engine, vec![])));
    orchestrator.register_test_agent(Arc::new(IntegrationTestAgent::new(5)));

    let results = orchestrator.run_all_tests().await;
    println!("Results: {:?}", results);

    Ok(())
}
