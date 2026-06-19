
use std::sync::Arc;
use tokio::sync::RwLock;
use axum::{
    Router,
    routing::{post, get, any},
    extract::{State, Request},
    http::{StatusCode, HeaderMap, Uri},
    response::{IntoResponse, Response},
    middleware::{self, Next},
};
use serde::{Serialize, Deserialize};
use serde_json::Value;
use chrono::Utc;
use thiserror::Error;
use tracing::{info, error, debug, warn};

use crate::substrato_8000::HeadroomBridge;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub listen_port: u16,
    pub listen_host: String,
    pub upstream_url: String,
    pub provider_type: LlmProviderType,
    pub compress_requests: bool,
    pub compress_responses: bool,
    pub compression_threshold: usize,
    pub ccr_enabled: bool,
    pub rate_limit_per_second: u32,
    pub upstream_auth_header: Option<String>,
    pub upstream_timeout_ms: u64,
    pub log_level: String,
    pub metrics_enabled: bool,
    pub metrics_port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LlmProviderType {
    Anthropic,
    OpenAI,
    Gemini,
    AzureOpenAI,
    Vllm,
    Ollama,
    Custom { api_path: String },
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            listen_port: 8787,
            listen_host: "0.0.0.0".to_string(),
            upstream_url: "http://localhost:8000".to_string(),
            provider_type: LlmProviderType::Vllm,
            compress_requests: true,
            compress_responses: true,
            compression_threshold: 4000,
            ccr_enabled: true,
            rate_limit_per_second: 100,
            upstream_auth_header: None,
            upstream_timeout_ms: 30000,
            log_level: "info".to_string(),
            metrics_enabled: true,
            metrics_port: 8788,
        }
    }
}

pub struct HeadroomProxy {
    config: ProxyConfig,
    bridge: Arc<RwLock<HeadroomBridge>>,
}

impl HeadroomProxy {
    pub fn new(
        config: ProxyConfig,
        bridge: Arc<RwLock<HeadroomBridge>>,
    ) -> Self {
        Self {
            config,
            bridge,
        }
    }

    pub fn create_router(self: Arc<Self>) -> Router {
        Router::new()
            .route("/health", get(health_handler))
            .with_state(self)
    }
}

async fn health_handler() -> impl IntoResponse {
    axum::Json(serde_json::json!({
        "status": "healthy",
    }))
}
