use anyhow::Result;

pub struct CompletionOptions {
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
}

#[async_trait::async_trait]
pub trait LlmBackend: Send + Sync {
    async fn complete(&self, prompt: &str, options: &CompletionOptions) -> Result<String>;
    async fn embed(&self, text: &str) -> Result<Vec<f32>>;
    fn supports_multimodal(&self) -> bool;
}

pub struct GeminiFlash;
pub struct ClaudeOpus;
pub struct GptInstant;
pub struct Grok;
pub struct DeepSeekV4;

#[async_trait::async_trait]
impl LlmBackend for GeminiFlash {
    async fn complete(&self, prompt: &str, _options: &CompletionOptions) -> Result<String> {
        Ok(format!("GeminiFlash complete: {}", prompt))
    }
    async fn embed(&self, _text: &str) -> Result<Vec<f32>> {
        Ok(vec![0.1, 0.2, 0.3])
    }
    fn supports_multimodal(&self) -> bool {
        true
    }
}

#[async_trait::async_trait]
impl LlmBackend for ClaudeOpus {
    async fn complete(&self, prompt: &str, _options: &CompletionOptions) -> Result<String> {
        Ok(format!("ClaudeOpus complete: {}", prompt))
    }
    async fn embed(&self, _text: &str) -> Result<Vec<f32>> {
        Ok(vec![0.4, 0.5, 0.6])
    }
    fn supports_multimodal(&self) -> bool {
        false
    }
}

#[async_trait::async_trait]
impl LlmBackend for GptInstant {
    async fn complete(&self, prompt: &str, _options: &CompletionOptions) -> Result<String> {
        Ok(format!("GptInstant complete: {}", prompt))
    }
    async fn embed(&self, _text: &str) -> Result<Vec<f32>> {
        Ok(vec![0.7, 0.8, 0.9])
    }
    fn supports_multimodal(&self) -> bool {
        true
    }
}

#[async_trait::async_trait]
impl LlmBackend for Grok {
    async fn complete(&self, prompt: &str, _options: &CompletionOptions) -> Result<String> {
        Ok(format!("Grok complete: {}", prompt))
    }
    async fn embed(&self, _text: &str) -> Result<Vec<f32>> {
        Ok(vec![1.0, 1.1, 1.2])
    }
    fn supports_multimodal(&self) -> bool {
        false
    }
}

#[async_trait::async_trait]
impl LlmBackend for DeepSeekV4 {
    async fn complete(&self, prompt: &str, _options: &CompletionOptions) -> Result<String> {
        Ok(format!("DeepSeekV4 complete: {}", prompt))
    }
    async fn embed(&self, _text: &str) -> Result<Vec<f32>> {
        Ok(vec![1.3, 1.4, 1.5])
    }
    fn supports_multimodal(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_gemini_flash() {
        let backend = GeminiFlash;
        let options = CompletionOptions {
            max_tokens: None,
            temperature: None,
        };
        let response = backend.complete("Hello", &options).await.unwrap();
        assert_eq!(response, "GeminiFlash complete: Hello");
        assert!(backend.supports_multimodal());
    }

    #[tokio::test]
    async fn test_claude_opus() {
        let backend = ClaudeOpus;
        let options = CompletionOptions {
            max_tokens: None,
            temperature: None,
        };
        let response = backend.complete("World", &options).await.unwrap();
        assert_eq!(response, "ClaudeOpus complete: World");
        assert!(!backend.supports_multimodal());
    }
}
