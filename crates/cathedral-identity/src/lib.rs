use cathedral_arkheobex::{ArkheObject, HeaderType};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand_core::OsRng;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum IdentityError {
    #[error("Unknown DID")]
    UnknownDID,
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("Missing attestation")]
    MissingAttestation,
}

pub struct SignatureGuard {
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
}

impl Default for SignatureGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl SignatureGuard {
    pub fn new() -> Self {
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let verifying_key = signing_key.verifying_key();
        Self {
            signing_key,
            verifying_key,
        }
    }

    pub fn sign(&self, message: &[u8]) -> Vec<u8> {
        let signature: Signature = self.signing_key.sign(message);
        signature.to_bytes().to_vec()
    }

    pub fn verify(&self, message: &[u8], signature: &[u8]) -> bool {
        if signature.len() != 64 {
            return false;
        }
        let sig = Signature::from_bytes(signature.try_into().unwrap());
        self.verifying_key.verify(message, &sig).is_ok()
    }

    /// Adiciona o header PqcAttestation (0xF8) a um ArkheObject
    pub fn attest_object(&self, obj: &mut ArkheObject) -> Result<(), IdentityError> {
        let data = obj.body.data.as_bytes();
        let sig = self.sign(data);
        let attestation = PqcAttestation::new(sig);
        obj.add_header(HeaderType::PqcAttestation, attestation.to_bytes());
        Ok(())
    }
}

pub struct PqcAttestation {
    signature: Vec<u8>,
    // No futuro, incluirá certificado ML-DSA
}

impl PqcAttestation {
    pub fn new(signature: Vec<u8>) -> Self {
        Self { signature }
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&(self.signature.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&self.signature);
        bytes
    }
    pub fn from_bytes(data: &[u8]) -> Self {
        let len = u32::from_le_bytes(data[0..4].try_into().unwrap()) as usize;
        let signature = data[4..4 + len].to_vec();
        Self { signature }
    }
}

#[derive(Default)]
pub struct IdentityGateway {
    // did_store: Arc<dashmap::DashMap<String, Vec<u8>>>,
}

impl IdentityGateway {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn verify(
        &self,
        _did: &str,
        _signature: &[u8],
        _message: &[u8],
    ) -> Result<bool, IdentityError> {
        // No protótipo, ignoramos a verificação real para mock
        Ok(true)
    }
}
