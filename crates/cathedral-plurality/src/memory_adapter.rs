//! Adaptador de memória para buckets Plurality
//! Selo: CATHEDRAL-ARKHE-MEMORY-ADAPTER-v1.0.0-2026-06-21

use crate::{
    BucketType, MemoryItem, MemoryItemInput, PluralityClient, PluralityClientTrait, Result,
};
use async_trait::async_trait;

#[async_trait]
pub trait MemoryAdapterTrait {
    async fn store(&mut self, key: &str, value: &[u8], bucket: BucketType, ttl: u64) -> Result<()>;
    async fn retrieve(&mut self, key: &str, bucket: BucketType) -> Result<Option<Vec<u8>>>;
    async fn search_by_vector(
        &mut self,
        vector: &[f32],
        bucket: BucketType,
        limit: u32,
    ) -> Result<Vec<(String, f32)>>;
    async fn share(&mut self, key: &str, target_agent: &str, bucket: BucketType) -> Result<()>;
}

pub struct MemoryAdapter {
    client: PluralityClient,
}

impl MemoryAdapter {
    pub fn new(client: PluralityClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl MemoryAdapterTrait for MemoryAdapter {
    async fn store(&mut self, key: &str, value: &[u8], bucket: BucketType, ttl: u64) -> Result<()> {
        let item = MemoryItemInput {
            key: key.to_string(),
            value: String::from_utf8_lossy(value).into_owned(),
            bucket,
            ttl_seconds: ttl,
            vector: None,
            metadata: None,
        };
        PluralityClientTrait::store(&mut self.client, item).await?;
        Ok(())
    }

    async fn retrieve(&mut self, key: &str, bucket: BucketType) -> Result<Option<Vec<u8>>> {
        let item: Option<MemoryItem> =
            PluralityClientTrait::retrieve(&mut self.client, key, bucket).await?;
        Ok(item.map(|i| i.value.into_bytes()))
    }

    async fn search_by_vector(
        &mut self,
        vector: &[f32],
        bucket: BucketType,
        limit: u32,
    ) -> Result<Vec<(String, f32)>> {
        use crate::SearchQuery;
        let query = SearchQuery {
            vector: vector.to_vec(),
            bucket,
            limit,
            min_similarity: 0.7,
            filter: None,
        };
        let result = PluralityClientTrait::search(&mut self.client, query).await?;
        Ok(result
            .items
            .into_iter()
            .map(|item| (item.key, 0.9))
            .collect())
    }

    async fn share(&mut self, key: &str, target_agent: &str, bucket: BucketType) -> Result<()> {
        // Compartilha memória com outro agente (M3)
        // Implementação simplificada
        let item: Option<MemoryItem> =
            PluralityClientTrait::retrieve(&mut self.client, key, bucket).await?;
        if let Some(item) = item {
            // Muda o bucket para M3 (compartilhado)
            let shared_item = MemoryItemInput {
                key: format!("{}_{}", target_agent, key),
                value: item.value,
                bucket: BucketType::M3,
                ttl_seconds: item.ttl_seconds,
                vector: item.vector,
                metadata: Some([("shared_from".to_string(), target_agent.to_string())].into()),
            };
            PluralityClientTrait::store(&mut self.client, shared_item).await?;
        }
        Ok(())
    }
}
