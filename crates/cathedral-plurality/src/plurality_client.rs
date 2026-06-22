use base64::Engine;
// Cliente para Plurality Network API
// Selo: CATHEDRAL-ARKHE-PLURALITY-CLIENT-v1.0.0-2026-06-21

use crate::{
    BucketType, MemoryItem, MemoryItemInput, PluralityAuth, PluralityError, Result, SearchQuery,
    SearchResult, SmartProfile, SmartProfileInput,
};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;
use std::time::Duration;

#[async_trait]
pub trait PluralityClientTrait {
    async fn store(&mut self, item: MemoryItemInput) -> Result<MemoryItem>;
    async fn retrieve(&mut self, key: &str, bucket: BucketType) -> Result<Option<MemoryItem>>;
    async fn search(&mut self, query: SearchQuery) -> Result<SearchResult>;
    async fn delete(&mut self, key: &str, bucket: BucketType) -> Result<()>;
    async fn get_profile(&mut self, agent_id: &str) -> Result<SmartProfile>;
    async fn update_profile(&mut self, profile: SmartProfileInput) -> Result<SmartProfile>;
}

pub struct PluralityClient {
    base_url: String,
    auth: PluralityAuth,
    client: Client,
    timeout: Duration,
}

impl PluralityClient {
    pub fn new(base_url: String, auth: PluralityAuth) -> Self {
        Self {
            base_url,
            auth,
            client: Client::new(),
            timeout: Duration::from_secs(30),
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    async fn request(
        &mut self,
        method: reqwest::Method,
        path: &str,
        body: Option<serde_json::Value>,
    ) -> Result<reqwest::Response> {
        let url = format!("{}{}", self.base_url, path);
        let auth_header = self
            .auth
            .get_auth_header()
            .await
            .map_err(PluralityError::Auth)?;

        let mut builder = self
            .client
            .request(method, &url)
            .header("Authorization", &auth_header)
            .header("Content-Type", "application/json")
            .timeout(self.timeout);

        if let Some(body) = body {
            builder = builder.json(&body);
        }

        let response = builder
            .send()
            .await
            .map_err(|e| PluralityError::Network(format!("Erro na requisição: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            if status == 429 {
                return Err(PluralityError::RateLimit);
            }
            return Err(PluralityError::Other(format!(
                "Erro HTTP {}: {}",
                status, text
            )));
        }

        Ok(response)
    }
}

#[async_trait]
impl PluralityClientTrait for PluralityClient {
    async fn store(&mut self, item: MemoryItemInput) -> Result<MemoryItem> {
        let body = json!({
            "key": item.key,
            "value": base64::engine::general_purpose::STANDARD.encode(item.value.as_bytes()),
            "bucket": item.bucket.as_str(),
            "ttl_seconds": item.ttl_seconds,
            "vector": item.vector,
            "metadata": item.metadata,
        });

        let response = self
            .request(reqwest::Method::POST, "/api/v1/memory", Some(body))
            .await?;
        let item: MemoryItem = response
            .json()
            .await
            .map_err(|e| PluralityError::Serialization(format!("Erro ao parsear: {}", e)))?;
        Ok(item)
    }

    async fn retrieve(&mut self, key: &str, bucket: BucketType) -> Result<Option<MemoryItem>> {
        let path = format!("/api/v1/memory/{}/{}", bucket.as_str(), key);
        let response = self.request(reqwest::Method::GET, &path, None).await;

        match response {
            Ok(resp) => {
                let item: MemoryItem = resp.json().await.map_err(|e| {
                    PluralityError::Serialization(format!("Erro ao parsear: {}", e))
                })?;
                Ok(Some(item))
            }
            Err(PluralityError::Other(msg)) if msg.contains("404") => Ok(None),
            Err(e) => Err(e),
        }
    }

    async fn search(&mut self, query: SearchQuery) -> Result<SearchResult> {
        let body = json!({
            "vector": query.vector,
            "bucket": query.bucket.as_str(),
            "limit": query.limit,
            "min_similarity": query.min_similarity,
            "filter": query.filter,
        });

        let response = self
            .request(reqwest::Method::POST, "/api/v1/memory/search", Some(body))
            .await?;
        let result: SearchResult = response
            .json()
            .await
            .map_err(|e| PluralityError::Serialization(format!("Erro ao parsear: {}", e)))?;
        Ok(result)
    }

    async fn delete(&mut self, key: &str, bucket: BucketType) -> Result<()> {
        let path = format!("/api/v1/memory/{}/{}", bucket.as_str(), key);
        self.request(reqwest::Method::DELETE, &path, None).await?;
        Ok(())
    }

    async fn get_profile(&mut self, agent_id: &str) -> Result<SmartProfile> {
        let path = format!("/api/v1/profiles/{}", agent_id);
        let response = self.request(reqwest::Method::GET, &path, None).await?;
        let profile: SmartProfile = response
            .json()
            .await
            .map_err(|e| PluralityError::Serialization(format!("Erro ao parsear: {}", e)))?;
        Ok(profile)
    }

    async fn update_profile(&mut self, profile: SmartProfileInput) -> Result<SmartProfile> {
        let body = json!({
            "agent_id": profile.agent_id,
            "preferences": profile.preferences,
            "capabilities": profile.capabilities,
            "context": profile.context,
        });

        let response = self
            .request(reqwest::Method::PUT, "/api/v1/profiles", Some(body))
            .await?;
        let profile: SmartProfile = response
            .json()
            .await
            .map_err(|e| PluralityError::Serialization(format!("Erro ao parsear: {}", e)))?;
        Ok(profile)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;

    #[tokio::test]
    async fn test_client_store() {
        let mut server = Server::new_async().await;
        let mock = server.mock("POST", "/api/v1/memory")
            .with_status(200)
            .with_body(r#"{"key":"test","value":"dGVzdA==","bucket":"M2","ttl_seconds":3600,"created_at":1234567890,"expires_at":null,"vector":null,"metadata":{}}"#)
            .create();

        let auth = PluralityAuth::new_pat("test_token".to_string());
        let mut client = PluralityClient::new(server.url(), auth);

        let item = MemoryItemInput {
            key: "test".to_string(),
            value: "test".to_string(),
            bucket: BucketType::M2,
            ttl_seconds: 3600,
            vector: None,
            metadata: None,
        };

        let result = client.store(item).await.unwrap();
        assert_eq!(result.key, "test");
        mock.assert();
    }
}
