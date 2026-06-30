import os

base_dir = "crates"
os.makedirs(base_dir, exist_ok=True)

# 6. FEATURE: Avaliação rs-merkle-tree (benchmark)
feature_merkle_dir = os.path.join(base_dir, "merkle-evaluation")
os.makedirs(os.path.join(feature_merkle_dir, "src"), exist_ok=True)
os.makedirs(os.path.join(feature_merkle_dir, "benches"), exist_ok=True)

merkle_cargo = '''[package]
name = "safe-core-merkle-evaluation"
version = "0.1.0"
edition = "2021"

[dependencies]
rs_merkle = "1.5"
merkle_light = "0.4"
sha2 = "0.10"
blake3 = "1.8"
serde = { version = "1.0", features = ["derive"] }
thiserror = "1.0"
tracing = "0.1"

[dev-dependencies]
criterion = { version = "0.5", features = ["async_tokio"] }
proptest = "1.6"

[[bench]]
name = "merkle_benchmark"
harness = false
'''

merkle_bench = '''//! Benchmark comparativo de implementações de Merkle Tree
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use rs_merkle::{MerkleTree, Hasher as RsMerkleHasher};
use sha2::{Sha256, Digest};

#[derive(Clone)]
struct Sha256Hasher;

impl RsMerkleHasher for Sha256Hasher {
    type Hash = [u8; 32];

    fn hash(data: &[u8]) -> Self::Hash {
        let result = Sha256::digest(data);
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
    }
}

fn build_tree_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("merkle_build");
    for size in [100, 1000, 10000].iter() {
        let leaves: Vec<[u8; 32]> = (0..*size)
            .map(|i| {
                let mut hasher = Sha256::new();
                hasher.update(&i.to_le_bytes());
                let mut hash = [0u8; 32];
                hash.copy_from_slice(&hasher.finalize());
                hash
            })
            .collect();
        group.bench_with_input(BenchmarkId::new("rs_merkle", size), size, |b, _| {
            b.iter(|| {
                let tree = MerkleTree::<Sha256Hasher>::from_leaves(&leaves);
                black_box(tree.root());
            });
        });
    }
    group.finish();
}

fn proof_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("merkle_proof");
    let leaves: Vec<[u8; 32]> = (0..1000)
        .map(|i| {
            let mut hasher = Sha256::new();
            hasher.update(&i.to_le_bytes());
            let mut hash = [0u8; 32];
            hash.copy_from_slice(&hasher.finalize());
            hash
        })
        .collect();
    let tree = MerkleTree::<Sha256Hasher>::from_leaves(&leaves);
    group.bench_function("generate_proof", |b| {
        b.iter(|| {
            let indices_to_prove = vec![500];
            let proof = tree.proof(&indices_to_prove);
            black_box(proof);
        });
    });
    group.bench_function("verify_proof", |b| {
        let indices_to_prove = vec![500];
        let proof = tree.proof(&indices_to_prove);
        let leaves_to_prove = vec![leaves[500]];
        b.iter(|| {
            let result = proof.verify(
                tree.root(),
                &indices_to_prove,
                &leaves_to_prove,
                leaves.len(),
            );
            black_box(result);
        });
    });
    group.finish();
}

criterion_group!(benches, build_tree_benchmark, proof_benchmark);
criterion_main!(benches);
'''

