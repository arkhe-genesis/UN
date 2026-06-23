use std::path::PathBuf;
use std::fs;
use walkdir::WalkDir;
use syn::{parse_file, Item};
use serde_json::json;
use cathedral_wormgraph::{ImprovementProposal, RiskLevel};

pub struct CathedralArchitect {
    root_path: PathBuf,
}

impl CathedralArchitect {
    pub fn new(root_path: PathBuf) -> Self {
        Self { root_path }
    }

    pub async fn analyze_monorepo(&self) -> Result<String, String> {
        let mut findings = Vec::new();
        let mut file_count = 0;
        let mut total_lines = 0;

        for entry in WalkDir::new(&self.root_path)
            .into_iter()
            .filter_entry(|e| !e.file_name().to_string_lossy().starts_with("target"))
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
        {
            let path = entry.path();
            file_count += 1;

            let content = fs::read_to_string(path).unwrap_or_default();
            total_lines += content.lines().count();

            if let Ok(syntax) = parse_file(&content) {
                let mut large_fns = Vec::new();
                let mut undocumented_structs = Vec::new();

                for item in syntax.items {
                    match item {
                        Item::Fn(fn_item) => {
                            // Simple heuristic for length: just checking string length since syn span mapping requires SourceFile
                            // For prototype, we'll just check if the function string representation is long
                            let fn_str = quote::quote!(#fn_item).to_string();
                            if fn_str.matches('\n').count() > 100 || fn_str.len() > 3000 {
                                large_fns.push(fn_item.sig.ident.to_string());
                            }
                        }
                        Item::Struct(struct_item) => {
                            let has_doc = struct_item.attrs.iter().any(|attr| attr.path().is_ident("doc"));
                            if !has_doc {
                                undocumented_structs.push(struct_item.ident.to_string());
                            }
                        }
                        _ => {}
                    }
                }

                if !large_fns.is_empty() {
                    findings.push(json!({
                        "file": path.display().to_string(),
                        "issue": "large_function",
                        "details": large_fns,
                    }));
                }

                if !undocumented_structs.is_empty() {
                    findings.push(json!({
                        "file": path.display().to_string(),
                        "issue": "undocumented_struct",
                        "details": undocumented_structs,
                    }));
                }
            }
        }

        Ok(json!({
            "files": file_count,
            "lines": total_lines,
            "findings": findings,
        }).to_string())
    }

    pub async fn generate_proposals(&self, analysis: &str) -> Result<Vec<ImprovementProposal>, String> {
        let parsed: serde_json::Value = serde_json::from_str(analysis)
            .map_err(|e| format!("Erro ao parsear análise: {}", e))?;

        let mut proposals = Vec::new();

        if let Some(findings) = parsed["findings"].as_array() {
            for finding in findings {
                let issue = finding["issue"].as_str().unwrap_or("unknown");
                let file = finding["file"].as_str().unwrap_or("");

                let (title, description, diff) = match issue {
                    "large_function" => {
                        let empty_vec = vec![];
                        let fns = finding["details"].as_array().unwrap_or(&empty_vec);
                        let fn_names = fns.iter()
                            .map(|v| v.as_str().unwrap_or(""))
                            .collect::<Vec<_>>()
                            .join(", ");
                        (
                            format!("Refatorar funções grandes em {}", file),
                            format!("As funções {} excedem 100 linhas. Considere extrair blocos em funções menores.", fn_names),
                            Some(format!("// TODO: Extrair blocos de {} em funções auxiliares", fn_names)),
                        )
                    }
                    "undocumented_struct" => {
                        let empty_vec = vec![];
                        let structs = finding["details"].as_array().unwrap_or(&empty_vec);
                        let struct_names = structs.iter()
                            .map(|v| v.as_str().unwrap_or(""))
                            .collect::<Vec<_>>()
                            .join(", ");
                        (
                            format!("Adicionar documentação para estruturas em {}", file),
                            format!("As estruturas {} não possuem documentação (///). Adicione comentários descritivos.", struct_names),
                            Some("//! Adicionar documentação com ///".to_string()),
                        )
                    }
                    _ => continue,
                };

                let proposal = ImprovementProposal::new(
                    title,
                    description,
                )
                .with_risk(RiskLevel::Medium)
                .with_code_diff(diff.unwrap_or_default())
                .with_impact("Melhoria na manutenibilidade e qualidade do código".to_string());

                proposals.push(proposal);
            }
        }

        Ok(proposals)
    }
}
