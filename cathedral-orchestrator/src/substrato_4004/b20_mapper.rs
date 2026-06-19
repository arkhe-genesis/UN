use ethers::types::{Address, U256, Bytes};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::substrato_4004::shared::{Action, FilterVerdict, EthicalFilter, extract_address, extract_u256, extract_optional_memo, extract_policy_scope, extract_u64, extract_pausable_features, hash_memo};
use crate::substrato_4004::policy_adapter::PolicyRegistryClient;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum B20Operation {
    Transfer {
        token: Address,
        from: Address,
        to: Address,
        amount: U256,
        memo: Option<[u8; 32]>,
        policy_scope: PolicyScope,
    },
    Mint {
        token: Address,
        to: Address,
        amount: U256,
        memo: Option<[u8; 32]>,
    },
    Burn {
        token: Address,
        from: Address,
        amount: U256,
        memo: Option<[u8; 32]>,
        burn_type: BurnType,
    },
    UpdatePolicy {
        token: Address,
        scope: PolicyScope,
        policy_id: u64,
    },
    Pause {
        token: Address,
        features: Vec<PausableFeature>,
        pause: bool,
    },
    UpdateMultiplier {
        token: Address,
        new_multiplier: U256, // WAD precision
    },
    Announce {
        token: Address,
        internal_calls: Vec<Bytes>,
        id: u64,
        description: String,
        uri: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyScope {
    TransferSender,
    TransferReceiver,
    TransferExecutor,
    MintReceiver,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BurnType {
    Caller,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PausableFeature {
    Transfer,
    Mint,
    Burn,
}

#[derive(Debug, Clone)]
pub enum MapperError {
    EthicalViolation(Vec<String>),
    PolicyDenied(String),
    SupplyCapExceeded,
    NotBlocked(Address),
    UnsupportedActionType(String),
    ExtractionError(String),
    PolicyError(crate::substrato_4004::policy_adapter::PolicyError),
}

impl std::fmt::Display for MapperError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for MapperError {}

impl From<crate::substrato_4004::policy_adapter::PolicyError> for MapperError {
    fn from(err: crate::substrato_4004::policy_adapter::PolicyError) -> Self {
        MapperError::PolicyError(err)
    }
}

pub struct B20TokenMapper {
    pub ethical_filter: Arc<EthicalFilter>,
    pub policy_registry: Arc<PolicyRegistryClient>,
}

impl B20TokenMapper {
    pub async fn get_total_supply(&self, _token: Address) -> Result<U256, MapperError> {
        Ok(U256::from(0))
    }

    pub async fn get_supply_cap(&self, _token: Address) -> Result<U256, MapperError> {
        Ok(U256::MAX)
    }

    pub async fn map_action(&self, action: &Action) -> Result<B20Operation, MapperError> {
        match self.ethical_filter.evaluate(action).await {
            FilterVerdict::Passed => {}
            FilterVerdict::Failed(v) => return Err(MapperError::EthicalViolation(v)),
        }

        match action.action_type.as_str() {
            "payment_b20" => {
                let token = extract_address(action, "token").map_err(|e| MapperError::ExtractionError(e))?;
                let from = extract_address(action, "from").map_err(|e| MapperError::ExtractionError(e))?;
                let to = extract_address(action, "to").map_err(|e| MapperError::ExtractionError(e))?;
                let amount = extract_u256(action, "amount").map_err(|e| MapperError::ExtractionError(e))?;
                let memo = extract_optional_memo(action).map_err(|e| MapperError::ExtractionError(e))?;

                let sender_policy = self.policy_registry
                    .get_policy(token, PolicyScope::TransferSender)
                    .await?;

                if !self.policy_registry.is_authorized(sender_policy, from).await? {
                    return Err(MapperError::PolicyDenied("sender".to_string()));
                }

                Ok(B20Operation::Transfer {
                    token,
                    from,
                    to,
                    amount,
                    memo,
                    policy_scope: PolicyScope::TransferSender,
                })
            }
            "mint_b20" => {
                let token = extract_address(action, "token").map_err(|e| MapperError::ExtractionError(e))?;
                let to = extract_address(action, "to").map_err(|e| MapperError::ExtractionError(e))?;
                let amount = extract_u256(action, "amount").map_err(|e| MapperError::ExtractionError(e))?;
                let memo = extract_optional_memo(action).map_err(|e| MapperError::ExtractionError(e))?;

                let current_supply = self.get_total_supply(token).await?;
                let cap = self.get_supply_cap(token).await?;

                if current_supply + amount > cap {
                    return Err(MapperError::SupplyCapExceeded);
                }

                Ok(B20Operation::Mint { token, to, amount, memo })
            }
            "freeze_and_seize" => {
                let token = extract_address(action, "token").map_err(|e| MapperError::ExtractionError(e))?;
                let target = extract_address(action, "target").map_err(|e| MapperError::ExtractionError(e))?;
                let amount = extract_u256(action, "amount").map_err(|e| MapperError::ExtractionError(e))?;

                let sender_policy = self.policy_registry
                    .get_policy(token, PolicyScope::TransferSender)
                    .await?;

                if self.policy_registry.is_authorized(sender_policy, target).await? {
                    return Err(MapperError::NotBlocked(target));
                }

                Ok(B20Operation::Burn {
                    token,
                    from: target,
                    amount,
                    memo: Some(hash_memo("freeze-and-seize", action)),
                    burn_type: BurnType::Blocked,
                })
            }
            "update_policy" => {
                let token = extract_address(action, "token").map_err(|e| MapperError::ExtractionError(e))?;
                let scope = extract_policy_scope(action).map_err(|e| MapperError::ExtractionError(e))?;
                let policy_id = extract_u64(action, "policy_id").map_err(|e| MapperError::ExtractionError(e))?;

                Ok(B20Operation::UpdatePolicy { token, scope, policy_id })
            }
            "pause_b20" => {
                let token = extract_address(action, "token").map_err(|e| MapperError::ExtractionError(e))?;
                let features = extract_pausable_features(action).map_err(|e| MapperError::ExtractionError(e))?;

                Ok(B20Operation::Pause { token, features, pause: true })
            }
            "unpause_b20" => {
                let token = extract_address(action, "token").map_err(|e| MapperError::ExtractionError(e))?;
                let features = extract_pausable_features(action).map_err(|e| MapperError::ExtractionError(e))?;

                Ok(B20Operation::Pause { token, features, pause: false })
            }
            "update_multiplier" => {
                let token = extract_address(action, "token").map_err(|e| MapperError::ExtractionError(e))?;
                let multiplier = extract_u256(action, "multiplier").map_err(|e| MapperError::ExtractionError(e))?;

                Ok(B20Operation::UpdateMultiplier { token, new_multiplier: multiplier })
            }
            _ => Err(MapperError::UnsupportedActionType(action.action_type.clone())),
        }
    }
}
