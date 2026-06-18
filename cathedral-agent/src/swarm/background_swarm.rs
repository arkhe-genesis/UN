use crate::skill::manager::SkillManager;
use crate::skill::types::SkillType;
use crate::swarm::second_self::SecondSelfOrchestrator;
use std::time::Duration;

pub struct BackgroundSwarm {
    orchestrator: SecondSelfOrchestrator,
    _interval: Duration,
}

impl BackgroundSwarm {
    pub fn new(orchestrator: SecondSelfOrchestrator, interval: Duration) -> Self {
        Self { orchestrator, _interval: interval }
    }

    /// Roda skills do tipo Background periodicamente
    pub async fn run_periodic_skills(&mut self, skill_mgr: &mut SkillManager) {
        let skills = skill_mgr.list_by_type(SkillType::Background);
        let mut skill_names = Vec::new();

        for skill in skills {
            skill_names.push(skill.name.clone());
        }

        for name in skill_names {
            let _ = self.orchestrator.execute_skill(skill_mgr, &name).await;
        }
    }
}
