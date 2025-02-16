use axum::Json;
use serde::Deserialize;

use crate::models::NetworkFeeResponse;

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum ChainName {
    Bitcoin,
}

// Trait for the blockchain implementations
pub trait Chain {
    async fn get_network_fee(&self) -> Json<NetworkFeeResponse>;
}
