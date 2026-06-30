//! Safe-Core Merkle Tree — Implementação otimizada
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
