use std::sync::Arc;
use async_trait::async_trait;
use cathedral_wormgraph::WormGraphClient;
use cathedral_permissions::PermissionEntry;
use serde_json::Value as ToolParams;
use serde_json::Value as ToolResult;

// Mock
pub struct ExplorationBudget;
pub struct Did(pub String);

impl Did {
    pub fn to_string(&self) -> String {
        self.0.clone()
    }
}

/// Ferramenta executável por agentes
#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn permissions(&self) -> Vec<PermissionEntry>;
    async fn execute(&self, params: &ToolParams, context: &ToolContext) -> Result<ToolResult, String>;
}

/// Contexto de execução de ferramenta
pub struct ToolContext {
    pub agent_did: Did,
    pub session_id: String,
    pub wormgraph: Arc<WormGraphClient>,
    pub execution_budget: Option<ExplorationBudget>,
}

impl ToolContext {
    pub async fn record_action(&self, action: &str, params: &ToolParams, result: &ToolResult) -> Result<(), String> {
        let content = serde_json::json!({
            "action": action,
            "params": params,
            "result": result,
            "session": self.session_id,
        }).to_string();

        self.wormgraph.record(
            &self.agent_did.to_string(),
            &content,
            &None,
            &[],
        ).await.map_err(|e| e.to_string())?;
        Ok(())
    }
}
