
pub mod mcp_headroom_server;
pub mod cache_aligner;
pub mod cross_agent_memory;
pub mod proxy_mode;
pub mod ema_integration;

use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use thiserror::Error;

use crate::substrato_8000::mcp_headroom_server::LlmMessage;

/// ============================================================
/// 1. CONFIGURAÇÃO DO SUBSTRATO 8000
/// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeadroomBridgeConfig {
    /// Modo de operação: Library, Proxy, MCP
    pub mode: HeadroomMode,
    /// Compressão ativada por padrão
    pub compression_enabled: bool,
    /// Threshold de compressão (ratio mínimo para comprimir)
    pub compression_threshold: f64,
    /// Máximo de tokens antes de forçar compressão
    pub max_tokens_before_compress: usize,
    /// CCR (reversible compression) ativado
    pub ccr_enabled: bool,
    /// TTL do CCR cache (segundos)
    pub ccr_ttl_seconds: u64,
    /// Cross-agent memory ativado
    pub cross_agent_memory: bool,
    /// CacheAligner para KV cache optimization
    pub cache_aligner_enabled: bool,
    /// Métricas exportadas para Prometheus
    pub metrics_export: bool,
    /// ZKP proofs para compressão semântica
    pub zkp_verification: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HeadroomMode {
    Library,
    Proxy { port: u16 },
    McpServer,
    Hybrid,
}

impl Default for HeadroomBridgeConfig {
    fn default() -> Self {
        Self {
            mode: HeadroomMode::Library,
            compression_enabled: true,
            compression_threshold: 0.3,
            max_tokens_before_compress: 4000,
            ccr_enabled: true,
            ccr_ttl_seconds: 3600,
            cross_agent_memory: true,
            cache_aligner_enabled: true,
            metrics_export: true,
            zkp_verification: false,
        }
    }
}

/// ============================================================
/// 2. HEADROOM BRIDGE — Interface Principal
/// ============================================================

pub struct HeadroomBridge {
    pub config: HeadroomBridgeConfig,
    pub compressor: Arc<HeadroomCompressor>,
    pub adapter: Arc<CathedralHeadroomAdapter>,
    pub metrics: Arc<RwLock<HeadroomMetricsCollector>>,
    pub ccr_cache: Arc<CcrCache>,
    pub cross_agent_store: Arc<cross_agent_memory::CrossAgentMemoryStore>,
}

impl HeadroomBridge {
    pub fn new(
        config: HeadroomBridgeConfig,
        compressor: Arc<HeadroomCompressor>,
        adapter: Arc<CathedralHeadroomAdapter>,
        ccr_cache: Arc<CcrCache>,
        cross_agent_store: Arc<cross_agent_memory::CrossAgentMemoryStore>,
    ) -> Self {
        Self {
            config,
            compressor,
            adapter,
            metrics: Arc::new(RwLock::new(HeadroomMetricsCollector::new())),
            ccr_cache,
            cross_agent_store,
        }
    }

    pub async fn compress_idt_context(
        &self,
        session_id: &str,
        branches: &[crate::substrato_9000::immersion_driven_thinking::ImmersionBranch],
        _anchor_objective: &str,
    ) -> Result<CompressedIdtContext, HeadroomBridgeError> {
        if !self.config.compression_enabled {
            return Ok(CompressedIdtContext {
                text: serde_json::to_string(branches).unwrap_or_default(),
                compression_ratio: 0.0,
                tokens_saved: 0,
                ccr_id: None,
                was_compressed: false,
            });
        }

        let start_time = std::time::Instant::now();
        let raw_json = serde_json::to_string(branches).unwrap_or_default();
        let raw_tokens = raw_json.len() / 4;

        if raw_tokens < self.config.max_tokens_before_compress {
            return Ok(CompressedIdtContext {
                text: raw_json,
                compression_ratio: 0.0,
                tokens_saved: 0,
                ccr_id: None,
                was_compressed: false,
            });
        }

        let compressed = self.compressor.compress(
            &raw_json,
            CompressionTarget::IdtContext {
                session_id: session_id.to_string(),
                branch_count: branches.len(),
            }
        ).await.map_err(|e| HeadroomBridgeError::CompressionFailed(e))?;

        let ccr_id = if self.config.ccr_enabled {
            Some(self.ccr_cache.store(
                session_id,
                &raw_json,
                self.config.ccr_ttl_seconds,
            ).await.unwrap_or_default())
        } else {
            None
        };

        Ok(CompressedIdtContext {
            text: compressed.text,
            compression_ratio: 1.0 - (compressed.tokens_after as f64 / raw_tokens as f64),
            tokens_saved: raw_tokens - compressed.tokens_after,
            ccr_id,
            was_compressed: true,
        })
    }

    pub async fn retrieve_ccr(&self, ccr_id: &str) -> Result<String, HeadroomBridgeError> {
        if !self.config.ccr_enabled {
            return Err(HeadroomBridgeError::CcrDisabled);
        }
        let original = self.ccr_cache.retrieve(ccr_id).await
            .map_err(|e| HeadroomBridgeError::CcrRetrieveFailed(e))?;
        Ok(original)
    }