merkle_lib = '''//! Safe-Core Merkle Tree — Implementação otimizada
use blake3::Hasher;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MerkleError {
    #[error("Invalid leaf index: {0}")]
    InvalidIndex(usize),
    #[error("Tree is empty")]
    EmptyTree,
    #[error("Proof verification failed")]
    VerificationFailed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleNode {
    pub hash: [u8; 32],
    pub left: Option<Box<MerkleNode>>,
    pub right: Option<Box<MerkleNode>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafeMerkleTree {
    root: Option<MerkleNode>,
    leaves: Vec<[u8; 32]>,
}

impl SafeMerkleTree {
    pub fn from_leaves(leaves: &[[u8; 32]]) -> Self {
        if leaves.is_empty() {
            return Self { root: None, leaves: vec![] };
        }
        let mut current_level: Vec<[u8; 32]> = leaves.to_vec();
        let next_pow2 = current_level.len().next_power_of_two();
        while current_level.len() < next_pow2 {
            current_level.push([0u8; 32]);
        }
        while current_level.len() > 1 {
            let mut next_level = Vec::new();
            for chunk in current_level.chunks(2) {
                let hash = Self::hash_pair(&chunk[0], &chunk[1]);
                next_level.push(hash);
            }
            current_level = next_level;
        }
        Self {
            root: Some(MerkleNode {
                hash: current_level[0],
                left: None,
                right: None,
            }),
            leaves: leaves.to_vec(),
        }
    }

    pub fn root(&self) -> Option<[u8; 32]> {
        self.root.as_ref().map(|n| n.hash)
    }

    pub fn proof(&self, index: usize) -> Result<MerkleProof, MerkleError> {
        if index >= self.leaves.len() {
            return Err(MerkleError::InvalidIndex(index));
        }
        let mut proof = Vec::new();
        let mut current_idx = index;
        let mut level_size = self.leaves.len().next_power_of_two();
        let mut level: Vec<[u8; 32]> = self.leaves.clone();
        while level_size > 1 {
            let sibling_idx = if current_idx % 2 == 0 { current_idx + 1 } else { current_idx - 1 };
            if sibling_idx < level.len() {
                proof.push(level[sibling_idx]);
            } else {
                proof.push([0u8; 32]);
            }
            let mut next_level = Vec::new();
            for chunk in level.chunks(2) {
                next_level.push(Self::hash_pair(&chunk[0], &chunk[1]));
            }
            level = next_level;
            current_idx /= 2;
            level_size /= 2;
        }
        Ok(MerkleProof { siblings: proof, leaf_index: index })
    }

    pub fn verify_proof(&self, leaf: &[u8; 32], proof: &MerkleProof) -> Result<bool, MerkleError> {
        let root = self.root().ok_or(MerkleError::EmptyTree)?;
        let mut current_hash = *leaf;
        let mut index = proof.leaf_index;
        for sibling in &proof.siblings {
            current_hash = if index % 2 == 0 {
                Self::hash_pair(&current_hash, sibling)
            } else {
                Self::hash_pair(sibling, &current_hash)
            };
            index /= 2;
        }
        Ok(current_hash == root)
    }

    fn hash_pair(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
        let mut hasher = Hasher::new();
        hasher.update(left);
        hasher.update(right);
        hasher.finalize().into()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleProof {
    pub siblings: Vec<[u8; 32]>,
    pub leaf_index: usize,
}
'''

with open(os.path.join(feature_merkle_dir, "Cargo.toml"), "w") as f:
    f.write(merkle_cargo)
with open(os.path.join(feature_merkle_dir, "src", "lib.rs"), "w") as f:
    f.write(merkle_lib)
with open(os.path.join(feature_merkle_dir, "benches", "merkle_benchmark.rs"), "w") as f:
    f.write(merkle_bench)

# 5. FEATURE: persistence-rocksdb
feature_rocks_dir = os.path.join(base_dir, "persistence-rocksdb")
os.makedirs(os.path.join(feature_rocks_dir, "src"), exist_ok=True)

rocks_cargo = '''[package]
name = "safe-core-persistence-rocksdb"
version = "0.1.0"
edition = "2021"

[features]
default = []

[dependencies]
thiserror = "1.0"
'''

rocks_lib = '''//! Safe-Core Persistence — Backend RocksDB
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PersistenceError {
    #[error("RocksDB error: {0}")]
    RocksDb(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Key not found: {0}")]
    KeyNotFound(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnFamily {
    Audit,
    Config,
    State,
    Merkle,
    Consensus,
}

impl ColumnFamily {
    pub fn as_str(&self) -> &str {
        match self {
            ColumnFamily::Audit => "audit",
            ColumnFamily::Config => "config",
            ColumnFamily::State => "state",
            ColumnFamily::Merkle => "merkle",
            ColumnFamily::Consensus => "consensus",
        }
    }
}

pub struct BatchOp {
    pub cf: ColumnFamily,
    pub key: Vec<u8>,
    pub value: Vec<u8>,
    pub op_type: BatchOpType,
}

pub enum BatchOpType {
    Put,
    Delete,
}
'''

