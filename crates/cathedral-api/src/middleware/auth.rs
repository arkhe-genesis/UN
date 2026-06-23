use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};

#[derive(Clone)]
pub struct AuthInfo {
    pub did: String,
    pub signature: Vec<u8>,
}

pub async fn auth_middleware(
    mut req: Request,
    next: Next,
) -> Response {
    let did = req.headers()
        .get("X-DID")
        .and_then(|v| v.to_str().ok())
        .map(String::from);

    let signature = req.headers()
        .get("X-Signature")
        .and_then(|v| hex::decode(v.as_bytes()).ok());

    if let (Some(did), Some(sig)) = (did, signature) {
        req.extensions_mut().insert(AuthInfo { did, signature: sig });
    }
    next.run(req).await
}
