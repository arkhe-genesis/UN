pub struct PublishedEvent {
    pub event_id_hex: String,
    pub relay_urls: Vec<String>,
}

pub struct NostrReplicator;
impl NostrReplicator {
    pub fn default_relays(&self) -> &[String] {
        &[]
    }
    pub async fn publish_to_relays(&self, _event: &nostr_sdk::Event, _relays: &[String]) -> Result<PublishedEvent, String> {
        Ok(PublishedEvent {
            event_id_hex: "".to_string(),
            relay_urls: vec![]
        })
    }
}
