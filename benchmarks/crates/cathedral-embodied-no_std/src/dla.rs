pub struct MemoryProof {
    pub merkle_root: String,
    pub timestamp: u64,
    pub state_count: u32,
}

pub async fn prove_memory_state() -> Result<MemoryProof, String> {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let merkle_root = format!("0xmockroot{}", timestamp);

    Ok(MemoryProof {
        merkle_root,
        timestamp,
        state_count: 47,
    })
}
