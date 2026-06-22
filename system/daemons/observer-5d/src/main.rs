//! Observer-5D — Daemon de governança com detecção de anomalias
//! Periodicidade: 5 minutos (configurável)
//! Selo: CATHEDRAL-ARKHE-OBSERVER-5D-v2.0.0-2026-06-21

mod wormgraph;
mod tree;
mod heuristics;
mod jira;

use std::sync::Arc;
use std::time::Duration;
use tokio::time;
use tracing::{info, warn, error};
use heuristics::HeuristicEngine;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuração do Observer-5D
#[derive(Debug, Clone, Deserialize)]
pub struct ObserverConfig {
    pub scan_interval_seconds: u64,
    pub wormgraph_path: PathBuf,
    pub jira_url: String,
    pub jira_token: String,
    pub jira_project: String,
    pub risk_threshold_high: f64,
    pub risk_threshold_medium: f64,
    pub risk_threshold_low: f64,
    pub enable_jira: bool,
    pub log_level: String,
}

impl Default for ObserverConfig {
    fn default() -> Self {
        Self {
            scan_interval_seconds: 300,
            wormgraph_path: PathBuf::from("/var/lib/cathedral/wormgraph"),
            jira_url: "https://cathedral.atlassian.net".to_string(),
            jira_token: String::new(),
            jira_project: "CEM".to_string(),
            risk_threshold_high: 0.7,
            risk_threshold_medium: 0.4,
            risk_threshold_low: 0.2,
            enable_jira: true,
            log_level: "info".to_string(),
        }
    }
}

/// Serviço principal Observer-5D
pub struct Observer5D {
    config: ObserverConfig,
    wormgraph: Arc<wormgraph::WormGraph>,
    tree_manager: Arc<tree::TreeManager>,
    heuristic_engine: Arc<HeuristicEngine>,
    jira_client: Option<Arc<jira::JiraClient>>,
}

impl Observer5D {
    pub async fn new(config: ObserverConfig) -> Result<Self, String> {
        let wormgraph = Arc::new(wormgraph::WormGraph::new());
        let tree_manager = Arc::new(tree::TreeManager::new());
        let heuristic_engine = Arc::new(HeuristicEngine::new());

        let jira_client = if config.enable_jira && !config.jira_token.is_empty() {
            Some(Arc::new(jira::JiraClient::new(
                &config.jira_url,
                &config.jira_token,
                &config.jira_project,
            )?))
        } else {
            info!("ℹ️ Jira integração desabilitada ou sem token");
            None
        };

        Ok(Self {
            config,
            wormgraph,
            tree_manager,
            heuristic_engine,
            jira_client,
        })
    }

    pub async fn run(&self) -> Result<(), String> {
        info!(
            "🚀 Observer-5D iniciado (intervalo: {}s, Jira: {})",
            self.config.scan_interval_seconds,
            if self.jira_client.is_some() { "✅" } else { "❌" }
        );

        let mut interval = time::interval(Duration::from_secs(self.config.scan_interval_seconds));

        loop {
            interval.tick().await;
            if let Err(e) = self.scan_and_report().await {
                error!("Erro no scan: {}", e);
            }
        }
    }

    async fn scan_and_report(&self) -> Result<(), String> {
        info!("🔍 Iniciando scan de governança...");

        let entries = self.wormgraph.get_entries().await;
        if entries.is_empty() {
            info!("ℹ️ Nenhum agente encontrado no WormGraph.");
            return Ok(());
        }

        // Atualiza a árvore de agentes
        for entry in &entries {
            if entry.decision_type == "create_agent" {
                let parent_id = self.extract_parent(&entry.after_state);
                self.tree_manager.add_node(entry.agent_id.clone(), parent_id).await;
            }
        }

        // Avalia heurísticas
        let history: Vec<_> = entries.iter().collect();
        let mut all_reports = Vec::new();

        for entry in &entries {
            let reports = self.heuristic_engine
                .evaluate(entry, &self.tree_manager, &history)
                .await;
            all_reports.extend(reports);
        }

        // Processa relatórios
        for report in all_reports {
            self.process_report(report).await;
        }

        info!("✅ Scan concluído. {} anomalias detectadas.", all_reports.len());
        Ok(())
    }

