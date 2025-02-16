use axum::Json;

use crate::{
    chain::Chain,
    models::{NetworkFeeResponse, NetworkFeeResponseData},
};

// Mempool API for network fee
const MEMPOOL_API_NETWORK_FEE_URL: &str = "https://mempool.space/api/v1/fees/recommended";

#[derive(Debug, Clone)]
pub struct Bitcoin {}

impl Chain for Bitcoin {
    async fn get_network_fee(&self) -> axum::Json<NetworkFeeResponse> {
        {
            let mut result = Json(NetworkFeeResponse {
                is_error: true,
                data: None,
                error_msg: None,
            });

            match reqwest::get(MEMPOOL_API_NETWORK_FEE_URL).await {
                Ok(response) => match response.json().await {
                    Ok(value) => match serde_json::from_value::<NetworkFeeResponseData>(value) {
                        Ok(network_fee_response) => {
                            result.is_error = false;
                            result.data = Some(network_fee_response);
                        }
                        Err(err) => {
                            result.error_msg = Some(err.to_string());
                        }
                    },
                    Err(err) => {
                        result.error_msg = Some(err.to_string());
                    }
                },
                Err(err) => {
                    result.error_msg = Some(err.to_string());
                }
            }

            result
        }
    }
}

impl Bitcoin {
    pub fn new() -> Self {
        Self {}
    }
}
