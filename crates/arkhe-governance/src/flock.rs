use std::path::PathBuf;

/// Configuração do Flock para provas de governança.
#[derive(Debug, Clone)]
pub struct FlockConfig {
    /// Caminho do binário flock_chain (ou None para PATH).
    pub flock_bin: Option<PathBuf>,
    /// Função de hash ("blake3", "sha256", etc).
    pub hash_function: String,
    /// Número de steps para a prova.
    pub steps: u64,
}

impl Default for FlockConfig {
    fn default() -> Self {
        Self {
            flock_bin: None,
            hash_function: "blake3".into(),
            steps: 256,
        }
    }
}
