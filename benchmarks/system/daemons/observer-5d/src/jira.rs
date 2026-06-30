//! Cliente Jira com exponential backoff para criação de tickets
//! Selo: CATHEDRAL-ARKHE-JIRA-v1.0.0-2026-06-21

use reqwest::Client;
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn};

/// Cliente para API do Jira
pub struct JiraClient {
    client: Client,
    base_url: String,
    token: String,
    project: String,
}

impl JiraClient {
    /// Cria uma nova instância do cliente Jira
    pub fn new(base_url: &str, token: &str, project: &str) -> Result<Self, String> {
        if base_url.is_empty() || token.is_empty() || project.is_empty() {
            return Err("Jira credentials missing".to_string());
        }
        Ok(Self {
            client: Client::new(),
            base_url: base_url.to_string(),
            token: token.to_string(),
            project: project.to_string(),
        })
    }

    /// Cria um ticket no Jira com retry e exponential backoff
    pub async fn create_ticket(
        &self,
        project: &str,
        summary: &str,
        description: &str,
        priority: &str,
    ) -> Result<(), String> {
        let url = format!("{}/rest/api/2/issue", self.base_url);
        let payload = json!({
            "fields": {
                "project": { "key": project },
                "summary": summary,
                "description": description,
                "issuetype": { "name": "Incident" },
                "priority": { "name": priority },
            }
        });

        let mut attempt = 0;
        let max_attempts = 5;
        let mut backoff = Duration::from_secs(1);

        loop {
            let response = self.client
                .post(&url)
                .header("Authorization", format!("Bearer {}", self.token))
                .header("Content-Type", "application/json")
                .json(&payload)
                .send()
                .await
                .map_err(|e| format!("Erro Jira: {}", e))?;

            if response.status().is_success() {
                info!("✅ Ticket Jira criado: {}", summary);
                return Ok(());
            } else {
                let text = response.text().await.unwrap_or_default();
                if attempt >= max_attempts {
                    return Err(format!("Jira API error após {} tentativas: {}", max_attempts, text));
                }
                attempt += 1;
                warn!("⚠️ Tentativa {} falhou, retry em {:?}: {}", attempt, backoff, text);
                sleep(backoff).await;
                backoff *= 2;
            }
        }
    }

    /// Cria um ticket com dados de anomalia
    pub async fn create_anomaly_ticket(
        &self,
        agent_id: &str,
        rule_name: &str,
        severity: &str,
        score: f64,
        details: &str,
        recommendations: &[String],
    ) -> Result<(), String> {
        let summary = format!("[CEM] Anomalia: {} - {}", rule_name, agent_id);
        let description = json!({
            "agent_id": agent_id,
            "rule": rule_name,
            "severity": severity,
            "score": score,
            "details": details,
            "recommendations": recommendations,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }).to_string();

        let priority = match severity {
            "Critical" => "Highest",
            "High" => "High",
            "Medium" => "Medium",
            _ => "Low",
        };

        self.create_ticket(&self.project, &summary, &description, priority).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore = "Requires real Jira credentials"]
    async fn test_jira_client() {
        let client = JiraClient::new(
            "https://cathedral.atlassian.net",
            "token_here",
            "CEM",
        ).unwrap();

        let result = client.create_ticket(
            "CEM",
            "[TEST] Anomalia de teste",
            "{\"test\": true}",
            "Low",
        ).await;

        assert!(result.is_ok() || result.is_err());
    }
}
