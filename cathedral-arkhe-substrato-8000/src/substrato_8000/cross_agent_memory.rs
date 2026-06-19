
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use chrono::Utc;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedMemoryEntry {
    pub entry_id: String,
    pub agent_id: String,
    pub task_id: String,
    pub memory_type: SharedMemoryType,
    pub content: String,
    pub compressed_content: Option<String>,
    pub ccr_id: Option<String>,
    pub embedding: Option<Vec<f32>>,
    pub metadata: MemoryMetadata,
    pub created_at: i64,
    pub last_accessed: i64,
    pub access_count: u64,
    pub ttl_seconds: u64,
    pub is_deduplicated: bool,
    pub duplicate_of: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SharedMemoryType {
    ConversationContext,
    ToolResult,
    IdtBranchState,
    EpisodicMemory,
    SemanticMemory,
    AgentCheckpoint,
    ConsensusMessage,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMetadata {
    pub priority: f64,
    pub relevance_score: f64,
    pub source_agent: String,
    pub target_agents: Vec<String>,
    pub tags: Vec<String>,
    pub compression_ratio: f64,
    pub original_size_bytes: usize,
    pub compressed_size_bytes: usize,
}

pub struct CrossAgentMemoryStore {
    store: Arc<RwLock<HashMap<String, SharedMemoryEntry>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossAgentMemoryConfig {
    pub max_entries: usize,
    pub default_ttl: u64,
    pub dedup_similarity_threshold: f64,
    pub auto_dedup: bool,
    pub semantic_indexing: bool,
    pub embedding_dimensions: usize,
    pub relevance_decay_factor: f64,
}

impl Default for CrossAgentMemoryConfig {
    fn default() -> Self {
        Self {
            max_entries: 100_000,
            default_ttl: 3600 * 24,
            dedup_similarity_threshold: 0.92,
            auto_dedup: true,
            semantic_indexing: true,
            embedding_dimensions: 384,
            relevance_decay_factor: 0.95,
        }
    }
}

impl CrossAgentMemoryStore {
    pub fn new(config: CrossAgentMemoryConfig) -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn store(
        &self,
        entry: SharedMemoryEntry,
    ) -> Result<String, MemoryStoreError> {
        let entry_id = entry.entry_id.clone();
        self.store.write().await.insert(entry_id.clone(), entry);
        Ok(entry_id)
    }

    pub async fn get(&self, entry_id: &str) -> Result<SharedMemoryEntry, MemoryStoreError> {
        let store = self.store.read().await;
        store.get(entry_id).cloned().ok_or_else(|| MemoryStoreError::EntryNotFound(entry_id.to_string()))
    }
}

#[derive(Debug, Error)]
pub enum MemoryStoreError {
    #[error("Entry not found: {0}")]
    EntryNotFound(String),
    #[error("Agent not found: {0}")]
    AgentNotFound(String),
    #[error("Task not found: {0}")]
    TaskNotFound(String),
    #[error("Store full")]
    StoreFull,
    #[error("Embedding computation failed: {0}")]
    EmbeddingFailed(String),
    #[error("Deduplication failed: {0}")]
    DeduplicationFailed(String),
}
