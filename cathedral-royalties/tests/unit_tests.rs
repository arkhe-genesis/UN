use cathedral_royalties::integrations::pix_openapi::{PixPaymentStatus, PixWebhookPayload};
use serde_json::json;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webhook_payload_deserialization() {
        let json = json!({
            "transaction_id": "pix_123456",
            "status": "PAID",
            "paid_amount": 5.00,
            "paid_at": "2025-06-17T14:30:00Z",
            "payer_document": "12345678901",
            "payer_name": "João Silva",
            "metadata": {
                "dpid": "46"
            }
        });

        let payload: PixWebhookPayload = serde_json::from_value(json).unwrap();
        assert_eq!(payload.transaction_id, "pix_123456");
        assert_eq!(payload.status, PixPaymentStatus::Paid);
        assert_eq!(payload.paid_amount.unwrap(), 5.00);
        assert_eq!(
            payload.metadata.as_ref().unwrap().get("dpid").unwrap(),
            "46"
        );
    }
}
