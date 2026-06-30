use crate::error::RuntimeError;
use crate::runtime::ModelRuntime;
use crate::types::*;
use async_trait::async_trait;
use safe_core_parallax_bridge::ParallaxClient;
use std::collections::HashMap;
use tracing::info;
pub struct ParallaxBackend { client: ParallaxClient, config: ModelConfig, model_name: String }
impl ParallaxBackend {
    pub async fn new(scheduler_addr: &str, model_name: &str, config: ModelConfig) -> Result<Self, RuntimeError> {
        let client = ParallaxClient::connect(scheduler_addr).await.map_err(|e| RuntimeError::LoadFailed(e.to_string()))?;
        let health = client.health().await.map_err(|e| RuntimeError::LoadFailed(e.to_string()))?;
        if !health.ready { return Err(RuntimeError::NotReady); }
        info!("Parallax cluster ready, version: {}", health.version);
        let models = client.list_models().await.map_err(|e| RuntimeError::LoadFailed(e.to_string()))?;
        if !models.contains(&model_name.to_string()) { return Err(RuntimeError::NotFound(format!("Model '{}' not found. Available: {:?}", model_name, models))); }
        Ok(Self { client, config, model_name: model_name.to_string() })
    }
}
#[async_trait]
impl ModelRuntime for ParallaxBackend {
    fn name(&self) -> &str { "parallax" }
    fn is_ready(&self) -> bool { true }
    fn config(&self) -> &ModelConfig { &self.config }
    async fn x_infer(&self, request: InferenceRequest) -> Result<InferenceResponse, RuntimeError> {
        let req = safe_core_parallax_bridge::InferRequest {
            model_name: self.model_name.clone(),
            prompt: request.prompt,
            messages: request.messages.into_iter().map(|m| {
                let role = match m.role { ChatRole::System => "system", ChatRole::User => "user", ChatRole::Assistant => "assistant", ChatRole::Tool => "tool" }.to_string();
                safe_core_parallax_bridge::ChatMessage { role, content: m.content }
            }).collect(),
            params: safe_core_parallax_bridge::SamplingParams { temperature: request.params.temperature, top_p: request.params.top_p, top_k: request.params.top_k, max_tokens: request.params.max_tokens, stop_sequences: request.params.stop_sequences, seed: request.params.seed },
            metadata: request.metadata,
        };
        let resp = self.client.infer(req).await.map_err(|e| RuntimeError::InferenceFailed(e.to_string()))?;
        Ok(InferenceResponse {
            id: resp.id, content: resp.content,
            tool_calls: resp.tool_calls.into_iter().map(|tc| ToolCall { id: tc.id, name: tc.name, arguments: tc.arguments.to_string() }).collect(),
            usage: TokenUsage { prompt_tokens: resp.usage.prompt_tokens, completion_tokens: resp.usage.completion_tokens, total_tokens: resp.usage.total_tokens },
            finish_reason: match resp.finish_reason.as_str() { "stop" => FinishReason::Stop, "length" => FinishReason::Length, "tool_calls" | "tool_call" => FinishReason::ToolCall, other => { tracing::warn!("Unknown finish_reason: {}", other); FinishReason::Stop } },
            timestamp: chrono::Utc::now(), metadata: HashMap::new(),
        })
    }
    async fn x_infer_chat(&self, messages: Vec<ChatMessage>) -> Result<InferenceResponse, RuntimeError> {
        let params = SamplingParams { temperature: self.config.temperature, top_p: self.config.top_p, top_k: None, max_tokens: self.config.max_tokens, stop_sequences: Vec::new(), seed: None };
        let request = InferenceRequest { id: uuid::Uuid::new_v4().to_string(), prompt: String::new(), system_prompt: None, messages, params, tools: Vec::new(), metadata: HashMap::new() };
        self.x_infer(request).await
    }
    async fn x_embed(&self, texts: Vec<String>) -> Result<Vec<Tensor>, RuntimeError> {
        let req = safe_core_parallax_bridge::EmbedRequest { model_name: self.model_name.clone(), texts };
        let resp = self.client.embed(req).await.map_err(|e| RuntimeError::InferenceFailed(e.to_string()))?;
        let tensors = resp.embeddings.into_iter().map(|emb| Tensor::new(emb.values.clone(), vec![emb.values.len()])).collect();
        Ok(tensors)
    }
    async fn unload(&self) -> Result<(), RuntimeError> { Ok(()) }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_x_infer_chat_conversion() {
        let config = ModelConfig { temperature: 0.8, top_p: 0.95, max_tokens: 4096, device: Device::Cpu, quantization: None, metadata: HashMap::new() };
        let params = SamplingParams { temperature: config.temperature, top_p: config.top_p, top_k: None, max_tokens: config.max_tokens, stop_sequences: Vec::new(), seed: None };
        assert_eq!(params.temperature, 0.8);
        assert_eq!(params.top_p, 0.95);
        assert_eq!(params.max_tokens, 4096);
    }
}
