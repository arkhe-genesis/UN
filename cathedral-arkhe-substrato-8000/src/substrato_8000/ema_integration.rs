
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use chrono::Utc;
use thiserror::Error;

use crate::substrato_8000::mcp_headroom_server::EmaToken;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmaConfig {
    pub max_token_lifetime: i64,
    pub check_revocation: bool,
    pub enforce_quota: bool,
    pub enforce_classification: bool,
    pub default_allowed_domains: Vec<String>,
    pub quota_warning_threshold: f64,
}

impl Default for EmaConfig {
    fn default() -> Self {
        Self {
            max_token_lifetime: 3600 * 24,
            check_revocation: true,
            enforce_quota: true,
            enforce_classification: true,
            default_allowed_domains: vec!["cathedral.local".to_string()],
            quota_warning_threshold: 0.8,
        }
    }
}

pub struct EmaVerifier {
    config: EmaConfig,
}

impl EmaVerifier {
    pub fn new(config: EmaConfig) -> Self {
        Self {
            config,
        }
    }
}

#[derive(Debug, Error)]
pub enum EmaError {
    #[error("Token expired")]
    TokenExpired,
}
