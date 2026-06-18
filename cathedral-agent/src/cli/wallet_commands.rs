use crate::swarm::second_self::SecondSelfOrchestrator;
use crate::evolution::wallet_resource::{WalletResource, WalletConfig, Chain, WalletNetwork};
use crate::evolution::identity_resource::IdentityResource;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum WalletCommand {
    CreateWallet { chain: String, seed: Option<String> },
    Balance { chain: String, address: String },
    Send { chain: String, to: String, amount: String },
    History { chain: String, address: String, limit: usize },
    CreateIdentity { npub: String, name: Option<String> },
    AddWalletToIdentity { chain: String, address: String },
    TrustAgent { npub: String, agent: String },
}

impl WalletCommand {
    pub fn parse(input: &str) -> Option<Self> {
        let parts: Vec<&str> = input.trim().split_whitespace().collect();
        if parts.is_empty() { return None; }

        match parts[0] {
            "/wallet-create" | "wallet-create" => {
                if parts.len() >= 2 {
                    Some(Self::CreateWallet {
                        chain: parts[1].to_string(),
                        seed: if parts.len() > 2 { Some(parts[2].to_string()) } else { None },
                    })
                } else { None }
            }
            "/wallet-balance" | "wallet-balance" => {
                if parts.len() >= 3 {
                    Some(Self::Balance {
                        chain: parts[1].to_string(),
                        address: parts[2].to_string(),
                    })
                } else { None }
            }
            "/wallet-send" | "wallet-send" => {
                if parts.len() >= 4 {
                    Some(Self::Send {
                        chain: parts[1].to_string(),
                        to: parts[2].to_string(),
                        amount: parts[3].to_string(),
                    })
                } else { None }
            }
            "/wallet-history" | "wallet-history" => {
                if parts.len() >= 3 {
                    Some(Self::History {
                        chain: parts[1].to_string(),
                        address: parts[2].to_string(),
                        limit: if parts.len() > 3 { parts[3].parse().unwrap_or(10) } else { 10 },
                    })
                } else { None }
            }
            "/identity-create" | "identity-create" => {
                if parts.len() >= 2 {
                    Some(Self::CreateIdentity {
                        npub: parts[1].to_string(),
                        name: if parts.len() > 2 { Some(parts[2].to_string()) } else { None },
                    })
                } else { None }
            }
            "/identity-add-wallet" | "identity-add-wallet" => {
                if parts.len() >= 3 {
                    Some(Self::AddWalletToIdentity {
                        chain: parts[1].to_string(),
                        address: parts[2].to_string(),
                    })
                } else { None }
            }
            "/identity-trust" | "identity-trust" => {
                if parts.len() >= 3 {
                    Some(Self::TrustAgent {
                        npub: parts[1].to_string(),
                        agent: parts[2].to_string(),
                    })
                } else { None }
            }
            _ => None,
        }
    }

    pub async fn execute(&self, orchestrator: &mut SecondSelfOrchestrator) -> Result<String, String> {
        match self {
            Self::CreateWallet { chain, seed } => {
                let chain_enum = Chain::from_str(chain);
                let config = WalletConfig {
                    chain: chain_enum.clone(),
                    derivation_path: None,
                    network: WalletNetwork::Mainnet,
                    metadata: HashMap::new(),
                };
                let wallet = WalletResource::new(config, seed.as_deref(), &orchestrator.identity.optimization_goal)?;
                let registry = orchestrator.resource_registry.as_mut()
                    .ok_or("ResourceRegistry não inicializado")?;
                registry.register_wallet(wallet.clone()).await?;
                Ok(format!("✅ Carteira criada para {}: {}", chain_enum, wallet.address))
            }
            Self::Balance { chain, address } => {
                let registry = orchestrator.resource_registry.as_mut()
                    .ok_or("ResourceRegistry não inicializado")?;
                if let Some(wallet) = registry.get_wallet(chain, address).await? {
                    let balance = wallet.get_balance(None).await?;
                    Ok(format!("💰 Saldo: {} ({} USD)", balance.native_balance, balance.usd_estimate.unwrap_or(0.0)))
                } else {
                    Err(format!("Carteira {} não encontrada", address))
                }
            }
            Self::Send { chain, to, amount } => {
                let registry = orchestrator.resource_registry.as_mut()
                    .ok_or("ResourceRegistry não inicializado")?;
                let mut identity = registry.get_identity(&orchestrator.identity.optimization_goal).await?
                    .ok_or("Identidade não encontrada")?;
                if let Some(wallet) = identity.get_wallet_mut(chain) {
                    let tx = wallet.send_transaction(to, amount, None).await?;
                    Ok(format!("✅ Transação enviada: {}", tx.hash))
                } else {
                    Err(format!("Nenhuma carteira para chain {}", chain))
                }
            }
            Self::CreateIdentity { npub, name } => {
                let identity = IdentityResource::new(npub, name.as_deref());
                let registry = orchestrator.resource_registry.as_mut()
                    .ok_or("ResourceRegistry não inicializado")?;
                registry.register_identity(identity).await?;
                Ok(format!("✅ Identidade criada para {}", npub))
            }
            Self::AddWalletToIdentity { chain, address } => {
                let registry = orchestrator.resource_registry.as_mut()
                    .ok_or("ResourceRegistry não inicializado")?;
                let mut identity = registry.get_identity(&orchestrator.identity.optimization_goal).await?
                    .ok_or("Identidade não encontrada")?;
                if let Some(wallet) = registry.get_wallet(chain, address).await? {
                    identity.add_wallet(wallet)?;
                    registry.register_identity(identity).await?;
                    Ok(format!("✅ Carteira {} adicionada à identidade", address))
                } else {
                    Err(format!("Carteira {} não encontrada", address))
                }
            }
            Self::TrustAgent { npub, agent } => {
                let registry = orchestrator.resource_registry.as_mut()
                    .ok_or("ResourceRegistry não inicializado")?;
                let mut identity = registry.get_identity(npub).await?
                    .ok_or("Identidade não encontrada")?;
                identity.trust_agent(agent);
                registry.register_identity(identity).await?;
                Ok(format!("✅ Agente {} agora é confiável para {}", agent, npub))
            }
            Self::History { chain, address, limit } => {
                let registry = orchestrator.resource_registry.as_mut()
                    .ok_or("ResourceRegistry não inicializado")?;
                if let Some(wallet) = registry.get_wallet(chain, address).await? {
                    let history = wallet.get_transaction_history(*limit).await?;
                    let output = history.iter().map(|tx| {
                        format!("  {}: {} {} -> {} ({:?})", tx.hash, tx.amount, tx.chain, tx.to, tx.status)
                    }).collect::<Vec<_>>().join("\n");
                    Ok(format!("📜 Histórico de transações:\n{}", output))
                } else {
                    Err(format!("Carteira {} não encontrada", address))
                }
            }
        }
    }
}
