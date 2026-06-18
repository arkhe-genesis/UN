use crate::evolution::identity_resource::IdentityResource;
use crate::evolution::resource::Resource;
use crate::evolution::secret_resource::SecretResource;
use crate::evolution::wallet_resource::WalletResource;
use std::collections::HashMap;

pub struct ResourceRegistry {
    resources: HashMap<String, Box<dyn Resource>>,
}

impl ResourceRegistry {
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }

    pub async fn register(&mut self, resource: Box<dyn Resource>) -> Result<(), String> {
        let id = resource.metadata().id.clone();
        self.resources.insert(id, resource);
        Ok(())
    }

    pub async fn get(&self, id: &str) -> Result<Option<&dyn Resource>, String> {
        Ok(self.resources.get(id).map(|r| r.as_ref()))
    }

    pub async fn get_mut(&mut self, id: &str) -> Result<Option<&mut Box<dyn Resource>>, String> {
        Ok(self.resources.get_mut(id))
    }

    pub async fn register_wallet(&mut self, wallet: WalletResource) -> Result<(), String> {
        let resource = Box::new(wallet);
        self.register(resource).await
    }

    pub async fn get_wallet(
        &self,
        chain: &str,
        address: &str,
    ) -> Result<Option<WalletResource>, String> {
        let id = format!("wallet:{}:{}", chain, address);
        match self.get(&id).await? {
            Some(res) => {
                if let Some(wallet) = res.as_any().downcast_ref::<WalletResource>() {
                    Ok(Some(wallet.clone()))
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }

    pub async fn register_identity(&mut self, identity: IdentityResource) -> Result<(), String> {
        let resource = Box::new(identity);
        self.register(resource).await
    }

    pub async fn get_identity(&self, npub: &str) -> Result<Option<IdentityResource>, String> {
        let id = format!("identity:{}", npub);
        match self.get(&id).await? {
            Some(res) => {
                if let Some(identity) = res.as_any().downcast_ref::<IdentityResource>() {
                    Ok(Some(identity.clone()))
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }

    pub async fn register_secret(&mut self, secret: SecretResource) -> Result<(), String> {
        let resource = Box::new(secret);
        self.register(resource).await
    }

    pub async fn get_secret(&self, identity: &str) -> Result<Option<SecretResource>, String> {
        let id = format!("secrets:{}", identity);
        match self.get(&id).await? {
            Some(res) => {
                if let Some(secret) = res.as_any().downcast_ref::<SecretResource>() {
                    Ok(Some(secret.clone()))
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }
}
