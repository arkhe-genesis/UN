//! GovernanceGuard — enforce do I_gov^v2 no ciclo de vida das propostas.
//!
//! # Design Choice: Síncrono
//!
//! O GovernanceGuard usa std::sync::Mutex por design — o TCB (Trusted
//! Computing Base) deve ser mínimo e não depende de runtime async.
//! Para uso em contexto Tokio/async, envolva as chamadas em
//! tokio::task::spawn_blocking ou use AsyncGovernanceGuard.
//!
//! # AsyncGovernanceGuard
//!
//! Veja crate::async_guard::AsyncGovernanceGuard para versão async.

use std::sync::Mutex;
use crate::invariants::{GovernanceAction, GovernanceInvariantChecker, ExecutedAction};
use crate::safe_core::SafeCoreHook;

#[derive(Debug, thiserror::Error, Clone)]
pub enum GuardError {
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
    #[error("Cancellation denied: {0}")]
    CancellationDenied(String),
    #[error("Not found: {0}")]
    NotFound(String),
}

#[derive(Debug, Clone)]
pub enum ExecutionResult {
    Success,
    Rejected(String),
    Cancelled,
}

pub struct GovernanceGuard {
    checker: Mutex<GovernanceInvariantChecker>,
    pending: Mutex<Vec<GovernanceAction>>,
    executed: Mutex<Vec<ExecutedAction>>,
    hooks: Mutex<Vec<Box<dyn SafeCoreHook>>>,
}

impl Default for GovernanceGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl GovernanceGuard {
    pub fn new() -> Self {
        Self {
            checker: Mutex::new(GovernanceInvariantChecker::default()),
            pending: Mutex::new(Vec::new()),
            executed: Mutex::new(Vec::new()),
            hooks: Mutex::new(Vec::new()),
        }
    }

    pub fn with_checker(checker: GovernanceInvariantChecker) -> Self {
        Self {
            checker: Mutex::new(checker),
            pending: Mutex::new(Vec::new()),
            executed: Mutex::new(Vec::new()),
            hooks: Mutex::new(Vec::new()),
        }
    }

    pub fn checker(&self) -> std::sync::MutexGuard<GovernanceInvariantChecker> {
        self.checker.lock().unwrap()
    }

    pub fn add_hook(&self, hook: Box<dyn SafeCoreHook>) {
        self.hooks.lock().unwrap().push(hook);
    }

    fn run_pre_submit_hooks(&self, action: &GovernanceAction) -> Result<(), GuardError> {
        let hooks = self.hooks.lock().unwrap();
        for hook in hooks.iter() {
            if let Err(e) = hook.pre_submit(action) {
                return Err(GuardError::CancellationDenied(e.to_string()));
            }
        }
        Ok(())
    }

    pub fn submit(&self, action: GovernanceAction) -> Result<String, GuardError> {
        self.run_pre_submit_hooks(&action)?;
        let id_hex = hex::encode(action.id);
        self.pending.lock().unwrap().push(action);
        Ok(id_hex)
    }

    pub fn execute<F, R>(&self, proposal_id: &str, action_fn: F) -> Result<R, GuardError>
    where
        F: FnOnce(&GovernanceAction) -> Result<R, String>,
    {
        let pending = self.pending.lock().unwrap().clone();
        let proposal = pending
            .iter()
            .find(|p| hex::encode(p.id) == proposal_id)
            .ok_or_else(|| GuardError::NotFound(proposal_id.to_string()))?
            .clone();

        let action_result = action_fn(&proposal);
        let success = action_result.is_ok();

        let execution_result = if success {
            ExecutionResult::Success
        } else {
            let err_msg = match &action_result {
                Err(e) => e.clone(),
                _ => unreachable!(),
            };
            ExecutionResult::Rejected(err_msg)
        };

        {
            let mut checker = self.checker.lock().unwrap();
            checker.record_execution(&proposal, execution_result.clone());
        }

        self.executed.lock().unwrap().push(ExecutedAction {
            id: proposal.id,
            class: proposal.class,
            executed_at: chrono::Utc::now(),
            action_hash: proposal.action_hash,
            result: execution_result,
        });

        action_result.map_err(GuardError::ExecutionFailed)
    }

    pub fn cancel(&self, proposal_id: &str, cancellation: &GovernanceAction) -> Result<(), GuardError> {
        let check = self.checker.lock().unwrap().check(cancellation);
        if !check.satisfied {
            return Err(GuardError::CancellationDenied(check.summary()));
        }

        let mut pending = self.pending.lock().unwrap();
        let pos = pending
            .iter()
            .position(|p| hex::encode(p.id) == proposal_id)
            .ok_or_else(|| GuardError::NotFound(proposal_id.to_string()))?;

        let target = pending[pos].clone();

        // Mocking an ExecutedAction to test revocation
        let simulated_target = ExecutedAction {
            id: target.id,
            class: target.class,
            executed_at: target.created_at, // rough proxy
            action_hash: target.action_hash,
            result: ExecutionResult::Success,
        };

        if let Err(e) = self.checker.lock().unwrap().check_revocation(&simulated_target) {
            return Err(GuardError::CancellationDenied(e.to_string()));
        }

        pending.remove(pos);
        Ok(())
    }

    pub fn pending_proposals(&self) -> Vec<GovernanceAction> {
        self.pending.lock().unwrap().clone()
    }

    pub fn executed_proposals(&self) -> Vec<ExecutedAction> {
        self.executed.lock().unwrap().clone()
    }

    pub fn audit_hash(&self) -> [u8; 32] {
        [0u8; 32] // placeholder
    }
}
