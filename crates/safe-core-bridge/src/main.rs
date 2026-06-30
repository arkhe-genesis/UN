use axum::Router;
use clap::Parser;
use safe_core_bridge::{handlers, state::BridgeState, SafeCoreMcpServer};
use std::sync::Arc;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

#[derive(Parser)]
#[command(name = "safe-core-bridge", version)]
struct Cli {
    /// Endereço HTTP (modo REST). Ignorado no modo MCP.
    #[arg(long, env = "ADDR", default_value = "0.0.0.0:8081")]
    addr: String,

    /// Ativa o modo MCP (stdio) em vez do servidor HTTP.
    /// Requer compilação com `--features mcp`.
    #[arg(long, env = "MCP")]
    mcp: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let state = Arc::new(BridgeState::new());

    if cli.mcp {
        // ── Modo MCP: stdio para coding agents ──────────────────────
        // Logging vai para stderr — stdout é do protocolo MCP
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "safe_core_bridge=warn".into()),
            )
            .with_writer(std::io::stderr)
            .init();

        tracing::info!("Safe-Core Bridge iniciando em modo MCP (stdio)");
        let server = SafeCoreMcpServer::new(state);
        server.run_stdio().await.unwrap();
    } else {
        // ── Modo HTTP: REST API programática ────────────────────────
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "safe_core_bridge=info".into()),
            )
            .init();

        let app = Router::new()
            .merge(handlers::router(state.clone()))
            .layer(CorsLayer::permissive())
            .layer(TraceLayer::new_for_http());

        tracing::info!("Safe-Core Bridge HTTP: http://{}/health", cli.addr);
        let listener = tokio::net::TcpListener::bind(&cli.addr).await?;
        axum::serve(listener, app).await?;
    }

    Ok(())
}
