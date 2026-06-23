use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Tipo de cabeçalho definido no ADR 017 (PqcAttestation = 0xF8)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum HeaderType {
    PqcAttestation = 0xF8,
    // Futuros headers: Timestamp, Routing, etc.
}

/// Corpo do objeto Arkhe.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ArkheBody {
    pub data: String, // Conteúdo principal (resposta, comando, etc.)
    pub timestamp: i64,
}

/// Objeto Arkhe: unidade de transporte do Cathedral-OS.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ArkheObject {
    pub id: String,
    pub source_did: String,
    pub target_did: Option<String>,
    pub body: ArkheBody,
    pub headers: Vec<(HeaderType, Vec<u8>)>,
}

impl ArkheObject {
    /// Cria um novo objeto com dados e DID de origem.
    pub fn new(data: String, source_did: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            source_did: source_did.to_string(),
            target_did: None,
            body: ArkheBody {
                data,
                timestamp: Utc::now().timestamp(),
            },
            headers: Vec::new(),
        }
    }

    /// Adiciona um cabeçalho.
    pub fn add_header(&mut self, typ: HeaderType, value: Vec<u8>) {
        self.headers.push((typ, value));
    }

    /// Obtém o valor de um cabeçalho, se existir.
    pub fn get_header(&self, typ: HeaderType) -> Option<&[u8]> {
        self.headers
            .iter()
            .find(|(t, _)| *t == typ)
            .map(|(_, v)| v.as_slice())
    }

    /// Serializa para bytes (usando bincode).
    pub fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        Ok(bincode::serialize(self)?)
    }

    /// Deserializa a partir de bytes.
    pub fn from_bytes(data: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(bincode::deserialize(data)?)
    }
}
