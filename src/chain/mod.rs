use axum::Json;
use serde::Deserialize;

use crate::models::{
    CreateTransactionParams, CreateTransactionResponse, NetworkFeeResponse,
    ValidateTransactionHashResponse,
};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub(crate) enum ChainName {
    Bitcoin,
}

// Trait for the blockchain implementations
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
}
