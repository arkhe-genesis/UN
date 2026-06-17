use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZkBalanceProof {
    pub commitment: [u8; 32],
    pub min_balance: u128,           // valor mínimo provado
    pub proof: Vec<u8>,              // seal do RISC Zero
    pub journal: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivateAccount {
    pub agent_id: [u8; 32],
    pub balance_commitment: [u8; 32], // commitment do saldo atual
    pub nonce: u64,
}
