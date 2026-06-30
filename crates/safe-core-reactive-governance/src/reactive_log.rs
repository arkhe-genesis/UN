use safe_core_hash_blake3::Hasher;
use crate::governance::{GovernanceAction, GovernanceEntry};
use crate::error::{GovernanceError, GovernanceResult};
use safe_core_dyn_signature::DynPublicKey;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error};

pub struct TransparencyLog<H: Hasher> {
    entries: Arc<RwLock<Vec<LogEntry>>>,
    _phantom: std::marker::PhantomData<H>,
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub issuer: String,
    pub entry_type: String,
    pub timestamp: i64,
    pub payload: String,
    pub signature: Vec<u8>,
}

impl<H: Hasher> TransparencyLog<H> {
    pub fn new() -> Self {
        Self { entries: Arc::new(RwLock::new(Vec::new())), _phantom: std::marker::PhantomData }
    }
    pub async fn append(&self, issuer: &str, entry_type: &str, timestamp: i64, payload: &str, sig: &[u8]) -> Result<(), String> {
        let mut entries = self.entries.write().await;
        entries.push(LogEntry {
            issuer: issuer.to_string(),
            entry_type: entry_type.to_string(),
            timestamp,
            payload: payload.to_string(),
            signature: sig.to_vec(),
        });
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct GovernanceState {
    pub frozen: bool,
    pub banned_routes: HashMap<String, Vec<String>>,
    pub reward_adjustments: HashMap<String, f64>,
    pub last_rollback_sth: Option<Vec<u8>>,
}

pub struct ReactiveLog<H: Hasher> {
    inner: TransparencyLog<H>,
    state: Arc<RwLock<GovernanceState>>,
    authorized_keys: Vec<DynPublicKey>,
}

impl<H: Hasher> ReactiveLog<H> {
    pub fn new(
        log: TransparencyLog<H>,
        authorized_keys: Vec<DynPublicKey>,
    ) -> Self {
        Self {
            inner: log,
            state: Arc::new(RwLock::new(GovernanceState::default())),
            authorized_keys,
        }
    }

    pub async fn apply_governance_entry(&mut self, entry: GovernanceEntry) -> GovernanceResult<()> {
        entry.verify()?;
        if !self.authorized_keys.contains(&entry.verifying_key) {
            return Err(GovernanceError::Unauthorized(entry.issued_by.clone()));
        }

        let entry_data = serde_json::to_vec(&entry).map_err(|e| GovernanceError::Serialization(e.to_string()))?;

        self.inner.append(
            &entry.issued_by,
            "governance/action",
            entry.timestamp,
            &hex::encode(entry_data),
            &entry.signature.to_bytes(),
        ).await.map_err(|e| GovernanceError::Log(e))?;

        let mut state = self.state.write().await;
        match entry.action {
            GovernanceAction::RollbackCurriculum { target_sth, reason } => {
                state.last_rollback_sth = Some(target_sth);
                warn!(reason, "Rollback curriculum to STH");
            }
            GovernanceAction::AdjustTeacherReward { teacher_id, environment_hash, reward_delta, reason } => {
                let current = state.reward_adjustments.entry(teacher_id.clone()).or_insert(0.0);
                *current += reward_delta;
                warn!(teacher_id, environment_hash, reward_delta, reason, "Teacher reward adjusted");
            }
            GovernanceAction::BanRoutingPath { router_id, from_module, to_module, reason } => {
                let path = format!("{}->{}", from_module, to_module);
                state.banned_routes.entry(router_id.clone()).or_default().push(path);
                warn!(router_id, from_module, to_module, reason, "Routing path banned");
            }
            GovernanceAction::EmergencyFreeze { reason, duration_seconds } => {
                state.frozen = true;
                error!(reason, duration_seconds, "🚨 SYSTEM FROZEN");
                let state_clone = self.state.clone();
                tokio::spawn(async move {
                    tokio::time::sleep(tokio::time::Duration::from_secs(duration_seconds)).await;
                    let mut state = state_clone.write().await;
                    state.frozen = false;
                    info!("System unfrozen automatically after {} seconds", duration_seconds);
                });
            }
            GovernanceAction::Unfreeze { reason } => {
                state.frozen = false;
                info!(reason, "System unfrozen by governance action");
            }
        }
        Ok(())
    }

    pub async fn is_frozen(&self) -> bool {
        self.state.read().await.frozen
    }

    pub async fn is_route_banned(&self, router_id: &str, from_module: &str, to_module: &str) -> bool {
        let state = self.state.read().await;
        if let Some(banned) = state.banned_routes.get(router_id) {
            banned.contains(&format!("{}->{}", from_module, to_module))
        } else {
            false
        }
    }

    pub async fn get_teacher_reward_delta(&self, teacher_id: &str) -> f64 {
        self.state.read().await
            .reward_adjustments.get(teacher_id)
            .copied()
            .unwrap_or(0.0)
    }

    pub async fn get_last_rollback_sth(&self) -> Option<Vec<u8>> {
        self.state.read().await.last_rollback_sth.clone()
    }

    pub fn inner(&self) -> &TransparencyLog<H> {
        &self.inner
    }
}
