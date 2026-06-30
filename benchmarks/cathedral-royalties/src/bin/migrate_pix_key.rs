use std::collections::HashMap;
use std::env;

// This is a stub for HashTreeStorage and DeSciNodeResource.
// In a real scenario, this would depend on the actual arkhe or cathedral crates.

#[derive(Debug)]
struct DeSciNodeResource {
    pub contributors: Vec<Contributor>,
}

#[derive(Debug)]
struct Contributor {
    pub npub: String,
    pub pix_key: Option<String>,
}

impl DeSciNodeResource {
    pub fn from_bytes(_bytes: &[u8]) -> Result<Self, String> {
        Ok(Self {
            contributors: vec![],
        })
    }
    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        Ok(vec![])
    }
}

struct HashTreeStorage;
impl HashTreeStorage {
    pub fn new(_path: &str) -> Result<Self, String> {
        Ok(Self)
    }
    pub async fn list_entries(&self, _path: &str) -> Result<Vec<String>, String> {
        Ok(vec![])
    }
    pub async fn get_bytes(&self, _path: &str) -> Result<Vec<u8>, String> {
        Ok(vec![])
    }
    pub async fn put_bytes(&self, _path: &str, _bytes: &[u8]) -> Result<(), String> {
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    let dry_run = args.contains(&"--dry-run".to_string());
    let hashtree_path = "./hashtree".to_string();

    println!("🔍 Iniciando migração de pix_key...");
    if dry_run {
        println!("⚠️ Modo DRY RUN: nenhuma alteração será salva");
    }

    let storage = HashTreeStorage::new(&hashtree_path)?;
    let mapping: HashMap<String, String> = HashMap::new();

    let nodes_path = "desci/nodes/";
    let entries = storage.list_entries(nodes_path).await?;

    let mut updated = 0;
    let mut skipped = 0;

    for entry in entries {
        let node_id = entry.trim_start_matches(nodes_path).trim_end_matches('/');
        println!("📂 Processando Node: {}", node_id);

        let node_data = storage
            .get_bytes(&format!("{}{}/latest", nodes_path, node_id))
            .await;
        if node_data.is_err() {
            println!("   ⚠️ Node sem versão 'latest', ignorado.");
            skipped += 1;
            continue;
        }

        let mut node = DeSciNodeResource::from_bytes(&node_data.unwrap())?;

        let has_pix = node.contributors.iter().any(|c| c.pix_key.is_some());
        if has_pix {
            println!("   ✅ Node já possui pix_key, ignorado.");
            skipped += 1;
            continue;
        }

        let mut changed = false;
        for contributor in &mut node.contributors {
            if let Some(pix_key) = mapping.get(&contributor.npub) {
                contributor.pix_key = Some(pix_key.clone());
                changed = true;
                println!("   ✅ pix_key adicionado para {}", contributor.npub);
            }
        }

        if !changed {
            println!("   ⏭️ Nenhuma mudança necessária.");
            skipped += 1;
            continue;
        }

        if !dry_run {
            let bytes = node.to_bytes()?;
            storage
                .put_bytes(&format!("{}{}/latest", nodes_path, node_id), &bytes)
                .await?;
            println!("   💾 Node {} atualizado.", node_id);
            updated += 1;
        } else {
            println!("   🧪 DRY RUN: Node {} seria atualizado.", node_id);
            updated += 1;
        }
    }

    println!("✅ Migração concluída!");
    println!("   📊 Nodes atualizados: {}", updated);
    println!("   ⏭️ Nodes ignorados: {}", skipped);

    Ok(())
}
