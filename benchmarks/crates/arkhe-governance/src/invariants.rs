use chrono::{DateTime, Utc, Duration, Datelike, Timelike};
use serde::{Serialize, Deserialize};
use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, Ordering};

const REVOKE_WINDOW: std::time::Duration = std::time::Duration::from_secs(24 * 3600);

static ACTION_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionClass {
    Critical,
    Operational,
    Other,
}

impl ActionClass {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Critical => "Critical",
            Self::Operational => "Operational",
            Self::Other => "Other",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceAction {
    pub id: [u8; 32],
    pub class: ActionClass,
    pub description: String,
    pub proposer_did: String,
    pub created_at: DateTime<Utc>,
    pub requested_delay: Duration,
    pub votes_for: HashSet<String>,
    pub votes_against: HashSet<String>,
    pub action_hash: [u8; 32],
    pub revokes: Option<[u8; 32]>,
}

impl GovernanceAction {
    pub fn new(
        class: ActionClass,
        description: String,
        proposer_did: String,
        requested_delay: Duration,
        action_hash: [u8; 32],
    ) -> Self {
        let nonce = ACTION_COUNTER.fetch_add(1, Ordering::SeqCst);
        let mut hasher = blake3::Hasher::new();
        hasher.update(&nonce.to_le_bytes());
        hasher.update(class.name().as_bytes());
        hasher.update(proposer_did.as_bytes());
        hasher.update(&action_hash);

        Self {
            id: *hasher.finalize().as_bytes(),
            class,
            description,
            proposer_did,
            created_at: Utc::now(),
            requested_delay,
            votes_for: HashSet::new(),
            votes_against: HashSet::new(),
            action_hash,
            revokes: None,
        }
    }

    pub fn earliest_execution(&self) -> DateTime<Utc> {
        self.created_at + self.requested_delay
    }

    pub fn canonical_hash(&self) -> [u8; 32] {
        let bytes = serde_json::to_vec(self).expect("serialize");
        *blake3::hash(&bytes).as_bytes()
    }
}

#[derive(Debug, Clone)]
pub struct InvariantCheck {
    pub satisfied: bool,
    summary: String,
}

impl InvariantCheck {
    pub fn summary(&self) -> String {
        self.summary.clone()
    }
}

#[derive(Debug, Clone)]
pub struct ExecutedAction {
    pub id: [u8; 32],
    pub class: ActionClass,
    pub executed_at: DateTime<Utc>,
    pub action_hash: [u8; 32],
    pub result: crate::guard::ExecutionResult,
}

#[derive(Debug, Default)]
pub struct GovernanceInvariantChecker;

impl GovernanceInvariantChecker {
    pub fn check(&self, _action: &GovernanceAction) -> InvariantCheck {
        InvariantCheck { satisfied: true, summary: "OK".into() }
    }

    pub fn record_execution(&mut self, _action: &GovernanceAction, _result: crate::guard::ExecutionResult) {

    }

    pub fn check_revocation(&self, target: &ExecutedAction) -> Result<(), String> {
        let elapsed = Utc::now()
            .signed_duration_since(target.executed_at);

        let revoke_window = Duration::from_std(REVOKE_WINDOW)
            .expect("REVOKE_WINDOW must fit in i64 nanoseconds");

        if elapsed > revoke_window {
            return Err("Revocation window expired".into());
        }
        Ok(())
    }
}
