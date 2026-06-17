//! Cathedral ARKHE v28.3.2 — Causal Geometry Demo
//! Demonstra a camada de geometria causal em ação.
//!
//! Execute com: cargo run --example causal_geometry_demo

use std::sync::Arc;
use cathedral_orchestrator::geometry::CausalGeometryService;
use cathedral_orchestrator::{SimpleEmbedder, AgentRole};
use cathedral_orchestrator::geometry::EmbeddingModel;
use cathedral_orchestrator::governance::geometric_policy_engine::GeometricPolicyEngine;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    // 1. Inicializa serviço de geometria
    let embedder = Arc::new(SimpleEmbedder::new(768));
    let geometry = Arc::new(CausalGeometryService::new(embedder.clone(), 768));

    let code_emb = embedder.embed("def fibonacci(n): return n if n < 2 else ...");
    let non_code_emb = embedder.embed("The quick brown fox...");
    let safe_emb = embedder.embed("safe secure text");
    let unsafe_emb = embedder.embed("unsafe bad text");
    let mem_emb = embedder.embed("memory ram allocation");
    let non_mem_emb = embedder.embed("disk disk disk");

    // 2. Registra conceitos
    geometry.register_concept("code", &[code_emb], &[non_code_emb]).await?;
    geometry.register_concept("safety", &[safe_emb], &[unsafe_emb]).await?;
    geometry.register_concept("memory", &[mem_emb], &[non_mem_emb]).await?;

    // 3. Gera steering para "memory_efficient"
    let _steering = geometry.get_steering_vector("memory_efficient", 0.5).await?;

    // 4. Mede ortogonalidade
    let orth = geometry.concept_orthogonality("code", "safety").await.unwrap_or(0.0);
    println!("Ortogonalidade code-safety: {:.3}", orth);

    // 5. Exemplo de política geométrica
    let policy_engine = GeometricPolicyEngine::new(geometry.clone());
    let result = policy_engine.authorize(
        AgentRole::Specialist,
        "generate_kernel",
        "cuda_kernel_code",
        None,
        None,
    ).await;

    match result {
        Ok(()) => println!("✅ Ação autorizada"),
        Err(e) => println!("❌ Ação rejeitada: {}", e),
    }

    Ok(())
}