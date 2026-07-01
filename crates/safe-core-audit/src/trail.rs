use crate::{AuditEvent, MerkleTree, MerkleProof};

#[derive(Debug, thiserror::Error)]
pub enum AuditError {
    #[error("JSON Error: {0}")]
    Json(#[from] serde_json::Error),
}

pub struct AuditTrail {
    events: Vec<AuditEvent>,
    merkle: MerkleTree,
}

impl AuditTrail {
    pub fn new() -> Self {
        Self { events: Vec::new(), merkle: MerkleTree::new() }
    }
    pub fn push(&mut self, event: AuditEvent) -> Result<(), AuditError> {
        let hash = blake3::hash(&serde_json::to_vec(&event)?);
        self.merkle.push(hash.into());
        self.events.push(event);
        Ok(())
    }
    pub fn root(&self) -> Option<[u8; 32]> { self.merkle.root() }
    pub fn prove(&self, index: usize) -> Option<MerkleProof> { self.merkle.prove(index) }
    pub fn events(&self) -> &[AuditEvent] { &self.events }
}
