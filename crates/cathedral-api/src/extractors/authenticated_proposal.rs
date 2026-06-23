use async_trait::async_trait;
use axum::{
    extract::{FromRef, FromRequestParts, Path, Query},
    http::request::Parts,
    response::{IntoResponse, Response},
};
use cathedral_inference_runtime::CathedralRuntime;
use cathedral_wormgraph::ImprovementProposal;
use std::sync::Arc;

pub struct ApiError;

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "API Error").into_response()
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(untagged)]
pub enum ProposalIdentifier {
    Path { id: String },
    Query { id: String },
    Body { id: String },
}

pub struct AuthenticatedProposal {
    pub proposal: ImprovementProposal,
    pub did: String,
    pub signature: Vec<u8>,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthenticatedProposal
where
    S: Send + Sync,
    Arc<CathedralRuntime>: FromRef<S>,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let did = parts
            .headers
            .get("X-DID")
            .and_then(|v| v.to_str().ok())
            .ok_or(ApiError.into_response())?
            .to_string();

        let signature = parts
            .headers
            .get("X-Signature")
            .and_then(|v| hex::decode(v.as_bytes()).ok())
            .ok_or(ApiError.into_response())?;

        let id = if let Some(path) = parts.extensions.get::<Path<String>>() {
            path.0.clone()
        } else if let Some(query) = parts.extensions.get::<Query<ProposalIdentifier>>() {
            match &query.0 {
                ProposalIdentifier::Query { id } => id.clone(),
                _ => return Err(ApiError.into_response()),
            }
        } else {
            return Err(ApiError.into_response());
        };

        let runtime = Arc::<CathedralRuntime>::from_ref(state);
        let proposal = runtime
            .wormgraph
            .get_proposal(&id)
            .await
            .map_err(|_| ApiError.into_response())?
            .ok_or(ApiError.into_response())?;

        if proposal.author_did != did {
            return Err(ApiError.into_response());
        }

        // Simplistic verification for the prototype. In real system, verify the payload.
        if !runtime
            .identity
            .verify(&did, &signature, b"dummy")
            .await
            .unwrap_or(false)
        {
            return Err(ApiError.into_response());
        }

        Ok(AuthenticatedProposal {
            proposal,
            did,
            signature,
        })
    }
}

pub struct AuthenticatedDid {
    pub did: String,
    pub signature: Vec<u8>,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthenticatedDid
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let did = parts
            .headers
            .get("X-DID")
            .and_then(|v| v.to_str().ok())
            .ok_or(ApiError.into_response())?
            .to_string();

        let signature = parts
            .headers
            .get("X-Signature")
            .and_then(|v| hex::decode(v.as_bytes()).ok())
            .ok_or(ApiError.into_response())?;

        Ok(AuthenticatedDid { did, signature })
    }
}
