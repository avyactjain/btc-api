use axum::{
    extract::{Query, State},
    Json,
};
use tracing::{debug, error};

use crate::{
    blockchains::{bitcoin::Bitcoin, blockchain_wrapper::BlockchainWrapper},
    models::{
        BroadcastTransactionParams, BroadcastTransactionResponse, CreateTransactionParams,
        CreateTransactionResponse, MethodNotAllowedResponse, NetworkFeeResponse,
        ValidateTransactionHashParams, ValidateTransactionHashResponse,
    },
};

#[axum::debug_handler]
pub(crate) async fn method_not_allowed_handler(
    State(_): State<BlockchainWrapper<Bitcoin>>,
) -> Json<MethodNotAllowedResponse> {
    error!("Method not allowed");
    Json(MethodNotAllowedResponse {
        is_error: true,
        error_msg: "Method not allowed".to_string(),
    })
}

#[axum::debug_handler]
pub(crate) async fn bitcoin_network_fee_handler(
    State(blockchain): State<BlockchainWrapper<Bitcoin>>,
) -> Json<NetworkFeeResponse> {
    debug!("Received request to get network fee");
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

    blockchain
        .validate_transaction_hash(params.transaction_hash)
        .await
}

#[axum::debug_handler]
pub(crate) async fn bitcoin_create_transaction_handler(
    State(blockchain): State<BlockchainWrapper<Bitcoin>>,
    Json(params): Json<CreateTransactionParams>,
) -> Json<CreateTransactionResponse> {
    debug!("Received request to create transaction: {:#?}", params);

    match params.validate() {
        Ok(params) => blockchain.create_transaction(params).await,
        Err(e) => Json(CreateTransactionResponse {
            is_error: true,
            data: None,
            error_msg: Some(e.to_string()),
        }),
    }
}

#[axum::debug_handler]
pub(crate) async fn bitcoin_broadcast_transaction_handler(
    State(blockchain): State<BlockchainWrapper<Bitcoin>>,
    Json(params): Json<BroadcastTransactionParams>,
) -> Json<BroadcastTransactionResponse> {
    debug!("Received request to broadcast transaction: {:#?}", params);

    blockchain.broadcast_transaction(params).await
}
