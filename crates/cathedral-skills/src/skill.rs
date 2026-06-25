use serde::{Deserialize, Serialize};
use cathedral_identity::SignatureGuard;
use cathedral_permissions::PermissionEntry;

// Mock Did
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Did(pub String);

impl Did {
    pub fn to_string(&self) -> String {
        self.0.clone()
    }
}

/// Uma Skill é uma capacidade reutilizável que pode ser invocada por agentes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub author_did: Did,
    pub signature: Vec<u8>,      // Assinatura ML‑DSA do autor
    pub metadata: SkillMetadata,
    pub implementation: SkillImplementation,
}

/// Metadados de uma Skill
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMetadata {
    pub version: String,
    pub dependencies: Vec<String>,
    pub permissions: Vec<PermissionEntry>,
    pub tags: Vec<String>,
}

/// Implementação de uma Skill (código executável)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SkillImplementation {
    Rust { code: String, entrypoint: String },
    Python { code: String, entrypoint: String },
    Shell { script: String },
    Wasm { module: Vec<u8> },
}

impl Skill {
    pub fn verify(&self) -> Result<bool, String> {
        let _guard = SignatureGuard::new();
        let _message = serde_json::to_vec(self).map_err(|e| e.to_string())?;
        // Mock verificação
        // Ok(guard.verify(&message, &self.signature))
        Ok(true)
    }
}
