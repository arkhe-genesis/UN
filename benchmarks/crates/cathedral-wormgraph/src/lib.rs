use sha3::Digest;


pub mod backends;

use backends::{JsonWormGraph, SqliteWormGraph, PostgresWormGraph};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Tiers of risk for an Improvement Proposal
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl RiskLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            RiskLevel::Low => "Low",
            RiskLevel::Medium => "Medium",
            RiskLevel::High => "High",
            RiskLevel::Critical => "Critical",
        }
    }
}

/// Validation status
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum ValidationStatus {
    Pending,
    Validating,
    Approved,
    Rejected,
    Implemented,
    Reverted,
}

impl ValidationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ValidationStatus::Pending => "Pending",
            ValidationStatus::Validating => "Validating",
            ValidationStatus::Approved => "Approved",
            ValidationStatus::Rejected => "Rejected",
            ValidationStatus::Implemented => "Implemented",
            ValidationStatus::Reverted => "Reverted",
        }
    }
}

/// A proposal for system improvement
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ImprovementProposal {
    pub id: String,
    pub title: String,
    pub description: String,
    pub code_diff: Option<String>,
    pub config_change: Option<String>,
    pub expected_impact: String,
    pub risk_level: RiskLevel,
    pub thinking_trace: Option<String>,
    pub validation_status: ValidationStatus,
    pub author_did: String,
    pub signature: Vec<u8>,
    pub created_at: DateTime<Utc>,
    pub validated_at: Option<DateTime<Utc>>,
    pub implemented_at: Option<DateTime<Utc>>,
    pub metrics_before: Option<String>,
    pub metrics_after: Option<String>,
}

impl ImprovementProposal {
    pub fn new(title: String, description: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            title,
            description,
            code_diff: None,
            config_change: None,
            expected_impact: String::new(),
            risk_level: RiskLevel::Low,
            thinking_trace: None,
            validation_status: ValidationStatus::Pending,
            author_did: String::new(),
            signature: Vec::new(),
            created_at: Utc::now(),
            validated_at: None,
            implemented_at: None,
            metrics_before: None,
            metrics_after: None,
        }
    }

    pub fn with_risk(mut self, risk_level: RiskLevel) -> Self {
        self.risk_level = risk_level;
        self
    }

    pub fn with_code_diff(mut self, diff: String) -> Self {
        self.code_diff = Some(diff);
        self
    }

    pub fn with_impact(mut self, impact: String) -> Self {
        self.expected_impact = impact;
        self
    }

    pub fn approve(&mut self) {
        self.validation_status = ValidationStatus::Approved;
        self.validated_at = Some(Utc::now());
    }
}