with open(os.path.join(feature_rocks_dir, "Cargo.toml"), "w") as f:
    f.write(rocks_cargo)
with open(os.path.join(feature_rocks_dir, "src", "lib.rs"), "w") as f:
    f.write(rocks_lib)

# 4. FEATURE: hw-yubihsm
feature_yubi_dir = os.path.join(base_dir, "hw-yubihsm")
os.makedirs(os.path.join(feature_yubi_dir, "src"), exist_ok=True)

yubi_cargo = '''[package]
name = "safe-core-hw-yubihsm"
version = "0.1.0"
edition = "2021"

[features]
default = []
mock = []

[dependencies]
thiserror = "1.0"
serde = { version = "1.0", features = ["derive"] }
rand = "0.8"
sha2 = "0.10"

[dev-dependencies]
hex = "0.4"
'''

yubi_lib = '''//! Safe-Core YubiHSM Bridge
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum YubiHsmError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    #[error("Key not found: {0}")]
    KeyNotFound(String),
    #[error("Signing failed: {0}")]
    SigningFailed(String),
    #[error("YubiHSM not available: {0}")]
    NotAvailable(String),
    #[error("Mock mode: {0}")]
    MockMode(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YubiHsmConfig {
    pub connector_url: String,
    pub auth_key_id: u16,
    pub password: String,
    pub timeout_ms: u64,
}

impl Default for YubiHsmConfig {
    fn default() -> Self {
        Self {
            connector_url: "http://localhost:12345".to_string(),
            auth_key_id: 1,
            password: String::new(),
            timeout_ms: 5000,
        }
    }
}

#[derive(Debug, Clone)]
pub struct YubiHsmKeyHandle {
    pub key_id: u16,
    pub algorithm: String,
    pub public_key: Vec<u8>,
}

#[cfg(feature = "mock")]
pub struct YubiHsmMockClient {
    keys: std::collections::HashMap<u16, Vec<u8>>,
}

#[cfg(feature = "mock")]
impl YubiHsmMockClient {
    pub fn new() -> Self {
        Self { keys: std::collections::HashMap::new() }
    }
    pub fn connect(_config: &YubiHsmConfig) -> Result<Self, YubiHsmError> { Ok(Self::new()) }
    pub fn authenticate(&mut self, _config: &YubiHsmConfig) -> Result<(), YubiHsmError> { Ok(()) }
    pub fn generate_ed25519_key(&mut self, key_id: u16, _label: &str) -> Result<YubiHsmKeyHandle, YubiHsmError> {
        let mut rng = rand::thread_rng();
        let mut public_key = vec![0u8; 32];
        rand::RngCore::fill_bytes(&mut rng, &mut public_key);
        self.keys.insert(key_id, public_key.clone());
        Ok(YubiHsmKeyHandle { key_id, algorithm: "Ed25519".to_string(), public_key })
    }
    pub fn sign_ed25519(&mut self, key_id: u16, data: &[u8]) -> Result<Vec<u8>, YubiHsmError> {
        let _ = self.keys.get(&key_id)
            .ok_or_else(|| YubiHsmError::KeyNotFound(format!("Key {} not found", key_id)))?;
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.update(&key_id.to_le_bytes());
        let result = hasher.finalize();
        Ok(result.to_vec())
    }
}

#[cfg(not(any(feature = "mock")))]
pub struct YubiHsmClient;

#[cfg(not(any(feature = "mock")))]
impl YubiHsmClient {
    pub fn connect(_config: &YubiHsmConfig) -> Result<Self, YubiHsmError> {
        Err(YubiHsmError::NotAvailable("Enable 'yubihsm' or 'mock' feature".into()))
    }
}
'''

with open(os.path.join(feature_yubi_dir, "Cargo.toml"), "w") as f:
    f.write(yubi_cargo)
