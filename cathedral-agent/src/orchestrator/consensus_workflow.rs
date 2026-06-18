//! Cathedral ARKHE v28.3 — Consensus Workflow
//! Workflow de consenso assíncrono para a Orquestração Multi-Agent.
//!
//! Selo: CATHEDRAL-ARKHE-v28.3-CONSENSUS-WORKFLOW-2026-06-16
//! Arquiteto ORCID: 0009-0005-2697-4668

use std::time::Instant;
use std::collections::HashMap;

/// Agent Identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AgentId(pub String);

/// Papel do Agente no Consenso
#[derive(Debug, Clone, PartialEq)]
pub enum AgentRole {
    Guardian,
    Architect,
    Curator,
    Validator,
    Observer,
}

impl AgentRole {
    pub fn authority_level(&self) -> u8 {
        match self {
            Self::Guardian => 255,
            Self::Architect => 200,
            Self::Curator => 150,
            Self::Validator => 100,
            Self::Observer => 50,
        }
    }
}

/// Modos de Consenso
#[derive(Debug, Clone, PartialEq)]
pub enum ConsensusMode {
    Dictator,
    MajorityVote,
    WeightedVote,
    Unanimous,
    Debate,
}

/// Estado do Workflow
#[derive(Debug, Clone, PartialEq)]
pub enum ConsensusWorkflowState {
    Draft,
    VotingOpen,
    VotingClosed,
    Tallying,
    Finalized,
    Failed,
    Overridden,
}

/// Representa o voto de um agente
#[derive(Debug, Clone)]
pub struct Vote {
    pub agent_id: AgentId,
    pub role: AgentRole,
    pub position: String,
    pub reasoning: String,
    pub confidence: f32, // 0.0 a 1.0
}

/// Resultado do Consenso
#[derive(Debug, Clone)]
pub enum ConsensusResult {
    Reached { agreement_ratio: f32, winning_position: String },
    Failed { reason: String },
}

/// Proposta de Consenso
#[derive(Debug, Clone)]
pub struct ConsensusProposal {
    pub proposal_id: String,
    pub title: String,
    pub description: String,
    pub proposed_by: AgentId,
    pub options: Vec<String>,
    pub required_roles: Vec<AgentRole>,
    pub consensus_mode: ConsensusMode,
    pub voting_deadline_secs: u64,
    pub created_at: u64,
    pub workflow_state: ConsensusWorkflowState,
    pub votes: HashMap<AgentId, Vote>,
    pub result: Option<ConsensusResult>,
    pub finalized_decision: Option<String>,
}

/// Workflow completo de consenso.
pub struct ConsensusWorkflow {
    pub proposal: ConsensusProposal,
    participants: Vec<AgentId>,
    start_time: Instant,
}

impl ConsensusWorkflow {
    /// Inicia um novo workflow de consenso.
    pub fn new(
        title: String,
        description: String,
        proposed_by: AgentId,
        options: Vec<String>,
        required_roles: Vec<AgentRole>,
        consensus_mode: ConsensusMode,
        voting_deadline_secs: u64,
    ) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let proposal = ConsensusProposal {
            proposal_id: "id_placeholder".to_string(), // In actual app, use proper uuid
            title,
            description,
            proposed_by,
            options,
            required_roles,
            consensus_mode,
            voting_deadline_secs,
            created_at: now,
            workflow_state: ConsensusWorkflowState::Draft,
            votes: HashMap::new(),
            result: None,
            finalized_decision: None,
        };

