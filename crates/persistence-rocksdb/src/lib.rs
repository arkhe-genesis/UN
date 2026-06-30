//! Safe-Core Persistence — Backend RocksDB
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PersistenceError {
    #[error("RocksDB error: {0}")]
    RocksDb(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Key not found: {0}")]
    KeyNotFound(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnFamily {
    Audit,
    Config,
    State,
    Merkle,
    Consensus,
}

impl ColumnFamily {
    pub fn as_str(&self) -> &str {
        match self {
            ColumnFamily::Audit => "audit",
            ColumnFamily::Config => "config",
            ColumnFamily::State => "state",
            ColumnFamily::Merkle => "merkle",
            ColumnFamily::Consensus => "consensus",
        }
    }
}

pub struct BatchOp {
    pub cf: ColumnFamily,
    pub key: Vec<u8>,
    pub value: Vec<u8>,
    pub op_type: BatchOpType,
}

pub enum BatchOpType {
    Put,
    Delete,
}
