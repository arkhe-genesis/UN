//! Trait de Hash Abstrato — Princípio da Independência Semântica
//!
//! O núcleo semântico não conhece algoritmos criptográficos.
//! Implementações concretas (blake3, sha256, keccak256) vivem em crates separados.

pub trait Hasher: Send + Sync {
    fn hash(&self, content: &[u8]) -> [u8; 32];
}

/// Hasher de identidade (para testes e modelagem)
pub struct IdentityHasher;
impl Hasher for IdentityHasher {
    fn hash(&self, content: &[u8]) -> [u8; 32] {
        let mut out = [0u8; 32];
        let len = content.len().min(32);
        out[..len].copy_from_slice(&content[..len]);
        out
    }
}

/// Hasher que usa uma função de hash externa via closure
pub struct FnHasher<F>(pub F);
impl<F: Fn(&[u8]) -> [u8; 32] + Send + Sync> Hasher for FnHasher<F> {
    fn hash(&self, content: &[u8]) -> [u8; 32] {
        (self.0)(content)
    }
}