    async fn process_report(&self, report: heuristics::HeuristicReport) {
        let score = report.score;
        let severity = format!("{:?}", report.severity);

        if score >= self.config.risk_threshold_high {
            info!(
                "🔴 Anomalia CRÍTICA: {} (score={:.2})",
                report.rule_id, score
            );

            if let Some(jira) = &self.jira_client {
                if let Err(e) = jira.create_anomaly_ticket(
                    &report.agent_id,
                    &report.rule_id,
                    &severity,
                    score,
                    &report.details,
                    &report.recommendations,
                ).await {
                    error!("Erro ao criar ticket Jira: {}", e);
                } else {
                    info!("✅ Ticket Jira criado para anomalia crítica");
                }
            }
        } else if score >= self.config.risk_threshold_medium {
            info!(
                "🟡 Anomalia MÉDIA: {} (score={:.2})",
                report.rule_id, score
            );
            // Log detalhado para análise
            warn!(
                "Detalhes: agent={}, details={}, recomendações={:?}",
                report.agent_id, report.details, report.recommendations
            );
        } else if score >= self.config.risk_threshold_low {
            info!(
                "🟢 Anomalia BAIXA: {} (score={:.2})",
                report.rule_id, score
            );
        } else {
            info!(
                "ℹ️ Anomalia menor: {} (score={:.2})",
                report.rule_id, score
            );
        }
    }

    fn extract_parent(&self, after_state: &str) -> Option<String> {
        // Extrai o campo "parent_id" do JSON
        if let Some(start) = after_state.find("\"parent_id\":\"") {
            let rest = &after_state[start + 13..];
            if let Some(end) = rest.find('\"') {
                return Some(rest[..end].to_string());
            }
        }
        None
    }
}

/// Carrega configuração de variáveis de ambiente ou arquivo
async fn load_config() -> Result<ObserverConfig, String> {
    // Tenta carregar de arquivo YAML
    let config_path = std::env::var("OBSERVER_CONFIG")
        .unwrap_or_else(|_| "/etc/cathedral/observer.yaml".to_string());

    if let Ok(content) = tokio::fs::read_to_string(&config_path).await {
        if let Ok(config) = serde_yaml::from_str::<ObserverConfig>(&content) {
            return Ok(config);
        }
    }

    // Fallback para variáveis de ambiente
    Ok(ObserverConfig {
        scan_interval_seconds: std::env::var("OBSERVER_INTERVAL")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(300),
        wormgraph_path: std::env::var("WORMGRAPH_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/var/lib/cathedral/wormgraph")),
        jira_url: std::env::var("JIRA_URL")
            .unwrap_or_else(|_| "https://cathedral.atlassian.net".to_string()),
        jira_token: std::env::var("JIRA_TOKEN").unwrap_or_default(),
        jira_project: std::env::var("JIRA_PROJECT").unwrap_or_else(|_| "CEM".to_string()),
        risk_threshold_high: std::env::var("RISK_THRESHOLD_HIGH")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0.7),
        risk_threshold_medium: std::env::var("RISK_THRESHOLD_MEDIUM")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0.4),
        risk_threshold_low: std::env::var("RISK_THRESHOLD_LOW")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0.2),
        enable_jira: std::env::var("ENABLE_JIRA")
            .ok()
            .map(|v| v != "false")
            .unwrap_or(true),
        log_level: std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
    })
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let config = load_config().await?;

    // Configura logging
    tracing_subscriber::fmt()
        .with_env_filter(&config.log_level)
        .with_target(false)
        .init();

    info!("🏛️ Cathedral ARKHE — Observer-5D v2.0.0");
    info!("📋 Configuração: interval={}s, wormgraph={:?}, jira={}",
        config.scan_interval_seconds,
        config.wormgraph_path,
        if config.enable_jira { "✅" } else { "❌" }
    );

    let observer = Observer5D::new(config).await?;
    observer.run().await
}
