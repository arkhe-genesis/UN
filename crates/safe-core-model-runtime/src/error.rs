use thiserror::Error;
use safe_core_parallax_bridge::ParallaxError;
#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("Policy denied: {0}")] Policy(String),
    #[error("Backend error: {0}")] Backend(String),
    #[error("Model not found: {0}")] ModelNotFound(String),
    #[error("Not ready")] NotReady,
    #[error("Operation not supported: {0}")] NotSupported(String),
    #[error("Inference failed: {0}")] InferenceFailed(String),
    #[error("Load failed: {0}")] LoadFailed(String),
    #[error("Not found: {0}")] NotFound(String),
}
impl From<ParallaxError> for RuntimeError {
    fn from(err: ParallaxError) -> Self { RuntimeError::Backend(err.to_string()) }
}
