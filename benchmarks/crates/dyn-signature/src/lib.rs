//! Safe-Core DynSignature
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SignatureError {
    #[error("Invalid key: {0}")]
    InvalidKey(String),
    #[error("Signing failed: {0}")]
    SigningFailed(String),
    #[error("Verification failed: {0}")]
    VerificationFailed(String),
    #[error("Algorithm mismatch: expected {expected}, got {actual}")]
    AlgorithmMismatch { expected: String, actual: String },
    #[error("Feature not enabled: {0}")]
    FeatureNotEnabled(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignatureAlgorithm {
    #[serde(rename = "P256")]
    P256,
    #[serde(rename = "Ed25519")]
    Ed25519,
}

impl SignatureAlgorithm {
    pub fn as_str(&self) -> &str {
        match self {
            SignatureAlgorithm::P256 => "P256",
            SignatureAlgorithm::Ed25519 => "Ed25519",
        }
    }
}

pub enum DynPrivateKey {
    #[cfg(feature = "p256")]
    P256(p256::ecdsa::SigningKey),
    #[cfg(feature = "ed25519")]
    Ed25519(ed25519_dalek::SigningKey),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DynPublicKey {
    #[cfg(feature = "p256")]
    P256(p256::ecdsa::VerifyingKey),
    #[cfg(feature = "ed25519")]
    Ed25519(ed25519_dalek::VerifyingKey),
}

#[derive(Debug, Clone)]
pub enum DynSignature {
    #[cfg(feature = "p256")]
    P256(p256::ecdsa::Signature),
    #[cfg(feature = "ed25519")]
    Ed25519(ed25519_dalek::Signature),
}

impl DynSignature {
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            #[cfg(feature = "p256")]
            DynSignature::P256(sig) => sig.to_vec(),
            #[cfg(feature = "ed25519")]
            DynSignature::Ed25519(sig) => sig.to_bytes().to_vec(),
        }
    }

    pub fn from_bytes(alg: SignatureAlgorithm, bytes: &[u8]) -> Result<Self, SignatureError> {
        match alg {
            #[cfg(feature = "p256")]
            SignatureAlgorithm::P256 => {
                let sig = p256::ecdsa::Signature::from_slice(bytes)
                    .map_err(|e| SignatureError::InvalidKey(e.to_string()))?;
                Ok(DynSignature::P256(sig))
            }
            #[cfg(feature = "ed25519")]
            SignatureAlgorithm::Ed25519 => {
                let bytes_arr: [u8; 64] = bytes.try_into()
                    .map_err(|_| SignatureError::InvalidKey("Ed25519 sig must be 64 bytes".into()))?;
                Ok(DynSignature::Ed25519(ed25519_dalek::Signature::from_bytes(&bytes_arr)))
            }
            #[allow(unreachable_patterns)]
            _ => Err(SignatureError::FeatureNotEnabled(alg.as_str().to_string())),
        }
    }
}

impl serde::Serialize for DynSignature {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bytes(&self.to_bytes())
    }
}

impl<'de> serde::Deserialize<'de> for DynSignature {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bytes = <Vec<u8>>::deserialize(deserializer)?;
        if bytes.len() == 64 {
            Self::from_bytes(SignatureAlgorithm::Ed25519, &bytes).map_err(serde::de::Error::custom)
        } else {
            Self::from_bytes(SignatureAlgorithm::P256, &bytes).map_err(serde::de::Error::custom)
        }
    }
}

impl serde::Serialize for DynPublicKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let bytes = match self {
            #[cfg(feature = "p256")]
            DynPublicKey::P256(key) => key.to_sec1_bytes().into_vec(),
            #[cfg(feature = "ed25519")]
            DynPublicKey::Ed25519(key) => key.to_bytes().to_vec(),
        };
        serializer.serialize_bytes(&bytes)
    }
}

impl<'de> serde::Deserialize<'de> for DynPublicKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bytes = <Vec<u8>>::deserialize(deserializer)?;
        if bytes.len() == 32 {
            #[cfg(feature = "ed25519")]
            {
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&bytes);
                let key = ed25519_dalek::VerifyingKey::from_bytes(&arr).map_err(serde::de::Error::custom)?;
                Ok(DynPublicKey::Ed25519(key))
            }
            #[cfg(not(feature = "ed25519"))]
            Err(serde::de::Error::custom("Unsupported algorithm"))
        } else {
            #[cfg(feature = "p256")]
            {
                let key = p256::ecdsa::VerifyingKey::from_sec1_bytes(&bytes).map_err(serde::de::Error::custom)?;
                Ok(DynPublicKey::P256(key))
            }
            #[cfg(not(feature = "p256"))]
            Err(serde::de::Error::custom("Unsupported algorithm"))
        }
    }
}

pub fn verify_dyn_signature(sig: &DynSignature, vk: &DynPublicKey, msg: &[u8]) -> Result<(), SignatureError> {
    match (sig, vk) {
        #[cfg(feature = "p256")]
        (DynSignature::P256(s), DynPublicKey::P256(k)) => {
            use ecdsa::signature::Verifier;
            k.verify(msg, s).map_err(|e| SignatureError::VerificationFailed(e.to_string()))
        }
        #[cfg(feature = "ed25519")]
        (DynSignature::Ed25519(s), DynPublicKey::Ed25519(k)) => {
            use ed25519_dalek::Verifier;
            k.verify(msg, s).map_err(|e| SignatureError::VerificationFailed(e.to_string()))
        }
        _ => Err(SignatureError::AlgorithmMismatch { expected: "unknown".into(), actual: "unknown".into() })
    }
}
