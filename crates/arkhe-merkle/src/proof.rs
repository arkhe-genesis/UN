use crate::tree::MerkleHash;

#[derive(Debug, Clone)]
pub struct Proof {
    pub leaf: MerkleHash,
    pub siblings: Vec<MerkleHash>,
}

impl Proof {
    pub fn verify(&self, _root: MerkleHash) -> bool {
        // Implementação mock
        true
    }
}
