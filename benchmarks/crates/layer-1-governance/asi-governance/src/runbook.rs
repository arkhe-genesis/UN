use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{info, warn, error, instrument};

/// Severidade de incidente
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    Sev3,
    Sev2,
    Sev1,
}

impl Severity {
    pub fn response_sla(&self) -> Duration {
        match self {
            Severity::Sev1 => Duration::from_secs(15 * 60),
            Severity::Sev2 => Duration::from_secs(60 * 60),
            Severity::Sev3 => Duration::from_secs(480 * 60),
        }
    }

    pub fn resolution_sla(&self) -> Duration {
        match self {
            Severity::Sev1 => Duration::from_secs(4 * 3600),
            Severity::Sev2 => Duration::from_secs(24 * 3600),
            Severity::Sev3 => Duration::from_secs(72 * 3600),
        }
    }
}

/// Fase do response playbook
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IncidentPhase {
    Detected,
    Triaged,
    Mitigated,
    Resolved,
    Closed,
}

/// Incidente ativo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Incident {
    pub id: String,
    pub severity: Severity,
    pub phase: IncidentPhase,
    pub invariant_id: String,
    pub description: String,
    pub detected_at: DateTime<Utc>,
    pub triaged_at: Option<DateTime<Utc>>,
    pub mitigated_at: Option<DateTime<Utc>>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub closed_at: Option<DateTime<Utc>>,
    pub assignee: Option<String>,
    pub actions: Vec<IncidentAction>,
    pub root_cause: Option<String>,
    pub slack_channel: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncidentAction {
    pub timestamp: DateTime<Utc>,
    pub actor: String,       // "system" ou email do humano
    pub action: String,
    pub result: ActionResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionResult {
    Pending,
    Success,
    Failed { reason: String },
}

/// Configuração de escalation de um runbook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationLevel {
    pub level: u8,
    pub role: String,
    pub action: String,
    pub sla_minutes: u64,
    pub auto_action: String,
    pub notification_channels: Vec<String>,
}

/// Definição completa de um runbook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Runbook {
    pub id: String,
    pub trigger_condition: String,
    pub invariant_ids: Vec<String>,
    pub escalation_path: Vec<EscalationLevel>,
    pub recovery_steps: Vec<String>,
    pub prevention_measures: Vec<String>,
    pub severity: Severity,
}

/// Motor de execução de runbooks
pub struct RunbookEngine {
    runbooks: HashMap<String, Runbook>,
    active_incidents: Arc<RwLock<HashMap<String, Incident>>>,
    notifier: Arc<dyn IncidentNotifier>,
    circuit_breaker: Arc<dyn CircuitBreaker>,
}

#[async_trait::async_trait]
pub trait IncidentNotifier: Send + Sync {
    async fn notify(&self, channel: &str, message: &str) -> Result<(), NotifyError>;
    async fn create_channel(&self, name: &str) -> Result<String, NotifyError>;
}

#[derive(Debug, thiserror::Error)]
pub enum NotifyError {
    #[error("Slack API error: {0}")]
    SlackApi(String),
    #[error("PagerDuty error: {0}")]
    PagerDuty(String),
    #[error("Email error: {0}")]
    Email(String),
}

#[async_trait::async_trait]
pub trait CircuitBreaker: Send + Sync {
    async fn trip(&self, endpoint: &str) -> bool;
    async fn reset(&self, endpoint: &str) -> bool;
    async fn is_open(&self, endpoint: &str) -> bool;
}

impl RunbookEngine {
    pub fn new(
        runbooks: HashMap<String, Runbook>,
        notifier: Arc<dyn IncidentNotifier>,
        circuit_breaker: Arc<dyn CircuitBreaker>,
    ) -> Self {
        Self {
            runbooks,
            active_incidents: Arc::new(RwLock::new(HashMap::new())),
            notifier,
            circuit_breaker,
        }
    }

