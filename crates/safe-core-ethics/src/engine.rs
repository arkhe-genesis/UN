use async_trait::async_trait;
use crate::{EthicsRule, EthicsVerdict, EthicsError};

#[async_trait]
pub trait EthicsEngine: Send + Sync {
    async fn evaluate(&self, action: &str, context: &serde_json::Value) -> Result<EthicsVerdict, EthicsError>;
    async fn load_rules(&mut self, rules: Vec<EthicsRule>) -> Result<(), EthicsError>;
    async fn list_rules(&self) -> Vec<EthicsRule>;
}

pub struct Lean4Verifier {
    rules: Vec<EthicsRule>,
}

impl Lean4Verifier {
    pub fn new(_db: Option<()>) -> Self {
        Self { rules: Vec::new() }
    }
}

#[async_trait]
impl EthicsEngine for Lean4Verifier {
    async fn evaluate(&self, _action: &str, _context: &serde_json::Value) -> Result<EthicsVerdict, EthicsError> {
        Ok(EthicsVerdict {
            verdict: crate::rule::Severity::Allow,
            reason: "Mock".to_string(),
            rule_id: None,
        })
    }
    async fn load_rules(&mut self, rules: Vec<EthicsRule>) -> Result<(), EthicsError> {
        self.rules = rules;
        Ok(())
    }
    async fn list_rules(&self) -> Vec<EthicsRule> {
        self.rules.clone()
    }
}
