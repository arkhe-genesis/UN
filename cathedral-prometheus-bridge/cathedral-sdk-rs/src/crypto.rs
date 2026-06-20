use anyhow::{anyhow, Result};
use common::crypto_config::{CryptoConfig, SignatureAlgorithm};
use ed25519_dalek::{Signer, SigningKey, Verifier, VerifyingKey};
use pqcrypto_dilithium::dilithium3::{self, PublicKey, SecretKey, detached_sign, verify_detached_signature, keypair};
use pqcrypto_traits::sign::{PublicKey as TraitPublicKey, SecretKey as TraitSecretKey, DetachedSignature};

/// Wrapper unificado para chave de assinatura
pub enum SigningKeyWrapper {
    Ed25519(SigningKey),
    MlDsa(SecretKey),
}

impl SigningKeyWrapper {
    pub fn generate(alg: SignatureAlgorithm) -> Result<Self> {
        match alg {
            SignatureAlgorithm::Ed25519 => {
                let mut rng = rand::thread_rng();
                Ok(Self::Ed25519(SigningKey::generate(&mut rng)))
            }
            SignatureAlgorithm::MlDsa => {
                let (_, sk) = keypair();
                Ok(Self::MlDsa(sk))
            }
            _ => Err(anyhow!("Algoritmo não suportado para geração de chave")),
        }
    }

    pub fn sign(&self, message: &[u8]) -> Result<Vec<u8>> {
        match self {
            Self::Ed25519(sk) => {
                let sig = sk.sign(message);
                Ok(sig.to_bytes().to_vec())
            }
            Self::MlDsa(sk) => {
                let sig = detached_sign(message, sk);
                Ok(sig.as_bytes().to_vec())
            }
        }
    }

    pub fn algorithm(&self) -> SignatureAlgorithm {
        match self {
            Self::Ed25519(_) => SignatureAlgorithm::Ed25519,
            Self::MlDsa(_) => SignatureAlgorithm::MlDsa,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Self::Ed25519(sk) => sk.to_bytes().to_vec(),
            Self::MlDsa(sk) => sk.as_bytes().to_vec(),
        }
    }

    pub fn from_bytes(alg: SignatureAlgorithm, bytes: &[u8]) -> Result<Self> {
        match alg {
            SignatureAlgorithm::Ed25519 => {
                let arr: [u8; 32] = bytes.try_into()
                    .map_err(|_| anyhow!("Tamanho inválido para chave Ed25519"))?;
                Ok(Self::Ed25519(SigningKey::from_bytes(&arr)))
            }
            SignatureAlgorithm::MlDsa => {
                let sk = SecretKey::from_bytes(bytes)
                    .map_err(|e| anyhow!("Falha ao carregar chave ML-DSA: {}", e))?;
                Ok(Self::MlDsa(sk))
            }
            _ => Err(anyhow!("Algoritmo não suportado para desserialização")),
        }
    }
}

/// Wrapper unificado para chave de verificação
pub enum VerifyingKeyWrapper {
    Ed25519(VerifyingKey),
    MlDsa(PublicKey),
}

impl VerifyingKeyWrapper {
    pub fn from_bytes(alg: SignatureAlgorithm, bytes: &[u8]) -> Result<Self> {
        match alg {
            SignatureAlgorithm::Ed25519 => {
                let arr: [u8; 32] = bytes.try_into()
                    .map_err(|_| anyhow!("Tamanho inválido para chave Ed25519"))?;
                Ok(Self::Ed25519(VerifyingKey::from_bytes(&arr).map_err(|_| anyhow!("Invalid public key"))?))
            }
            SignatureAlgorithm::MlDsa => {
                let pk = PublicKey::from_bytes(bytes)
                    .map_err(|e| anyhow!("Falha ao carregar chave ML-DSA: {}", e))?;
                Ok(Self::MlDsa(pk))
            }
            _ => Err(anyhow!("Algoritmo não suportado")),
        }
    }

    pub fn verify(&self, message: &[u8], signature: &[u8]) -> Result<bool> {
        match self {
            Self::Ed25519(vk) => {
                let sig = ed25519_dalek::Signature::from_slice(signature).map_err(|_| anyhow!("Invalid signature format"))?;
                Ok(vk.verify(message, &sig).is_ok())
            }
            Self::MlDsa(vk) => {
                let sig = dilithium3::DetachedSignature::from_bytes(signature)
                    .map_err(|e| anyhow!("Signature inválida: {}", e))?;
                Ok(verify_detached_signature(&sig, message, vk).is_ok())
            }
        }
    }

    pub fn algorithm(&self) -> SignatureAlgorithm {
        match self {
            Self::Ed25519(_) => SignatureAlgorithm::Ed25519,
            Self::MlDsa(_) => SignatureAlgorithm::MlDsa,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Self::Ed25519(vk) => vk.to_bytes().to_vec(),
            Self::MlDsa(vk) => vk.as_bytes().to_vec(),
        }
    }
}

/// Fábrica criptográfica com suporte a dual‑stack
pub struct CryptoFactory {
    config: CryptoConfig,
}

impl CryptoFactory {
    pub fn new(config: CryptoConfig) -> Self {
        Self { config }
    }

    /// Gera um par de chaves para o algoritmo principal
    pub fn generate_signing_key(&self) -> Result<SigningKeyWrapper> {
        SigningKeyWrapper::generate(self.config.signature_algorithm)
    }

    /// Gera um par de chaves para o algoritmo de fallback (se configurado)
    pub fn generate_fallback_key(&self) -> Result<Option<SigningKeyWrapper>> {
        if let Some(alg) = self.config.fallback_signature_algorithm {
            Ok(Some(SigningKeyWrapper::generate(alg)?))
        } else {
            Ok(None)
        }
    }

    /// Carrega chave de verificação a partir de bytes (tenta ambos os algoritmos se dual‑stack)
    pub fn load_verifying_key(&self, bytes: &[u8]) -> Result<VerifyingKeyWrapper> {
        // Primeiro tenta o algoritmo principal
        if let Ok(key) = VerifyingKeyWrapper::from_bytes(self.config.signature_algorithm, bytes) {
            return Ok(key);
        }
        // Se falhar e houver fallback, tenta o fallback
        if let Some(fallback) = self.config.fallback_signature_algorithm {
            if let Ok(key) = VerifyingKeyWrapper::from_bytes(fallback, bytes) {
                return Ok(key);
            }
        }
        Err(anyhow!("Não foi possível carregar chave de verificação"))
    }

    /// Assina uma mensagem usando o algoritmo principal (e opcionalmente o fallback para dual‑stack)
    pub fn sign(&self, key: &SigningKeyWrapper, message: &[u8]) -> Result<Vec<u8>> {
        key.sign(message)
    }

    /// Verifica uma assinatura, tentando ambos os algoritmos se dual‑stack
    pub fn verify_dual(
        &self,
        primary_key: &VerifyingKeyWrapper,
        fallback_key: Option<&VerifyingKeyWrapper>,
        message: &[u8],
        signature: &[u8],
    ) -> Result<bool> {
        // Tenta verificar com a chave primária
        if primary_key.verify(message, signature)? {
            return Ok(true);
        }
        // Se falhar e houver fallback, tenta com o fallback
        if let Some(fb_key) = fallback_key {
            if fb_key.verify(message, signature)? {
                return Ok(true);
            }
        }
        Ok(false)
    }
}
