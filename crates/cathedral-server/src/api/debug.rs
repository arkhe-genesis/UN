use axum::Json;

pub async fn create_session() -> Json<serde_json::Value> { Json(serde_json::json!({})) }
pub async fn step() -> Json<serde_json::Value> { Json(serde_json::json!({})) }
pub async fn get_state() -> Json<serde_json::Value> { Json(serde_json::json!({})) }
pub async fn stop_session() -> Json<serde_json::Value> { Json(serde_json::json!({})) }