with open(os.path.join(feature_yubi_dir, "src", "lib.rs"), "w") as f:
    f.write(yubi_lib)

# 3. FEATURE: tpm-bridge
feature_tpm_dir = os.path.join(base_dir, "tpm-bridge")
os.makedirs(os.path.join(feature_tpm_dir, "src"), exist_ok=True)

tpm_cargo = '''[package]
name = "safe-core-tpm-bridge"
version = "0.1.0"
edition = "2021"

[features]
default = []

[dependencies]
thiserror = "1.0"
serde = { version = "1.0", features = ["derive"] }
'''

tpm_lib = '''//! Safe-Core TPM Bridge
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TpmError {
    #[error("TPM context creation failed: {0}")]
    ContextCreation(String),
    #[error("Key generation failed: {0}")]
    KeyGeneration(String),
    #[error("Signing failed: {0}")]
    SigningFailed(String),
    #[error("PCR read failed: {0}")]
    PcrReadFailed(String),
    #[error("TPM not available: {0}")]
    TpmNotAvailable(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TpmConfig {
    pub tcti: String,
    pub owner_auth: Vec<u8>,
    pub endorsement_auth: Vec<u8>,
}

impl Default for TpmConfig {
    fn default() -> Self {
        Self {
            tcti: "device:/dev/tpm0".to_string(),
            owner_auth: vec![],
            endorsement_auth: vec![],
        }
    }
}

#[derive(Debug, Clone)]
pub struct TpmKeyHandle {
    pub handle: u32,
    pub public_key: Vec<u8>,
    pub algorithm: String,
}

pub struct TpmBridge;

impl TpmBridge {
    pub fn new(_config: &TpmConfig) -> Result<Self, TpmError> {
        Err(TpmError::TpmNotAvailable("tss-esapi feature not enabled".into()))
    }
}
'''

with open(os.path.join(feature_tpm_dir, "Cargo.toml"), "w") as f:
    f.write(tpm_cargo)
with open(os.path.join(feature_tpm_dir, "src", "lib.rs"), "w") as f:
    f.write(tpm_lib)

# 2. FEATURE: dyn-signature
feature_sig_dir = os.path.join(base_dir, "dyn-signature")
os.makedirs(os.path.join(feature_sig_dir, "src"), exist_ok=True)

sig_cargo = '''[package]
name = "safe-core-dyn-signature"
version = "0.1.0"
edition = "2021"

[features]
default = ["p256", "ed25519"]
p256 = ["dep:p256", "dep:ecdsa", "dep:elliptic-curve"]
ed25519 = ["dep:ed25519-dalek"]

[dependencies]
p256 = { version = "0.13", optional = true, features = ["ecdsa", "pem"] }
ecdsa = { version = "0.16", optional = true }
elliptic-curve = { version = "0.13", optional = true, features = ["pkcs8"] }
ed25519-dalek = { version = "2.1", optional = true, features = ["rand_core"] }
rand = "0.8"
thiserror = "1.0"
serde = { version = "1.0", features = ["derive"] }

[dev-dependencies]
hex = "0.4"
'''

sig_lib = '''//! Safe-Core DynSignature
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
'''

with open(os.path.join(feature_sig_dir, "Cargo.toml"), "w") as f:
    f.write(sig_cargo)
with open(os.path.join(feature_sig_dir, "src", "lib.rs"), "w") as f:
    f.write(sig_lib)

# 1. FEATURE: hash-blake3
feature_blake3_dir = os.path.join(base_dir, "hash-blake3")
os.makedirs(os.path.join(feature_blake3_dir, "src"), exist_ok=True)

blake3_cargo = '''[package]
name = "safe-core-hash-blake3"
version = "0.1.0"
edition = "2021"

[features]
default = ["blake3"]
blake3 = ["dep:blake3"]
sha2-fallback = ["dep:sha2"]

[dependencies]
blake3 = { version = "1.8", optional = true }
sha2 = { version = "0.10", optional = true }
thiserror = "1.0"

[dev-dependencies]
hex = "0.4"
'''

