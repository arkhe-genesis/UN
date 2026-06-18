use crate::skill::manager::SkillManager;
use crate::skill::types::SkillType;
use crate::skill::executor::SkillExecutor;
use crate::swarm::orchestrator::SwarmOrchestrator;
use crate::swarm::types::SwarmResult;

use crate::skill::builtin::qvac_inference::{QVACInferenceExecutor, QVACConfig};
use crate::evolution::sepl::AutogenesisOperator;
use crate::evolution::pipeline::EvolutionPipeline;
use crate::sandbox::WasiPreview2Sandbox;
use crate::hashtree_adapter::HashTreeStorage;
use crate::trace_manager::TraceManager;
use crate::thread_index::ThreadIndex;
use crate::version_manager::VersionManager;
use crate::integrations::x402::{X402RoyaltyServer, X402Client};
use crate::evolution::desci_node_resource::{DeSciNodeResource, RoyaltySplit, FreeTier};
use bytes::Bytes;
use std::sync::Arc;

pub struct AgentIdentity {
    pub name: String,
    pub optimization_goal: String,
}

impl AgentIdentity {
    pub fn get_orcid_by_npub(&self, _npub: &str) -> Option<String> {
        None
    }

    pub fn add_provenance(&self, _action: &str, _author: &str, _desc: &str, _tx_hash: Option<&str>, _artifact_hash: Option<&str>) {
        // mocked
    }
}

pub struct SecondSelfOrchestrator {
    pub orchestrator: SwarmOrchestrator,
    pub identity: AgentIdentity,
    pub qvac_executor: Option<QVACInferenceExecutor>,
    pub evolution_pipeline: Option<EvolutionPipeline>,
    pub storage: HashTreeStorage,
    pub trace_manager: Option<Arc<TraceManager>>,
    pub thread_index: Option<ThreadIndex>,
    pub version_manager: Option<VersionManager>,
    pub resource_registry: Option<crate::evolution::registry::ResourceRegistry>,
    pub x402_server: X402RoyaltyServer,
    pub x402_client: X402Client,
    pub base_url: String,
}

impl SecondSelfOrchestrator {
    pub fn new() -> Self {
        let facilitator_url = std::env::var("X402_FACILITATOR_URL")
            .unwrap_or_else(|_| "https://api.x402.org/v1".to_string());

        Self {
            orchestrator: SwarmOrchestrator {},
            identity: AgentIdentity {
                name: "Second Self".to_string(),
                optimization_goal: "Assist".to_string(),
            },
            qvac_executor: None,
            evolution_pipeline: None,
            storage: HashTreeStorage::new(),
            trace_manager: Some(Arc::new(TraceManager::new())),
            thread_index: Some(ThreadIndex {}),
            version_manager: Some(VersionManager {}),
            resource_registry: Some(crate::evolution::registry::ResourceRegistry::new()),
            x402_server: X402RoyaltyServer::new(&facilitator_url),
            x402_client: X402Client::new(),
            base_url: "http://localhost:3000".to_string(),
        }
    }

    pub async fn init_evolution_system_with_qvac(
        &mut self,
        default_model_hash: &str,
        qvac_config: QVACConfig,
    ) -> Result<(), String> {
        let storage = self.storage.clone();
        let trace_manager = self.trace_manager.as_ref()
            .ok_or("TraceManager não inicializado")?.clone();
        let _thread_index = self.thread_index.as_ref()
            .ok_or("ThreadIndex não inicializado")?;
        let eve_client = crate::eve_client::EveClient {};

        let operator = AutogenesisOperator::new_with_qvac(
            eve_client,
            ThreadIndex {},
            storage.clone(),
            trace_manager.clone(),
            default_model_hash,
            qvac_config.clone(),
            5,
        ).await?;

        let operator = Box::new(operator);
        let sandbox = WasiPreview2Sandbox::new().await?;
        let version_manager = self.version_manager.as_ref()
            .ok_or("VersionManager não inicializado")?;

        let pipeline = EvolutionPipeline::new(
            operator,
            sandbox,
            version_manager.clone(),
            5,
        );

        self.evolution_pipeline = Some(pipeline);
        self.qvac_executor = Some(QVACInferenceExecutor::new(
            storage.clone(),
            trace_manager.clone(),
            qvac_config,
            default_model_hash,
        ));

        Ok(())
    }

