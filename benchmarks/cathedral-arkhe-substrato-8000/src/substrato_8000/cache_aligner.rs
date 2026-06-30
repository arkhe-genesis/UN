
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::substrato_8000::mcp_headroom_server::LlmMessage;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum LlmProvider {
    Anthropic,
    OpenAI,
    Gemini,
    Rio35,
    Vllm,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderCacheConfig {
    pub provider: LlmProvider,
    pub stable_prefix_size: usize,
    pub alignment_strategy: AlignmentStrategy,
    pub native_prefix_caching: bool,
    pub message_format: MessageFormat,
    pub system_prompt_tokens: Vec<String>,
    pub context_separator: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlignmentStrategy {
    FixedPrefix,
    RotatingPrefix,
    Hierarchical,
    DomainSpecific { domain: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageFormat {
    AnthropicClaude,
    OpenAIChat,
    GeminiPro,
    CathedralNative,
    Raw,
}

pub struct CacheAligner {
    configs: HashMap<LlmProvider, ProviderCacheConfig>,
    prefix_cache: HashMap<String, Vec<String>>,
    stats: CacheAlignerStats,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CacheAlignerStats {
    pub total_alignments: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub avg_prefix_size: f64,
    pub provider_hits: HashMap<String, u64>,
}

impl CacheAligner {
    pub fn new() -> Self {
        Self {
            configs: HashMap::new(),
            prefix_cache: HashMap::new(),
            stats: CacheAlignerStats::default(),
        }
    }

    pub fn align_messages(
        &mut self,
        provider: &LlmProvider,
        messages: &[LlmMessage],
        context_id: &str,
    ) -> Result<Vec<LlmMessage>, CacheAlignError> {
        Ok(messages.to_vec())
    }

    pub fn format_for_provider(
        &self,
        provider: &LlmProvider,
        messages: &[LlmMessage],
    ) -> Result<String, CacheAlignError> {
        Ok(messages.iter().map(|m| m.content.clone()).collect::<Vec<_>>().join("\n"))
    }

    pub fn get_stats(&self) -> &CacheAlignerStats {
        &self.stats
    }

    pub fn get_cache_hit_rate(&self) -> f64 {
        0.0
    }

    pub fn clear_prefix_cache(&mut self) {
        self.prefix_cache.clear();
    }
}

#[derive(Debug, Error)]
pub enum CacheAlignError {
    #[error("Unsupported provider: {0}")]
    UnsupportedProvider(String),
    #[error("Format error: {0}")]
    FormatError(String),
    #[error("Prefix computation failed: {0}")]
    PrefixComputationFailed(String),
}
