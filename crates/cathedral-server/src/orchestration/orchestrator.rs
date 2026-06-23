use std::sync::Arc;
use cathedral_wormgraph::WormGraphClient;
use cathedral_zk::ZKGateway;
use cathedral_remix_bridge::{RemixClient, CompileRequest, DeployRequest};
use crate::api::auth::Did;

pub struct Orchestrator {
    remix: Arc<RemixClient>,
    wormgraph: Arc<WormGraphClient>,
    zk: Arc<ZKGateway>,
}

impl Orchestrator {
    pub fn new(
        remix: Arc<RemixClient>,
        wormgraph: Arc<WormGraphClient>,
        zk: Arc<ZKGateway>,
    ) -> Self {
        Self { remix, wormgraph, zk }
    }

    pub async fn compile_contract(
        &self,
        did: &Did,
        source: &str,
        version: &str,
        optimize: bool,
        runs: u32,
    ) -> Result<(serde_json::Value, String, String, String, String), String> {
        let action_id = "mock_action_id".to_string(); // Mocking recording

        let req = CompileRequest {
            source: source.to_string(),
            version: version.to_string(),
            optimize,
            runs,
            did: did.to_string(),
            signature: "".to_string(), // Será assinado pelo cliente
        };

        let resp = self.remix.compile(req).await?;
        if !resp.success {
            return Err(resp.error.unwrap_or_else(|| "Compilation failed".to_string()));
        }

        let abi = resp.abi.unwrap_or_else(|| serde_json::json!([]));
        let bytecode = resp.bytecode.unwrap_or_default();
        let bytecode_hash = resp.bytecode_hash.unwrap_or_default();

        let sampled = self.zk.sample(&format!("Compilação Solidity v{} para DID {}", version, did.to_string()), 1.0).await.map_err(|e| e.to_string())?;
        let proof = self.zk.prove_nanozk(sampled).await.map_err(|e| e.to_string())?;

        Ok((abi, bytecode, bytecode_hash, proof.hash, action_id))
    }

    pub async fn deploy_contract(
        &self,
        did: &Did,
        bytecode: &str,
        abi: &serde_json::Value,
        network: &str,
        from: &str,
        gas_limit: u64,
    ) -> Result<(String, String, String), String> {
        let action_id = "mock_deploy_action_id".to_string();

        let req = DeployRequest {
            bytecode: bytecode.to_string(),
            abi: abi.clone(),
            network: network.to_string(),
            from: from.to_string(),
            gas_limit,
            did: did.to_string(),
            signature: "".to_string(),
        };

        let resp = self.remix.deploy(req).await?;
        if !resp.success {
            return Err(resp.error.unwrap_or_else(|| "Deployment failed".to_string()));
        }

        let contract_address = resp.contract_address.unwrap_or_default();
        let tx_hash = resp.transaction_hash.unwrap_or_default();

        Ok((contract_address, tx_hash, action_id))
    }
}
