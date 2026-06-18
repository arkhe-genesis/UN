use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ─── Cobrança (Charge) ──────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PixChargeRequest {
    pub amount: f64,
    pub description: String,
    pub payer_name: Option<String>,
    pub payer_document: Option<String>,
    pub expiration_seconds: Option<u32>,
    pub callback_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PixChargeResponse {
    pub transaction_id: String,
    pub qr_code: String,
    pub copy_paste: String,
    pub status: PixChargeStatus,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub payment_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum PixChargeStatus {
    Created,
    Waiting,
    Paid,
    Expired,
    Cancelled,
    Failed,
}

// ─── Pagamento (Payment) ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PixPayment {
    pub transaction_id: String,
    pub charge_id: String,
    pub amount: f64,
    pub status: PixPaymentStatus,
    pub paid_at: Option<DateTime<Utc>>,
    pub payer_document: Option<String>,
    pub payer_name: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum PixPaymentStatus {
    Created,
    Processing,
    Paid,
    Failed,
    Refunded,
}

// ─── Webhook ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PixWebhookPayload {
    pub transaction_id: String,
    pub status: PixPaymentStatus,
    pub paid_amount: Option<f64>,
    pub paid_at: Option<DateTime<Utc>>,
    pub payer_document: Option<String>,
    pub payer_name: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

// ─── Pix Key (DICT) ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PixKey {
    pub key: String,
    pub key_type: PixKeyType,
    pub holder_name: String,
    pub holder_document: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum PixKeyType {
    Cpf,
    Cnpj,
    Phone,
    Email,
    Evp,
}

// ─── Open Finance ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenFinanceConsent {
    pub consent_id: String,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: DateTime<Utc>,
    pub scope: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenFinanceBalance {
    pub balance: f64,
    pub currency: String,
    pub account_type: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenFinanceTransferRequest {
    pub pix_key: String,
    pub amount: f64,
    pub description: String,
    pub consent_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenFinanceTransferResponse {
    pub transaction_id: String,
    pub status: TransferStatus,
    pub processed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum TransferStatus {
    Pending,
    Success,
    Failed,
}
