#![no_std]

extern crate alloc;
use alloc::string::{String, ToString};
use alloc::collections::{BTreeMap, BTreeSet};
use sha2::{Sha256, Digest};

pub const SUBSTRATE_ID: &str = "1010";
pub const SEAL: &str = "ZKCBDC-1010-2026-05-31";

#[derive(Debug, Clone, PartialEq)]
pub enum TransactionStatus {
    Pending,
    Proven,
    Rejected,
    Anchored,
    DoubleSpend,
}

#[derive(Debug, Clone)]
pub struct AccountState {
    pub account_id: String,
    pub commitment_balance: String,
    pub nonce: u32,
    pub is_frozen: bool,
    pub kyc_level: u8,
}

#[derive(Debug, Clone)]
pub struct ConfidentialTransaction {
    pub tx_id: String,
    pub commitment_sender: String,
    pub commitment_receiver: String,
    pub commitment_amount: String,
    pub nullifier: String,
    pub zk_proof: String,
    pub kyc_proof: String,
    pub sanctions_proof: String,
    pub status: TransactionStatus,
    pub temporal_anchor: Option<String>,
    pub seal: String,
}

impl ConfidentialTransaction {
    pub fn compute_seal(&mut self) -> String {
        let payload = alloc::format!("{}:{}:{}", self.tx_id, self.nullifier, &self.zk_proof[..32]);
        let hash = sha256_hash(&payload);
        self.seal = alloc::format!("ZKCBDC-{}", &hash[..16].to_ascii_uppercase());
        self.seal.clone()
    }
}

pub struct ZkcbdcEngine {
    pub total_supply: u64,
    pub central_bank_key: String,
    pub nullifiers: BTreeSet<String>,
    pub transactions: BTreeMap<String, ConfidentialTransaction>,
    pub accounts: BTreeMap<String, AccountState>,
    pub mint_proofs: BTreeMap<String, String>,
    pub sanctions_list: BTreeSet<String>,
    pub frozen_accounts: BTreeSet<String>,
    pub total_transactions: u64,
    pub total_volume: u64,
}

impl ZkcbdcEngine {
    pub fn new(total_supply: u64, central_bank_key: String) -> Self {
        Self {
            total_supply,
            central_bank_key,
            nullifiers: BTreeSet::new(),
            transactions: BTreeMap::new(),
            accounts: BTreeMap::new(),
            mint_proofs: BTreeMap::new(),
            sanctions_list: BTreeSet::new(),
            frozen_accounts: BTreeSet::new(),
            total_transactions: 0,
            total_volume: 0,
        }
    }

    pub fn create_account(&mut self, account_id: &str, initial_balance: u64) -> Result<AccountState, &'static str> {
        if self.accounts.contains_key(account_id) {
            return Err("Account already exists");
        }
        let r = "0123456789abcdef0123456789abcdef"; // static in no_std, mock
        let commitment = sha256_hash(&alloc::format!("{}:{}", initial_balance, r));
        let state = AccountState {
            account_id: account_id.to_string(),
            commitment_balance: commitment,
            nonce: 0,
            is_frozen: false,
            kyc_level: 0,
        };
        self.accounts.insert(account_id.to_string(), state.clone());
        Ok(state)
    }

    pub fn add_to_sanctions_list(&mut self, account_id: &str) {
        self.sanctions_list.insert(account_id.to_string());
    }

    pub fn freeze_account(&mut self, account_id: &str) {
        if let Some(acc) = self.accounts.get_mut(account_id) {
            acc.is_frozen = true;
            self.frozen_accounts.insert(account_id.to_string());
        }
    }

    pub fn create_transaction(&mut self, sender_priv: &str, receiver_pub: &str, amount: u64) -> Result<ConfidentialTransaction, &'static str> {
        if amount == 0 {
            return Err("Amount must be positive");
        }
        if sender_priv == receiver_pub {
            return Err("Self-transfer not allowed");
        }

        let mock_entropy = "0123456789abcdef";
        let tx_id = sha256_hash(&alloc::format!("{}:{}:{}:{}", sender_priv, receiver_pub, amount, mock_entropy));
        let tx_id = tx_id[..32].to_string();

        let nullifier = sha256_hash(&alloc::format!("{}:mock_utxo", sender_priv));
        if self.nullifiers.contains(&nullifier) {
            return Err("DOUBLE SPEND DETECTED");
        }

        let r1 = "mock_r1";
        let r2 = "mock_r2";
        let r3 = "mock_r3";

        let commitment_sender = sha256_hash(&alloc::format!("{}:{}", sender_priv, r1));
        let commitment_receiver = sha256_hash(&alloc::format!("{}:{}", receiver_pub, r2));
        let commitment_amount = sha256_hash(&alloc::format!("{}:{}", amount, r3));

        let zk_proof = sha256_hash(&alloc::format!("{}:{}:{}:verify", commitment_amount, commitment_sender, commitment_receiver));
        let kyc_proof = sha256_hash(&alloc::format!("{}:{}:humanity:verified", sender_priv, receiver_pub));
        let sanctions_proof = sha256_hash(&alloc::format!("{}:{}:no_sanctions", sender_priv, receiver_pub));

        let mut tx = ConfidentialTransaction {
            tx_id: tx_id.clone(),
            commitment_sender,
            commitment_receiver,
            commitment_amount,
            nullifier: nullifier.clone(),
            zk_proof,
            kyc_proof,
            sanctions_proof,
            status: TransactionStatus::Pending,
            temporal_anchor: None,
            seal: String::new(),
        };

        tx.compute_seal();

        if self.sanctions_list.contains(sender_priv) || self.sanctions_list.contains(receiver_pub) {
            tx.status = TransactionStatus::Rejected;
            return Ok(tx);
        }

        if self.frozen_accounts.contains(sender_priv) {
            tx.status = TransactionStatus::Rejected;
            return Ok(tx);
        }

        self.nullifiers.insert(nullifier);
        tx.status = TransactionStatus::Proven;
        self.total_transactions += 1;
        self.total_volume += amount;

        let mint_proof = sha256_hash(&alloc::format!("supply:{}:{}:{}", self.total_supply, tx_id, self.total_volume));
        self.mint_proofs.insert(tx_id.clone(), mint_proof);

        let anchor_hash = sha256_hash(&tx.seal);
        tx.temporal_anchor = Some(alloc::format!("923-ANCHOR-{}", &anchor_hash[..16].to_ascii_uppercase()));
        tx.status = TransactionStatus::Anchored;

        self.transactions.insert(tx_id, tx.clone());

        Ok(tx)
    }

    pub fn verify_proof(&self, tx: &mut ConfidentialTransaction) -> bool {
        let recalculated = sha256_hash(&alloc::format!("{}:{}:{}:verify", tx.commitment_amount, tx.commitment_sender, tx.commitment_receiver));
        if recalculated[..16] != tx.zk_proof[..16] {
            tx.status = TransactionStatus::Rejected;
            return false;
        }
        tx.status = TransactionStatus::Proven;
        true
    }
}

fn sha256_hash(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();

    let mut hex_string = String::with_capacity(result.len() * 2);
    for byte in result {
        hex_string.push_str(&alloc::format!("{:02x}", byte));
    }
    hex_string
}
