use axum::Json;

use crate::{chain::Chain, models::NetworkFeeResponse};

#[derive(Debug, Clone)]
pub struct BlockchainWrapper<T: Chain>(T);

impl<T: Chain> BlockchainWrapper<T> {
    pub fn new(blockchain: T) -> Self {
        Self(blockchain)
    }

    pub async fn get_network_fee(&self) -> Json<NetworkFeeResponse> {
        self.0.get_network_fee().await
    }
}
