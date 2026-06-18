use crate::skill::types::ExecutionStatus;
use crate::skill::manager::SkillManager;
use crate::swarm::orchestrator::SwarmOrchestrator;
use crate::swarm::types::SwarmResult;

pub struct SkillExecutor {
    orchestrator: SwarmOrchestrator,
    skill_manager: SkillManager,
}

impl SkillExecutor {
    pub fn new(orchestrator: SwarmOrchestrator, skill_manager: SkillManager) -> Self {
        Self { orchestrator, skill_manager }
    }

    /// Executa uma skill como SwarmSpec
    pub async fn execute_skill(&mut self, skill_name: &str) -> Result<SwarmResult, String> {
        // Carrega a skill
        let skill = self.skill_manager.load_skill(skill_name).await
            .ok_or_else(|| format!("Skill '{}' não encontrada", skill_name))?
            .clone();

        // Converte para SwarmSpec
        let spec = skill.to_swarm_spec();

        // Executa
        let result = self.orchestrator.run_spec(spec).await?;

        // Registra execução
        self.skill_manager.record_execution(
            skill_name,
            ExecutionStatus::Completed,
            Some(format!("{:?}", result).into_bytes()),
            None,
        );

        Ok(result)
    }

    /// Executa uma skill em background (sem retorno)
    pub async fn execute_skill_background(&mut self, skill_name: &str) {
        match self.execute_skill(skill_name).await {
            Ok(_result) => {
            }
            Err(e) => {
                self.skill_manager.record_execution(
                    skill_name,
                    ExecutionStatus::Failed,
                    None,
                    Some(e),
                );
            }
        }
    }
}
