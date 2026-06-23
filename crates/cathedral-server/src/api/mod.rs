use axum::{
    routing::{get, post},
    Router,
    middleware,
};
use std::sync::Arc;
use crate::orchestration::orchestrator::Orchestrator;
use crate::api::auth::did_auth_middleware;

pub mod auth;
pub mod compile;
pub mod debug;
pub mod deploy;
pub mod plugins;
pub mod ledger;

pub fn create_routes(orchestrator: Arc<Orchestrator>) -> Router {
    Router::new()
        // Autenticação (sem middleware)
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login))
        // Rotas protegidas
        .nest(
            "/api",
            Router::new()
                // Compilação
                .route("/compile", post(compile::compile_contract))
                .route("/compile/versions", get(compile::list_versions))
                // Debugging
                .route("/debug/session", post(debug::create_session))
                .route("/debug/session/:id/step", post(debug::step))
                .route("/debug/session/:id/state", get(debug::get_state))
                .route("/debug/session/:id/stop", post(debug::stop_session))
                // Deployment
                .route("/deploy", post(deploy::deploy_contract))
                .route("/deploy/:tx_hash/status", get(deploy::get_status))
                // Plugins
                .route("/plugins", get(plugins::list_plugins))
                .route("/plugins/:id/activate", post(plugins::activate_plugin))
                .route("/plugins/:id/deactivate", post(plugins::deactivate_plugin))
                .route("/plugins/:id/call", post(plugins::call_plugin))
                // Ledger
                .route("/ledger/actions", get(ledger::list_actions))
                .route("/ledger/actions/:id", get(ledger::get_action))
                .route("/ledger/verify/:id", get(ledger::verify_action))
                // Health
                .route("/health", get(health_check))
                .layer(middleware::from_fn(did_auth_middleware))
                .with_state(orchestrator),
        )
}

async fn health_check() -> &'static str {
    "OK"
}
