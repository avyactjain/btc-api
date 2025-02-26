use axum::Json;
use serde::Deserialize;

use crate::models::{
    BroadcastTransactionParams, BroadcastTransactionResponse, CreateTransactionParams,
    CreateTransactionResponse, NetworkFeeResponse, ValidateTransactionHashResponse,
};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub(crate) enum ChainName {
    Bitcoin,
}

// Trait for the blockchain implementations
// Every blockchain should implement this trait
pub trait Chain {
    async fn get_network_fee(&self) -> Json<NetworkFeeResponse>;
    async fn validate_transaction_hash(
        &self,
        transaction_hash: String,
    ) -> Json<ValidateTransactionHashResponse>;
    async fn create_transaction(
        &self,
        transaction: CreateTransactionParams,
    ) -> Json<CreateTransactionResponse>;
    async fn broadcast_transaction(
        &self,
        transaction: BroadcastTransactionParams,
    ) -> Json<BroadcastTransactionResponse>;
}
