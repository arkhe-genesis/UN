pub mod desci_publish;
pub mod desci_review;
pub mod diagnose;
pub mod grill_me;
pub mod improve_architecture;
pub mod qvac_inference;
pub mod tdd;
pub mod to_prd;
pub mod triage;

use crate::skill::manager::SkillManager;

/// Registra todas as skills built-in
pub async fn register_all(skill_mgr: &mut SkillManager) -> Result<Vec<String>, String> {
    let skills = vec![
        grill_me::grill_me_skill(),
        to_prd::to_prd_skill(),
        diagnose::diagnose_skill(),
        tdd::tdd_skill(),
        improve_architecture::improve_architecture_skill(),
        triage::triage_skill(),
        qvac_inference::qvac_inference_skill(),
    ];

    let mut registered = Vec::new();
    for skill in skills {
        let _hash = skill_mgr.save_skill(&skill).await?;
        registered.push(skill.name);
        // tracing::info!("✅ Skill '{}' registrada (hash: {})", skill.name, hash);
    }

    Ok(registered)
}
