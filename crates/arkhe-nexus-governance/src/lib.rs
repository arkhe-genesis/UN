//! Bridge entre GovernanceGuard e NEXUS/Safe Core.
//!
//! Esta crate fornece adaptadores que permitem ao NEXUS/Safe Core
//! utilizar o GovernanceGuard para enforce do invariante I_gov
//! em todas as ações administrativas.

pub mod adapters;
pub mod admin_actions;
pub mod migration;

pub use adapters::*;
pub use admin_actions::*;
