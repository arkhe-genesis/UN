//! AsyncGovernanceGuard — versão async do GovernanceGuard.
//!
//! ✅ P9: Fornece alternativa async para contextos Tokio.

use crate::guard::{GovernanceGuard, GuardError};
use crate::invariants::{GovernanceAction};

/// Versão async do GovernanceGuard.
pub struct AsyncGovernanceGuard {
    inner: tokio::sync::Mutex<GovernanceGuard>,
}

impl Default for AsyncGovernanceGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl AsyncGovernanceGuard {
    pub fn new() -> Self {
        Self {
            inner: tokio::sync::Mutex::new(GovernanceGuard::new()),
        }
    }

    pub async fn submit(&self, action: GovernanceAction) -> Result<String, GuardError> {
        let guard = self.inner.lock().await;
        guard.submit(action)
    }

    pub async fn execute<F, R>(
        &self,
        proposal_id: &str,
        action: F,
    ) -> Result<R, GuardError>
    where
        F: FnOnce(&GovernanceAction) -> Result<R, String>,
    {
        let guard = self.inner.lock().await;
        guard.execute(proposal_id, action)
    }
}
