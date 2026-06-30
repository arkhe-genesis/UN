use lazy_static::lazy_static;
use prometheus::{IntCounter, IntGauge, Registry};

lazy_static! {
    pub static ref REGISTRY: Registry = Registry::new();

    pub static ref B20_COMPLIANCE_PASSED_TOTAL: IntCounter =
        IntCounter::new("b20_compliance_passed_total", "Total B20 compliance checks passed").unwrap();

    pub static ref B20_COMPLIANCE_CHECKED_TOTAL: IntCounter =
        IntCounter::new("b20_compliance_checked_total", "Total B20 compliance checks performed").unwrap();

    pub static ref B20_POLICY_DENIED_TOTAL: IntCounter =
        IntCounter::new("b20_policy_denied_total", "Total B20 policy denials").unwrap();

    pub static ref B20_SETTLEMENT_AMOUNT: IntGauge =
        IntGauge::new("b20_settlement_amount", "B20 Settlement Amount").unwrap();

    pub static ref B20_XRPL_BRIDGE_TRANSFERS_TOTAL: IntCounter =
        IntCounter::new("b20_xrpl_bridge_transfers_total", "Total B20 <-> XRPL transfers").unwrap();

    pub static ref B20_MEMO_INDEXED_TOTAL: IntCounter =
        IntCounter::new("b20_memo_indexed_total", "Total B20 memos indexed").unwrap();

    pub static ref B20_FREEZE_SEIZE_TOTAL: IntCounter =
        IntCounter::new("b20_freeze_seize_total", "Total B20 freeze and seize operations").unwrap();
}

pub fn register_metrics() {
    REGISTRY.register(Box::new(B20_COMPLIANCE_PASSED_TOTAL.clone())).unwrap();
    REGISTRY.register(Box::new(B20_COMPLIANCE_CHECKED_TOTAL.clone())).unwrap();
    REGISTRY.register(Box::new(B20_POLICY_DENIED_TOTAL.clone())).unwrap();
    REGISTRY.register(Box::new(B20_SETTLEMENT_AMOUNT.clone())).unwrap();
    REGISTRY.register(Box::new(B20_XRPL_BRIDGE_TRANSFERS_TOTAL.clone())).unwrap();
    REGISTRY.register(Box::new(B20_MEMO_INDEXED_TOTAL.clone())).unwrap();
    REGISTRY.register(Box::new(B20_FREEZE_SEIZE_TOTAL.clone())).unwrap();
}
