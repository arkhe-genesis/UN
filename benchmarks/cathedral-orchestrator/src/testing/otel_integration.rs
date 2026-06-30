use opentelemetry::global;
use opentelemetry::metrics::{Counter, Histogram};
use std::sync::OnceLock;
use tracing::{error, info, span, Level};

use crate::testing::test_agent::{TestAgent, TestContext, TestResult};

static METRICS: OnceLock<TestMetrics> = OnceLock::new();

struct TestMetrics {
    test_duration: Histogram<f64>,
    test_success: Counter<u64>,
    test_failure: Counter<u64>,
    test_total: Counter<u64>,
}

impl TestMetrics {
    fn init() -> &'static Self {
        METRICS.get_or_init(|| {
            let meter = global::meter("cathedral-arkhe-testing");
            Self {
                test_duration: meter
                    .f64_histogram("test.duration_seconds")
                    .with_description("Duration of test execution in seconds")
                    .init(),
                test_success: meter
                    .u64_counter("test.success_total")
                    .with_description("Total number of successful tests")
                    .init(),
                test_failure: meter
                    .u64_counter("test.failure_total")
                    .with_description("Total number of failed tests")
                    .init(),
                test_total: meter
                    .u64_counter("test.total")
                    .with_description("Total number of tests executed")
                    .init(),
            }
        })
    }
}

pub trait TraceableTestAgent: TestAgent {
    async fn run_test_with_tracing_and_metrics(
        &self,
        context: &TestContext,
    ) -> Result<TestResult, String> {
        let span = span!(
            Level::INFO,
            "test.agent",
            test_name = %self.test_name(),
            test_type = ?self.test_type(),
            agent_id = %context.agent_id,
        );
        let _enter = span.enter();

        info!("🔄 Executando teste com tracing: {}", self.test_name());

        let start = std::time::Instant::now();
        let result = self.run_test(context).await;
        let duration = start.elapsed().as_secs_f64();

        let metrics = TestMetrics::init();
        metrics.test_duration.record(duration, &[]);
        metrics.test_total.add(1, &[]);

        match &result {
            Ok(test_result) => {
                span.record("passed", test_result.passed);
                span.record("duration_ms", test_result.duration_ms);
                if test_result.passed {
                    metrics.test_success.add(1, &[]);
                    info!("✅ Teste concluído: {} (passou)", test_result.test_name);
                } else {
                    metrics.test_failure.add(1, &[]);
                    info!("❌ Teste concluído: {} (falhou)", test_result.test_name);
                }
            }
            Err(e) => {
                metrics.test_failure.add(1, &[]);
                span.record("error", e.as_str());
                error!("❌ Teste falhou: {} - {}", self.test_name(), e);
            }
        }

        result
    }
}

impl<T: ?Sized + TestAgent> TraceableTestAgent for T {}

pub fn init_test_metrics() {
    let _ = TestMetrics::init();
}