    /// Dispara runbook quando invariante é violada
    #[instrument(skip(self), fields(runbook_id = %runbook_id))]
    pub async fn trigger(
        &self,
        runbook_id: &str,
        invariant_id: &str,
        description: &str,
        violating_value: &str,
    ) -> Result<Incident, RunbookError> {
        let runbook = self.runbooks.get(runbook_id)
            .ok_or(RunbookError::RunbookNotFound(runbook_id.to_string()))?;

        let incident_id = format!("INC-{}-{}",
            Utc::now().format("%Y%m%d"),
            uuid::Uuid::new_v4().to_string()[..8].to_string()
        );

        // Criar canal Slack
        let channel_name = format!("inc-{}", incident_id.to_lowercase().replace("_", "-"));
        let slack_channel = match self.notifier.create_channel(&channel_name).await {
            Ok(ch) => Some(ch),
            Err(e) => {
                warn!(error = %e, "Failed to create Slack channel, continuing");
                None
            }
        };

        let mut incident = Incident {
            id: incident_id.clone(),
            severity: runbook.severity,
            phase: IncidentPhase::Detected,
            invariant_id: invariant_id.to_string(),
            description: description.to_string(),
            detected_at: Utc::now(),
            triaged_at: None,
            mitigated_at: None,
            resolved_at: None,
            closed_at: None,
            assignee: None,
            actions: Vec::new(),
            root_cause: None,
            slack_channel: slack_channel.clone(),
        };

        // ── PHASE 1: DETECT ──
        let notify_msg = format!(
            "🚨 **{}** | Invariante: {} | {}\nValor: {}\nSLA Resposta: {}min",
            incident.id,
            invariant_id,
            description,
            violating_value,
            runbook.severity.response_sla().as_secs() / 60
        );

        if let Some(ref ch) = slack_channel {
            let _ = self.notifier.notify(ch, &notify_msg).await;
        }

        // Notificar canais de severidade
        for level in &runbook.escalation_path {
            if level.level == 1 {
                for channel in &level.notification_channels {
                    let _ = self.notifier.notify(channel, &notify_msg).await;
                }
            }
        }

        // ── AUTO-MITIGATION para SEV-1 ──
        if runbook.severity == Severity::Sev1 {
            let auto_action = runbook.escalation_path.first()
                .map(|l| l.auto_action.as_str())
                .unwrap_or("No auto-action defined");

            let result = self.circuit_breaker.trip("agi-endpoint").await;
            incident.actions.push(IncidentAction {
                timestamp: Utc::now(),
                actor: "system".to_string(),
                action: format!("Auto-mitigation: {} | Circuit breaker: {}", auto_action, if result { "TRIPPED" } else { "FAILED" }),
                result: if result { ActionResult::Success } else { ActionResult::Failed { reason: "Circuit breaker failed".to_string() } },
            });
            incident.phase = IncidentPhase::Mitigated;
            incident.mitigated_at = Some(Utc::now());
        }

        // Armazenar incidente ativo
        self.active_incidents.write().await.insert(incident_id.clone(), incident.clone());

        info!(
            incident_id = %incident.id,
            severity = ?incident.severity,
            phase = ?incident.phase,
            "Incident created and processed"
        );

        Ok(incident)
    }

    /// Escala incidente se SLA de resposta foi excedido
    pub async fn check_escalation(&self) -> Vec<EscalationAction> {
        let incidents = self.active_incidents.read().await;
        let mut actions = Vec::new();

        for (id, incident) in incidents.iter() {
            if incident.phase != IncidentPhase::Detected {
                continue;
            }

            let elapsed = Utc::now() - incident.detected_at;
            let sla = incident.severity.response_sla();

            if elapsed > chrono::Duration::from_std(sla).unwrap_or_default() {
                if let Some(runbook_id) = self.find_runbook_for_invariant(&incident.invariant_id) {
                    if let Some(runbook) = self.runbooks.get(&runbook_id) {
                        let current_level = incident.actions.len() as u8 + 1;
                        if let Some(next_level) = runbook.escalation_path.iter()
                            .find(|l| l.level == current_level)
                        {
                            actions.push(EscalationAction {
                                incident_id: id.clone(),
                                escalate_to: next_level.role.clone(),
                                action: next_level.action.clone(),
                                channels: next_level.notification_channels.clone(),
                            });
                        }
                    }
                }
            }
        }

        actions
    }

    fn find_runbook_for_invariant(&self, invariant_id: &str) -> Option<String> {
        for (id, runbook) in &self.runbooks {
            if runbook.invariant_ids.iter().any(|i| i == invariant_id) {
                return Some(id.clone());
            }
        }
        None
    }

    /// Lista incidentes ativos com filtros
    pub async fn list_incidents(
        &self,
        severity: Option<Severity>,
        phase: Option<IncidentPhase>,
    ) -> Vec<Incident> {
        let incidents = self.active_incidents.read().await;
        incidents.values()
            .filter(|i| {
                severity.map_or(true, |s| i.severity == s)
                    && phase.map_or(true, |p| i.phase == p)
            })
            .cloned()
            .collect()
    }

    /// Resolve incidente
    #[instrument(skip(self))]
    pub async fn resolve(
        &self,
        incident_id: &str,
        root_cause: &str,
        resolver: &str,
    ) -> Result<Incident, RunbookError> {
        let mut incidents = self.active_incidents.write().await;
        let incident = incidents.get_mut(incident_id)
            .ok_or(RunbookError::IncidentNotFound(incident_id.to_string()))?;

        incident.phase = IncidentPhase::Resolved;
        incident.resolved_at = Some(Utc::now());
        incident.root_cause = Some(root_cause.to_string());
        incident.assignee = Some(resolver.to_string());

        incident.actions.push(IncidentAction {
            timestamp: Utc::now(),
            actor: resolver.to_string(),
            action: format!("Resolved. Root cause: {}", root_cause),
            result: ActionResult::Success,
        });

        // Reset circuit breaker se estava tripado
        let _ = self.circuit_breaker.reset("agi-endpoint").await;

        Ok(incident.clone())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationAction {
    pub incident_id: String,
    pub escalate_to: String,
    pub action: String,
    pub channels: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum RunbookError {
    #[error("Runbook not found: {0}")]
    RunbookNotFound(String),
    #[error("Incident not found: {0}")]
    IncidentNotFound(String),
    #[error("Notification failed: {0}")]
    NotificationFailed(NotifyError),
}
