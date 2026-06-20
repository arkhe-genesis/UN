use tonic::{Request, Response, Status};

pub mod bridge_proto {
    tonic::include_proto!("cathedral.v1");
}

use bridge_proto::cathedral_bridge_server::CathedralBridge;
use bridge_proto::{
    IngestRequest, IngestResponse,
    GovernanceRequest, GovernanceResponse, GovernanceVerdict,
    QueryProvenanceRequest, QueryProvenanceResponse,
};

#[derive(Default)]
pub struct CathedralBridgeService {}

#[tonic::async_trait]
impl CathedralBridge for CathedralBridgeService {
    async fn ingest(
        &self,
        request: Request<IngestRequest>,
    ) -> Result<Response<IngestResponse>, Status> {
        let req = request.into_inner();
        println!("Received Ingest request for project: {}", req.project_id);

        let events_count = req.events.len() as u32;

        Ok(Response::new(IngestResponse {
            success: true,
            message: "Events ingested successfully".into(),
            events_accepted: events_count,
            rejected_event_ids: vec![],
        }))
    }

    async fn request_governance(
        &self,
        request: Request<GovernanceRequest>,
    ) -> Result<Response<GovernanceResponse>, Status> {
        let req = request.into_inner();
        println!("Received Governance request for event_type: {:?}", req.event_type);

        Ok(Response::new(GovernanceResponse {
            request_id: req.request_id,
            verdict: GovernanceVerdict::Approved.into(),
            rationale: "Approved by CathedralBridgeService".into(),
            conditions: vec![],
            evaluated_by: "MockEthicalGuardian".into(),
            evaluated_at: Some(prost_types::Timestamp {
                seconds: chrono::Utc::now().timestamp(),
                nanos: chrono::Utc::now().timestamp_subsec_nanos() as i32,
            }),
        }))
    }

    async fn query_provenance(
        &self,
        _request: Request<QueryProvenanceRequest>,
    ) -> Result<Response<QueryProvenanceResponse>, Status> {
        Ok(Response::new(QueryProvenanceResponse {
            entries: vec![],
            has_more: false,
        }))
    }
}
