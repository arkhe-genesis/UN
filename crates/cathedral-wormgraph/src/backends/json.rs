use dashmap::DashMap;
use crate::{WormGraphBackend, LedgerEntry, ImprovementProposal, ProposalFilter, Result, MemoryFilter};
use std::sync::Arc;

pub struct JsonWormGraph {
    entries: Arc<DashMap<String, LedgerEntry>>,
    proposals: Arc<DashMap<String, ImprovementProposal>>,
}

impl JsonWormGraph {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(DashMap::new()),
            proposals: Arc::new(DashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl WormGraphBackend for JsonWormGraph {
    async fn append_entry(&self, entry: LedgerEntry) -> Result<()> {
        self.entries.insert(entry.id.clone(), entry);
        Ok(())
    }

    async fn get_entries(&self, limit: Option<usize>) -> Result<Vec<LedgerEntry>> {
        self.list_memories(MemoryFilter { limit, ..Default::default() }).await
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
        let mut vec: Vec<_> = self.proposals.iter().map(|kv| kv.value().clone()).collect();
        if let Some(risk) = filter.risk_level { vec.retain(|p| p.risk_level == risk); }
        if let Some(status) = filter.status { vec.retain(|p| p.validation_status == status); }
        if let Some(author) = filter.author_did { vec.retain(|p| p.author_did == author); }
        vec.sort_by_key(|p| -p.created_at.timestamp());
        let offset = filter.offset.unwrap_or(0);
        let limit = filter.limit.unwrap_or(100);
        Ok(vec.into_iter().skip(offset).take(limit).collect())
    }

    async fn delete_proposal(&self, id: &str, _author_did: &str, _signature: &[u8]) -> Result<()> {
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
