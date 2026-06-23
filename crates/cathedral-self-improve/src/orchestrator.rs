use crate::architect::CathedralArchitect;
use cathedral_wormgraph::{ImprovementProposal, WormGraphClient};
use std::sync::Arc;

pub struct SelfImprovementOrchestrator {
    architect: CathedralArchitect,
    wormgraph: Arc<WormGraphClient>,
}

impl SelfImprovementOrchestrator {
    pub fn new(architect: CathedralArchitect, wormgraph: Arc<WormGraphClient>) -> Self {
        Self {
            architect,
            wormgraph,
        }
    }

    pub async fn run_cycle(&self) -> Result<Vec<ImprovementProposal>, String> {
        // 2. CathedralArchitect analisa o monorepo
        let analysis = self.architect.analyze_monorepo().await?;

        // 3. Gera propostas com base na análise
        let mut proposals = self.architect.generate_proposals(&analysis).await?;

        // 4. Valida e persiste (simulando que todas são válidas)
        for proposal in proposals.iter_mut() {
            proposal.approve();
            if let Err(e) = self.wormgraph.save_proposal(proposal).await {
                return Err(format!("Erro ao salvar proposta: {}", e));
            }
        }

        Ok(proposals)
    }
}
