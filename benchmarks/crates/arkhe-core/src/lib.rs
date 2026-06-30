pub mod string_safe;
#[derive(Debug)]
pub struct ArkheError(pub String);
impl std::fmt::Display for ArkheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ArkheError: {}", self.0)
    }
}
impl std::error::Error for ArkheError {}
