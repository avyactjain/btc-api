use axum::Json;
use reqwest::Client;
use response_models::GetRawTransactionResponse;

use crate::{
    chain::Chain,
    models::{
        NetworkFeeResponse, NetworkFeeResponseData, ValidateTransactionHashResponse,
        ValidateTransactionHashResponseData,
    },
};
mod request_models;
mod response_models;
mod utils;

// Mempool API for network fee
const MEMPOOL_API_NETWORK_FEE_URL: &str = "https://mempool.space/api/v1/fees/recommended";

#[derive(Debug, Clone)]
pub struct Bitcoin {
    pub rpc_url: String,
    pub client: Client,
}

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

    async fn validate_transaction_hash(
        &self,
        transaction_hash: String,
    ) -> axum::Json<ValidateTransactionHashResponse> {
        {
            let mut validate_transaction_hash_response = ValidateTransactionHashResponse {
                is_error: true,
                data: None,
            };

            let request_body = utils::create_validate_transaction_request_body(transaction_hash, 1);

            let rpc_response = self
                .client
                .post(self.rpc_url.clone())
                .json(&request_body)
                .send()
                .await;

            match rpc_response {
                Ok(response) => match response.json().await {
                    Ok(value) => match serde_json::from_value::<GetRawTransactionResponse>(value) {
                        Ok(_get_raw_transaction_response) => {
                            validate_transaction_hash_response.is_error = false;
                            validate_transaction_hash_response.data =
                                Some(ValidateTransactionHashResponseData { is_valid: true });
                        }
                        Err(_err) => {todo!()}
                    },
                    Err(_err) => {todo!()}
                },
                Err(_err) => {todo!()}
            }

            axum::Json(validate_transaction_hash_response)
        }
    }
}

impl Bitcoin {
    pub fn new(rpc_url: String) -> Self {
        Self {
            rpc_url,
            client: Client::new(),
        }
    }
}
