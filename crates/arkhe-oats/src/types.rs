use arkhe_core::types::BeliefID;

pub struct EvaluationContext {
    pub belief_ids: Vec<BeliefID>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum PolicyDecision {
    Allow,
    Deny { reason: String },
}
