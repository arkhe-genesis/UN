use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;

use asi_governance::runbook::{IncidentNotifier, NotifyError};

pub struct SlackNotifier {
    client: Client,
    bot_token: String,
    base_url: String,
}

impl SlackNotifier {
    pub fn new(bot_token: &str) -> Self {
        Self {
            client: Client::new(),
            bot_token: bot_token.to_string(),
            base_url: "https://slack.com/api".to_string(),
        }
    }
}

#[async_trait]
impl IncidentNotifier for SlackNotifier {
    async fn notify(&self, channel: &str, message: &str) -> Result<(), NotifyError> {
        let url = format!("{}/chat.postMessage", self.base_url);

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.bot_token))
            .json(&json!({
                "channel": channel,
                "text": message,
                "mrkdwn": true
            }))
            .send()
            .await
            .map_err(|e| NotifyError::SlackApi(e.to_string()))?;

        let body: serde_json::Value = response.json().await
            .map_err(|e| NotifyError::SlackApi(e.to_string()))?;

        if body["ok"].as_bool() != Some(true) {
            let error = body["error"].as_str().unwrap_or("unknown");
            return Err(NotifyError::SlackApi(error.to_string()));
        }

        Ok(())
    }

    async fn create_channel(&self, name: &str) -> Result<String, NotifyError> {
        let url = format!("{}/conversations.create", self.base_url);

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.bot_token))
            .json(&json!({
                "name": name,
                "is_private": true
            }))
            .send()
            .await
            .map_err(|e| NotifyError::SlackApi(e.to_string()))?;

        let body: serde_json::Value = response.json().await
            .map_err(|e| NotifyError::SlackApi(e.to_string()))?;

        if body["ok"].as_bool() != Some(true) {
            let error = body["error"].as_str().unwrap_or("unknown");
            return Err(NotifyError::SlackApi(error.to_string()));
        }

        body["channel"]["id"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| NotifyError::SlackApi("No channel ID in response".to_string()))
    }
}
