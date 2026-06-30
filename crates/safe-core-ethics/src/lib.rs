//! Motor de Ética para Agentes Autônomos

pub mod engine;
pub mod rule;
pub mod verdict;

pub use engine::{EthicsEngine, Lean4Verifier};
pub use rule::{EthicsRule, Severity};
pub use verdict::EthicsVerdict;

#[derive(Debug, thiserror::Error)]
pub enum EthicsError {
    #[error("Regra não encontrada: {0}")]
    RuleNotFound(String),
    #[error("Ação desconhecida: {0}")]
    UnknownAction(String),
    #[error("Erro de validação: {0}")]
    Validation(String),
    #[error("Erro Lean4: {0}")]
    Lean4(String),
}
