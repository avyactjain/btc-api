use axum::Json;

use crate::{chain::Chain, models::{NetworkFeeResponse, ValidateTransactionHashResponse}};

#[derive(Debug, Clone)]
pub struct BlockchainWrapper<T: Chain>(T);

impl<T: Chain> BlockchainWrapper<T> {
    pub fn new(blockchain: T) -> Self {
        Self(blockchain)
    }

    pub async fn get_network_fee(&self) -> Json<NetworkFeeResponse> {
        self.0.get_network_fee().await
    }

    pub async fn validate_transaction_hash(
        &self,
        transaction_hash: String,
    ) -> Json<ValidateTransactionHashResponse> {
        self.0.validate_transaction_hash(transaction_hash).await
    }
}
