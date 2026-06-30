use crate::error::RuntimeError;
use crate::types::{InferenceRequest, InferenceResponse, ChatMessage, Tensor, ModelConfig};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
#[async_trait]
pub trait ModelRuntime: Send + Sync {
    fn name(&self) -> &str;
    fn is_ready(&self) -> bool;
    fn config(&self) -> &ModelConfig;
    async fn x_infer(&self, request: InferenceRequest) -> Result<InferenceResponse, RuntimeError>;
    async fn x_infer_chat(&self, messages: Vec<ChatMessage>) -> Result<InferenceResponse, RuntimeError>;
    async fn x_embed(&self, _texts: Vec<String>) -> Result<Vec<Tensor>, RuntimeError> { Err(RuntimeError::NotSupported("Embedding not supported by this backend".into())) }
    async fn unload(&self) -> Result<(), RuntimeError>;
}
pub struct RuntimeRegistry { backends: Arc<RwLock<HashMap<String, Arc<dyn ModelRuntime>>>>, default: Arc<RwLock<Option<String>>> }
impl RuntimeRegistry {
    pub fn new() -> Self { Self { backends: Arc::new(RwLock::new(HashMap::new())), default: Arc::new(RwLock::new(None)) } }
    pub async fn register(&self, runtime: Arc<dyn ModelRuntime>) -> Result<(), RuntimeError> {
        let name = runtime.name().to_string();
        let mut map = self.backends.write().await;
        if map.contains_key(&name) { return Err(RuntimeError::Backend(format!("Backend '{}' already registered", name))); }
        map.insert(name.clone(), runtime);
        let mut default = self.default.write().await;
        if default.is_none() { *default = Some(name); }
        Ok(())
    }
    pub async fn get(&self, name: &str) -> Option<Arc<dyn ModelRuntime>> { let map = self.backends.read().await; map.get(name).cloned() }
    pub async fn default_runtime(&self) -> Option<Arc<dyn ModelRuntime>> { let default_name = self.default.read().await.clone(); if let Some(name) = default_name { self.get(&name).await } else { None } }
    pub async fn list(&self) -> Vec<String> { let map = self.backends.read().await; map.keys().cloned().collect() }
    pub async fn set_default(&self, name: &str) -> Result<(), RuntimeError> {
        let map = self.backends.read().await;
        if !map.contains_key(name) { return Err(RuntimeError::Backend(format!("Backend '{}' not found", name))); }
        let mut default = self.default.write().await;
        *default = Some(name.to_string());
        Ok(())
    }
}
impl Default for RuntimeRegistry { fn default() -> Self { Self::new() } }
