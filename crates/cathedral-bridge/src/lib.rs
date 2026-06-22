pub mod handlers {
    pub mod nostr;
    pub mod zk;
}
pub mod proto {
    pub struct ZkVerifyRequest {
        pub circuit_id: String,
        pub agent_id: String,
        pub design_hash: String,
        pub proof_bytes: Vec<u8>,
        pub public_inputs: Vec<u8>,
    }
    pub struct ZkVerifyResponse {
        pub valid: bool,
        pub circuit_id: String,
        pub verification_time_ms: String,
        pub error: Option<String>,
        pub verification_hash: Vec<u8>,
    }
    pub struct NostrPublishRequest {
        pub project_id: String,
        pub design_hash: String,
        pub wormgraph_json: String,
        pub tags: Vec<Vec<String>>,
        pub relay_urls: Vec<String>,
    }
    pub struct NostrPublishResponse {
        pub success: bool,
        pub event_id_hex: String,
        pub relay_urls: Vec<String>,
        pub error: Option<String>,
        pub published_at: u64,
    }
}
pub mod server {
    use std::collections::HashMap;
    use tokio::sync::RwLock;

    #[derive(Clone)]
    pub struct VerificationKey {
        pub hash: String,
        pub elf: Vec<u8>,
    }

    pub struct WormgraphMock;
    impl WormgraphMock {
        pub async fn append(&self, _entry: cathedral_wormgraph::ProvenanceEntry) -> Result<(), ()> {
            Ok(())
        }
    }

    pub struct BridgeState {
        pub verification_keys: RwLock<HashMap<String, VerificationKey>>,
        pub wormgraph: WormgraphMock,
        pub nostr_replicator: Option<cathedral_nostr::NostrReplicator>,
    }
}
