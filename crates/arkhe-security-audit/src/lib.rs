pub mod recon;
pub mod hunt;
pub mod validate;
pub mod report;
pub mod structured_output;
pub mod independent_verification;
pub mod types;

pub use types::{Finding, Severity, AttackClass};
pub use recon::ReconnaissancePhase;
pub use hunt::HuntPhase;
pub use validate::ValidationPhase;
pub use report::ReportPhase;
pub use structured_output::StructuredOutputPhase;
pub use independent_verification::IndependentVerificationPhase;

pub struct AuditOrchestrator {
    target_dir: String,
    llm: std::sync::Arc<dyn arkhe_llm::engine::InferenceEngine>,
}

impl AuditOrchestrator {
    pub fn new(target_dir: &str, llm: std::sync::Arc<dyn arkhe_llm::engine::InferenceEngine>) -> Self {
        Self {
            target_dir: target_dir.to_string(),
            llm,
        }
    }

    pub async fn run(&self) -> Result<Vec<Finding>, arkhe_core::ArkheError> {
        tracing::info!("🔄 Iniciando auditoria de segurança (6 fases)");

        let recon = ReconnaissancePhase::new(&self.target_dir, self.llm.clone());
        let architecture = recon.run().await?;

        let hunt = HuntPhase::new(self.llm.clone());
        let findings = hunt.run(&architecture).await?;

        let validate = ValidationPhase::new(self.llm.clone());
        let validated_findings = validate.run(findings).await?;

        let report = ReportPhase::new();
        report.generate(&validated_findings).await?;

        let structured = StructuredOutputPhase::new();
        structured.generate(&validated_findings).await?;

        let verify = IndependentVerificationPhase::new(self.llm.clone());
        let verified_findings = verify.run(validated_findings).await?;

        Ok(verified_findings)
    }
}
