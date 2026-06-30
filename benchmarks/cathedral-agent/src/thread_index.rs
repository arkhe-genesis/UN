pub struct ThreadIndex {}
impl ThreadIndex {
    pub async fn get_usage_metrics(&self, _id: &str) -> Result<Vec<u8>, String> {
        Ok(vec![])
    }
}
