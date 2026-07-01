use arkhe_merkle::tree::MerkleTree;

fn main() {
    let mut tree = MerkleTree::new();
    let hash1 = [1u8; 32];
    let hash2 = [2u8; 32];
    tree.append_leaf(hash1);
    tree.append_leaf(hash2);
    println!("OK");
}