blake3_lib = '''//! Safe-Core Hash
use thiserror::Error;

#[derive(Debug, Error)]
pub enum HashError {
    #[error("Hashing failed: {0}")]
    Internal(String),
}

pub trait Hasher: Send + Sync {
    fn update(&mut self, data: &[u8]);
    fn finalize(self) -> [u8; 32];
    fn hash(data: &[u8]) -> [u8; 32] where Self: Sized;
}

#[cfg(feature = "blake3")]
pub struct Blake3Hasher {
    state: blake3::Hasher,
}

#[cfg(feature = "blake3")]
impl Blake3Hasher {
    pub fn new() -> Self {
        Self { state: blake3::Hasher::new() }
    }
}

#[cfg(feature = "blake3")]
impl Hasher for Blake3Hasher {
    fn update(&mut self, data: &[u8]) {
        self.state.update(data);
    }
    fn finalize(self) -> [u8; 32] {
        self.state.finalize().into()
    }
    fn hash(data: &[u8]) -> [u8; 32] {
        blake3::hash(data).into()
    }
}
'''

with open(os.path.join(feature_blake3_dir, "Cargo.toml"), "w") as f:
    f.write(blake3_cargo)
with open(os.path.join(feature_blake3_dir, "src", "lib.rs"), "w") as f:
    f.write(blake3_lib)

# REATIVE GOVERNANCE
feature_gov_dir = os.path.join(base_dir, "safe-core-reactive-governance")
os.makedirs(os.path.join(feature_gov_dir, "src"), exist_ok=True)

gov_cargo = '''[package]
name = "safe-core-reactive-governance"
version = "0.1.0"
edition = "2021"

[dependencies]
safe-core-dyn-signature = { path = "../dyn-signature" }
safe-core-hash-blake3 = { path = "../hash-blake3" }
safe-core-hw-yubihsm = { path = "../hw-yubihsm", optional = true }

serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "1.0"
tracing = "0.1"
chrono = { version = "0.4", features = ["clock"] }
tokio = { version = "1.43", features = ["full"] }
prometheus = { version = "0.13", optional = true }
metrics = "0.23"
hex = "0.4"
async-trait = "0.1"

[dev-dependencies]
tempfile = "3.14"
rand = "0.8"

[features]
default = ["watchdog"]
watchdog = ["dep:prometheus"]
'''

gov_lib = '''//! Reactive Governance Module for Dark Bio + AGISAFE.
pub mod error;
pub mod governance;
pub mod reactive_log;
pub mod watchdog;
pub mod integration;

pub use governance::{GovernanceAction, GovernanceEntry};
pub use reactive_log::ReactiveLog;
pub use watchdog::GovernanceWatchdog;
pub use integration::{UedGovernance, SparseRouterGovernance};

pub trait HsmBackend: Send + Sync {
    fn sign(&self, key_id: &str, payload: &[u8]) -> Result<safe_core_dyn_signature::DynSignature, error::GovernanceError>;
    fn export_public_key(&self, key_id: &str) -> Result<safe_core_dyn_signature::DynPublicKey, error::GovernanceError>;
}
'''

gov_error = '''use thiserror::Error;
#[derive(Debug, Error)]
pub enum GovernanceError {
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Invalid signature: {0}")]
    InvalidSignature(String),
    #[error("Unauthorized issuer: {0}")]
    Unauthorized(String),
    #[error("Governance action not supported: {0}")]
    UnsupportedAction(String),
    #[error("Log Error: {0}")]
    Log(String)
}
pub type GovernanceResult<T> = Result<T, GovernanceError>;
'''

