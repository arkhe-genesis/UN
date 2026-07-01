use std::collections::HashMap;

pub type Hash = String;
pub type ArtifactID = u64;
pub type EvidenceID = u64;
pub type ClaimID = u64;
pub type DecisionID = u64;
pub type BeliefID = u64;
pub type Payload = String;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Artifact {
    pub payload: Payload,
    pub metadata: String,
    pub hash: Hash,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Evidence {
    pub artifact_id: ArtifactID,
    pub content: Payload,
    pub signature: Hash,
    pub timestamp: u64,
    pub parent_hash: Option<Hash>,
    pub hash: Hash,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Claim {
    pub proposition: String,
    pub evidence_ids: Vec<EvidenceID>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Belief {
    pub claim_id: ClaimID,
    pub confidence: u8,
    pub justification: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decision {
    pub goal: String,
    pub belief_ids: Vec<BeliefID>,
    pub timestamp: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct State {
    pub artifacts: HashMap<ArtifactID, Artifact>,
    pub evidences: HashMap<EvidenceID, Evidence>,
    pub claims: HashMap<ClaimID, Claim>,
    pub beliefs: HashMap<BeliefID, Belief>,
    pub decisions: HashMap<DecisionID, Decision>,
}

impl State {
    pub fn new() -> Self {
        Self {
            artifacts: HashMap::new(),
            evidences: HashMap::new(),
            claims: HashMap::new(),
            beliefs: HashMap::new(),
            decisions: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    ArtifactAdded(ArtifactID, Payload, String),
    EvidenceAdded(EvidenceID, ArtifactID, Payload, Hash, u64, Option<Hash>),
    ClaimAdded(ClaimID, String, Vec<EvidenceID>),
    BeliefAdded(BeliefID, ClaimID, u8, String),
    DecisionAdded(DecisionID, String, Vec<BeliefID>, u64),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransitionError {
    IdAlreadyExists,
    ReferencedIdNotFound,
    InvalidParentHash,
}
