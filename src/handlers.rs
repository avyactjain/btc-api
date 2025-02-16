use axum::{extract::State, Json};

use crate::{
    blockchains::{bitcoin::Bitcoin, blockchain_wrapper::BlockchainWrapper},
    models::NetworkFeeResponse,
};

#[axum::debug_handler]
pub(crate) async fn bitcoin_network_fee_handler(
    State(blockchain): State<BlockchainWrapper<Bitcoin>>,
) -> Json<NetworkFeeResponse> {
    blockchain.get_network_fee().await
}
