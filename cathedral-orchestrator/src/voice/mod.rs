use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceEvidence {
    pub phrase_matched: bool,
    pub coercion_score: f64,
}

pub struct VoiceCore;

impl VoiceCore {
    pub async fn capture_phrase_for_proof_of_life(&self, phrase: Option<&str>) -> Result<VoiceEvidence, String> {
        Ok(VoiceEvidence {
            phrase_matched: phrase.is_some(),
            coercion_score: 0.1,
        })
    }
}
