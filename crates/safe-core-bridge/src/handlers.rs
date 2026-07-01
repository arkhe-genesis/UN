use axum::{routing::{get, post}, Router, Json};
use std::sync::Arc;
use crate::state::BridgeState;

pub fn router(state: Arc<BridgeState>) -> Router {
    Router::new()
        .route("/health", get(|| async { "OK" }))
}
