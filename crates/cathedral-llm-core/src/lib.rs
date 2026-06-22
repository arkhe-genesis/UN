use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ModelTier {
    Pro,
    Plus,
    Standard,
    Lite,
}

impl Display for ModelTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelTier::Pro => write!(f, "Pro"),
            ModelTier::Plus => write!(f, "Plus"),
            ModelTier::Standard => write!(f, "Standard"),
            ModelTier::Lite => write!(f, "Lite"),
        }
    }
}

pub struct CathedralCore {
    // pesos e tokenizer no futuro
}

impl CathedralCore {
    pub async fn new() -> Self {
        Self {}
    }

    pub fn for_tier(&self, tier: ModelTier) -> CathedralModelInstance {
        CathedralModelInstance { tier }
    }
}

pub struct CathedralModelInstance {
    tier: ModelTier,
}

impl CathedralModelInstance {
    pub async fn generate_with_thinking(
        &self,
        prompt: &str,
    ) -> Result<(String, Option<String>), String> {
        // Mock implementation
        let thinking = if prompt.contains("L0") {
            None
        } else {
            Some("<think>\nThinking process mock...\n</think>".to_string())
        };

        let response = if prompt.contains("nome") {
            "Mock response: Meu nome é João.".to_string()
        } else if prompt.contains("café") {
            "Mock response: Eu gosto de café.".to_string()
        } else {
            format!(
                "Mock response for prompt length {}. Tier: {}",
                prompt.len(),
                self.tier
            )
        };
        Ok((response, thinking))
    }
}
