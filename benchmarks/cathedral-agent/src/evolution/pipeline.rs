use crate::evolution::sepl::AutogenesisOperator;
use crate::sandbox::WasiPreview2Sandbox;
use crate::version_manager::VersionManager;

pub struct EvolutionPipeline {
    pub operator: Box<AutogenesisOperator>,
    pub sandbox: WasiPreview2Sandbox,
    pub version_manager: VersionManager,
}
impl EvolutionPipeline {
    pub fn new(
        operator: Box<AutogenesisOperator>,
        sandbox: WasiPreview2Sandbox,
        version_manager: VersionManager,
        _retries: usize,
    ) -> Self {
        Self {
            operator,
            sandbox,
            version_manager,
        }
    }
}
