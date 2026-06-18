use crate::evolution::resource::{Resource, ResourceMetadata, ResourceInterface, ResourceState, ProvenanceEntry};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ResearchComponentType {
    Manuscript,
    Dataset,
    Code,
    Model,
    Pipeline,
    Supplementary,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchComponent {
    pub component_type: ResearchComponentType,
    pub name: String,
    pub hash: String,
    pub cid: Option<String>,
    pub size_bytes: Option<u64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContributorCredit {
    pub npub: String,
    pub orcid: Option<String>,
    pub role: String,
    pub contribution_score: f64,
    pub contribution_description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeVersion {
    pub version: String,
    pub hash: String,
    pub created_at: u64,
    pub created_by: String,
    pub changelog: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeSciNodeResource {
    pub metadata: ResourceMetadata,
    pub node_id: String,
    pub dpid: String,
    pub title: String,
    pub abstract_text: Option<String>,
    pub components: Vec<ResearchComponent>,
    pub contributors: Vec<ContributorCredit>,
    pub orcid_links: Vec<String>,
    pub versions: Vec<NodeVersion>,
    pub current_version: String,
    pub license: Option<String>,
    pub keywords: Vec<String>,
}

impl DeSciNodeResource {
    pub fn new(
        title: &str,
        dpid: &str,
        author_npub: &str,
        author_orcid: Option<&str>,
    ) -> Self {
        let now = chrono::Utc::now().timestamp() as u64;
        let node_id = format!("desci:{}", uuid::Uuid::new_v4());

        let contributors = vec![ContributorCredit {
            npub: author_npub.to_string(),
            orcid: author_orcid.map(|s| s.to_string()),
            role: "author".to_string(),
            contribution_score: 1.0,
            contribution_description: Some("Initial creation".to_string()),
        }];

        Self {
            metadata: ResourceMetadata {
                id: node_id.clone(),
                version: "1.0.0".to_string(),
                state: ResourceState::Active,
                interface: ResourceInterface {
                    input_schema: serde_json::json!({}),
                    output_schema: serde_json::json!({}),
                    side_effects: vec!["publishes_research".to_string()],
                    dependencies: vec!["hash_tree".to_string()],
                },
                created_at: now,
                updated_at: now,
                author: author_npub.to_string(),
                provenance: Vec::new(),
                tags: vec!["desci".to_string(), "research".to_string()],
            },
            node_id,
            dpid: dpid.to_string(),
            title: title.to_string(),
            abstract_text: None,
            components: Vec::new(),
            contributors,
            orcid_links: author_orcid.map(|o| vec![o.to_string()]).unwrap_or_default(),
            versions: vec![NodeVersion {
                version: "v1".to_string(),
                hash: "".to_string(),
                created_at: now,
                created_by: author_npub.to_string(),
                changelog: "Initial version".to_string(),
            }],
            current_version: "v1".to_string(),
            license: Some("CC-BY-4.0".to_string()),
            keywords: vec![],
        }
    }

    pub fn add_component(&mut self, component: ResearchComponent) {
        self.components.push(component);
        self.metadata.updated_at = chrono::Utc::now().timestamp() as u64;
    }

    pub fn add_contributor(&mut self, contributor: ContributorCredit) {
        self.contributors.push(contributor);
        self.metadata.updated_at = chrono::Utc::now().timestamp() as u64;
    }

    pub fn create_new_version(&mut self, changelog: &str, author: &str) -> String {
        let new_version = format!("v{}", self.versions.len() + 1);
        let now = chrono::Utc::now().timestamp() as u64;

        self.versions.push(NodeVersion {
            version: new_version.clone(),
            hash: "".to_string(),
            created_at: now,
            created_by: author.to_string(),
            changelog: changelog.to_string(),
        });

        self.current_version = new_version.clone();
        self.metadata.updated_at = now;
        new_version
    }
}

impl Resource for DeSciNodeResource {
    fn metadata(&self) -> &ResourceMetadata { &self.metadata }
    fn metadata_mut(&mut self) -> &mut ResourceMetadata { &mut self.metadata }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }

    fn to_bytes(&self) -> Result<Vec<u8>, String> {
        serde_json::to_vec(self).map_err(|e| format!("Erro ao serializar DeSciNodeResource: {}", e))
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        serde_json::from_slice(bytes).map_err(|e| format!("Erro ao deserializar DeSciNodeResource: {}", e))
    }
}
