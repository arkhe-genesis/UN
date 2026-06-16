// cathedral-embodied-no_std/src/picoads/client.rs
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendationRequest {
    pub query: String,
    pub hub: Option<String>,
    pub max_results: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PicoAdsRecommendation {
    pub title: String,
    pub description: String,
    pub hub: String,
    pub url: String,
    pub estimated_value_usd: Option<f64>,
}

pub struct PicoAdsClient {
    client: Client,
    api_key: String,
    base_url: String,
}

impl PicoAdsClient {
    pub fn new(api_key: impl Into<String>, base_url: Option<String>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            api_key: api_key.into(),
            base_url: base_url.unwrap_or_else(|| "https://picoads.xyz".to_string()),
        }
    }

    pub async fn get_recommendations(
        &self,
        query: &str,
        hub: Option<&str>,
        max_results: Option<u32>,
    ) -> Result<Vec<PicoAdsRecommendation>, reqwest::Error> {
        let url = format!("{}/recommendations", self.base_url);

        let body = RecommendationRequest {
            query: query.to_string(),
            hub: hub.map(|s| s.to_string()),
            max_results,
        };

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await?;

        let recommendations: Vec<PicoAdsRecommendation> = response.json().await?;
        Ok(recommendations)
    }

    /// Optional: Register agent (if you want to do it from Rust)
    pub async fn register_agent(&self, _name: &str, _description: &str) -> Result<String, reqwest::Error> {
        // let url = format!("{}/agents/register", self.base_url);
        // Implementation...
        todo!()
    }
}
