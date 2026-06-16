#![cfg(feature = "std")] // Requires std for file IO
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct OrchestraConfig {
    pub max_parallel_subagents: u32,
    pub delegation_threshold: f64,
    pub dla_memory_budget: u32,
    pub zk_proof_budget: u32,
    pub use_symbolic_planner: bool,
}

pub fn update_orchestra_config(config: OrchestraConfig) -> Result<(), std::io::Error> {
    let path = "/etc/orchestra/config.json";
    let data = serde_json::to_string(&config)?;
    std::fs::write(path, data)?;
    Ok(())
}
