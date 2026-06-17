use std::sync::Arc;
use tokio::sync::Mutex;
use super::ledger_relayer::ConsensusLedger;

pub struct AsyncRLConfig {}
pub struct CathedralAgent {}
pub trait RewardModel {}

pub struct DebateConsensusRewardModel {}
impl DebateConsensusRewardModel {
    pub fn new() -> Self { Self {} }
}
impl RewardModel for DebateConsensusRewardModel {}

pub struct AsyncRLOrchestrator {
    config: AsyncRLConfig,
    agent: Arc<Mutex<CathedralAgent>>,
    reward_model: Arc<dyn RewardModel>,
    ledger: Option<Arc<dyn ConsensusLedger>>,
}

impl AsyncRLOrchestrator {
    pub fn new(
        config: AsyncRLConfig,
        agent: Arc<Mutex<CathedralAgent>>,
        reward_model: Arc<dyn RewardModel>,
        ledger: Option<Arc<dyn ConsensusLedger>>,
    ) -> Self {
        Self {
            config,
            agent,
            reward_model,
            ledger,
        }
    }

    pub fn new_with_debate(
        config: AsyncRLConfig,
        agent: Arc<Mutex<CathedralAgent>>,
        ledger: Option<Arc<dyn ConsensusLedger>>,
    ) -> Self {
        let reward_model = Arc::new(DebateConsensusRewardModel::new());
        Self::new(config, agent, reward_model, ledger)
    }
}