gov_governance = '''use safe_core_dyn_signature::{DynSignature, DynPublicKey, verify_dyn_signature};
use crate::error::{GovernanceError, GovernanceResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GovernanceAction {
    RollbackCurriculum {
        target_sth: Vec<u8>,
        reason: String,
    },
    AdjustTeacherReward {
        teacher_id: String,
        environment_hash: String,
        reward_delta: f64,
        reason: String,
    },
    BanRoutingPath {
        router_id: String,
        from_module: String,
        to_module: String,
        reason: String,
    },
    EmergencyFreeze {
        reason: String,
        duration_seconds: u64,
    },
    Unfreeze {
        reason: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceEntry {
    pub action: GovernanceAction,
    pub issued_by: String,
    pub timestamp: i64,
    pub signature: DynSignature,
    pub verifying_key: DynPublicKey,
}

impl GovernanceEntry {
    pub fn verify(&self) -> GovernanceResult<()> {
        // Use canonical JSON representation before signing
        let value = serde_json::to_value(&self.action)
            .map_err(|e| GovernanceError::Serialization(e.to_string()))?;

        let payload = serde_json::to_vec(&value)
             .map_err(|e| GovernanceError::Serialization(e.to_string()))?;

        verify_dyn_signature(&self.signature, &self.verifying_key, &payload)
            .map_err(|e| GovernanceError::InvalidSignature(e.to_string()))
    }
}
'''

gov_reactive_log = '''use safe_core_hash_blake3::Hasher;
use crate::governance::{GovernanceAction, GovernanceEntry};
use crate::error::{GovernanceError, GovernanceResult};
use safe_core_dyn_signature::DynPublicKey;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error};

pub struct TransparencyLog<H: Hasher> {
    entries: Arc<RwLock<Vec<LogEntry>>>,
    _phantom: std::marker::PhantomData<H>,
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub issuer: String,
    pub entry_type: String,
    pub timestamp: i64,
    pub payload: String,
    pub signature: Vec<u8>,
}

impl<H: Hasher> TransparencyLog<H> {
    pub fn new() -> Self {
        Self { entries: Arc::new(RwLock::new(Vec::new())), _phantom: std::marker::PhantomData }
    }
    pub async fn append(&self, issuer: &str, entry_type: &str, timestamp: i64, payload: &str, sig: &[u8]) -> Result<(), String> {
        let mut entries = self.entries.write().await;
        entries.push(LogEntry {
            issuer: issuer.to_string(),
            entry_type: entry_type.to_string(),
            timestamp,
            payload: payload.to_string(),
            signature: sig.to_vec(),
        });
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct GovernanceState {
    pub frozen: bool,
    pub banned_routes: HashMap<String, Vec<String>>,
    pub reward_adjustments: HashMap<String, f64>,
    pub last_rollback_sth: Option<Vec<u8>>,
}

pub struct ReactiveLog<H: Hasher> {
    inner: TransparencyLog<H>,
    state: Arc<RwLock<GovernanceState>>,
    authorized_keys: Vec<DynPublicKey>,
}

impl<H: Hasher> ReactiveLog<H> {
    pub fn new(
        log: TransparencyLog<H>,
        authorized_keys: Vec<DynPublicKey>,
    ) -> Self {
        Self {
            inner: log,
            state: Arc::new(RwLock::new(GovernanceState::default())),
            authorized_keys,
        }
    }

    pub async fn apply_governance_entry(&mut self, entry: GovernanceEntry) -> GovernanceResult<()> {
        entry.verify()?;
        if !self.authorized_keys.contains(&entry.verifying_key) {
            return Err(GovernanceError::Unauthorized(entry.issued_by.clone()));
        }

        let entry_data = serde_json::to_vec(&entry).map_err(|e| GovernanceError::Serialization(e.to_string()))?;

        self.inner.append(
            &entry.issued_by,
            "governance/action",
            entry.timestamp,
            &hex::encode(entry_data),
            &entry.signature.to_bytes(),
        ).await.map_err(|e| GovernanceError::Log(e))?;

        let mut state = self.state.write().await;
        match entry.action {
            GovernanceAction::RollbackCurriculum { target_sth, reason } => {
                state.last_rollback_sth = Some(target_sth);
                warn!(reason, "Rollback curriculum to STH");
            }
            GovernanceAction::AdjustTeacherReward { teacher_id, environment_hash, reward_delta, reason } => {
                let current = state.reward_adjustments.entry(teacher_id.clone()).or_insert(0.0);
                *current += reward_delta;
                warn!(teacher_id, environment_hash, reward_delta, reason, "Teacher reward adjusted");
            }
            GovernanceAction::BanRoutingPath { router_id, from_module, to_module, reason } => {
                let path = format!("{}->{}", from_module, to_module);
                state.banned_routes.entry(router_id.clone()).or_default().push(path);
                warn!(router_id, from_module, to_module, reason, "Routing path banned");
            }
            GovernanceAction::EmergencyFreeze { reason, duration_seconds } => {
                state.frozen = true;
                error!(reason, duration_seconds, "🚨 SYSTEM FROZEN");
                let state_clone = self.state.clone();
                tokio::spawn(async move {
                    tokio::time::sleep(tokio::time::Duration::from_secs(duration_seconds)).await;
                    let mut state = state_clone.write().await;
                    state.frozen = false;
                    info!("System unfrozen automatically after {} seconds", duration_seconds);
                });
            }
            GovernanceAction::Unfreeze { reason } => {
                state.frozen = false;
                info!(reason, "System unfrozen by governance action");
            }
        }
        Ok(())
    }

    pub async fn is_frozen(&self) -> bool {
        self.state.read().await.frozen
    }

    pub async fn is_route_banned(&self, router_id: &str, from_module: &str, to_module: &str) -> bool {
        let state = self.state.read().await;
        if let Some(banned) = state.banned_routes.get(router_id) {
            banned.contains(&format!("{}->{}", from_module, to_module))
        } else {
            false
        }
    }

    pub async fn get_teacher_reward_delta(&self, teacher_id: &str) -> f64 {
        self.state.read().await
            .reward_adjustments.get(teacher_id)
            .copied()
            .unwrap_or(0.0)
    }

    pub async fn get_last_rollback_sth(&self) -> Option<Vec<u8>> {
        self.state.read().await.last_rollback_sth.clone()
    }

    pub fn inner(&self) -> &TransparencyLog<H> {
        &self.inner
    }
}
'''

