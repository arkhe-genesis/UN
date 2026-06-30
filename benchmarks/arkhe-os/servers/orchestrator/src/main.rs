use std::sync::Arc;
use tokio;
use cathedral_orchestrator::attestation::{AttestationManager, CathedralComputeProvider};
use cathedral_orchestrator::identity_attestation::IdentityAttestationProvider;
use cathedral_orchestrator::voice::VoiceCore;
use cathedral_orchestrator::mcp::server::start_mcp_server;

struct DummyIdentityProvider;

#[async_trait::async_trait]
impl IdentityAttestationProvider for DummyIdentityProvider {
    async fn attest_identity(&self, _force_refresh: bool) -> Result<cathedral_orchestrator::identity_attestation::IdentityAttestation, String> {
        Ok(cathedral_orchestrator::identity_attestation::IdentityAttestation {
            confidence: 0.99,
            identity_verified: true,
            timestamp: chrono::Utc::now().timestamp(),
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("100T Orchestrator starting...");

    let attestation_manager = Arc::new(AttestationManager::new());
    let compute_provider = Arc::new(CathedralComputeProvider::new());
    let identity_provider = Arc::new(DummyIdentityProvider);
    let voice_core = Some(Arc::new(VoiceCore));
    let architect_verifier = None;
    let mcp_port = 3032;

    let mcp_enabled = std::env::var("ENABLE_MCP_SERVER")
        .unwrap_or_else(|_| "true".to_string())
        .parse::<bool>()
        .unwrap_or(true);

    if mcp_enabled {
        println!("Starting MCP Server on port {}", mcp_port);
        start_mcp_server(
            attestation_manager,
            identity_provider,
            compute_provider,
            architect_verifier,
            voice_core,
            mcp_port,
        ).await?;
    }

    Ok(())
}
