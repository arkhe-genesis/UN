#[derive(Debug, PartialEq, Eq)]
pub enum MerkleError {
    EmptyTree,
    InvalidProof,
}
