use crate::ethics::{EthicsEngine, Lean4Verifier, EthicsRule, EthicsVerdict};
use crate::persistence::{StateRepository, RepositoryError};
use crate::verifier::{Verifier, Constraint, ConstraintResult};
use crate::audit::{AuditTrail, AuditEvent, EventType};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

pub struct GovernanceEngine {
    pub ethics: Arc<RwLock<dyn EthicsEngine>>,
    pub repository: Arc<StateRepository>,
    pub verifier: Arc<dyn Verifier>,
    pub audit: Arc<RwLock<AuditTrail>>,
}

impl GovernanceEngine {
    pub async fn new(database_url: &str) -> Result<Self, GovernanceError> {
        let repository = Arc::new(StateRepository::new(database_url).await?);
        let rules = repository.load_all_rules().await?;
        let mut ethics = Lean4Verifier::new(None);
        ethics.load_rules(rules).await?;
        let ethics = Arc::new(RwLock::new(ethics));
        let verifier = Arc::new(SimpleVerifier);
        let audit = Arc::new(RwLock::new(AuditTrail::new()));
        info!("GovernanceEngine inicializada");
        Ok(Self { ethics, repository, verifier, audit })
    }

    pub async fn enforce_action(
        &self,
        action: &str,
        context: &serde_json::Value,
        agent_id: Option<&str>,
    ) -> Result<EthicsVerdict, GovernanceError> {
        let engine = self.ethics.read().await;
        let verdict = engine.evaluate(action, context).await?;

        let event = AuditEvent {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            event_type: EventType::ActionEvaluated,
            action: action.to_string(),
            verdict: format!("{:?}", verdict.verdict),
            rule_id: verdict.rule_id.clone(),
            agent_id: agent_id.map(|s| s.to_string()),
            signature: None,
        };
        let mut audit = self.audit.write().await;
        audit.push(event)?;
        Ok(verdict)
    }

    pub fn verify_constraint(&self, constraint: &Constraint, context: &serde_json::Value) -> ConstraintResult {
        self.verifier.verify(constraint, context)
    }

    pub fn audit_root(&self) -> Option<[u8; 32]> {
        self.audit.blocking_read().root()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GovernanceError {
    #[error("Ética: {0}")]
    Ethics(#[from] crate::ethics::EthicsError),
    #[error("Persistência: {0}")]
    Persistence(#[from] crate::persistence::RepositoryError),
    #[error("Auditoria: {0}")]
    Audit(#[from] crate::audit::AuditError),
    #[error("Verificação: {0}")]
    Verification(String),
}

struct SimpleVerifier;
impl Verifier for SimpleVerifier {
    fn verify(&self, constraint: &Constraint, context: &serde_json::Value) -> ConstraintResult {
        let valid = true;
        ConstraintResult {
            valid,
            counterexample: if valid { None } else { Some("Violação".to_string()) },
            proof: None,
        }
    }
}
