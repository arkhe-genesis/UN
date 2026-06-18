use crate::skill::manager::SkillManager;
use crate::skill::types::SkillType;
use crate::skill::executor::SkillExecutor;
use crate::swarm::orchestrator::SwarmOrchestrator;
use crate::swarm::types::SwarmResult;

pub struct AgentIdentity {
    pub name: String,
    pub optimization_goal: String,
}

pub struct SecondSelfOrchestrator {
    pub orchestrator: SwarmOrchestrator,
    pub identity: AgentIdentity,
}

impl SecondSelfOrchestrator {
    pub fn new() -> Self {
        Self {
            orchestrator: SwarmOrchestrator {},
            identity: AgentIdentity {
                name: "Second Self".to_string(),
                optimization_goal: "Assist".to_string(),
            },
        }
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
}
