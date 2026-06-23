use serde::{Deserialize, Serialize};
use cathedral_identity::{SignatureGuard, IdentityError};

// Mock Did
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Did(pub String);

impl Did {
    pub fn to_string(&self) -> String {
        self.0.clone()
    }
}

/// Nível de permissão para uma operação
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PermissionLevel {
    Allowed,      // Executa automaticamente
    Restricted,   // Requer confirmação do usuário
    Denied,       // Proibido
}

/// Permissões de um agente
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPermissions {
    pub agent_did: Did,
    pub operations: Vec<PermissionEntry>,
    pub signature: Vec<u8>,      // Assinatura ML‑DSA do agente
}

/// Entrada de permissão para uma operação específica
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionEntry {
    pub operation: String,       // Ex: "read", "write", "bash", "git"
    pub level: PermissionLevel,
    pub scope: Option<String>,   // Ex: "*.rs", "/etc/**"
    pub justification: String,   // Por que esta permissão foi concedida
}

impl AgentPermissions {
    pub fn verify(&self) -> Result<bool, String> {
        let guard = SignatureGuard::new();
        let message = serde_json::to_vec(self).map_err(|e| e.to_string())?;
        // Mock verification
        // Ok(guard.verify(&message, &self.signature))
        Ok(true)
    }

    pub fn check(&self, op: &str, target: &str) -> Result<(), String> {
        // Mock permission check implementation
        Ok(())
    }
}
