use serde::Deserialize;

use crate::models::{
    BroadcastTransactionParams, BroadcastTransactionResponse, CreateTransactionParams,
    CreateTransactionResponse, NetworkFeeResponse, ValidateTransactionHashResponse,
    WalletBalanceResponse,
};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub(crate) enum ChainName {
    Bitcoin,
}

// Trait for the blockchain implementations
// Every blockchain should implement this trait
#[async_trait::async_trait]
#[mockall::automock]
pub trait Chain {
    async fn get_network_fee(&self) -> NetworkFeeResponse;
    async fn validate_transaction_hash(
        &self,
        transaction_hash: String,
    ) -> ValidateTransactionHashResponse;
    async fn create_transaction(
        &self,
        transaction: CreateTransactionParams,
    ) -> CreateTransactionResponse;
    async fn broadcast_transaction(
        &self,
        transaction: BroadcastTransactionParams,
    ) -> BroadcastTransactionResponse;
    async fn get_wallet_balance(&self, address: String) -> WalletBalanceResponse;
}
