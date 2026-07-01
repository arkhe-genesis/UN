use std::collections::HashMap;

pub type EvidenceID = u64;
pub type ArtifactID = u64;
pub type ClaimID = u64;
pub type DecisionID = u64;
pub type Hash = String;
pub type Payload = String;

#[derive(Debug, Clone, PartialEq)]
pub struct Evidence {
    pub artifact_id: ArtifactID,
    pub content: Payload,
    pub signature: Hash,
    pub timestamp: u64,
    pub parent_hash: Option<Hash>,
    pub hash: Hash,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Artifact {
    pub payload: Payload,
    pub metadata: String,
    pub hash: Hash,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Claim {
    pub proposition: String,
    pub evidence_ids: Vec<EvidenceID>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Belief {
    pub claim_id: ClaimID,
    pub confidence: u8,
    pub justification: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Decision {
    pub goal: String,
    pub belief_ids: Vec<u64>,
    pub timestamp: u64,
}

pub enum Event {
    ArtifactAdded(ArtifactID, Payload, String),
    EvidenceAdded(EvidenceID, ArtifactID, Payload, Hash, u64, Option<Hash>),
    ClaimAdded(ClaimID, String, Vec<EvidenceID>),
    BeliefAdded(u64, ClaimID, u8, String),
    DecisionAdded(DecisionID, String, Vec<u64>, u64),
}

#[derive(Debug, Clone, Default)]
pub struct State {
    pub artifacts: HashMap<ArtifactID, Artifact>,
    pub evidences: HashMap<EvidenceID, Evidence>,
    pub claims: HashMap<ClaimID, Claim>,
    pub beliefs: HashMap<u64, Belief>,
    pub decisions: HashMap<DecisionID, Decision>,
}

impl State {
    pub fn new() -> Self {
        Self::default()
    }
}
