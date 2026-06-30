use crate::error::ParallaxError;
use crate::types::*;

use tokio::sync::Mutex;
use tonic::transport::Channel;
pub mod parallax {
    tonic::include_proto!("parallax");
}
use parallax::inference_service_client::InferenceServiceClient;
use parallax::{HealthRequest, InferRequest as ProtoInferRequest, ListModelsRequest};
pub struct ParallaxClient {
    inner: Mutex<InferenceServiceClient<Channel>>,
}
impl ParallaxClient {
    pub async fn connect(addr: &str) -> Result<Self, ParallaxError> {
        let channel = tonic::transport::Endpoint::try_from(addr.to_string())
            .map_err(|e| ParallaxError::Connection(e.to_string()))?
            .connect()
            .await
            .map_err(|e| ParallaxError::Connection(e.to_string()))?;
        Ok(Self { inner: Mutex::new(InferenceServiceClient::new(channel)) })
    }
    pub async fn health(&self) -> Result<HealthResponse, ParallaxError> {
        let mut client = self.inner.lock().await;
        let resp = client.health(HealthRequest {}).await.map_err(|e| ParallaxError::Grpc(e.to_string()))?.into_inner();
        Ok(HealthResponse { ready: resp.ready, version: resp.version })
    }
    pub async fn list_models(&self) -> Result<Vec<String>, ParallaxError> {
        let mut client = self.inner.lock().await;
        let resp = client.list_models(ListModelsRequest {}).await.map_err(|e| ParallaxError::Grpc(e.to_string()))?.into_inner();
        Ok(resp.model_names)
    }
    pub async fn infer(&self, req: InferRequest) -> Result<InferResponse, ParallaxError> {
        let top_k = req.params.top_k.map(|v| i32::try_from(v).unwrap_or(0)).unwrap_or(0);
        let max_tokens = i32::try_from(req.params.max_tokens).unwrap_or(i32::MAX);
        let seed = req.params.seed.map(|v| i64::try_from(v).unwrap_or(0)).unwrap_or(0);
        let proto_req = ProtoInferRequest {
            model_name: req.model_name,
            prompt: req.prompt,
            messages: req.messages.into_iter().map(|m| parallax::ChatMessage { role: m.role, content: m.content }).collect(),
            params: Some(parallax::SamplingParams { temperature: req.params.temperature, top_p: req.params.top_p, top_k, max_tokens, stop: req.params.stop_sequences, seed }),
            metadata: req.metadata,
        };
        let mut client = self.inner.lock().await;
        let response = client.infer(proto_req).await.map_err(|e| ParallaxError::Grpc(e.to_string()))?.into_inner();
        Ok(InferResponse {
            id: response.id,
            content: response.content,
            tool_calls: response.tool_calls.into_iter().map(|tc| ToolCall {
                id: tc.id, name: tc.name, arguments: match serde_json::from_str(&tc.arguments) {
                    Ok(v) => v,
                    Err(e) => { tracing::warn!("Failed to parse tool call arguments: {}", e); serde_json::json!({}) }
                },
            }).collect(),
            usage: TokenUsage {
                prompt_tokens: response.usage.as_ref().map(|u| u.prompt_tokens as u32).unwrap_or(0),
                completion_tokens: response.usage.as_ref().map(|u| u.completion_tokens as u32).unwrap_or(0),
                total_tokens: response.usage.as_ref().map(|u| u.total_tokens as u32).unwrap_or(0),
            },
            finish_reason: match response.finish_reason.as_str() {
                "stop" => "stop".to_string(), "length" => "length".to_string(), "tool_calls" | "tool_call" => "tool_calls".to_string(),
                other => { tracing::warn!("Unknown finish_reason: {}", other); "stop".to_string() }
            },
        })
    }
    pub async fn embed(&self, req: crate::types::EmbedRequest) -> Result<EmbedResponse, ParallaxError> {
        let mut client = self.inner.lock().await;
        let proto_req = parallax::EmbedRequest { model_name: req.model_name, texts: req.texts };
        let response = client.embed(proto_req).await.map_err(|e| ParallaxError::Grpc(e.to_string()))?.into_inner();
        Ok(EmbedResponse { embeddings: response.embeddings.into_iter().map(|e| Embedding { values: e.values }).collect() })
    }
}
#[cfg(test)]
mod tests {
    use super::*; use std::collections::HashMap;
    #[test]
    fn test_infer_request_conversion() {
        let req = InferRequest { model_name: "test".to_string(), prompt: "hello".to_string(), messages: vec![], params: SamplingParams { temperature: 0.7, top_p: 0.9, top_k: None, max_tokens: 100, stop_sequences: vec![], seed: None }, metadata: HashMap::new() };
        let proto_req = parallax::InferRequest { model_name: req.model_name.clone(), prompt: req.prompt.clone(), messages: vec![], params: Some(parallax::SamplingParams { temperature: req.params.temperature, top_p: req.params.top_p, top_k: 0, max_tokens: 100, stop: vec![], seed: 0 }), metadata: HashMap::new() };
        assert_eq!(proto_req.model_name, "test");
        assert_eq!(proto_req.prompt, "hello");
    }
    #[test]
    fn test_safe_numeric_conversion() {
        let large_value: usize = usize::MAX;
        let converted = i32::try_from(large_value).unwrap_or(i32::MAX);
        assert_eq!(converted, i32::MAX);
        let normal_value: usize = 100;
        let converted = i32::try_from(normal_value).unwrap_or(0);
        assert_eq!(converted, 100);
    }
    #[test]
    fn test_tool_call_arguments_parsing() {
        let valid_json = r#"{"key": "value"}"#;
        let parsed: serde_json::Value = serde_json::from_str(valid_json).unwrap();
        assert_eq!(parsed["key"], "value");
        let invalid_json = "not json";
        let parsed: serde_json::Value = serde_json::from_str(invalid_json).unwrap_or(serde_json::json!({}));
        assert_eq!(parsed, serde_json::json!({}));
    }
}
