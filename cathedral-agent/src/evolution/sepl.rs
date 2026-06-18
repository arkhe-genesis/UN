use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionContext {
    pub resource_id: String,
    pub agent_id: String,
    pub goal: String,
    pub constraints: Vec<String>,
    pub available_tools: Vec<String>,
    pub memory_keys: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observation {
    pub resource_id: String,
    pub current_version: String,
    pub performance_metrics: Vec<u8>,
    pub usage_patterns: Vec<String>,
    pub errors: Vec<String>,
    pub context: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    pub resource_id: String,
    pub target_version: String,
    pub changes: Vec<Change>,
    pub rationale: String,
    pub expected_improvement: HashMap<String, f64>,
    pub proposed_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Change {
    pub change_type: ChangeType,
    pub path: String,
    pub before: Option<String>,
    pub after: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeType {
    ParameterTuning,
    CodeModification,
    ToolAddition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Verification {
    pub success: bool,
    pub reward_score: RewardScore,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardScore {
    pub total: f64,
}

use crate::eve_client::{EveClient, EveTask, EveStrategy};
use crate::thread_index::ThreadIndex;
use crate::hashtree_adapter::HashTreeStorage;
use crate::trace_manager::TraceManager;
use crate::skill::builtin::qvac_inference::{QVACInferenceExecutor, QVACConfig};
use crate::error_handling::retry::{RetryConfig, RetryContext};
use crate::error_handling::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};
use crate::evolution::resource::Resource;
use std::sync::Arc;
use tracing::{info, warn};

pub struct AutogenesisOperator {
    pub eve_client: EveClient,
    pub thread_index: ThreadIndex,
    pub storage: HashTreeStorage,
    pub trace_manager: Arc<TraceManager>,
    pub qvac_executor: Option<QVACInferenceExecutor>,
    pub max_iterations: usize,
    pub use_qvac: bool,
    pub circuit_breaker: CircuitBreaker,
}

impl AutogenesisOperator {
    pub async fn new_with_qvac(
        eve_client: EveClient,
        thread_index: ThreadIndex,
        storage: HashTreeStorage,
        trace_manager: Arc<TraceManager>,
        default_model_hash: &str,
        qvac_config: QVACConfig,
        max_iterations: usize,
    ) -> Result<Self, String> {
        let qvac_executor = QVACInferenceExecutor::new(
            storage.clone(),
            trace_manager.clone(),
            qvac_config,
            default_model_hash,
        );

        Ok(Self {
            eve_client,
            thread_index,
            storage,
            trace_manager,
            qvac_executor: Some(qvac_executor),
            max_iterations,
            use_qvac: true,
            circuit_breaker: CircuitBreaker::new(CircuitBreakerConfig::default()),
        })
    }

    pub fn disable_qvac(&mut self) {
        self.use_qvac = false;
    }

    pub async fn infer_with_strategy(&self, prompt: &str, trace_id: Option<&str>) -> Result<String, String> {
        if self.use_qvac {
            if let Some(qvac) = &self.qvac_executor {
                match qvac.infer(prompt, None, trace_id).await {
                    Ok(result) => {
                        info!("✅ Inferência via QVAC local bem-sucedida");
                        return Ok(result);
                    }
                    Err(e) => {
                        warn!("❌ QVAC falhou: {}, fallback para Eve", e);
                    }
                }
            }
        }

        info!("☁️ Usando Eve (cloud) para inferência");
        let prompt_clone = prompt.to_string();
        let client = self.eve_client.clone();

        let result: Result<String, String> = self.circuit_breaker.call(|| {
            let client = client.clone();
            let prompt = prompt_clone.clone();
            Box::pin(async move {
                let mut retry_ctx = RetryContext::new(RetryConfig::default());
                retry_ctx.retry_async(|| {
                    let client = client.clone();
                    let prompt = prompt.clone();
                    Box::pin(async move {
                        let task = EveTask::new(&prompt).with_strategy(EveStrategy::Prototype);
                        let result = client.execute_task_blocking(&task, 60).await?;
                        Ok(result.code.unwrap_or_default())
                    })
                }).await
            })
        }).await;

        result.map_err(|e| format!("Falha na inferência após fallback e retries: {}", e))
    }

    pub async fn reflect(&self, context: &EvolutionContext, resource: &dyn Resource) -> Result<Observation, String> {
        info!("🔍 [SEPL] Refletindo sobre recurso: {}", context.resource_id);

        let metrics = self.thread_index.get_usage_metrics(&context.resource_id).await?;

        let prompt = format!(
            "Analyze resource '{}' (version {}). Metrics: {:?}. Goal: {}. Produce structured observation.",
            context.resource_id, resource.metadata().version, metrics, context.goal
        );

        let trace_id = self.trace_manager.start_trace(&context.resource_id).await.ok();
        let response = self.infer_with_strategy(&prompt, trace_id.as_deref()).await?;

        Ok(Observation {
            resource_id: context.resource_id.clone(),
            current_version: resource.metadata().version.clone(),
            performance_metrics: metrics,
            usage_patterns: Vec::new(),
            errors: Vec::new(),
            context: response,
            timestamp: chrono::Utc::now().timestamp() as u64,
        })
    }

    pub async fn propose(&self, observation: &Observation, context: &EvolutionContext) -> Result<Proposal, String> {
        info!("💡 [SEPL] Propondo evolução para: {}", observation.resource_id);

        let prompt = format!(
            "Based on observation: {:?}, propose concrete changes with rationale and expected improvement.",
            observation
        );

        let trace_id = self.trace_manager.start_trace(&context.resource_id).await.ok();
        let _response = self.infer_with_strategy(&prompt, trace_id.as_deref()).await?;

        Ok(Proposal {
            resource_id: observation.resource_id.clone(),
            target_version: format!("{}-proposed", observation.current_version),
            changes: Vec::new(),
            rationale: "Improvement needed".to_string(),
            expected_improvement: HashMap::new(),
            proposed_by: context.agent_id.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_infer_with_strategy_fallback() {
        let eve_client = EveClient {};
        let thread_index = ThreadIndex {};
        let storage = HashTreeStorage::new();
        let trace_manager = Arc::new(TraceManager::new());
        let qvac_config = QVACConfig::default();

        let operator = AutogenesisOperator::new_with_qvac(
            eve_client,
            thread_index,
            storage,
            trace_manager,
            "test_hash",
            qvac_config,
            5
        ).await.unwrap();

        let res = operator.infer_with_strategy("test prompt", None).await;
        assert!(res.is_ok());

        let mut operator = AutogenesisOperator::new_with_qvac(
            EveClient {},
            ThreadIndex {},
            HashTreeStorage::new(),
            Arc::new(TraceManager::new()),
            "test_hash",
            QVACConfig::default(),
            5
        ).await.unwrap();
        operator.disable_qvac();

        let res = operator.infer_with_strategy("test prompt", None).await;
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), "");
    }
}
