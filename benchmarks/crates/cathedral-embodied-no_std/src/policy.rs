#[derive(Default, Debug, Clone)]
pub struct ZkMemoryProofPolicy {
    pub require_memory_proof_for_recommendations: bool,
    pub min_recommendation_value_usd: f32,
    pub allowed_recommendation_hubs: Vec<String>,
    pub always_require_memory_proof_for_unknown_hubs: bool,
}

impl ZkMemoryProofPolicy {
    pub fn should_require_memory_proof_for_recommendation(
        &self,
        hub: Option<&str>,
        estimated_value_usd: f32,
    ) -> bool {
        if !self.require_memory_proof_for_recommendations {
            return false;
        }

        if let Some(h) = hub {
            if !self.allowed_recommendation_hubs.iter().any(|allowed| allowed == h) {
                return self.always_require_memory_proof_for_unknown_hubs;
            }
        } else {
            return self.always_require_memory_proof_for_unknown_hubs;
        }

        if estimated_value_usd >= self.min_recommendation_value_usd {
            return true;
        }

        false
    }
}
