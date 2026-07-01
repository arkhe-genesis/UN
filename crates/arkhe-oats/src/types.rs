pub struct EvaluationContext {
    pub belief_ids: Vec<u64>,
}

pub enum PolicyDecision {
    Allow,
    Deny { reason: String },
}
