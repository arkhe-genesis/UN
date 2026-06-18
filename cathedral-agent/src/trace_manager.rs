
pub struct TraceManager {}

impl TraceManager {
    pub fn new() -> Self { Self {} }
    pub async fn start_trace(&self, _id: &str) -> Result<String, String> { Ok("trace_id".to_string()) }
    pub async fn add_artifact(&self, _trace_id: &str, _name: &str, _data: Vec<u8>, _mime: &str, _desc: &str) -> Result<(), String> { Ok(()) }
}