    pub async fn store_cross_agent_context(
        &self,
        agent_id: &str,
        task_id: &str,
        context: &CompressedIdtContext,
    ) -> Result<String, HeadroomBridgeError> {
        if !self.config.cross_agent_memory {
            return Err(HeadroomBridgeError::CrossAgentMemoryDisabled);
        }

        let shared_context = cross_agent_memory::SharedMemoryEntry {
            entry_id: format!("{}_{}", agent_id, task_id),
            agent_id: agent_id.to_string(),
            task_id: task_id.to_string(),
            memory_type: cross_agent_memory::SharedMemoryType::ConversationContext,
            content: context.text.clone(),
            compressed_content: None,
            ccr_id: context.ccr_id.clone(),
            embedding: None,
            metadata: cross_agent_memory::MemoryMetadata {
                priority: 0.0,
                relevance_score: 0.0,
                source_agent: "".to_string(),
                target_agents: vec![],
                tags: vec![],
                compression_ratio: 0.0,
                original_size_bytes: 0,
                compressed_size_bytes: 0,
            },
            created_at: 0,
            last_accessed: 0,
            access_count: 0,
            ttl_seconds: 0,
            is_deduplicated: false,
            duplicate_of: None,
        };

        let context_id = self.cross_agent_store.store(shared_context).await
            .map_err(|e| HeadroomBridgeError::CrossAgentRetrieveFailed(e.to_string()))?;

        Ok(context_id)
    }

    pub async fn get_cross_agent_context(
        &self,
        context_id: &str,
    ) -> Result<cross_agent_memory::SharedMemoryEntry, HeadroomBridgeError> {
        let context = self.cross_agent_store.get(context_id).await
            .map_err(|e| HeadroomBridgeError::CrossAgentRetrieveFailed(e.to_string()))?;
        Ok(context)
    }

    pub async fn get_metrics_report(&self) -> HeadroomMetricsReport {
        let metrics = self.metrics.read().await;
        metrics.generate_report()
    }
}

/// ============================================================
/// 3. TIPOS DE DOMÍNIO
/// ============================================================

#[derive(Debug, Clone)]
pub struct CompressedIdtContext {
    pub text: String,
    pub compression_ratio: f64,
    pub tokens_saved: usize,
    pub ccr_id: Option<String>,
    pub was_compressed: bool,
}

#[derive(Debug, Clone)]
pub struct CompressionResult {
    pub text: String,
    pub compressed_text: String,
    pub tokens_after: usize,
    pub ratio: f64,
    pub compressor_name: String,
}

#[derive(Debug, Clone)]
pub enum CompressionTarget {
    IdtContext { session_id: String, branch_count: usize },
    AgentContext { model: String, max_tokens: usize },
    MemoryIndex,
    ToolOutput { tool_name: String },
    RagChunk { source: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeadroomMetricsReport {
    pub total_compressions: u64,
    pub total_tokens_saved: u64,
    pub avg_compression_ratio: f64,
    pub ccr_retrieve_count: u64,
    pub cross_agent_stores: u64,
    pub cache_hit_rate: f64,
    pub top_compressors: Vec<(String, f64)>,
}

/// ============================================================
/// 4. ERROS
/// ============================================================

#[derive(Debug, Error)]
pub enum HeadroomBridgeError {
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Compression failed: {0}")]
    CompressionFailed(String),
    #[error("CCR is disabled")]
    CcrDisabled,
    #[error("CCR retrieve failed: {0}")]
    CcrRetrieveFailed(String),
    #[error("Cross-agent memory is disabled")]
    CrossAgentMemoryDisabled,
    #[error("Cross-agent retrieve failed: {0}")]
    CrossAgentRetrieveFailed(String),
    #[error("Cache align failed: {0}")]
    CacheAlignFailed(String),
    #[error("Adapter error: {0}")]
    AdapterError(String),
}

#[derive(Debug, Clone)] pub struct HeadroomCompressor;
impl HeadroomCompressor {
    pub async fn compress(&self, _input: &str, _target: CompressionTarget) -> Result<CompressionResult, String> {
        Ok(CompressionResult { text: _input.to_string(), compressed_text: _input.to_string(), tokens_after: _input.len() / 4, ratio: 0.0, compressor_name: "Mock".to_string() })
    }
    pub async fn align_cache_prefixes(&self, messages: &[LlmMessage]) -> Result<Vec<LlmMessage>, String> {
        Ok(messages.to_vec())
    }
}

#[derive(Debug, Clone)] pub struct CathedralHeadroomAdapter;
impl CathedralHeadroomAdapter {
    pub async fn log_compression_event(
        &self, _session: &str, _type: &str, _before: usize, _after: usize, _ccr: Option<&str>
    ) -> Result<(), HeadroomBridgeError> {
        Ok(())
    }
}

#[derive(Debug, Clone)] pub struct CcrCache;
impl CcrCache {
    pub async fn store(&self, _key: &str, _value: &str, _ttl: u64) -> Result<String, String> {
        Ok(format!("ccr_{}", _key))
    }
    pub async fn retrieve(&self, _id: &str) -> Result<String, String> {
        Ok("original".to_string())
    }
}

#[derive(Debug, Clone, Default)] pub struct HeadroomMetricsCollector;
impl HeadroomMetricsCollector {
    pub fn new() -> Self { Self }
    pub fn record_compression(&mut self, _type: &str, _before: usize, _after: usize, _latency_ms: u64) {}
    pub fn record_ccr_retrieve(&mut self, _id: &str) {}
    pub fn generate_report(&self) -> HeadroomMetricsReport {
        HeadroomMetricsReport {
            total_compressions: 0,
            total_tokens_saved: 0,
            avg_compression_ratio: 0.0,
            ccr_retrieve_count: 0,
            cross_agent_stores: 0,
            cache_hit_rate: 0.0,
            top_compressors: vec![],
        }
    }
    pub fn to_prometheus_format(&self) -> String {
        "# headroom metrics".to_string()
    }
}
