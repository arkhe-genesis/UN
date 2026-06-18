use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::post, Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, warn};

use crate::integrations::pix_openapi::PixPaymentStatus;

#[derive(Debug, Deserialize)]
pub struct PixWebhookRequest {
    pub transaction_id: String,
    pub status: String, // "PAID", "EXPIRED", "CANCELLED"
    pub paid_amount: Option<f64>,
    pub paid_at: Option<DateTime<Utc>>,
    pub payer_document: Option<String>,
    pub payer_name: Option<String>,
    pub metadata: Option<serde_json::Value>, // dados adicionais (ex: dpid)
}

#[derive(Debug, Serialize)]
pub struct PixWebhookResponse {
    pub status: String,
    pub message: String,
}

pub struct PixWebhookHandler {}

impl PixWebhookHandler {
    pub async fn handle_webhook(
        State(_state): State<Arc<PixWebhookHandler>>,
        Json(payload): Json<PixWebhookRequest>,
    ) -> impl IntoResponse {
        info!(
            "📨 Webhook Pix recebido: tx={}, status={}",
            payload.transaction_id, payload.status
        );

        // Processa apenas pagamentos confirmados
        let status = match payload.status.as_str() {
            "PAID" => PixPaymentStatus::Paid,
            "EXPIRED" => {
                warn!("Status ignorado: {}", payload.status);
                return (
                    StatusCode::OK,
                    Json(PixWebhookResponse {
                        status: "ignored".to_string(),
                        message: format!("Status {} ignorado", payload.status),
                    }),
                );
            }
            "CANCELLED" => {
                warn!("Status ignorado: {}", payload.status);
                return (
                    StatusCode::OK,
                    Json(PixWebhookResponse {
                        status: "ignored".to_string(),
                        message: format!("Status {} ignorado", payload.status),
                    }),
                );
            }
            _ => {
                warn!("Status desconhecido: {}", payload.status);
                return (
                    StatusCode::BAD_REQUEST,
                    Json(PixWebhookResponse {
                        status: "error".to_string(),
                        message: format!("Status desconhecido: {}", payload.status),
                    }),
                );
            }
        };

        if status != PixPaymentStatus::Paid {
            info!("⏭️ Webhook ignorado (status: {:?})", status);
            return (
                StatusCode::OK,
                Json(PixWebhookResponse {
                    status: "ignored".to_string(),
                    message: format!("Status {:?} ignorado", status),
                }),
            );
        }

        // Extrai metadados (dpid, node_id, etc.)
        let dpid = payload
            .metadata
            .as_ref()
            .and_then(|m| m.get("dpid").and_then(|v| v.as_str()))
            .unwrap_or("unknown");

        let amount = payload.paid_amount.unwrap_or(0.0);

        info!(
            "💰 Pagamento Pix confirmado: dPID={}, BRL={:.2}",
            dpid, amount
        );

        (
            StatusCode::OK,
            Json(PixWebhookResponse {
                status: "success".to_string(),
                message: "Processed".to_string(),
            }),
        )
    }
}

pub fn create_router(handler: Arc<PixWebhookHandler>) -> Router {
    Router::new()
        .route("/pix", post(PixWebhookHandler::handle_webhook))
        .with_state(handler)
}
