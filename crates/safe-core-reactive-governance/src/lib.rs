//! Reactive Governance Module for Dark Bio + AGISAFE.
pub mod error;
pub mod governance;
pub mod reactive_log;
pub mod watchdog;
pub mod integration;

pub use governance::{GovernanceAction, GovernanceEntry};
pub use reactive_log::ReactiveLog;
pub use watchdog::GovernanceWatchdog;
pub use integration::{UedGovernance, SparseRouterGovernance};

pub trait HsmBackend: Send + Sync {
    fn sign(&self, key_id: &str, payload: &[u8]) -> Result<safe_core_dyn_signature::DynSignature, error::GovernanceError>;
    fn export_public_key(&self, key_id: &str) -> Result<safe_core_dyn_signature::DynPublicKey, error::GovernanceError>;
}
