use axum::{
    extract::{State, Extension, Path},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::api::auth::Did;
use crate::orchestration::orchestrator::Orchestrator;

#[derive(Debug, Deserialize)]
pub struct DeployRequest {
    pub bytecode: String,
    pub abi: serde_json::Value,
    pub network: String,
    pub from: String,
    pub gas_limit: u64,
}

#[derive(Debug, Serialize)]
pub struct DeployResponse {
    pub success: bool,
    pub contract_address: Option<String>,
    pub transaction_hash: Option<String>,
    pub action_id: Option<String>,
    pub error: Option<String>,
}

pub async fn deploy_contract(
    State(orchestrator): State<Arc<Orchestrator>>,
    Extension(did): Extension<Did>,
    Json(req): Json<DeployRequest>,
) -> Json<DeployResponse> {
    let result = orchestrator
        .deploy_contract(
            &did,
            &req.bytecode,
            &req.abi,
            &req.network,
            &req.from,
            req.gas_limit,
        )
        .await;

    match result {
        Ok((contract_address, tx_hash, action_id)) => Json(DeployResponse {
            success: true,
            contract_address: Some(contract_address),
            transaction_hash: Some(tx_hash),
            action_id: Some(action_id),
            error: None,
        }),
        Err(e) => Json(DeployResponse {
            success: false,
            contract_address: None,
            transaction_hash: None,
            action_id: None,
            error: Some(e),
        }),
    }
}

pub async fn get_status(Path(_tx_hash): Path<String>) -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "pending"}))
}