    /// Carrega skills built-in e do diretório
    pub async fn load_all_skills(
        &mut self,
        skill_mgr: &mut SkillManager,
        skills_dir: Option<&str>,
    ) -> Result<Vec<String>, String> {
        let mut loaded = Vec::new();

        let builtin = crate::skill::builtin::register_all(skill_mgr).await?;
        loaded.extend(builtin);

        if let Some(dir) = skills_dir {
            let imported = skill_mgr.import_from_dir(dir).await?;
            loaded.extend(imported);
        }

        let context = skill_mgr.generate_context();
        tokio::fs::write("CONTEXT.md", context)
            .await
            .map_err(|e| format!("Erro ao escrever CONTEXT.md: {}", e))?;

        Ok(loaded)
    }

    /// Executa uma skill como SwarmSpec (via SkillExecutor)
    pub async fn execute_skill(
        &mut self,
        skill_mgr: &mut SkillManager,
        skill_name: &str,
    ) -> Result<crate::swarm::types::SwarmResult, String> {
        let orchestrator = self.orchestrator.clone();
        let mut executor = SkillExecutor::new(orchestrator, skill_mgr.clone());
        executor.execute_skill(skill_name).await
    }

    /// Aplica skills model-invoked automaticamente
    pub async fn apply_model_skills(
        &mut self,
        input_text: &str,
        skill_mgr: &mut SkillManager,
    ) -> Result<Vec<String>, String> {
        let triggered = skill_mgr.find_by_trigger(input_text);
        let mut applied = Vec::new();

        for skill in triggered {
            if skill.skill_type == SkillType::ModelInvoked {
                applied.push(skill.name.clone());
            }
        }

        Ok(applied)
    }

    /// Executa automaticamente skills model-invoked que fazem match com o input
    pub async fn execute_triggered_skills(
        &mut self,
        input: &str,
        skill_mgr: &mut SkillManager,
    ) -> Result<Vec<SwarmResult>, String> {
        let triggered_skills = skill_mgr.find_by_trigger(input);
        let mut results = Vec::new();
        let mut skills_to_execute = Vec::new();

        for skill in triggered_skills {
            if skill.skill_type == SkillType::ModelInvoked {
                skills_to_execute.push(skill.name.clone());
            }
        }

        for skill_name in skills_to_execute {
            let result = self.execute_skill(skill_mgr, &skill_name).await?;
            results.push(result);
        }

        Ok(results)
    }

    pub fn get_desci_node_mut(&mut self, _node_id: &str) -> Option<&mut DeSciNodeResource> {
        // Mocked - in reality would get from registry
        None
    }

    pub fn get_desci_node(&self, _node_id: &str) -> Option<DeSciNodeResource> {
        // Mocked - in reality would get from registry
        None
    }

    pub async fn save_node_version(&self, _node: &mut DeSciNodeResource) -> Result<(), String> {
        Ok(())
    }

    pub async fn enable_royalties(
        &mut self,
        _node_id: &str,
        _price: &str,                    // "0.001 USDC"
        splits: Vec<(String, f32)>,     // (npub, share)
        _free_tier: Option<FreeTier>,
    ) -> Result<(), String> {
        // Using mocked implementations
        let _now = chrono::Utc::now().timestamp() as u64;

        let royalty_splits: Vec<RoyaltySplit> = splits.into_iter()
            .map(|(npub, share)| {
                let orcid = self.identity.get_orcid_by_npub(&npub);
                let eth_address = self.x402_server.npub_to_eth_address(&npub);
                RoyaltySplit {
                    npub,
                    share,
                    orcid,
                    eth_address: Some(eth_address),
                }
            })
            .collect();

        let total_share: f32 = royalty_splits.iter().map(|s| s.share).sum();
        if (total_share - 1.0).abs() > 0.001 {
            return Err("A soma das participações deve ser 1.0".to_string());
        }

        Ok(())
    }

    pub async fn download_desci_component(
        &self,
        dpid: &str,
        component_id: &str,
        wallet_private_key: &str,
    ) -> Result<Bytes, String> {
        let node = self.get_desci_node(dpid)
            .ok_or_else(|| format!("Node {} não encontrado", dpid))?;

        let url = format!("{}/desci/{}/components/{}", self.base_url, dpid, component_id);

        if let Some(royalty) = &node.royalty_config {
            if royalty.enabled {
                return self.x402_client.download_with_payment(&url, wallet_private_key).await;
            }
        }

        Ok(Bytes::from(Vec::new()))
    }
}
