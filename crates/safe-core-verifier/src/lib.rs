pub mod lean4;
pub mod constraint;

pub use lean4::Lean4Verifier;
pub use constraint::{Constraint, ConstraintResult};

pub trait Verifier: Send + Sync {
    fn verify(&self, constraint: &Constraint, context: &serde_json::Value) -> ConstraintResult;
}
