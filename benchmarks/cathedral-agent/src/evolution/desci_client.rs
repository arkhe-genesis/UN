use reqwest::Client;
use serde_json::Value;

pub struct DeSciClient {
    client: Client,
    api_url: String,
}

impl DeSciClient {
    pub fn new(api_url: &str) -> Self {
        Self {
            client: Client::new(),
            api_url: api_url.to_string(),
        }
    }

    pub async fn register_node(&self, payload: Value) -> Result<String, String> {
        let url = format!("{}/v1/nodes/register", self.api_url);
        let res = self.client.post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if res.status().is_success() {
            let json: Value = res.json().await.map_err(|e| format!("Failed to parse response: {}", e))?;
            let dpid = json["dpid"].as_str().ok_or("No dpid in response")?;
            Ok(dpid.to_string())
        } else {
            Err(format!("Failed to register node: {}", res.status()))
        }
    }

    pub async fn resolve_dpid(&self, dpid: &str) -> Result<Value, String> {
        let url = format!("{}/v1/dpid/{}", self.api_url, dpid);
        let res = self.client.get(&url)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if res.status().is_success() {
            let json: Value = res.json().await.map_err(|e| format!("Failed to parse response: {}", e))?;
            Ok(json)
        } else {
            Err(format!("Failed to resolve dpid: {}", res.status()))
        }
    }
}
