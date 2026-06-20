use tonic::transport::Server;

mod grpc_service;
use grpc_service::bridge_proto::cathedral_bridge_server::CathedralBridgeServer;
use grpc_service::CathedralBridgeService;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:9002".parse()?;
    let service = CathedralBridgeService::default();

    println!("CathedralBridgeServer listening on {}", addr);

    Server::builder()
        .add_service(CathedralBridgeServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
