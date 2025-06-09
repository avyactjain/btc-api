use std::sync::Arc;

use crate::{
    chain::Chain,
    models::{
        BroadcastTransactionParams, BroadcastTransactionResponse, CreateTransactionParams,
        CreateTransactionResponse, NetworkFeeResponse, ValidateTransactionHashResponse,
        WalletBalanceResponse,
    },
};

#[derive(Debug, Clone)]
// State Abstraction for the blockchain instance
// All the blockchain specific methods are implemented in the blockchain trait
// Every blockchain should implement the blockchain trait
// Todo : Add a type parameter for the blockchain instance, so that we can use the same wrapper for different blockchains.
pub struct BtcApiState<T: Chain> {
    pub inner: Arc<T>,
}

impl<T: Chain> BtcApiState<T> {
    pub fn new(blockchain: T) -> Self {
        Self {
            inner: Arc::new(blockchain),
        }
    }

    pub async fn get_network_fee(&self) -> NetworkFeeResponse {
        self.inner.get_network_fee().await
    }

    pub async fn validate_transaction_hash(
        &self,
        transaction_hash: String,
    ) -> ValidateTransactionHashResponse {
        self.inner.validate_transaction_hash(transaction_hash).await
    }

    pub async fn create_transaction(
        &self,
        transaction: CreateTransactionParams,
    ) -> CreateTransactionResponse {
        self.inner.create_transaction(transaction).await
    }

    pub async fn broadcast_transaction(
        &self,
        transaction: BroadcastTransactionParams,
    ) -> BroadcastTransactionResponse {
        self.inner.broadcast_transaction(transaction).await
    }

    pub async fn get_wallet_balance(&self, address: String) -> WalletBalanceResponse {
        self.inner.get_wallet_balance(address).await
    }
}
