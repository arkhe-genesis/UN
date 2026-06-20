use anyhow::{anyhow, Result};
use common::crypto_config::{CryptoConfig, SignatureAlgorithm};
use cathedral_sdk_rs::crypto::{SigningKeyWrapper, VerifyingKeyWrapper, CryptoFactory};
use cathedral_sdk_rs::bridge_proto::{Event, EventType, EventMetadata}; // Adjust depending on path
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

// Mock AgentIdentity logic assuming it gets added to the proto
pub struct AgentIdentity {
    pub ed25519_public_key: Option<Vec<u8>>,
    pub ml_dsa_public_key: Option<Vec<u8>>,
    pub slh_dsa_public_key: Option<Vec<u8>>,
}

/// Registro de chaves públicas com suporte a múltiplos algoritmos
pub struct PublicKeyRegistry {
    keys: Arc<RwLock<HashMap<String, Vec<VerifyingKeyWrapper>>>>,
    config: CryptoConfig,
    factory: CryptoFactory,
}

impl PublicKeyRegistry {
    pub fn new(config: CryptoConfig) -> Self {
        let factory = CryptoFactory::new(config.clone());
        Self {
            keys: Arc::new(RwLock::new(HashMap::new())),
            config,
            factory,
        }
    }

    /// Registra uma chave pública para um agente
    pub async fn register_key(&self, agent_id: &str, key_bytes: &[u8], alg: SignatureAlgorithm) -> Result<()> {
        let key = VerifyingKeyWrapper::from_bytes(alg, key_bytes)?;
        let mut cache = self.keys.write().await;
        let entry = cache.entry(agent_id.to_string()).or_insert_with(Vec::new);
        // Evita duplicatas
        if !entry.iter().any(|k| k.algorithm() == alg) {
            entry.push(key);
            info!("Registrada chave {} para agente {}", alg.as_str(), agent_id);
        }
        Ok(())
    }

    /// Verifica uma assinatura usando qualquer chave registrada (dual‑stack)
    pub async fn verify_signature(&self, agent_id: &str, message: &[u8], signature: &[u8]) -> Result<bool> {
        let cache = self.keys.read().await;
        let keys = cache.get(agent_id)
            .ok_or_else(|| anyhow!("Agente {} não possui chave registrada", agent_id))?;

        for key in keys {
            if key.verify(message, signature).unwrap_or(false) {
                debug!("Assinatura verificada com algoritmo {}", key.algorithm().as_str());
                return Ok(true);
            }
        }
        warn!("Nenhuma chave verificou a assinatura para agente {}", agent_id);
        Ok(false)
    }

    /// Registra chaves a partir de um payload (útil para agentes que enviam sua chave pública)
    pub async fn register_from_identity(&self, agent_id: &str, identity: &AgentIdentity) -> Result<()> {
        // Supondo que AgentIdentity tenha campos para chaves públicas
        if let Some(ref ed25519_key) = identity.ed25519_public_key {
            self.register_key(agent_id, ed25519_key, SignatureAlgorithm::Ed25519).await?;
        }
        if let Some(ref ml_dsa_key) = identity.ml_dsa_public_key {
            self.register_key(agent_id, ml_dsa_key, SignatureAlgorithm::MlDsa).await?;
        }
        Ok(())
    }
}

/// Verificador de assinaturas que integra o registro
pub struct SignatureVerifier {
    registry: Arc<PublicKeyRegistry>,
    config: CryptoConfig,
}

impl SignatureVerifier {
    pub fn new(registry: Arc<PublicKeyRegistry>, config: CryptoConfig) -> Self {
        Self { registry, config }
    }

    pub async fn verify_batch(&self, agent_id: &str, events: &[Event], signature: &[u8]) -> Result<bool> {
        // Mock Blake3 usage, normally would depend on blake3
        // use std::hash::Hasher to fake it
        use std::collections::hash_map::DefaultHasher;
        use std::hash::Hasher;
        let mut hasher = DefaultHasher::new();
        for event in events {
            hasher.write(event.event_id.as_bytes());
            hasher.write(event.payload_json.as_bytes());
        }
        let hash = hasher.finish().to_le_bytes().to_vec();
        self.registry.verify_signature(agent_id, &hash, signature).await
    }
}
