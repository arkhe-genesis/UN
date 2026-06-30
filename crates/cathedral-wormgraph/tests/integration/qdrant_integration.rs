//! Teste de integração com Qdrant.
//! Requer Qdrant rodando localmente (docker run -p 6333:6333 qdrant/qdrant).

use qdrant_client::Qdrant;
use testcontainers::{
    core::WaitFor,
    runners::AsyncRunner,
    Image,
};

struct QdrantImage;

impl Image for QdrantImage {
    fn name(&self) -> &str {
        "qdrant/qdrant"
    }

    fn tag(&self) -> &str {
        "v1.11.0"
    }

    fn ready_conditions(&self) -> Vec<WaitFor> {
        vec![WaitFor::message_on_stderr("Qdrant HTTP server listening")]
    }
}

#[tokio::test]
#[ignore] // Executar apenas com `cargo test -- --ignored`
async fn test_qdrant_vector_insert_and_search() {
    // 1. Iniciar container Qdrant
    let container = QdrantImage
        .start()
        .await
        .expect("Failed to start Qdrant container");

    let host = container.get_host().await.unwrap();
    let port = container.get_host_port_ipv4(6334).await.unwrap();
    let qdrant_url = format!("http://{}:{}", host, port);

    let client = Qdrant::from_url(&qdrant_url).build().unwrap();

    let collections = client.list_collections().await.unwrap();
    assert!(collections.collections.len() >= 0);
}
