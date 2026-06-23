use cathedral_wormgraph::{WormGraphBackend, LedgerEntry, ImprovementProposal, ProposalFilter, Result, WormGraphError, MemoryFilter};
use dashmap::DashMap;
use std::sync::atomic::{AtomicU64, Ordering};

pub struct TestWormGraph {
    entries: DashMap<String, LedgerEntry>,
    proposals: DashMap<String, ImprovementProposal>,
    next_id: AtomicU64,
}

impl TestWormGraph {
    pub fn new() -> Self {
        Self {
            entries: DashMap::new(),
            proposals: DashMap::new(),
            next_id: AtomicU64::new(1),
        }
    }

    pub fn insert_proposal_sync(&self, mut proposal: ImprovementProposal) -> Result<()> {
        if proposal.id.is_empty() {
            proposal.id = format!("prop_{}", self.next_id.fetch_add(1, Ordering::SeqCst));
        }
        self.proposals.insert(proposal.id.clone(), proposal);
        Ok(())
    }

    pub fn populate_with_proposals(&self, count: usize, _author_did: &str) -> Result<()> {
        for i in 0..count {
            let proposal = ImprovementProposal::new(
                format!("Proposta {}", i),
                format!("Descrição da proposta {}", i),
            );
            // author_did field logic left to the instantiator (not going to mess with exact fields as they might differ from proposal vs db representation for tests)
            self.insert_proposal_sync(proposal)?;
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl WormGraphBackend for TestWormGraph {
    async fn append_entry(&self, entry: LedgerEntry) -> Result<()> {
        self.entries.insert(entry.id.clone(), entry);
        Ok(())
    }

    async fn get_entries(&self, limit: Option<usize>) -> Result<Vec<LedgerEntry>> {
        let mut vec: Vec<_> = self.entries.iter().map(|kv| kv.value().clone()).collect();
        vec.sort_by_key(|e| -e.timestamp);
        let limit = limit.unwrap_or(100);
        Ok(vec.into_iter().take(limit).collect())
    }

    async fn list_memories(&self, filter: MemoryFilter) -> Result<Vec<LedgerEntry>> {
        let mut vec: Vec<_> = self.entries.iter().map(|kv| kv.value().clone()).collect();
        if let Some(agent) = filter.agent_id {
            vec.retain(|e| e.agent_id == agent);
        }
        if let Some(decision) = filter.decision_type {
            vec.retain(|e| e.decision_type == decision);
        }
        if let Some(since) = filter.since_timestamp {
            vec.retain(|e| e.timestamp >= since);
        }
        vec.sort_by_key(|e| -e.timestamp);
        let offset = filter.offset.unwrap_or(0);
        let limit = filter.limit.unwrap_or(100);
        Ok(vec.into_iter().skip(offset).take(limit).collect())
    }

    async fn save_proposal(&self, proposal: &ImprovementProposal) -> Result<()> {
        self.proposals.insert(proposal.id.clone(), proposal.clone());
        Ok(())
    }

    async fn list_proposals(&self, filter: ProposalFilter) -> Result<Vec<ImprovementProposal>> {
        let mut vec: Vec<ImprovementProposal> = self.proposals.iter()
            .map(|kv| kv.value().clone())
            .collect();

        if let Some(risk) = filter.risk_level {
            vec.retain(|p| p.risk_level == risk);
        }
        if let Some(status) = filter.status {
            vec.retain(|p| p.validation_status == status);
        }
        if let Some(author) = filter.author_did {
            vec.retain(|p| p.author_did == author);
        }

        vec.sort_by_key(|p| -p.created_at.timestamp());

        let offset = filter.offset.unwrap_or(0);
        let limit = filter.limit.unwrap_or(100);
        Ok(vec.into_iter().skip(offset).take(limit).collect())
    }

    async fn delete_proposal(&self, id: &str, author_did: &str, _signature: &[u8]) -> Result<()> {
        if let Some(proposal) = self.proposals.get(id) {
            if proposal.author_did != author_did {
                return Err(WormGraphError::Forbidden);
            }
        }
        self.proposals.remove(id);
        Ok(())
    }

    async fn get_proposal(&self, id: &str) -> Result<Option<ImprovementProposal>> {
        Ok(self.proposals.get(id).map(|p| p.clone()))
    }

    async fn ping(&self) -> Result<()> {
        Ok(())
    }
}
