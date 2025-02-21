use axum::Json;
use reqwest::{Client, Error, Response};
use response_models::BlockchaincomRawTxn;
use serde::Serialize;

use crate::{
    btc_api_error::BtcApiError,
    chain::Chain,
    models::{
        AddressSpent, NetworkFeeResponse, NetworkFeeResponseData, TxnData, TxnStatus,
        ValidateTransactionHashResponse, ValidateTransactionHashResponseData,
    },
};
mod request_models;
mod response_models;
mod utils;

// Mempool API for network fee
const MEMPOOL_API_NETWORK_FEE_URL: &str = "https://mempool.space/api/v1/fees/recommended";
// Blockchain API for raw transaction
const BLOCKCHAIN_API_RAW_TRANSACTION_URL: &str = "https://blockchain.info/rawtx/";

#[derive(Debug, Clone)]
pub struct Bitcoin {
    pub rpc_url: String,
    pub rpc_client: Client,
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
            let mut result = ValidateTransactionHashResponse {
                is_error: true,
                data: None,
                error_msg: None,
            };
            let get_raw_txn_response = self.get_raw_transaction(transaction_hash).await;

            match get_raw_txn_response {
                Ok(validate_txn_data) => {
                    result.is_error = false;
                    result.data = Some(validate_txn_data);
                    result.error_msg = None;
                }
                Err(e) => {
                    result.error_msg = Some(e.to_string());
                }
            }

            Json(result)
        }
    }
}

impl Bitcoin {
    pub fn new(rpc_url: String) -> Self {
        Self {
            rpc_url,
            rpc_client: Client::new(),
        }
    }

    async fn send_post_rpc_request<T: Serialize + ?Sized>(
        &self,
        request_body: &T,
    ) -> Result<Response, Error> {
        self.rpc_client
            .post(self.rpc_url.clone())
            .json(&request_body)
            .send()
            .await
    }

    async fn get_raw_transaction(
        &self,
        transaction_hash: String,
    ) -> Result<ValidateTransactionHashResponseData, BtcApiError> {
        //call blockchain api to get raw transaction

        let url = format!("{}{}", BLOCKCHAIN_API_RAW_TRANSACTION_URL, transaction_hash);
        let response_body = reqwest::get(url).await?.text().await?;

        let blockchaincom_raw_txn = serde_json::from_str::<BlockchaincomRawTxn>(&response_body)?;

        let txn_block_index = blockchaincom_raw_txn.block_index;
        let txn_block_height = blockchaincom_raw_txn.block_height;
        let double_spend = blockchaincom_raw_txn.double_spend;
        let rbf = blockchaincom_raw_txn.rbf;

        match (txn_block_index, txn_block_height, double_spend, rbf) {
            (None, None, true, _) => {
                //CASE : INVALID TXN
                //Transaction is invalid/cacelled if
                //txn_block_index is None
                //txn_block_height is None
                //double_spend is true

                let result = ValidateTransactionHashResponseData {
                    txn_hash: transaction_hash,
                    txn_status: TxnStatus::Cancelled,
                    txn_data: Some(TxnData {
                        //TODO: Remove Unwrap
                        block_index: None,
                        block_height: None,
                        consumed_fees_in_satoshis: blockchaincom_raw_txn.get_total_fee(),
                        //TODO: Calculate txn amount in satoshis
                        txn_input_amount_in_satoshis: blockchaincom_raw_txn
                            .get_total_input_amount(),
                        txn_output_amount_in_satoshis: blockchaincom_raw_txn
                            .get_total_output_amount(),
                        input_txns: blockchaincom_raw_txn.get_input_txns(),
                        output_txns: blockchaincom_raw_txn.get_output_txns(),
                    }),
                };

                Ok(result)
            }
            (Some(block_index), Some(block_height), false, None) => {
                //Transaction is valid if
                //txn_block_index is not None
                //txn_block_height is not None
                //double_spend is false
                //rbf is None/false

                let result = ValidateTransactionHashResponseData {
                    txn_hash: transaction_hash,
                    txn_status: TxnStatus::Confirmed,
                    txn_data: Some(TxnData {
                        //TODO: Remove Unwrap
                        block_index: Some(block_index),
                        block_height: Some(block_height),
                        consumed_fees_in_satoshis: blockchaincom_raw_txn.get_total_fee(),
                        //TODO: Calculate txn amount in satoshis
                        txn_input_amount_in_satoshis: blockchaincom_raw_txn
                            .get_total_input_amount(),
                        txn_output_amount_in_satoshis: blockchaincom_raw_txn
                            .get_total_output_amount(),
                        input_txns: blockchaincom_raw_txn.get_input_txns(),
                        output_txns: blockchaincom_raw_txn.get_output_txns(),
                    }),
                };

                Ok(result)
            }
            (None, None, false, Some(true)) => {
                //Transaction is still mining if
                //txn_block_index is None
                //txn_block_height is None
                //double_spend is false
                //rbf is true

                let result = ValidateTransactionHashResponseData {
                    txn_hash: transaction_hash,
                    txn_status: TxnStatus::Pending,
                    txn_data: Some(TxnData {
                        //TODO: Remove Unwrap
                        block_index: None,
                        block_height: None,
                        consumed_fees_in_satoshis: blockchaincom_raw_txn.get_total_fee(),
                        //TODO: Calculate txn amount in satoshis
                        txn_input_amount_in_satoshis: blockchaincom_raw_txn
                            .get_total_input_amount(),
                        txn_output_amount_in_satoshis: blockchaincom_raw_txn
                            .get_total_output_amount(),
                        input_txns: blockchaincom_raw_txn.get_input_txns(),
                        output_txns: blockchaincom_raw_txn.get_output_txns(),
                    }),
                };

                Ok(result)
            }
            _ => Err(BtcApiError::UnableToVerifyTxnStatus),
        }
    }
}
