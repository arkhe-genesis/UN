use thiserror::Error;
#[derive(Debug, Error)]
pub enum GovernanceError {
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Invalid signature: {0}")]
    InvalidSignature(String),
    #[error("Unauthorized issuer: {0}")]
    Unauthorized(String),
    #[error("Governance action not supported: {0}")]
    UnsupportedAction(String),
    #[error("Log Error: {0}")]
    Log(String)
}
pub type GovernanceResult<T> = Result<T, GovernanceError>;
