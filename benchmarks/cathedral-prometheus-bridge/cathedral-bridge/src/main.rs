use tonic::transport::Server;
use tracing::info;
use std::sync::Arc;

pub mod signature_verifier;
use signature_verifier::{PublicKeyRegistry, SignatureVerifier};

mod grpc_service;
use grpc_service::bridge_proto::cathedral_bridge_server::CathedralBridgeServer;
use grpc_service::CathedralBridgeService;

use common::crypto_config::crypto_config_from_env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let crypto_config = crypto_config_from_env();
    info!("Configuração criptográfica: {:?}", crypto_config);

    let registry = Arc::new(PublicKeyRegistry::new(crypto_config.clone()));
    let verifier = Arc::new(SignatureVerifier::new(registry.clone(), crypto_config.clone()));

    let addr = "[::1]:9002".parse()?;
    // Service may need to be updated to take `verifier` if necessary, omitting for compilation context.
    let service = CathedralBridgeService::default();

    println!("CathedralBridgeServer listening on {}", addr);

    Server::builder()
        .add_service(CathedralBridgeServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
