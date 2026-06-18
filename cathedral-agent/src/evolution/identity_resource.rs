use crate::evolution::resource::{Resource, ResourceMetadata, ResourceInterface, ResourceState, ProvenanceEntry};
use crate::evolution::desci_node_resource::{DeSciNodeResource};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityResource {
    pub metadata: ResourceMetadata,
    pub npub: String,
    pub name: String,
    pub desci_profile: Option<DeSciProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeSciProfile {
    pub nodes: Vec<NodeReference>,
    pub peer_reviews: Vec<PeerReview>,
    pub funding_contributions: Vec<FundingContribution>,
    pub reputation_score: DeSciReputation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeReference {
    pub dpid: String,
    pub title: String,
    pub version: String,
    pub published_at: u64,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerReview {
    pub node_id: String,
    pub score: u8,
    pub comments: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundingContribution {
    pub node_id: String,
    pub amount: f64,
    pub currency: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeSciReputation {
    pub publication_count: u32,
    pub review_count: u32,
    pub citation_count: u32,
    pub overall_score: f64,
}

impl IdentityResource {
    pub fn ensure_desci_profile(&mut self) {
        if self.desci_profile.is_none() {
            self.desci_profile = Some(DeSciProfile {
                nodes: Vec::new(),
                peer_reviews: Vec::new(),
                funding_contributions: Vec::new(),
                reputation_score: DeSciReputation {
                    publication_count: 0,
                    review_count: 0,
                    citation_count: 0,
                    overall_score: 0.0,
                },
            });
        }
    }

    pub fn add_desci_contribution(&mut self, node_ref: NodeReference, _role: &str) {
        self.ensure_desci_profile();
        if let Some(profile) = &mut self.desci_profile {
            profile.nodes.push(node_ref);
            profile.reputation_score.publication_count += 1;
        }
        self.metadata.updated_at = chrono::Utc::now().timestamp() as u64;
    }

    pub fn add_peer_review(&mut self, review: PeerReview) {
        self.ensure_desci_profile();
        if let Some(profile) = &mut self.desci_profile {
            profile.peer_reviews.push(review);
            profile.reputation_score.review_count += 1;
        }
        self.metadata.updated_at = chrono::Utc::now().timestamp() as u64;
    }
}

impl Resource for IdentityResource {
    fn metadata(&self) -> &ResourceMetadata { &self.metadata }
    fn metadata_mut(&mut self) -> &mut ResourceMetadata { &mut self.metadata }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }

    fn to_bytes(&self) -> Result<Vec<u8>, String> {
        serde_json::to_vec(self).map_err(|e| format!("Erro ao serializar IdentityResource: {}", e))
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        serde_json::from_slice(bytes).map_err(|e| format!("Erro ao deserializar IdentityResource: {}", e))
    }
}
