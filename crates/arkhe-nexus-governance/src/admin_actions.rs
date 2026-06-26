//! Ações administrativas do NEXUS mapeadas para AdministrativeAction.
//!
//! Esta enum mapeia as ações específicas do NEXUS para os tipos
//! genéricos de AdministrativeAction do arkhe-governance.

use arkhe_governance::ActionClass;
use chrono::Duration;

/// Ações administrativas específicas do NEXUS/Safe Core.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NexusAdminAction {
    /// Atualização do kernel do NEXUS.
    KernelUpdate,
    /// Modificação de políticas de segurança.
    SecurityPolicyChange,
    /// Alteração em Capsules (privilégios, isolamento).
    CapsuleModification,
    /// Atualização de regras de ComplianceReport.
    ComplianceRulesUpdate,
    /// Modificação de parâmetros Flock.
    FlockConfigUpdate,
    /// Operação no WormGraph (routing, anchoring).
    WormGraphOperation,
    /// Alteração no Hashtree de bundles.
    BundleHashtreeUpdate,
    /// Outra ação administrativa.
    Other,
}

impl NexusAdminAction {
    /// Mapeia para ActionClass genérico.
    pub fn to_generic(&self) -> ActionClass {
        match self {
            Self::KernelUpdate => ActionClass::Critical,
            Self::SecurityPolicyChange => ActionClass::Critical,
            Self::CapsuleModification => ActionClass::Operational,
            Self::ComplianceRulesUpdate => ActionClass::Operational,
            Self::FlockConfigUpdate => ActionClass::Operational,
            Self::WormGraphOperation => ActionClass::Operational,
            Self::BundleHashtreeUpdate => ActionClass::Operational,
            Self::Other => ActionClass::Other,
        }
    }

    /// Cria proposta de governança para esta ação.
    pub fn to_proposal(
        &self,
        _id: String, // id in the prompt, but GovernanceAction::new doesn't take id (generated)
        description: String,
        _total_voters: u64, // Not present in patch GovernanceAction signature
        delay_hours: i64,
    ) -> arkhe_governance::GovernanceProposal {
        arkhe_governance::GovernanceProposal::new(
            self.to_generic(),
            description,
            "did:arkhe:proposer".into(), // mocked for this adapter method based on patch struct
            Duration::hours(delay_hours),
            [0u8; 32], // mocked action_hash
        )
    }
}
