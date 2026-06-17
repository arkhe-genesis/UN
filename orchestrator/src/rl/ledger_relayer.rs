use async_trait::async_trait;
use ethers::prelude::*;
use std::sync::Arc;
use tokio::sync::mpsc;
use serde_json::json;
use tracing::{info, error};

#[derive(Debug, Clone)]
pub struct LedgerEvent {
    pub event_type: String,
    pub payload: String,
    pub timestamp: u64,
    pub policy_version: u64,
    pub signature: Option<String>,
}

#[async_trait]
pub trait ConsensusLedger: Send + Sync {
    async fn record_event(&self, event: LedgerEvent) -> Result<(), String>;
}

abigen!(
    CathedralConsensusLedger,
    "../governance/CathedralConsensusLedger.abi.json"
);

pub struct SolanaJsonRpcRelayer {
    endpoint: String,
    program_id: String,
    client: reqwest::Client,
}

impl SolanaJsonRpcRelayer {
    pub async fn new(endpoint: &str, program_id: &str) -> Result<Self, String> {
        Ok(Self {
            endpoint: endpoint.to_string(),
            program_id: program_id.to_string(),
            client: reqwest::Client::new(),
        })
    }
}

#[async_trait]
impl ConsensusLedger for SolanaJsonRpcRelayer {
    async fn record_event(&self, event: LedgerEvent) -> Result<(), String> {
        let instruction_data = json!({
            "event_type": event.event_type,
            "payload": event.payload,
            "timestamp": event.timestamp,
            "policy_version": event.policy_version,
            "signature": event.signature.unwrap_or_default(),
        });
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "sendTransaction",
            "params": [
                hex::encode(&serde_json::to_vec(&instruction_data).unwrap_or_default())
            ]
        });

        let response = self.client
            .post(&self.endpoint)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("RPC error: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("RPC returned {}", response.status()));
        }

        let result: serde_json::Value = response.json().await
            .map_err(|e| format!("RPC parse error: {}", e))?;

        if result.get("error").is_some() {
            Err(format!("RPC error: {:?}", result["error"]))
        } else {
            Ok(())
        }
    }
}

pub struct EthereumRelayer<M> {
    sender: mpsc::Sender<LedgerEvent>,
    _contract: Arc<CathedralConsensusLedger<M>>,
}

impl<M: Middleware + 'static> EthereumRelayer<M> {
    pub fn new(client: Arc<M>, address: Address, queue_capacity: usize) -> Self {
        let contract = Arc::new(CathedralConsensusLedger::new(address, client));
        let (tx, mut rx) = mpsc::channel::<LedgerEvent>(queue_capacity);

        let contract_clone = contract.clone();
        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                let signature_bytes = if let Some(sig) = &event.signature {
                    hex::decode(sig).unwrap_or_default().into()
                } else {
                    ethers::types::Bytes::from(vec![])
                };

                let mut workflow_id = [0u8; 32];
                let bytes = event.event_type.as_bytes();
                let len = std::cmp::min(32, bytes.len());
                workflow_id[..len].copy_from_slice(&bytes[..len]);

                let mut proposal_hash = [0u8; 32];
                let pbytes = event.payload.as_bytes();
                let plen = std::cmp::min(32, pbytes.len());
                proposal_hash[..plen].copy_from_slice(&pbytes[..plen]);

                let call = contract_clone.record_decision(
                    workflow_id,
                    proposal_hash,
                    vec![],
                    true,
                    0,
                    0,
                    signature_bytes,
                );

                let result = call.send().await;

                match result {
                    Ok(pending_tx) => {
                        match pending_tx.await {
                            Ok(Some(receipt)) => {
                                info!("Event recorded successfully. Tx hash: {:?}", receipt.transaction_hash);
                            }
                            Ok(None) => {
                                error!("Transaction receipt not found.");
                            }
                            Err(e) => {
                                error!("Transaction failed: {:?}", e);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to send transaction: {:?}", e);
                    }
                }
            }
        });

        Self {
            sender: tx,
            _contract: contract,
        }
    }
}

#[async_trait]
impl<M: Middleware + 'static> ConsensusLedger for EthereumRelayer<M> {
    async fn record_event(&self, event: LedgerEvent) -> Result<(), String> {
        self.sender.send(event).await.map_err(|e| format!("Failed to queue event: {}", e))
    }
}
