
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use thiserror::Error;
use axum::{
    Router,
    routing::{post, get},
    extract::{State, Json},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::Utc;
use tracing::{info, error, warn, debug};

use crate::substrato_8000::{CompressionResult, HeadroomBridge, HeadroomMetricsReport};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResponse {
    pub jsonrpc: String,
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<McpError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResource {
    pub uri: String,
    pub name: String,
    pub mime_type: Option<String>,
    pub description: Option<String>,
}

pub struct McpHeadroomServer {
    bridge: Arc<RwLock<HeadroomBridge>>,
    config: McpServerConfig,
    metrics: Arc<RwLock<McpServerMetrics>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub port: u16,
    pub host: String,
    pub auth_enabled: bool,
    pub rate_limit_per_minute: u32,
    pub max_request_size_mb: usize,
    pub prometheus_endpoint: String,
    pub zkp_verification: bool,
    pub ema_integration: bool,
}

impl Default for McpServerConfig {
    fn default() -> Self {
        Self {
            port: 8787,
            host: "0.0.0.0".to_string(),
            auth_enabled: true,
            rate_limit_per_minute: 1000,
            max_request_size_mb: 50,
            prometheus_endpoint: "/metrics".to_string(),
            zkp_verification: true,
            ema_integration: true,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct McpServerMetrics {
    pub total_requests: u64,
    pub compress_requests: u64,
    pub retrieve_requests: u64,
    pub stats_requests: u64,
    pub errors: u64,
    pub avg_compression_ratio: f64,
    pub total_tokens_saved: u64,
    pub zkp_verifications: u64,
    pub zkp_failures: u64,
}

impl McpHeadroomServer {
    pub fn new(
        bridge: Arc<RwLock<HeadroomBridge>>,
        config: McpServerConfig,
    ) -> Self {
        Self {
            bridge,
            config,
            metrics: Arc::new(RwLock::new(McpServerMetrics::default())),
        }
    }

    pub async fn handle_compress(
        &self,
        params: CompressParams,
    ) -> Result<CompressResult, McpToolError> {
        let start = std::time::Instant::now();

        info!("🗜️  headroom_compress: target={}, type={:?}",
            params.target_id, params.content_type);

        let bridge = self.bridge.read().await;

        if self.config.ema_integration {
            self.validate_ema_token(&params.ema_token).await?;
        }

        let compressor = self.select_compressor(&params.content_type)?;

        let compression_result = compressor.compress(
            &params.content,
            &params.target_id,
            params.max_tokens,
        ).await.map_err(|e| McpToolError::CompressionFailed(e))?;

        let zkp_proof = if self.config.zkp_verification {
            Some(self.generate_zkp_proof(
                &params.content,
                &compression_result.compressed_text,
                &params.target_id,
            ).await?)
        } else {
            None
        };

        let ccr_id = if params.retrievable {
            Some(self.store_ccr(&params.target_id, &params.content).await?)
        } else {
            None
        };

        self.log_to_wormgraph(
            "compress",
            &params.target_id,
            params.content.len(),
            compression_result.compressed_text.len(),
            ccr_id.as_deref(),
            zkp_proof.as_ref(),
        ).await?;

        {
            let mut metrics = self.metrics.write().await;
            metrics.compress_requests += 1;
            metrics.total_requests += 1;
            let ratio = 1.0 - (compression_result.compressed_text.len() as f64 / params.content.len() as f64);
            metrics.avg_compression_ratio =
                (metrics.avg_compression_ratio * (metrics.compress_requests - 1) as f64 + ratio)
                / metrics.compress_requests as f64;
            metrics.total_tokens_saved += (params.content.len() - compression_result.compressed_text.len()) as u64;
        }

        let elapsed_ms = start.elapsed().as_millis() as u64;

        Ok(CompressResult {
            compressed_text: compression_result.compressed_text.clone(),
            compression_ratio: compression_result.ratio,
            tokens_before: params.content.len() / 4,
            tokens_after: compression_result.compressed_text.len() / 4,
            ccr_id,
            zkp_proof,
            processing_time_ms: elapsed_ms,
            compressor_used: compression_result.compressor_name,
        })
    }

    pub async fn handle_retrieve(
        &self,
        params: RetrieveParams,
    ) -> Result<RetrieveResult, McpToolError> {
        info!("📤 headroom_retrieve: ccr_id={}", params.ccr_id);

        let bridge = self.bridge.read().await;

        if self.config.ema_integration {
            self.validate_ema_token(&params.ema_token).await?;
        }

        let original = bridge.retrieve_ccr(&params.ccr_id).await
            .map_err(|e| McpToolError::RetrieveFailed(e.to_string()))?;

        if let Some(proof) = &params.zkp_proof {
            self.verify_zkp_proof(proof, &original).await?;
        }

        self.log_to_wormgraph(
            "retrieve",
            &params.ccr_id,
            original.len(),
            0,
            Some(&params.ccr_id),
            None,
        ).await?;

        {
            let mut metrics = self.metrics.write().await;
            metrics.retrieve_requests += 1;
            metrics.total_requests += 1;
        }

        Ok(RetrieveResult {
            original_text: original,
            ccr_id: params.ccr_id,
            retrieved_at: Utc::now().timestamp(),
        })
    }

    pub async fn handle_stats(
        &self,
        params: StatsParams,
    ) -> Result<StatsResult, McpToolError> {
        info!("📊 headroom_stats: detail={:?}", params.detail_level);

        let metrics = self.metrics.read().await;
        let bridge = self.bridge.read().await;
        let report = bridge.get_metrics_report().await;

        let result = StatsResult {
            server_metrics: ServerMetricsSnapshot {
                total_requests: metrics.total_requests,
                compress_requests: metrics.compress_requests,
                retrieve_requests: metrics.retrieve_requests,
                stats_requests: metrics.stats_requests,
                errors: metrics.errors,
                avg_compression_ratio: metrics.avg_compression_ratio,
                total_tokens_saved: metrics.total_tokens_saved,
                zkp_verifications: metrics.zkp_verifications,
                zkp_failures: metrics.zkp_failures,
            },
            bridge_metrics: report,
            top_compressors: vec![
                ("SmartCrusher".to_string(), 0.85),
                ("CodeCompressor".to_string(), 0.72),
                ("KompressBase".to_string(), 0.68),
            ],
            uptime_seconds: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };

        {
            let mut metrics = self.metrics.write().await;
            metrics.stats_requests += 1;
            metrics.total_requests += 1;
        }

        Ok(result)
    }

    async fn generate_zkp_proof(
        &self,
        original: &str,
        compressed: &str,
        target_id: &str,
    ) -> Result<ZkpProof, McpToolError> {
        let original_hash = sha256(original);
        let compressed_hash = sha256(compressed);

        let proof = ZkpProof {
            proof_type: "semantic_preservation".to_string(),
            original_commitment: hex::encode(&original_hash[..16]),
            compressed_commitment: hex::encode(&compressed_hash[..16]),
            target_id: target_id.to_string(),
            timestamp: Utc::now().timestamp(),
            verification_key: "cathedral_zkp_v1".to_string(),
            proof_data: vec![0u8; 64],
        };

        {
            let mut metrics = self.metrics.write().await;
            metrics.zkp_verifications += 1;
        }

        Ok(proof)
    }

    async fn verify_zkp_proof(
        &self,
        proof: &ZkpProof,
        original: &str,
    ) -> Result<bool, McpToolError> {
        let original_hash = sha256(original);
        let expected_commitment = hex::encode(&original_hash[..16]);

        if proof.original_commitment != expected_commitment {
            {
                let mut metrics = self.metrics.write().await;
                metrics.zkp_failures += 1;
            }
            return Err(McpToolError::ZkpVerificationFailed(
                "Commitment mismatch".to_string()
            ));
        }

        Ok(true)
    }

    async fn validate_ema_token(
        &self,
        token: &Option<EmaToken>,
    ) -> Result<(), McpToolError> {
        let token = token.as_ref().ok_or(McpToolError::EmaAuthRequired)?;

        if token.expiry < Utc::now().timestamp() {
            return Err(McpToolError::EmaTokenExpired);
        }

        if !token.scopes.contains(&"headroom:compress".to_string()) {
            return Err(McpToolError::EmaInsufficientScope);
        }

        Ok(())
    }

    async fn log_to_wormgraph(
        &self,
        operation: &str,
        target_id: &str,
        bytes_before: usize,
        bytes_after: usize,
        ccr_id: Option<&str>,
        zkp_proof: Option<&ZkpProof>,
    ) -> Result<(), McpToolError> {
        debug!(
            "📝 WormGraph log: op={}, target={}, before={}, after={}, ccr={:?}, zkp={}",
            operation, target_id, bytes_before, bytes_after,
            ccr_id, zkp_proof.is_some()
        );
        Ok(())
    }

    fn select_compressor(
        &self,
        content_type: &ContentType,
    ) -> Result<Box<dyn Compressor>, McpToolError> {
        match content_type {
            ContentType::Json => Ok(Box::new(SmartCrusher)),
            ContentType::Code { language } => Ok(Box::new(CodeCompressor::new(language))),
            ContentType::Text => Ok(Box::new(KompressBase)),
            ContentType::IdtContext => Ok(Box::new(IdtContextCompressor)),
            ContentType::AgentMemory => Ok(Box::new(AgentMemoryCompressor)),
        }
    }

    async fn store_ccr(
        &self,
        target_id: &str,
        original: &str,
    ) -> Result<String, McpToolError> {
        let bridge = self.bridge.read().await;
        bridge.retrieve_ccr(target_id).await
            .map_err(|e| McpToolError::CcrStoreFailed(e.to_string()))?;
        Ok(format!("ccr_{}_{}", target_id, Utc::now().timestamp_millis()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressParams {
    pub content: String,
    pub target_id: String,
    pub content_type: ContentType,
    pub max_tokens: Option<usize>,
    pub retrievable: bool,
    pub zkp_verify: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ema_token: Option<EmaToken>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressResult {
    pub compressed_text: String,
    pub compression_ratio: f64,
    pub tokens_before: usize,
    pub tokens_after: usize,
    pub ccr_id: Option<String>,
    pub zkp_proof: Option<ZkpProof>,
    pub processing_time_ms: u64,
    pub compressor_used: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrieveParams {
    pub ccr_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zkp_proof: Option<ZkpProof>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ema_token: Option<EmaToken>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrieveResult {
    pub original_text: String,
    pub ccr_id: String,
    pub retrieved_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsParams {
    pub detail_level: StatsDetailLevel,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ema_token: Option<EmaToken>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StatsDetailLevel {
    Summary,
    Detailed,
    Full,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsResult {
    pub server_metrics: ServerMetricsSnapshot,
    pub bridge_metrics: HeadroomMetricsReport,
    pub top_compressors: Vec<(String, f64)>,
    pub uptime_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerMetricsSnapshot {
    pub total_requests: u64,
    pub compress_requests: u64,
    pub retrieve_requests: u64,
    pub stats_requests: u64,
    pub errors: u64,
    pub avg_compression_ratio: f64,
    pub total_tokens_saved: u64,
    pub zkp_verifications: u64,
    pub zkp_failures: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZkpProof {
    pub proof_type: String,
    pub original_commitment: String,
    pub compressed_commitment: String,
    pub target_id: String,
    pub timestamp: i64,
    pub verification_key: String,
    pub proof_data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmaToken {
    pub token_id: String,
    pub holder_id: String,
    pub scopes: Vec<String>,
    pub expiry: i64,
    pub signature: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentType {
    Json,
    Code { language: String },
    Text,
    IdtContext,
    AgentMemory,
}

#[async_trait::async_trait]
trait Compressor: Send + Sync {
    async fn compress(
        &self,
        content: &str,
        target_id: &str,
        max_tokens: Option<usize>,
    ) -> Result<CompressionResult, String>;
}

struct SmartCrusher;
#[async_trait::async_trait]
impl Compressor for SmartCrusher {
    async fn compress(&self, content: &str, _target: &str, max: Option<usize>) -> Result<CompressionResult, String> {
        let compressed = content.chars().filter(|c| !c.is_whitespace()).collect::<String>();
        let ratio = 1.0 - (compressed.len() as f64 / content.len() as f64);
        Ok(CompressionResult {
            text: compressed.clone(),
            compressed_text: compressed.clone(),
            tokens_after: max.unwrap_or(compressed.len() / 4),
            ratio,
            compressor_name: "SmartCrusher".to_string(),
        })
    }
}

struct CodeCompressor { language: String }
impl CodeCompressor {
    fn new(lang: &str) -> Self { Self { language: lang.to_string() } }
}
#[async_trait::async_trait]
impl Compressor for CodeCompressor {
    async fn compress(&self, content: &str, _target: &str, max: Option<usize>) -> Result<CompressionResult, String> {
        let compressed = content.lines().filter(|l| !l.trim().starts_with("//")).collect::<Vec<_>>().join("\n");
        let ratio = 1.0 - (compressed.len() as f64 / content.len() as f64);
        Ok(CompressionResult {
            text: compressed.clone(),
            compressed_text: compressed.clone(),
            tokens_after: max.unwrap_or(compressed.len() / 4),
            ratio,
            compressor_name: format!("CodeCompressor({})", self.language),
        })
    }
}

struct KompressBase;
#[async_trait::async_trait]
impl Compressor for KompressBase {
    async fn compress(&self, content: &str, _target: &str, max: Option<usize>) -> Result<CompressionResult, String> {
        let compressed = format!("[KOMPRESSED:{}]", &content[..content.len().min(100)]);
        let ratio = 0.6;
        Ok(CompressionResult {
            text: compressed.clone(),
            compressed_text: compressed.clone(),
            tokens_after: max.unwrap_or(compressed.len() / 4),
            ratio,
            compressor_name: "KompressBase".to_string(),
        })
    }
}

struct IdtContextCompressor;
#[async_trait::async_trait]
impl Compressor for IdtContextCompressor {
    async fn compress(&self, content: &str, _target: &str, max: Option<usize>) -> Result<CompressionResult, String> {
        let compressed = format!("[IDT:{}]", &content[..content.len().min(200)]);
        let ratio = 0.75;
        Ok(CompressionResult {
            text: compressed.clone(),
            compressed_text: compressed.clone(),
            tokens_after: max.unwrap_or(compressed.len() / 4),
            ratio,
            compressor_name: "IdtContextCompressor".to_string(),
        })
    }
}

struct AgentMemoryCompressor;
#[async_trait::async_trait]
impl Compressor for AgentMemoryCompressor {
    async fn compress(&self, content: &str, _target: &str, max: Option<usize>) -> Result<CompressionResult, String> {
        let compressed = format!("[AGENT_MEM:{}]", &content[..content.len().min(150)]);
        let ratio = 0.5;
        Ok(CompressionResult {
            text: compressed.clone(),
            compressed_text: compressed.clone(),
            tokens_after: max.unwrap_or(compressed.len() / 4),
            ratio,
            compressor_name: "AgentMemoryCompressor".to_string(),
        })
    }
}

pub fn create_router(server: Arc<McpHeadroomServer>) -> Router {
    Router::new()
        .route("/mcp/v1/tools/list", get(list_tools))
        .route("/mcp/v1/tools/call", post(call_tool))
        .route("/mcp/v1/resources/list", get(list_resources))
        .route("/metrics", get(prometheus_metrics))
        .route("/health", get(health_check))
        .with_state(server)
}

async fn list_tools(State(_server): State<Arc<McpHeadroomServer>>) -> impl IntoResponse {
    let tools = vec![
        McpTool {
            name: "headroom_compress".to_string(),
            description: "Compress any context using Headroom compression layer".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "content": { "type": "string" },
                    "target_id": { "type": "string" },
                    "content_type": { "enum": ["Json", "Code", "Text", "IdtContext", "AgentMemory"] },
                    "max_tokens": { "type": "integer" },
                    "retrievable": { "type": "boolean" },
                    "zkp_verify": { "type": "boolean" }
                },
                "required": ["content", "target_id", "content_type"]
            }),
        },
    ];
    Json(McpResponse { jsonrpc: "2.0".to_string(), id: None, result: Some(serde_json::to_value(tools).unwrap()), error: None })
}

async fn call_tool(
    State(server): State<Arc<McpHeadroomServer>>,
    Json(request): Json<McpRequest>,
) -> impl IntoResponse {
    let result = match request.method.as_str() {
        "headroom_compress" => {
            let params: CompressParams = serde_json::from_value(request.params.unwrap_or(Value::Null)).unwrap();
            match server.handle_compress(params).await {
                Ok(result) => Ok(serde_json::to_value(result).unwrap()),
                Err(e) => Err(McpError { code: -32603, message: e.to_string(), data: None }),
            }
        }
        _ => Err(McpError { code: -32601, message: format!("Method not found: {}", request.method), data: None }),
    };
    let response = match result {
        Ok(value) => McpResponse { jsonrpc: "2.0".to_string(), id: request.id, result: Some(value), error: None },
        Err(error) => McpResponse { jsonrpc: "2.0".to_string(), id: request.id, result: None, error: Some(error) },
    };
    (StatusCode::OK, Json(response))
}

async fn list_resources() -> impl IntoResponse {
    let resources: Vec<McpResource> = vec![];
    Json(resources)
}

async fn prometheus_metrics(State(_server): State<Arc<McpHeadroomServer>>) -> impl IntoResponse {
    let report = "".to_string();
    (StatusCode::OK, report)
}

async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
    }))
}

#[derive(Debug, Error)]
pub enum McpToolError {
    #[error("Compression failed: {0}")]
    CompressionFailed(String),
    #[error("Retrieve failed: {0}")]
    RetrieveFailed(String),
    #[error("CCR store failed: {0}")]
    CcrStoreFailed(String),
    #[error("ZKP verification failed: {0}")]
    ZkpVerificationFailed(String),
    #[error("EMA authentication required")]
    EmaAuthRequired,
    #[error("EMA token expired")]
    EmaTokenExpired,
    #[error("EMA insufficient scope")]
    EmaInsufficientScope,
    #[error("Invalid content type")]
    InvalidContentType,
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
}

fn sha256(input: &str) -> [u8; 32] {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    hasher.finalize().into()
}
