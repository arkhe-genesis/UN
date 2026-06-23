use axum::Router;
use std::sync::Arc;
use cathedral_server::api;
use cathedral_server::orchestration::orchestrator::Orchestrator;
use cathedral_wormgraph::WormGraphClient;
use cathedral_zk::ZKGateway;
use cathedral_remix_bridge::RemixClient;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    println!("Cathedral Server starting...");

    let remix = Arc::new(RemixClient::new("http://localhost:3000".to_string()));
    let wormgraph = Arc::new(WormGraphClient::new());
    let zk = Arc::new(ZKGateway::new());

    let orchestrator = Arc::new(Orchestrator::new(remix, wormgraph, zk));

    let app = api::create_routes(orchestrator);

    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    println!("Cathedral Server listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
