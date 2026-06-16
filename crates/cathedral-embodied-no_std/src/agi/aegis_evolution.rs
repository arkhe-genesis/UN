use crate::context::ContextEmbedding;
use crate::policy::ZkMemoryProofPolicy;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct HubPerformance {
    pub acceptance_rate: f32,
    pub recommendation_volume: u32,
    pub roas: f32,
}

pub struct AegisEvolution {
    picoads_api_key: Option<String>,
    picoads_backend: Option<String>,
    hub_performances: HashMap<String, HubPerformance>,
}

impl AegisEvolution {
    pub fn new(picoads_api_key: Option<String>, picoads_backend: Option<String>) -> Self {
        Self {
            picoads_api_key,
            picoads_backend,
            hub_performances: HashMap::new(),
        }
    }

    pub fn update_hub_performance(&mut self, hub: String, acceptance_rate: f32, volume: u32) {
        let entry = self.hub_performances.entry(hub).or_default();
        entry.acceptance_rate = acceptance_rate;
        entry.recommendation_volume = volume;
    }

    /// Decides whether to use PicoAds based on current context and policy
    pub fn should_use_picoads(&self, context: &ContextEmbedding, hub: Option<&str>) -> bool {
        // Example logic (can be evolved by AEGIS later)
        if context.high_risk_action_rate > 0.35 {
            return true;
        }

        if let Some(h) = hub {
            if h == "defi-yield" || h == "agent-tools" {
                return context.acceptance_rate > 0.6;
            }
        }

        context.memory_proof_usage_rate > 0.3
    }

    pub fn evolve_policy(
        &mut self,
        _current_policy: &mut ZkMemoryProofPolicy,
        context: &ContextEmbedding,
    ) {
        // ... existing logic ...

        // === PicoAds Integration ===
        if self.should_use_picoads(&context, None) {
            // You can trigger a recommendation request here
            // or let the agent decide via tools.
            // For now we just log the decision.
            println!(
                "[AegisEvolution] PicoAds recommendation recommended this round (acceptance={:.2})",
                context.acceptance_rate
            );
        }

        // ... rest of evolution logic ...
    }
}
