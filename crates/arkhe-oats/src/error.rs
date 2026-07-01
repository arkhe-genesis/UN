#[derive(Debug, PartialEq, Eq)]
pub enum OatsError {
    InternalError(String),
}

impl std::fmt::Display for OatsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InternalError(s) => write!(f, "OATS Internal Error: {}", s),
        }
    }
}
impl std::error::Error for OatsError {}