        Self {
            proposal,
            participants: Vec::new(),
            start_time: Instant::now(),
        }
    }

    /// Define os participantes (agentes que podem votar).
    pub fn with_participants(mut self, participants: Vec<AgentId>) -> Self {
        self.participants = participants;
        self
    }

    /// Abre a votação (transição Draft → VotingOpen).
    pub fn open_voting(&mut self) -> Result<(), String> {
        if self.proposal.workflow_state != ConsensusWorkflowState::Draft {
            return Err(format!("Cannot open voting from state {:?}", self.proposal.workflow_state));
        }
        self.proposal.workflow_state = ConsensusWorkflowState::VotingOpen;
        Ok(())
    }

    /// Registra um voto de um agente.
    pub fn cast_vote(&mut self, agent_id: AgentId, position: String, reasoning: String, confidence: f32) -> Result<(), String> {
        if self.proposal.workflow_state != ConsensusWorkflowState::VotingOpen {
            return Err(format!("Voting is not open (state: {:?})", self.proposal.workflow_state));
        }

        // Verifica se o agente está autorizado a votar
        if !self.participants.contains(&agent_id) {
            return Err(format!("Agent {:?} not in participant list", agent_id));
        }

        // Verifica se a opção é válida
        if !self.proposal.options.contains(&position) {
            return Err(format!("Invalid option '{}'. Valid: {:?}", position, self.proposal.options));
        }

        let vote = Vote {
            agent_id: agent_id.clone(),
            role: AgentRole::Observer, // Em produção, obter role real
            position,
            reasoning,
            confidence,
        };

        self.proposal.votes.insert(agent_id, vote);
        Ok(())
    }

    /// Fecha a votação (por tempo ou manualmente) e apura o resultado.
    pub async fn close_and_tally(&mut self) -> Result<ConsensusResult, String> {
        if self.proposal.workflow_state != ConsensusWorkflowState::VotingOpen {
            return Err(format!("Cannot close voting from state {:?}", self.proposal.workflow_state));
        }

        // Se já passou do deadline, pode fechar automaticamente
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        if now > self.proposal.created_at + self.proposal.voting_deadline_secs {
            self.proposal.workflow_state = ConsensusWorkflowState::VotingClosed;
        } else {
            // Fechamento manual
            self.proposal.workflow_state = ConsensusWorkflowState::VotingClosed;
        }

        let result = self.tally_votes();
        self.proposal.result = Some(result.clone());

        // Se consenso alcançado, finaliza
        if let ConsensusResult::Reached { agreement_ratio: _, winning_position } = &result {
            self.proposal.finalized_decision = Some(winning_position.clone());
            self.proposal.workflow_state = ConsensusWorkflowState::Finalized;
        } else {
            self.proposal.workflow_state = ConsensusWorkflowState::Failed;
        }

        Ok(result)
    }

    /// Apura os votos de acordo com o modo de consenso.
    fn tally_votes(&self) -> ConsensusResult {
        if self.proposal.votes.is_empty() {
            return ConsensusResult::Failed { reason: "No votes cast".to_string() };
        }

        match self.proposal.consensus_mode {
            ConsensusMode::MajorityVote => {
                let mut counts: HashMap<String, u32> = HashMap::new();
                for vote in self.proposal.votes.values() {
                    *counts.entry(vote.position.clone()).or_insert(0) += 1;
                }
                let total = self.proposal.votes.len() as u32;
                let (winning, count) = counts.into_iter().max_by_key(|(_, c)| *c).unwrap_or_default();
                let ratio = count as f32 / total as f32;
                if ratio >= 0.75 { // CONSENSUS_THRESHOLD
                    ConsensusResult::Reached { agreement_ratio: ratio, winning_position: winning }
                } else {
                    ConsensusResult::Failed { reason: format!("Majority not reached: {}%", ratio * 100.0) }
                }
            }
            ConsensusMode::Unanimous => {
                let first = self.proposal.votes.values().next().unwrap().position.clone();
                if self.proposal.votes.values().all(|v| v.position == first) {
                    ConsensusResult::Reached { agreement_ratio: 1.0, winning_position: first }
                } else {
                    ConsensusResult::Failed { reason: "Not unanimous".to_string() }
                }
            }
            ConsensusMode::WeightedVote => {
                let mut scores: HashMap<String, f32> = HashMap::new();
                for vote in self.proposal.votes.values() {
                    let weight = vote.confidence * (vote.role.authority_level() as f32 / 255.0);
                    *scores.entry(vote.position.clone()).or_insert(0.0) += weight;
                }
                let total_weight: f32 = scores.values().sum();
                let (winning, score) = scores.into_iter().max_by(|a, b| a.1.partial_cmp(&b.1).unwrap()).unwrap_or_default();
                let ratio = score / total_weight;
                if ratio >= 0.75 {
                    ConsensusResult::Reached { agreement_ratio: ratio, winning_position: winning }
                } else {
                    ConsensusResult::Failed { reason: "Weighted consensus not reached".to_string() }
                }
            }
            _ => ConsensusResult::Failed { reason: "Consensus mode not implemented in this workflow".to_string() },
        }
    }

    /// Sobrescreve a decisão manualmente (apenas Guardian).
    pub fn override_decision(&mut self, by: AgentId, decision: String) -> Result<(), String> {
        if !self.participants.contains(&by) {
            return Err(format!("Agent {:?} not in participants", by));
        }
        // Em produção, verificar role == Guardian
        self.proposal.workflow_state = ConsensusWorkflowState::Overridden;
        self.proposal.finalized_decision = Some(decision);
        Ok(())
    }
}
