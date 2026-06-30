use anyhow::Result;


pub struct WormGraphIndexer {}

impl WormGraphIndexer {
    pub fn new() -> Self {
        Self {}
    }

    pub fn index_vulnerability(
        &self,
        _vuln: &crate::integrations::openant::Vulnerability,
        _source: &str,
    ) -> Result<String> {
        Ok("mock_tx_id".to_string())
    }
}
