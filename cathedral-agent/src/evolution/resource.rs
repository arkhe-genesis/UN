use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetadata {
    pub id: String,
    pub version: String,
    pub state: ResourceState,
    pub interface: ResourceInterface,
    pub created_at: u64,
    pub updated_at: u64,
    pub author: String,
    pub provenance: Vec<ProvenanceEntry>,
    pub tags: Vec<String>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceInterface {
    pub input_schema: serde_json::Value,
    pub output_schema: serde_json::Value,
    pub side_effects: Vec<String>,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ResourceState { Active, Inactive }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceEntry {
    pub action: String,
    pub author: String,
    pub timestamp: u64,
    pub description: String,
    pub tx_hash: Option<String>,
    pub artifact_hash: Option<String>,
}

pub trait Resource: Send + Sync + std::fmt::Debug {
    fn metadata(&self) -> &ResourceMetadata;
    fn metadata_mut(&mut self) -> &mut ResourceMetadata;
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
    fn to_bytes(&self) -> Result<Vec<u8>, String>;
    fn from_bytes(bytes: &[u8]) -> Result<Self, String> where Self: Sized;

    fn bump_version(&mut self, _rationale: &str) {
        // Simple version bump logic for mock
        self.metadata_mut().version = format!("{}-new", self.metadata().version);
    }

    fn add_provenance(&mut self, action: &str, author: &str, desc: &str, tx_hash: Option<&str>, artifact_hash: Option<&str>) {
        self.metadata_mut().provenance.push(ProvenanceEntry {
            action: action.to_string(),
            author: author.to_string(),
            timestamp: chrono::Utc::now().timestamp() as u64,
            description: desc.to_string(),
            tx_hash: tx_hash.map(|s| s.to_string()),
            artifact_hash: artifact_hash.map(|s| s.to_string()),
        });
    }
}
