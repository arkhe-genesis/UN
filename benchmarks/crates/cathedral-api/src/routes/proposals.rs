use axum::{
    extract::{Path, Query, State},
    Json,
};
use cathedral_inference_runtime::CathedralRuntime;
use cathedral_wormgraph::{ImprovementProposal, ProposalFilter};
use std::sync::Arc;
use crate::extractors::authenticated_proposal::AuthenticatedDid;

pub async fn create_proposal(
    State(runtime): State<Arc<CathedralRuntime>>,
    auth: AuthenticatedDid,
    Json(mut proposal): Json<ImprovementProposal>,
) -> Json<ImprovementProposal> {
    proposal.author_did = auth.did;
    proposal.signature = auth.signature;

    let _ = runtime.wormgraph.save_proposal(&proposal).await;
    Json(proposal)
}

pub async fn list_proposals(
    State(runtime): State<Arc<CathedralRuntime>>,
    Query(filter): Query<ProposalFilter>,
) -> Json<Vec<ImprovementProposal>> {
    let proposals = runtime.wormgraph.list_proposals(filter).await.unwrap_or_default();
    Json(proposals)
}

pub async fn delete_proposal(
    State(runtime): State<Arc<CathedralRuntime>>,
    auth: AuthenticatedDid,
    Path(id): Path<String>,
) -> axum::response::Result<()> {
    // Verificações simplificadas.
    let _ = runtime.wormgraph.delete_proposal(&id, &auth.did, &auth.signature).await;
    Ok(())
}