gov_watchdog = '''use crate::reactive_log::ReactiveLog;
use crate::governance::{GovernanceAction, GovernanceEntry};
use crate::error::GovernanceError;
use crate::HsmBackend;
use safe_core_hash_blake3::Hasher;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info};
use metrics::{gauge};

#[derive(Clone)]
pub struct WatchdogConfig {
    pub check_interval_secs: u64,
    pub consecutive_failures_threshold: u32,
    pub governance_key_id: String,
    pub governance_hsm: Arc<dyn HsmBackend>,
}

pub struct GovernanceWatchdog<H: Hasher> {
    log: Arc<tokio::sync::RwLock<ReactiveLog<H>>>,
    config: WatchdogConfig,
    consecutive_attestation_failures: u32,
}

#[derive(Default)]
struct MetricsSnapshot {
    attestation_trusted: f64,
    ued_teacher_failure_rate: f64,
}

impl<H: Hasher> GovernanceWatchdog<H> {
    pub fn new(log: Arc<tokio::sync::RwLock<ReactiveLog<H>>>, config: WatchdogConfig) -> Self {
        Self {
            log,
            config,
            consecutive_attestation_failures: 0,
        }
    }

    pub async fn run(&mut self) {
        let mut interval = tokio::time::interval(Duration::from_secs(self.config.check_interval_secs));
        loop {
            interval.tick().await;
            self.check_and_act().await;
        }
    }

    async fn check_and_act(&mut self) {
        let metrics = self.collect_metrics().await;
        if metrics.attestation_trusted == 0.0 {
            self.consecutive_attestation_failures += 1;
        } else {
            self.consecutive_attestation_failures = 0;
        }

        if self.consecutive_attestation_failures >= self.config.consecutive_failures_threshold {
            let action = GovernanceAction::EmergencyFreeze {
                reason: format!(
                    "Attestation failure for {} consecutive checks",
                    self.consecutive_attestation_failures
                ),
                duration_seconds: 300,
            };
            if let Err(e) = self.propose_governance(action).await {
                error!("Failed to propose governance action: {}", e);
            }
            self.consecutive_attestation_failures = 0;
        }

        if metrics.ued_teacher_failure_rate > 0.5 {
            let action = GovernanceAction::AdjustTeacherReward {
                teacher_id: "default-teacher".to_string(),
                environment_hash: "".to_string(),
                reward_delta: -0.2,
                reason: "High failure rate detected".to_string(),
            };
            if let Err(e) = self.propose_governance(action).await {
                error!("Failed to propose teacher reward adjustment: {}", e);
            }
        }
        gauge!("watchdog_attestation_failures").set(self.consecutive_attestation_failures as f64);
    }

    async fn collect_metrics(&self) -> MetricsSnapshot {
        let attestation = 1.0;
        let teacher_failure = 0.0;
        MetricsSnapshot {
            attestation_trusted: attestation,
            ued_teacher_failure_rate: teacher_failure,
        }
    }

    async fn propose_governance(&self, action: GovernanceAction) -> Result<(), GovernanceError> {
        let value = serde_json::to_value(&action)
            .map_err(|e| GovernanceError::Serialization(e.to_string()))?;
        let action_data = serde_json::to_vec(&value)
            .map_err(|e| GovernanceError::Serialization(e.to_string()))?;

        let signature = self.config.governance_hsm
            .sign(&self.config.governance_key_id, &action_data)?;
        let verifying_key = self.config.governance_hsm
            .export_public_key(&self.config.governance_key_id)?;
        let entry = GovernanceEntry {
            action,
            issued_by: "watchdog".to_string(),
            timestamp: chrono::Utc::now().timestamp(),
            signature,
            verifying_key,
        };
        info!("Watchdog proposing action: {:?}", entry);
        let mut log = self.log.write().await;
        log.apply_governance_entry(entry).await?;
        Ok(())
    }
}
'''

