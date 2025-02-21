use axum::{
    extract::{Query, State},
    Json,
};
use tracing::debug;

use crate::{
    blockchains::{bitcoin::Bitcoin, blockchain_wrapper::BlockchainWrapper},
    models::{NetworkFeeResponse, ValidateTransactionHashParams, ValidateTransactionHashResponse},
};

#[axum::debug_handler]
pub(crate) async fn bitcoin_network_fee_handler(
    State(blockchain): State<BlockchainWrapper<Bitcoin>>,
) -> Json<NetworkFeeResponse> {
    blockchain.get_network_fee().await
}

#[axum::debug_handler]
pub(crate) async fn bitcoin_validate_transaction_hash_handler(
    Query(params): Query<ValidateTransactionHashParams>,
    State(blockchain): State<BlockchainWrapper<Bitcoin>>,
) -> Json<ValidateTransactionHashResponse> {
    debug!(
        "Received request to validate transaction hash: {:#?}",
        params
    );
    //todo: Validate the request here

    blockchain
        .validate_transaction_hash(params.transaction_hash)
        .await
}
