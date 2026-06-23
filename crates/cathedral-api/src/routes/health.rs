use axum::{Json, extract::State};
use cathedral_inference_runtime::CathedralRuntime;
use std::sync::Arc;

pub async fn health_check(
    State(runtime): State<Arc<CathedralRuntime>>,
) -> Json<serde_json::Value> {
    let db_ok = runtime.wormgraph.ping().await.is_ok();
    Json(serde_json::json!({
        "status": "ok",
        "database": db_ok,
    }))
}
