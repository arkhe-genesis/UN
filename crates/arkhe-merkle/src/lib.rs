pub mod error;
pub mod proof;
pub mod tree;

#[cfg(test)]
mod tests {
    use crate::tree::MerkleTree;

    #[test]
    fn test_append() {
        let mut tree = MerkleTree::new();
        let hash1 = [1u8; 32];
        let hash2 = [2u8; 32];
        tree.append_leaf(hash1);
        tree.append_leaf(hash2);
    }
}