/// Filter for listing proposals
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProposalFilter {
    pub risk_level: Option<RiskLevel>,
    pub status: Option<ValidationStatus>,
    pub author_did: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

/// Ledger Entry for WormGraph
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct LedgerEntry {
    pub id: String,
    pub version: i64,
    pub decision_type: String,
    pub before_state: Option<String>,
    pub after_state: Option<String>,
    pub rationale: Option<String>,
    pub timestamp: i64,
    pub agent_id: String,
    pub entry_hash: Vec<u8>,
    pub parent_hash: Option<Vec<u8>>,
    pub signature: Option<Vec<u8>>,
    pub public_key: Option<Vec<u8>>,
    pub nostr_event_id: Option<String>,
    pub tree_id: Option<String>,
    pub parent_event_id: Option<String>,
    pub zk_proof_hash: Option<Vec<u8>>,
}

/// Filter for listing memories
#[derive(Clone, Debug, Default)]
pub struct MemoryFilter {
    pub agent_id: Option<String>,
    pub decision_type: Option<String>,
    pub since_timestamp: Option<i64>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

/// Execution Receipt
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ExecutionReceipt {
    pub id: String,
    pub merkle_root: String,
    pub timestamp: i64,
}

#[derive(Debug, Error)]
pub enum WormGraphError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Agent not found")]
    NotFound,
    #[error("Forbidden: You are not authorized to perform this action")]
    Forbidden,
}

pub type Result<T> = std::result::Result<T, WormGraphError>;

#[async_trait::async_trait]
pub trait WormGraphBackend: Send + Sync {
    async fn append_entry(&self, entry: LedgerEntry) -> Result<()>;
    async fn get_entries(&self, limit: Option<usize>) -> Result<Vec<LedgerEntry>>;
    async fn list_memories(&self, filter: MemoryFilter) -> Result<Vec<LedgerEntry>>;
    async fn save_proposal(&self, proposal: &ImprovementProposal) -> Result<()>;
    async fn list_proposals(&self, filter: ProposalFilter) -> Result<Vec<ImprovementProposal>>;
    async fn delete_proposal(&self, id: &str, author_did: &str, signature: &[u8]) -> Result<()>;
    async fn get_proposal(&self, id: &str) -> Result<Option<ImprovementProposal>>;
    async fn ping(&self) -> Result<()>;
}

pub enum WormGraphBackendType {
    Json(JsonWormGraph),
    Sqlite(SqliteWormGraph),
    Postgres(PostgresWormGraph),
}

pub struct WormGraphClient {
    backend: Box<dyn WormGraphBackend>,
}

impl WormGraphClient {
    pub fn new(backend: impl WormGraphBackend + 'static) -> Self {
        Self {
            backend: Box::new(backend),
        }
    }

    pub async fn append_entry(&self, entry: LedgerEntry) -> Result<()> {
        self.backend.append_entry(entry).await
    }

    pub async fn get_entries(&self, limit: Option<usize>) -> Result<Vec<LedgerEntry>> {
        self.backend.get_entries(limit).await
    }

    pub async fn list_memories(&self, filter: MemoryFilter) -> Result<Vec<LedgerEntry>> {
        self.backend.list_memories(filter).await
    }

    pub async fn get_memories(&self, did: &str, limit: usize) -> Result<Vec<LedgerEntry>> {
        self.list_memories(MemoryFilter {
            agent_id: Some(did.to_string()),
            limit: Some(limit),
            ..Default::default()
        }).await
    }

    pub async fn save_proposal(&self, proposal: &ImprovementProposal) -> Result<()> {
        self.backend.save_proposal(proposal).await
    }

    pub async fn list_proposals(&self, filter: ProposalFilter) -> Result<Vec<ImprovementProposal>> {
        self.backend.list_proposals(filter).await
    }

    pub async fn delete_proposal(&self, id: &str, author_did: &str, signature: &[u8]) -> Result<()> {
        self.backend.delete_proposal(id, author_did, signature).await
    }

    pub async fn get_proposal(&self, id: &str) -> Result<Option<ImprovementProposal>> {
        self.backend.get_proposal(id).await
    }

    pub async fn ping(&self) -> Result<()> {
        self.backend.ping().await
    }

    pub async fn search_similar(&self, did: &str, query: &str, limit: usize) -> Result<Vec<LedgerEntry>> {
        // Stub for API compatibility
        let entries = self.get_memories(did, 100).await?;
        let query_lower = query.to_lowercase();
        let mut scored: Vec<_> = entries
            .into_iter()
            .filter_map(|e| {
                let content = e.rationale.clone().unwrap_or_default();
                if content.to_lowercase().contains(&query_lower) {
                    Some((e, 1))
                } else {
                    None
                }
            })
            .collect();
        scored.sort_by_key(|(_, score)| -score);
        let results = scored.into_iter().take(limit).map(|(e, _)| e).collect();
        Ok(results)
    }

    pub async fn record(&self, did: &str, content: &str, _thinking: &Option<String>, signature: &[u8]) -> Result<ExecutionReceipt> {
        // Stub for API compatibility
        let id = uuid::Uuid::new_v4().to_string();
        let timestamp = Utc::now().timestamp();
        let entry = LedgerEntry {
            id: id.clone(),
            version: 1,
            decision_type: "inference".to_string(),
            before_state: None,
            after_state: None,
            rationale: Some(content.to_string()),
            timestamp,
            agent_id: did.to_string(),
            entry_hash: vec![],
            parent_hash: None,
            signature: Some(signature.to_vec()),
            public_key: None,
            nostr_event_id: None,
            tree_id: None,
            parent_event_id: None,
            zk_proof_hash: None,
        };
        self.append_entry(entry).await?;

        Ok(ExecutionReceipt {
            id,
            merkle_root: format!("0x{:x}", sha3::Sha3_256::digest(b"mock")),
            timestamp,
        })
    }
}
