pub mod bridge_proto {
    tonic::include_proto!("cathedral.v1");
}

use bridge_proto::cathedral_bridge_client::CathedralBridgeClient;
use bridge_proto::{
    IngestRequest, Event, EventType, EventMetadata,
    GovernanceRequest, GovernanceResponse
};
use tonic::transport::Channel;
use anyhow::Result;

pub struct CathedralGrpcSdk {
    client: CathedralBridgeClient<Channel>,
    project_id: String,
    agent_id: String,
}

impl CathedralGrpcSdk {
    pub async fn new(endpoint: String, project_id: String, agent_id: String) -> Result<Self> {
        let client = CathedralBridgeClient::connect(endpoint).await?;
        Ok(Self {
            client,
            project_id,
            agent_id,
        })
    }

    pub async fn emit_design_proposed(
        &mut self,
        design_hash: String,
        parent_hashes: Vec<String>,
        payload_json: String,
    ) -> Result<()> {
        let event = Event {
            event_id: uuid::Uuid::new_v4().to_string(),
            timestamp: Some(prost_types::Timestamp {
                seconds: chrono::Utc::now().timestamp(),
                nanos: chrono::Utc::now().timestamp_subsec_nanos() as i32,
            }),
            event_type: EventType::DesignProposed.into(),
            design_hash,
            parent_hashes,
            payload_json,
            metadata: Some(EventMetadata {
                domain: "general".to_string(),
                confidence: 0.9,
                compute_cost_usd: 0.0,
                tags: vec!["design".to_string()],
            }),
        };

        let request = tonic::Request::new(IngestRequest {
            project_id: self.project_id.clone(),
            agent_id: self.agent_id.clone(),
            events: vec![event],
            batch_id: None,
        });

        self.client.ingest(request).await?;
        Ok(())
    }

    pub async fn request_governance(
        &mut self,
        event_type: EventType,
        proposed_state_json: String,
    ) -> Result<GovernanceResponse> {
        let request = tonic::Request::new(GovernanceRequest {
            request_id: uuid::Uuid::new_v4().to_string(),
            project_id: self.project_id.clone(),
            agent_id: self.agent_id.clone(),
            event_type: event_type.into(),
            proposed_state_json,
            current_state_json: "{}".to_string(),
            agent_risk_score: 0.5,
            domain: "general".to_string(),
            metadata: std::collections::HashMap::new(),
        });

        let response = self.client.request_governance(request).await?;
        Ok(response.into_inner())
    }
}