gov_integration = '''use crate::reactive_log::ReactiveLog;
use safe_core_hash_blake3::Hasher;

#[async_trait::async_trait]
pub trait UedGovernance<H: Hasher> {
    async fn is_frozen(&self) -> bool;
    async fn get_reward_adjustment(&self, teacher_id: &str) -> f64;
    async fn get_rollback_sth(&self) -> Option<Vec<u8>>;
}

#[async_trait::async_trait]
pub trait SparseRouterGovernance<H: Hasher> {
    async fn is_route_banned(&self, router_id: &str, from_module: &str, to_module: &str) -> bool;
    async fn is_frozen(&self) -> bool;
}

#[async_trait::async_trait]
impl<H: Hasher + Send + Sync> UedGovernance<H> for ReactiveLog<H> {
    async fn is_frozen(&self) -> bool {
        self.is_frozen().await
    }
    async fn get_reward_adjustment(&self, teacher_id: &str) -> f64 {
        self.get_teacher_reward_delta(teacher_id).await
    }
    async fn get_rollback_sth(&self) -> Option<Vec<u8>> {
        self.get_last_rollback_sth().await
    }
}

#[async_trait::async_trait]
impl<H: Hasher + Send + Sync> SparseRouterGovernance<H> for ReactiveLog<H> {
    async fn is_route_banned(&self, router_id: &str, from_module: &str, to_module: &str) -> bool {
        self.is_route_banned(router_id, from_module, to_module).await
    }
    async fn is_frozen(&self) -> bool {
        self.is_frozen().await
    }
}
'''

with open(os.path.join(feature_gov_dir, "Cargo.toml"), "w") as f: f.write(gov_cargo)
with open(os.path.join(feature_gov_dir, "src", "lib.rs"), "w") as f: f.write(gov_lib)
with open(os.path.join(feature_gov_dir, "src", "error.rs"), "w") as f: f.write(gov_error)
with open(os.path.join(feature_gov_dir, "src", "governance.rs"), "w") as f: f.write(gov_governance)
with open(os.path.join(feature_gov_dir, "src", "reactive_log.rs"), "w") as f: f.write(gov_reactive_log)
with open(os.path.join(feature_gov_dir, "src", "watchdog.rs"), "w") as f: f.write(gov_watchdog)
with open(os.path.join(feature_gov_dir, "src", "integration.rs"), "w") as f: f.write(gov_integration)
