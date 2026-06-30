use serde::{Deserialize, Serialize};
use std::collections::HashMap;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceRequest { pub id: String, pub prompt: String, pub system_prompt: Option<String>, pub messages: Vec<ChatMessage>, pub params: SamplingParams, pub tools: Vec<ToolDefinition>, pub metadata: HashMap<String, String> }
impl InferenceRequest {
    pub fn simple(prompt: impl Into<String>) -> Self { Self { id: uuid::Uuid::new_v4().to_string(), prompt: prompt.into(), system_prompt: None, messages: Vec::new(), params: SamplingParams::default(), tools: Vec::new(), metadata: HashMap::new() } }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage { pub role: ChatRole, pub content: String }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChatRole { System, User, Assistant, Tool }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingParams { pub temperature: f32, pub top_p: f32, pub top_k: Option<usize>, pub max_tokens: usize, pub stop_sequences: Vec<String>, pub seed: Option<u64> }
impl Default for SamplingParams { fn default() -> Self { Self { temperature: 0.7, top_p: 0.9, top_k: None, max_tokens: 2048, stop_sequences: Vec::new(), seed: None } } }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition { pub name: String, pub description: String, pub parameters: serde_json::Value }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceResponse { pub id: String, pub content: String, pub tool_calls: Vec<ToolCall>, pub usage: TokenUsage, pub finish_reason: FinishReason, pub timestamp: chrono::DateTime<chrono::Utc>, pub metadata: HashMap<String, String> }
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum FinishReason { Stop, Length, ToolCall, Error }
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenUsage { pub prompt_tokens: u32, pub completion_tokens: u32, pub total_tokens: u32 }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall { pub id: String, pub name: String, pub arguments: String }
#[derive(Debug, Clone)]
pub struct Tensor { pub data: Vec<f32>, pub shape: Vec<usize> }
impl Tensor { pub fn new(data: Vec<f32>, shape: Vec<usize>) -> Self { Self { data, shape } } }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig { pub temperature: f32, pub top_p: f32, pub max_tokens: usize, pub device: Device, pub quantization: Option<String>, pub metadata: HashMap<String, String> }
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Device { Cpu, Gpu }
