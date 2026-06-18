use crate::evolution::desci_node_resource::{RoyaltyConfig, RoyaltySplit};
use bytes::Bytes;
use tracing::info;

// Mocks for x402 functionality to ensure compilation without actual x402 implementation packages
pub struct X402Middleware {
    pub facilitator_url: String,
}

impl X402Middleware {
    pub fn new(facilitator_url: &str) -> Self {
        Self { facilitator_url: facilitator_url.to_string() }
    }

    pub fn clone(&self) -> Self {
        Self { facilitator_url: self.facilitator_url.clone() }
    }

    pub fn with_price_tag(&self, _price_tag: String) -> Self {
        self.clone()
    }
}

pub struct V2Eip155Exact {}
impl V2Eip155Exact {
    pub fn price_tag(_receiver: String, _amount: u64) -> String {
        "mock_price_tag".to_string()
    }
}

pub struct USDC {}
impl USDC {
    pub fn base() -> Self { Self {} }
    pub fn amount(&self, _amount: u64) -> u64 { _amount }
}

pub struct X402RoyaltyServer {
    pub middleware: X402Middleware,
    pub facilitator_url: String,
}

impl X402RoyaltyServer {
    pub fn new(facilitator_url: &str) -> Self {
        Self {
            middleware: X402Middleware::new(facilitator_url),
            facilitator_url: facilitator_url.to_string(),
        }
    }

    pub fn npub_to_eth_address(&self, _npub: &str) -> String {
        format!("0xmock_address")
    }

    pub fn protect_route(
        &self,
        royalty_config: &RoyaltyConfig,
    ) {
        let price_str = royalty_config.price_per_access
            .split_whitespace()
            .next()
            .unwrap_or("0.001");

        let price = (price_str.parse::<f64>().unwrap_or(0.001) * 1_000_000.0) as u64;

        let receiver = royalty_config.royalty_split
            .first()
            .map(|s| self.npub_to_eth_address(&s.npub))
            .unwrap_or_else(|| "0x0000000000000000000000000000000000000000".to_string());

        let price_tag = V2Eip155Exact::price_tag(
            receiver,
            USDC::base().amount(price),
        );

        let _protected_middleware = self.middleware.clone().with_price_tag(price_tag);
    }

    pub async fn distribute_royalties(
        &self,
        payment_amount: u64,
        splits: &[RoyaltySplit],
    ) -> Result<(), String> {
        for split in splits {
            let address = self.npub_to_eth_address(&split.npub);
            let amount = (payment_amount as f64 * split.share as f64) as u64;
            info!("📤 Enviando {} USDC para {}", amount, address);
        }
        Ok(())
    }
}

pub struct X402Client {
    pub client: String, // mocked reqwest client
}

impl X402Client {
    pub fn new() -> Self {
        Self {
            client: "mock_client".to_string(),
        }
    }

    pub async fn download_with_payment(
        &self,
        _url: &str,
        wallet_private_key: &str,
    ) -> Result<Bytes, String> {
        let _signature = self.sign_payment("instructions", wallet_private_key);
        Ok(Bytes::from(Vec::new()))
    }

    fn sign_payment(&self, _instructions: &str, _private_key: &str) -> String {
        "signed_payment".to_string()
    }
}
