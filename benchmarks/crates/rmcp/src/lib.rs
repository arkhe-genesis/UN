pub mod transport {
    pub fn stdio() -> () {}
}
pub mod model {
    pub struct ServerInfo {
        pub name: String,
        pub version: String,
        pub instructions: Option<String>,
        pub capabilities: ServerCapabilities,
    }
    pub struct ServerCapabilities;
    impl ServerCapabilities {
        pub fn builder() -> CapabilitiesBuilder { CapabilitiesBuilder }
    }
    pub struct CapabilitiesBuilder;
    impl CapabilitiesBuilder {
        pub fn enable_tools(self) -> Self { self }
        pub fn build(self) -> ServerCapabilities { ServerCapabilities }
    }
    pub struct CallToolResult;
    impl CallToolResult {
        pub fn success(_c: Vec<Content>) -> Self { Self }
    }
    pub struct Content;
    impl Content {
        pub fn text(_s: String) -> Self { Self }
    }
}
pub trait ServerHandler {
    fn get_info(&self) -> model::ServerInfo {
        model::ServerInfo {
            name: String::new(), version: String::new(), instructions: None,
            capabilities: model::ServerCapabilities,
        }
    }
}
pub trait ServiceExt {
    fn serve(&self, _t: ()) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self, anyhow::Error>> + Send>> where Self: Sized {
        Box::pin(async { unreachable!() }) // mock
    }
    fn waiting(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), anyhow::Error>> + Send>> {
        Box::pin(async { Ok(()) })
    }
}
impl<T> ServiceExt for T {}

// Macros mock
pub use rmcp_macros::{tool_handler, tool_router, tool};
