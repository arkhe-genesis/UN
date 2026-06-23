use clap::Parser;
use std::path::PathBuf;
use std::fs;
use cathedral_identity::pqc::{PqcAlgorithm, PqcKeyPair};

#[derive(Parser)]
pub struct SignArgs {
    #[arg(short, long)]
    payload: PathBuf,

    #[arg(long, default_value = "ed25519")]
    algorithm: String,

    #[arg(long, default_value = "hex")]
    output_format: String,
}

pub async fn execute(args: SignArgs) -> Result<(), String> {
    let payload_bytes = fs::read(&args.payload)
        .map_err(|e| format!("Erro ao ler payload: {}", e))?;

    let algorithm = match args.algorithm.as_str() {
        "ed25519" => PqcAlgorithm::Ed25519,
        #[cfg(feature = "mldsa")]
        "mldsa" => PqcAlgorithm::Mldsa65,
        _ => return Err("Algoritmo não suportado (apenas ed25519 e mldsa (se habilitado) sao validos)".to_string()),
    };

    let keypair = PqcKeyPair::generate(algorithm).map_err(|_| "Failed to generate keypair".to_string())?;

    let sig_bytes = keypair.sign(&payload_bytes).map_err(|_| "Failed to sign".to_string())?;

    match args.output_format.as_str() {
        "hex" => println!("{}", hex::encode(&sig_bytes)),
        "base64" => {
            // Using a stub for base64 since it requires adding the base64 crate
            // and I want to avoid workspace compilation breakage from new deps unless strictly required
            // In a real implementation this would use the base64 crate.
            println!("Base64 output not implemented in prototype. Use hex.");
        },
        _ => return Err("Formato de saída inválido".to_string()),
    }

    Ok(())
}
