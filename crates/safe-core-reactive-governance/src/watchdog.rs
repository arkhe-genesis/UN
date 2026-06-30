use crate::reactive_log::ReactiveLog;
use crate::governance::{GovernanceAction, GovernanceEntry};
use crate::error::GovernanceError;
use crate::HsmBackend;
use safe_core_hash_blake3::Hasher;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info};
use metrics::{gauge};

#[derive(Clone)]
pub struct WatchdogConfig {
    pub check_interval_secs: u64,
    pub consecutive_failures_threshold: u32,
    pub governance_key_id: String,
    pub governance_hsm: Arc<dyn HsmBackend>,
}

pub struct GovernanceWatchdog<H: Hasher> {
    log: Arc<tokio::sync::RwLock<ReactiveLog<H>>>,
    config: WatchdogConfig,
    consecutive_attestation_failures: u32,
}

#[derive(Default)]
struct MetricsSnapshot {
    attestation_trusted: f64,
    ued_teacher_failure_rate: f64,
}

impl<H: Hasher> GovernanceWatchdog<H> {
    pub fn new(log: Arc<tokio::sync::RwLock<ReactiveLog<H>>>, config: WatchdogConfig) -> Self {
        Self {
            log,
            config,
            consecutive_attestation_failures: 0,
        }
    }

    pub async fn run(&mut self) {
        let mut interval = tokio::time::interval(Duration::from_secs(self.config.check_interval_secs));
        loop {
            interval.tick().await;
            self.check_and_act().await;
        }
    }

    async fn check_and_act(&mut self) {
        let metrics = self.collect_metrics().await;
        if metrics.attestation_trusted == 0.0 {
            self.consecutive_attestation_failures += 1;
        } else {
            self.consecutive_attestation_failures = 0;
        }

        if self.consecutive_attestation_failures >= self.config.consecutive_failures_threshold {
            let action = GovernanceAction::EmergencyFreeze {
                reason: format!(
                    "Attestation failure for {} consecutive checks",
                    self.consecutive_attestation_failures
                ),
                duration_seconds: 300,
            };
            if let Err(e) = self.propose_governance(action).await {
                error!("Failed to propose governance action: {}", e);
            }
            self.consecutive_attestation_failures = 0;
        }

        if metrics.ued_teacher_failure_rate > 0.5 {
            let action = GovernanceAction::AdjustTeacherReward {
                teacher_id: "default-teacher".to_string(),
                environment_hash: "".to_string(),
                reward_delta: -0.2,
                reason: "High failure rate detected".to_string(),
            };
            if let Err(e) = self.propose_governance(action).await {
                error!("Failed to propose teacher reward adjustment: {}", e);
            }
        }
        gauge!("watchdog_attestation_failures").set(self.consecutive_attestation_failures as f64);
    }

    async fn collect_metrics(&self) -> MetricsSnapshot {
        let attestation = 1.0;
        let teacher_failure = 0.0;
        MetricsSnapshot {
            attestation_trusted: attestation,
            ued_teacher_failure_rate: teacher_failure,
        }
    }

    async fn propose_governance(&self, action: GovernanceAction) -> Result<(), GovernanceError> {
        let value = serde_json::to_value(&action)
            .map_err(|e| GovernanceError::Serialization(e.to_string()))?;
        let action_data = serde_json::to_vec(&value)
            .map_err(|e| GovernanceError::Serialization(e.to_string()))?;

        let signature = self.config.governance_hsm
            .sign(&self.config.governance_key_id, &action_data)?;
        let verifying_key = self.config.governance_hsm
            .export_public_key(&self.config.governance_key_id)?;
        let entry = GovernanceEntry {
            action,
            issued_by: "watchdog".to_string(),
            timestamp: chrono::Utc::now().timestamp(),
            signature,
            verifying_key,
        };
        info!("Watchdog proposing action: {:?}", entry);
        let mut log = self.log.write().await;
        log.apply_governance_entry(entry).await?;
        Ok(())
    }
}
