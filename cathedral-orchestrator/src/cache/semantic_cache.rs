//! Cathedral ARKHE v28.3 — Semantic Cache via Qdrant (ACP)
//! Armazena embeddings de prompts e respostas para evitar chamadas repetidas ao LLM.
//! Selo: CATHEDRAL-ARKHE-v28.3-SEMANTIC-CACHE-2026-06-16
//! Arquiteto ORCID: 0009-0005-2697-4668

use anyhow::Result;
use qdrant_client::Qdrant;
use qdrant_client::qdrant::{
    self, CreateCollection, PointStruct, SearchPoints, VectorParams, VectorsConfig, Distance
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};
use uuid::Uuid;

/// Configuração do cache semântico.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticCacheConfig {
    pub qdrant_url: String,
    pub collection_name: String,
    pub embedding_model: String,
    pub similarity_threshold: f32,
    pub ttl_seconds: u64,
}

impl Default for SemanticCacheConfig {
    fn default() -> Self {
        Self {
            qdrant_url: "http://localhost:6333".into(),
            collection_name: "oracle_cache".into(),
            embedding_model: "all-MiniLM-L6-v2".into(),
            similarity_threshold: 0.95,
            ttl_seconds: 3600,
        }
    }
}

/// Cache semântico usando Qdrant.
pub struct SemanticCache {
    client: Qdrant,
    config: SemanticCacheConfig,
}

impl SemanticCache {
    pub async fn new(config: SemanticCacheConfig) -> Result<Self> {
        let client = Qdrant::from_url(&config.qdrant_url).build()?;
        // Garantir coleção
        if !client.collection_exists(&config.collection_name).await? {
            client
                .create_collection(CreateCollection {
                    collection_name: config.collection_name.clone(),
                    vectors_config: Some(VectorsConfig {
                        config: Some(qdrant::vectors_config::Config::Params(VectorParams {
                            size: 384, // all-MiniLM-L6-v2 dim
                            distance: Distance::Cosine.into(),
                            ..Default::default()
                        })),
                    }),
                    ..Default::default()
                })
                .await?;
        }
        Ok(Self { client, config })
    }

    /// Busca uma resposta cacheada para um prompt.
    /// Retorna `None` se nenhum cache com similaridade acima do threshold.
    pub async fn get(&self, prompt: &str) -> Option<String> {
        let embedding = self.embed(prompt).await.ok()?;
        let search_result = self.client
            .search_points(SearchPoints {
                collection_name: self.config.collection_name.clone(),
                vector: embedding,
                limit: 1,
                score_threshold: Some(self.config.similarity_threshold),
                ..Default::default()
            })
            .await
            .ok()?;
        if let Some(point) = search_result.result.first() {
            let response = point.payload.get("response")?.as_str()?.to_string();
            debug!("Cache hit for prompt: {}", prompt);
            return Some(response);
        }
        None
    }

    /// Armazena um par prompt‑resposta no cache.
    pub async fn set(&self, prompt: &str, response: &str) -> Result<()> {
        let embedding = self.embed(prompt).await?;
        let mut payload: HashMap<String, qdrant_client::qdrant::Value> = HashMap::new();
        payload.insert("prompt".to_string(), serde_json::json!(prompt).into());
        payload.insert("response".to_string(), serde_json::json!(response).into());
        let point = PointStruct::new(
            Uuid::new_v4().to_string(),
            embedding,
            payload,
        );
        use qdrant_client::qdrant::UpsertPoints;

        self.client
            .upsert_points(UpsertPoints {
                collection_name: self.config.collection_name.clone(),
                points: vec![point],
                ..Default::default()
            })
            .await?;
        Ok(())
    }

    /// Obtém embedding de texto (stub; em produção, chamar serviço de embeddings).
    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        // Stub: usa comprimento da string como embedding (não real)
        let mut vec = vec![0.0; 384];
        for i in 0..vec.len() {
            vec[i] = ((text.len() >> (i % 64)) & 0xFF) as f32 / 255.0;
        }
        Ok(vec)
    }
}

/// Wrapper ACP para expor o cache via Agent Communication Protocol.
pub struct AcpSemanticCache {
    cache: SemanticCache,
}

impl AcpSemanticCache {
    pub fn new(cache: SemanticCache) -> Self {
        Self { cache }
    }

    /// Verifica o cache antes de enviar para o Oracle.
    /// Se encontrado, retorna a resposta; caso contrário, retorna None.
    pub async fn check_oracle_cache(&self, prompt: &str) -> Option<String> {
        self.cache.get(prompt).await
    }

    /// Após obter resposta do Oracle, armazena no cache.
    pub async fn store_oracle_response(&self, prompt: &str, response: &str) -> Result<()> {
        self.cache.set(prompt, response).await
    }
}