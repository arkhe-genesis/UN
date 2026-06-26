pub mod guard;
pub mod invariants;
pub mod flock;
pub mod safe_core;
pub mod async_guard;

pub use guard::{GovernanceGuard, GuardError, ExecutionResult};
pub use invariants::{GovernanceAction, GovernanceInvariantChecker, ActionClass, ExecutedAction};
pub use async_guard::AsyncGovernanceGuard;
pub use flock::FlockConfig;

// Provide GovernanceProposal as alias to GovernanceAction if needed by nexus
pub type GovernanceProposal = GovernanceAction;
pub type GovernanceError = GuardError;

#[derive(Debug, thiserror::Error)]
#[error("Governance violation: {0}")]
pub struct GovernanceViolation(pub String);

// For error mapping compatibility
impl From<GovernanceViolation> for GuardError {
    fn from(v: GovernanceViolation) -> Self {
        GuardError::CancellationDenied(v.0)
    }
}

// Ensure execution result matches the one expected by nexus
impl ExecutionResult {
    pub fn is_success(&self) -> bool {
        matches!(self, ExecutionResult::Success)
    }
}
