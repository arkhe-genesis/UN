#[cfg(feature = "mcp")]
pub mod mcp_impl {
    use crate::tools;
    use crate::state::BridgeState;
    use rmcp::{ServerHandler, ServiceExt, tool, tool_router, transport::stdio};
    use std::sync::Arc;
    use tracing::info;

    #[derive(Clone)]
    pub struct SafeCoreMcpServer {
        state: Arc<BridgeState>,
    }

    impl SafeCoreMcpServer {
        pub fn new(state: Arc<BridgeState>) -> Self {
            Self { state }
        }

        pub async fn run_stdio(self) -> Result<(), Box<dyn std::error::Error>> {
            info!("MCP server iniciando no modo stdio");
            self.serve(stdio()).await;
            Ok(())
        }
    }

    #[tool_router]
    impl SafeCoreMcpServer {
        #[tool(description = "Check if an action is ethically permitted by Safe-Core AGI. Returns allowed/blocked/requires_approval with constraint details.")]
        pub async fn enforce_action(
            &self,
            #[tool(description = "Action identifier (e.g. 'deploy_model', 'send_email', 'delete_user')")]
            action: String,
            #[tool(description = "JSON context: harm_to_humans (bool), violates_autonomy (bool), transparent (bool), privacy_violation (bool)")]
            context: serde_json::Value,
        ) -> Result<serde_json::Value, String> {
            let result = tools::enforce_action(&self.state, &action, &context)
                .await
                .map_err(|e| e.to_string())?;
            serde_json::to_value(&result).map_err(|e| e.to_string())
        }

        #[tool(description = "List all recorded ethics violations with constraint details")]
        pub async fn get_violations(&self, _params: serde_json::Value) -> Result<serde_json::Value, String> {
            let resp = tools::get_violations(&self.state).await;
            serde_json::to_value(&resp).map_err(|e| e.to_string())
        }

        #[tool(description = "Clear all recorded ethics violations")]
        pub async fn clear_violations(&self, _params: serde_json::Value) -> Result<serde_json::Value, String> {
            let result = tools::clear_violations(&self.state).await;
            Ok(result)
        }

        #[tool(description = "List all safety invariants with their Lean 4 formal specifications and severity levels")]
        pub async fn list_invariants(&self, _params: serde_json::Value) -> Result<serde_json::Value, String> {
            let resp = tools::list_invariants(&self.state);
            serde_json::to_value(&resp).map_err(|e| e.to_string())
        }

        #[tool(description = "Export safety invariants as Lean 4 pseudo-code specifications to /tmp/safe-core-lean4-export/")]
        pub async fn export_invariants(&self, _params: serde_json::Value) -> Result<serde_json::Value, String> {
            let result = tools::export_invariants(&self.state)
                .await
                .map_err(|e| e.to_string())?;
            Ok(result)
        }

        #[tool(description = "Health check — returns component status, constraint count, and invariant count")]
        pub async fn health(&self, _params: serde_json::Value) -> Result<serde_json::Value, String> {
            let resp = tools::health_check(&self.state).await;
            serde_json::to_value(&resp).map_err(|e| e.to_string())
        }
    }

    impl ServerHandler for SafeCoreMcpServer {}
}

#[cfg(not(feature = "mcp"))]
pub mod mcp_impl {
    use crate::state::BridgeState;
    use std::sync::Arc;

    #[derive(Clone)]
    pub struct SafeCoreMcpServer;

    impl SafeCoreMcpServer {
        pub fn new(_: Arc<BridgeState>) -> Self {
            Self
        }

        pub async fn run_stdio(self) -> Result<(), Box<dyn std::error::Error>> {
            Err("MCP não disponível. Compile com: cargo build -p safe-core-bridge --features mcp".into())
        }
    }
}

pub use mcp_impl::SafeCoreMcpServer;
