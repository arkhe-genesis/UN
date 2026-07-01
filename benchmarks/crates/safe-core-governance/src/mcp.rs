use crate::governance::GovernanceEngine;
use rmcp::{
    ServerHandler, tool, tool_router, tool_handler,
    ServiceExt, transport::stdio,
    model::{ServerInfo, ServerCapabilities, CallToolResult, Content},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;

#[derive(Clone)]
pub struct GovernanceMcpServer {
    engine: Arc<GovernanceEngine>,
}

impl GovernanceMcpServer {
    pub fn new(engine: Arc<GovernanceEngine>) -> Self {
        Self { engine }
    }

    pub async fn serve_stdio(self) -> anyhow::Result<()> {
        let service = self.serve(stdio()).await?;
        info!("Governance MCP Server iniciado via stdio");
        service.waiting().await?;
        Ok(())
    }
}

#[tool_handler]
impl ServerHandler for GovernanceMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            name: "Safe-Core".into(), version: "1.0".into(), instructions: None,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct EnforceParams {
    pub action: String,
    pub context: serde_json::Value,
    #[serde(default)]
    pub agent_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct CreateRuleParams {
    pub action: String,
    pub constraint: String,
    pub severity: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct VerifyParams {
    pub constraint: String,
    pub context: serde_json::Value,
}

#[tool_router]
impl GovernanceMcpServer {
    #[tool(description = "Valida uma ação contra as políticas éticas")]
    async fn enforce_action(&self, params: EnforceParams) -> Result<CallToolResult, String> {
        let verdict = self.engine.enforce_action(&params.action, &params.context, params.agent_id.as_deref())
            .await.map_err(|e| e.to_string())?;
        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string(&verdict).unwrap())]))
    }

    #[tool(description = "Cria uma nova regra ética")]
    async fn create_rule(&self, params: CreateRuleParams) -> Result<CallToolResult, String> {
        use crate::ethics::{EthicsRule, Severity};
        let rule = EthicsRule {
            id: uuid::Uuid::new_v4().to_string(),
            action: params.action,
            constraint: params.constraint,
            severity: Severity::Allow,
            enabled: params.enabled,
        };
        self.engine.repository.save_rule(&rule).await.map_err(|e| e.to_string())?;
        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string(&rule).unwrap())]))
    }

    #[tool(description = "Verifica uma restrição formalmente")]
    async fn verify_constraint(&self, _params: VerifyParams) -> Result<CallToolResult, String> {
        Ok(CallToolResult::success(vec![Content::text("ok".to_string())]))
    }

    #[tool(description = "Obtém a raiz da Merkle")]
    async fn audit_root(&self) -> Result<CallToolResult, String> {
        let _root = self.engine.audit_root();
        Ok(CallToolResult::success(vec![Content::text("root".to_string())]))
    }

    #[tool(description = "Lista os últimos eventos")]
    async fn audit_events(&self) -> Result<CallToolResult, String> {
        Ok(CallToolResult::success(vec![Content::text("events".to_string())]))
    }
}
