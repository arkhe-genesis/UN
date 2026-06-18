use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmSpec {
    pub project: String,
    pub goal: String,
    pub scope: String,
    pub rules: Vec<String>,
    pub sources: Vec<String>,
    pub output: OutputSpec,
    pub on_conflict: ConflictResolution,
    pub stop_condition: StopCondition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputSpec {
    pub file_type: String,
    pub count: usize,
    pub naming_convention: String,
    pub format_details: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictResolution {
    Flag,
    Resolve,
    Ignore,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StopCondition {
    MaxSteps(usize),
    GoalReached,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmResult {
    pub agent_count: usize,
    pub total_steps: usize,
    pub outputs: Vec<String>,
}
