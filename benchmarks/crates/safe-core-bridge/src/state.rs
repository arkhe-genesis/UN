use std::sync::Arc;
use tokio::sync::RwLock;

pub struct DummyEngine {
    pub violations: RwLock<Vec<DummyViolation>>,
}

impl DummyEngine {
    pub fn new() -> Self {
        Self { violations: RwLock::new(Vec::new()) }
    }
    pub async fn check_action(&self, action: &str, ctx: &serde_json::Value) -> Result<DummyResult, safe_core_ethics::EthicsError> {
        let is_allowed = !ctx.get("harm_to_humans").and_then(|v| v.as_bool()).unwrap_or(false)
            && ctx.get("transparent").and_then(|v| v.as_bool()).unwrap_or(false);

        let t = if is_allowed { "allowed" } else {
            if ctx.get("harm_to_humans").and_then(|v| v.as_bool()).unwrap_or(false) {
                let mut v = self.violations.write().await;
                v.push(DummyViolation { constraint_id: "ETH-001".to_string() });
                "blocked"
            } else {
                "requires_approval"
            }
        };

        Ok(DummyResult {
            allowed: is_allowed,
            typ: t.to_string(),
        })
    }
    pub async fn get_violations(&self) -> Vec<DummyViolation> {
        self.violations.read().await.clone()
    }
    pub async fn clear_violations(&self) {
        self.violations.write().await.clear();
    }
    pub async fn constraint_count(&self) -> usize { 4 }
}

pub struct DummyResult {
    allowed: bool,
    typ: String,
}

impl DummyResult {
    pub fn is_allowed(&self) -> bool { self.allowed }
}

impl From<&DummyResult> for Option<serde_json::Value> {
    fn from(res: &DummyResult) -> Self {
        Some(serde_json::json!({ "type": res.typ }))
    }
}

#[derive(Clone)]
pub struct DummyViolation {
    constraint_id: String,
}

impl From<&DummyViolation> for crate::api::ViolationView {
    fn from(v: &DummyViolation) -> Self {
        crate::api::ViolationView { constraint_id: v.constraint_id.clone() }
    }
}

pub struct DummyInvariant {
    pub id: String,
    pub severity: String,
}

impl From<&DummyInvariant> for crate::api::InvariantView {
    fn from(inv: &DummyInvariant) -> Self {
        crate::api::InvariantView { id: inv.id.clone(), severity: inv.severity.clone() }
    }
}

pub struct BridgeState {
    pub ethics_engine: Arc<DummyEngine>,
    pub invariants: Vec<DummyInvariant>,
}

impl BridgeState {
    pub fn new() -> Self {
        Self {
            ethics_engine: Arc::new(DummyEngine::new()),
            invariants: vec![
                DummyInvariant { id: "INV-001".to_string(), severity: "Critical".to_string() },
                DummyInvariant { id: "INV-002".to_string(), severity: "High".to_string() },
                DummyInvariant { id: "INV-003".to_string(), severity: "High".to_string() },
                DummyInvariant { id: "INV-004".to_string(), severity: "Medium".to_string() },
                DummyInvariant { id: "INV-005".to_string(), severity: "Low".to_string() },
            ],
        }
    }
}
