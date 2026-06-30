use crate::integrations::picnic::IPicnicBasket;
use ethers::prelude::*;
use tracing::info;

pub struct X402RoyaltyServer {
    pub picnic_basket_address: Address,
    pub provider: std::sync::Arc<Provider<Http>>,
    pub wallet: LocalWallet,
}

impl X402RoyaltyServer {
    pub async fn settle_payment_with_picnic(
        &self,
        payment_amount: u64,
        royalty_splits: &[(String, f32)], // (eth_address, share)
    ) -> Result<(), String> {
        let client = std::sync::Arc::new(SignerMiddleware::new(
            (*self.provider).clone(),
            self.wallet.clone(),
        ));
        let contract = IPicnicBasket::new(self.picnic_basket_address, client.clone());

        // 1. Deposita
        let receipt = contract
            .deposit(U256::from(payment_amount), self.wallet.address())
            .send()
            .await
            .map_err(|e| format!("Erro no depósito: {}", e))?
            .await
            .map_err(|e| e.to_string())?
            .unwrap();

        info!(
            "✅ USDC depositado no Picnic Basket: tx={:?}",
            receipt.transaction_hash
        );

        // 2. Distribui shares entre os criadores (split)
        let total_shares = contract
            .total_assets()
            .call()
            .await
            .map_err(|e| e.to_string())?;
        let mut recipients = Vec::new();
        let mut amounts = Vec::new();

        for (eth_address, share) in royalty_splits {
            let share_amount = (total_shares.as_u64() as f64 * (*share as f64)) as u64;
            let address: Address = eth_address.parse().unwrap();
            recipients.push(address);
            amounts.push(U256::from(share_amount));
        }

        let receipt2 = contract
            .distribute_rewards(recipients, amounts)
            .send()
            .await
            .map_err(|e| format!("Erro na distribuição: {}", e))?
            .await
            .map_err(|e| e.to_string())?
            .unwrap();

        info!(
            "✅ Royalties distribuídos: tx={:?}",
            receipt2.transaction_hash
        );

        Ok(())
    }
}
