//! Guia de migração do Safe Core para GovernanceGuard.
/// Documentação de migração para desenvolvedores do NEXUS.
///
/// # Passo 1: Substituir SafeCoreGuard
///
/// Antes:
/// ```rust,ignore
/// pub struct SafeCoreGuard {
///     // stubs existentes
/// }
///
/// impl SafeCoreGuard {
///     pub fn execute(&self, action: AdminAction) {
///         action.run(); // ❌ Sem verificação de governança
///     }
/// }
/// ```
///
/// Depois:
/// ```rust,ignore
/// use arkhe_nexus_governance::NexusGovernanceAdapter;
///
/// pub struct SafeCoreGuard {
///     governance: NexusGovernanceAdapter,
/// }
///
/// impl SafeCoreGuard {
///     pub fn execute(&self, proposal: GovernanceProposal, action: AdminActionFn) {
///         self.governance.execute_admin_action(proposal, action)?;
///     }
/// }
/// ```
///
/// # Passo 2: Atualizar chamadores
///
/// Antes:
/// ```rust,ignore
/// guard.execute(AdminAction::UpdateKernel { ... });
/// ```
///
/// Depois:
/// ```rust,ignore
/// let proposal = NexusAdminAction::KernelUpdate.to_proposal(
///     "prop-001".into(),
///     "Atualizar kernel para v2.0".into(),
///     8,  // total voters
///     48, // delay hours
/// );
///
/// // Votar (em um sistema real, isso viria de DIDs)
/// for i in 0..5 {
///     proposal.votes_for.insert(format!("did:arkhe:voter:{}", i));
/// }
///
/// guard.execute_admin_action(proposal, |p| {
///     // Ação real do NEXUS
///     kernel.update()?;
///     Ok(())
/// })?;
/// ```
///
/// # Passo 3: Adicionar ao Cargo.toml do NEXUS
///
/// ```toml
/// [dependencies]
/// arkhe-nexus-governance = { path = "../arkhe-nexus-governance" }
/// ```
pub struct MigrationGuide;

/// Lista de stubs do NEXUS que precisam ser substituídos.
pub const STUBS_TO_REPLACE: &[&str] = &[
    "SafeCoreGuard::execute",
    "SafeCoreGuard::update_kernel",
    "SafeCoreGuard::modify_capsule",
    "SafeCoreGuard::update_compliance",
    "NEXUS::admin_action",
    "NEXUS::privileged_operation",
];

/// Verifica se um módulo ainda contém stubs não migrados.
pub fn check_migration_status(code: &str) -> Vec<&'static str> {
    STUBS_TO_REPLACE
        .iter()
        .filter(|stub| code.contains(*stub))
        .copied()
        .collect()
}
