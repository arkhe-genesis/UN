//! Benchmark comparativo de implementações de Merkle Tree
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
