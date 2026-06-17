use cathedral_orchestrator::geometry::service::CausalGeometryService;
use cathedral_orchestrator::governance::geometric_policy_engine::GeometricPolicyEngine;
use cathedral_orchestrator::simulation::trajectory_store::TrajectoryStore;
use cathedral_orchestrator::testing::{
    ChaosTestAgent, ComplianceTestAgent, IntegrationTestAgent, IntegrityTestAgent,
    PerformanceTestAgent, SecurityTestAgent, TestAgent, TestContext, TestOrchestrator, TestResult,
    TestType,
};
use cathedral_orchestrator::{PrivacyGuard, SemanticCache, SimpleEmbedder};
use std::sync::Arc;

async fn setup_test_environment() -> (Arc<TrajectoryStore>, Arc<GeometricPolicyEngine>) {
    let cache = Arc::new(SemanticCache::new(()).await.unwrap());
    let guard = Arc::new(PrivacyGuard::load("test", None).unwrap());
    let store = Arc::new(TrajectoryStore::new(cache, guard, 10));
    let engine = Arc::new(GeometricPolicyEngine::new(Arc::new(
        CausalGeometryService::new(Arc::new(SimpleEmbedder::new(384)), 384),
    )));
    (store, engine)
}

#[tokio::test]
async fn test_integrity_agent_success() {
    let (store, _) = setup_test_environment().await;
    let agent = IntegrityTestAgent::new(store, 10);
    let context = TestContext::new("test");
    let result = agent.run_test(&context).await.unwrap();
    assert!(result.passed);
}

#[tokio::test]
async fn test_performance_agent_basic() {
    let agent = PerformanceTestAgent::new(2);
    let mut context = TestContext::new("test");
    context = context.with_parameter("concurrency", 2);
    context = context.with_parameter("tasks", 5);
    let result = agent.run_test(&context).await.unwrap();
    assert!(result.passed);
}

#[tokio::test]
async fn test_chaos_agent_basic() {
    let agent = ChaosTestAgent::new(0.3, 20.0);
    let context = TestContext::new("test");
    let result = agent.run_test(&context).await.unwrap();
    assert!(result.passed);
}

#[tokio::test]
async fn test_security_agent_basic() {
    let agent = SecurityTestAgent::new();
    let context = TestContext::new("test");
    let result = agent.run_test(&context).await.unwrap();
    assert!(result.passed);
}

#[tokio::test]
async fn test_compliance_agent_basic() {
    let (_, engine) = setup_test_environment().await;
    let required_policies = vec!["pii_prohibition".to_string(), "steering_safety".to_string()];
    let agent = ComplianceTestAgent::new(engine, required_policies);
    let context = TestContext::new("test");
    let result = agent.run_test(&context).await.unwrap();
    assert!(result.passed);
}

#[tokio::test]
async fn test_integration_agent_basic() {
    let agent = IntegrationTestAgent::new(3);
    let context = TestContext::new("test");
    let result = agent.run_test(&context).await.unwrap();
    assert!(result.passed);
}

#[test]
fn test_test_result_serialization() {
    let result = TestResult {
        test_id: "test-123".to_string(),
        test_name: "unit_test".to_string(),
        test_type: TestType::Integrity,
        passed: true,
        duration_ms: 100,
        details: serde_json::json!({ "detail": "test" }),
        attestation_id: None,
        timestamp: chrono::Utc::now(),
    };

    let json = serde_json::to_string(&result).unwrap();
    let deserialized: TestResult = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.test_id, result.test_id);
    assert_eq!(deserialized.passed, result.passed);
}
