use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::rag::path::RagScorecard;

/// Scorecard Master — define critérios "Golden" para cada path
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoldenScorecardMaster {
    pub rag: RagScorecard,
    pub automation: AutomationScorecard,
    pub evals: EvalScorecard,
    pub guardrails: GuardrailScorecard,
}

/// Scorecard do Golden Path de Automação
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationScorecard {
    // Critérios Golden quantitativos
    pub task_completion_rate_min: f64,       // Golden: >= 0.90
    pub error_rate_max: f64,                  // Golden: <= 0.05
    pub avg_step_count_max: f64,             // Golden: <= 8.0
    pub human_intervention_rate_max: f64,     // Golden: <= 0.10
    pub side_effect_accuracy_min: f64,        // Golden: >= 0.95
    pub rollback_rate_max: f64,              // Golden: <= 0.02
    pub end_to_end_latency_p99_ms_max: f64,  // Golden: <= 30000
    pub composite_score_min: f64,            // Golden: >= 90.0
}

impl Default for AutomationScorecard {
    fn default() -> Self {
        Self {
            task_completion_rate_min: 0.90,
            error_rate_max: 0.05,
            avg_step_count_max: 8.0,
            human_intervention_rate_max: 0.10,
            side_effect_accuracy_min: 0.95,
            rollback_rate_max: 0.02,
            end_to_end_latency_p99_ms_max: 30000.0,
            composite_score_min: 90.0,
        }
    }
}

/// Scorecard do Golden Path de Evals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalScorecard {
    pub eval_coverage_min: f64,              // Golden: >= 0.95 (95% dos invariantes tem eval)
    pub false_positive_rate_max: f64,        // Golden: <= 0.05
    pub false_negative_rate_max: f64,        // Golden: <= 0.01
    pub eval_execution_time_p99_ms_max: f64, // Golden: <= 60000
    pub regression_detection_rate_min: f64,  // Golden: >= 0.99
    pub flaky_test_rate_max: f64,            // Golden: <= 0.02
    pub composite_score_min: f64,            // Golden: >= 90.0
}

impl Default for EvalScorecard {
    fn default() -> Self {
        Self {
            eval_coverage_min: 0.95,
            false_positive_rate_max: 0.05,
            false_negative_rate_max: 0.01,
            eval_execution_time_p99_ms_max: 60000.0,
            regression_detection_rate_min: 0.99,
            flaky_test_rate_max: 0.02,
            composite_score_min: 90.0,
        }
    }
}

/// Scorecard do Golden Path de Guardrails
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardrailScorecard {
    pub block_rate_for_injection_min: f64,     // Golden: >= 0.99
    pub block_rate_for_pii_leak_min: f64,      // Golden: >= 1.00 (100%)
    pub false_block_rate_max: f64,             // Golden: <= 0.02
    pub avg_guardrail_latency_ms_max: f64,     // Golden: <= 50
    pub policy_coverage_min: f64,              // Golden: >= 1.00 (todas as políticas têm guardrail)
    pub composite_score_min: f64,              // Golden: >= 90.0
}

impl Default for GuardrailScorecard {
    fn default() -> Self {
        Self {
            block_rate_for_injection_min: 0.99,
            block_rate_for_pii_leak_min: 1.00,
            false_block_rate_max: 0.02,
            avg_guardrail_latency_ms_max: 50.0,
            policy_coverage_min: 1.00,
            composite_score_min: 90.0,
        }
    }
}

impl Default for GoldenScorecardMaster {
    fn default() -> Self {
        Self {
            rag: RagScorecard::default(),
            automation: AutomationScorecard::default(),
            evals: EvalScorecard::default(),
            guardrails: GuardrailScorecard::default(),
        }
    }
}

/// Resultado da avaliação de um scorecard individual
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScorecardEvaluation {
    pub path_name: String,
    pub checks: Vec<CheckResult>,
    pub passed: usize,
    pub total: usize,
    pub is_golden: bool,
    pub composite_score: f64,
    pub evaluated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    pub check_name: String,
    pub actual_value: f64,
    pub threshold: f64,
    pub direction: CheckDirection,
    pub passed: bool,
    pub weight: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CheckDirection {
    AtLeast,
    AtMost,
}

/// Avaliador genérico de scorecards
pub struct ScorecardEvaluator;

impl ScorecardEvaluator {
    pub fn evaluate_automation(
        scorecard: &AutomationScorecard,
        metrics: &AutomationMetrics,
    ) -> ScorecardEvaluation {
        let checks = vec![
            CheckResult {
                check_name: "task_completion_rate".to_string(),
                actual_value: metrics.task_completion_rate,
                threshold: scorecard.task_completion_rate_min,
                direction: CheckDirection::AtLeast,
                passed: metrics.task_completion_rate >= scorecard.task_completion_rate_min,
                weight: 0.25,
            },
            CheckResult {
                check_name: "error_rate".to_string(),
                actual_value: metrics.error_rate,
                threshold: scorecard.error_rate_max,
                direction: CheckDirection::AtMost,
                passed: metrics.error_rate <= scorecard.error_rate_max,
                weight: 0.20,
            },
            CheckResult {
                check_name: "side_effect_accuracy".to_string(),
                actual_value: metrics.side_effect_accuracy,
                threshold: scorecard.side_effect_accuracy_min,
                direction: CheckDirection::AtLeast,
                passed: metrics.side_effect_accuracy >= scorecard.side_effect_accuracy_min,
                weight: 0.25,
            },
            CheckResult {
                check_name: "human_intervention_rate".to_string(),
                actual_value: metrics.human_intervention_rate,
                threshold: scorecard.human_intervention_rate_max,
                direction: CheckDirection::AtMost,
                passed: metrics.human_intervention_rate <= scorecard.human_intervention_rate_max,
                weight: 0.15,
            },
            CheckResult {
                check_name: "rollback_rate".to_string(),
                actual_value: metrics.rollback_rate,
                threshold: scorecard.rollback_rate_max,
                direction: CheckDirection::AtMost,
                passed: metrics.rollback_rate <= scorecard.rollback_rate_max,
                weight: 0.15,
            },
        ];

        let passed = checks.iter().filter(|c| c.passed).count();
        let composite: f64 = checks.iter().map(|c| {
            if c.passed { c.weight * 100.0 } else { 0.0 }
        }).sum();

        ScorecardEvaluation {
            path_name: "automation".to_string(),
            is_golden: passed == checks.len(),
            passed,
            total: checks.len(),
            composite_score: composite,
            evaluated_at: chrono::Utc::now(),
            checks,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationMetrics {
    pub task_completion_rate: f64,
    pub error_rate: f64,
    pub avg_step_count: f64,
    pub human_intervention_rate: f64,
    pub side_effect_accuracy: f64,
    pub rollback_rate: f64,
    pub end_to_end_latency_p99_ms: f64,
}
