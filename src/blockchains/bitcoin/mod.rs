use std::str::FromStr;

use axum::Json;
use bitcoin::{
    absolute::LockTime, consensus::encode::serialize_hex, transaction::Version, Address, Amount,
    Network, OutPoint, Script, Transaction, TxIn, TxOut, Txid, Witness,
};
use reqwest::Url;
use response_models::{BlockchaincomResponse, BlockstreamUtxo};
use tracing::{debug, error, info};

use crate::{
    btc_api_error::BtcApiError,
    chain::Chain,
    config::ChainVariant,
    models::{
        CreateTransactionParams, CreateTransactionResponse, CreateTransactionResponseData,
        NetworkFeeResponse, NetworkFeeResponseData, TxnData, TxnStatus,
        ValidateTransactionHashResponse, ValidateTransactionHashResponseData,
    },
};
mod request_models;
mod response_models;
mod utils;

// Mempool API for network fee
const MEMPOOL_API_NETWORK_FEE_URL: &str = "https://mempool.space/api/v1/fees/recommended";
// Blockchain API for raw transaction, always mainnet
const BLOCKCHAIN_API_RAW_TRANSACTION_URL: &str = "https://blockchain.info/rawtx/";

#[derive(Debug, Clone)]
pub struct Bitcoin {
    pub rpc_url: String,
    pub network: Network,
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

    async fn create_transaction(
        &self,
        transaction_params: crate::models::CreateTransactionParams,
    ) -> Json<crate::models::CreateTransactionResponse> {
        let mut result = CreateTransactionResponse {
            is_error: true,
            data: None,
            error_msg: None,
        };

        match self.create_transaction(transaction_params).await {
            Ok(transaction) => {
                let unsigned_raw_txn = serialize_hex(&transaction);

                result.is_error = false;
                result.data = Some(CreateTransactionResponseData { unsigned_raw_txn })
            }
            Err(err) => {
                result.error_msg = Some(err.to_string());
            }
        }

        Json(result)
    }
}

impl Bitcoin {
    pub fn new(rpc_url: &str, variant: &ChainVariant) -> Self {
        let network = match variant {
            ChainVariant::Mainnet => Network::Bitcoin,
            ChainVariant::Testnet => Network::Testnet,
        };

        info!(
            "Creating Bitcoin instance with rpc_url: {} on network: {}",
            rpc_url, network
        );

        Self {
            rpc_url: rpc_url.to_owned(),
            network,
        }
    }

    async fn get_raw_transaction(
        &self,
        transaction_hash: String,
    ) -> Result<ValidateTransactionHashResponseData, BtcApiError> {
        //call blockchain api to get raw transaction

        let url = BLOCKCHAIN_API_RAW_TRANSACTION_URL
            .parse::<Url>()?
            .join(&transaction_hash)?;

        let response_body = reqwest::get(url).await?.text().await?;

        let blockchaincom_response = serde_json::from_str::<BlockchaincomResponse>(&response_body)?;

        match blockchaincom_response {
            BlockchaincomResponse::RawTxn(blockchaincom_raw_txn) => {
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
                                block_index: None,
                                block_height: None,
                                consumed_fees_in_satoshis: blockchaincom_raw_txn.get_total_fee(),
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
                        //CASE: VALID TXN
                        //Transaction is valid if
                        //txn_block_index is not None
                        //txn_block_height is not None
                        //double_spend is false
                        //rbf is None/false

                        let result = ValidateTransactionHashResponseData {
                            txn_hash: transaction_hash,
                            txn_status: TxnStatus::Confirmed,
                            txn_data: Some(TxnData {
                                block_index: Some(block_index),
                                block_height: Some(block_height),
                                consumed_fees_in_satoshis: blockchaincom_raw_txn.get_total_fee(),
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
                        //CASE : PENDING TXN
                        //Transaction is still mining if
                        //txn_block_index is None
                        //txn_block_height is None
                        //double_spend is false
                        //rbf is true

                        let result = ValidateTransactionHashResponseData {
                            txn_hash: transaction_hash,
                            txn_status: TxnStatus::Pending,
                            txn_data: Some(TxnData {
                                block_index: None,
                                block_height: None,
                                consumed_fees_in_satoshis: blockchaincom_raw_txn.get_total_fee(),
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
                    _ => {
                        error!(
                            "Unable to verify transaction status for txn hash: {}.",
                            transaction_hash
                        );

                        Err(BtcApiError::UnableToVerifyTxnStatus)
                    }
                }
            }
            BlockchaincomResponse::ApiError(blockchaincom_api_error) => {
                Err(BtcApiError::ExternalApiError(format!(
                    "{}: {}",
                    blockchaincom_api_error.error, blockchaincom_api_error.message
                )))
            }
        }
    }

    async fn create_transaction(
        &self,
        transaction_params: CreateTransactionParams,
    ) -> Result<Transaction, BtcApiError> {
        let send_amount = transaction_params.amount;

        let receiver_address =
            Address::from_str(&transaction_params.to_address)?.require_network(self.network)?;

        //This will be the change address as well
        let sender_address =
            Address::from_str(&transaction_params.from_address)?.require_network(self.network)?;

        //1. Get the Txn inputs based on the UTXOs and the change amount
        let (inputs, change_amount) = self
            .get_input_txns_and_change_amount(transaction_params)
            .await?;

        debug!("Inputs: {:#?}", inputs);
        debug!("Change amount: {}", change_amount);

        let txout_receiver = TxOut {
            value: Amount::from_sat(send_amount),
            script_pubkey: receiver_address.script_pubkey(),
        };

        let txout_change = TxOut {
            value: Amount::from_sat(change_amount),
            script_pubkey: sender_address.script_pubkey(),
        };

        // Create the unsigned transaction
        debug!("Creating unsigned transaction");

        let txn = Transaction {
            version: Version::TWO,
            lock_time: LockTime::ZERO,
            input: inputs,
            output: vec![txout_receiver, txout_change],
        };

        debug!("Unsigned transaction created: {:#?}", txn);

        Ok(txn)
    }

    async fn find_spendable_utxos(
        &self,
        address: String,
    ) -> Result<Vec<BlockstreamUtxo>, BtcApiError> {
        //todo: Do URL parsing here
        let url = self
            .rpc_url
            .parse::<Url>()?
            .join(&format!("address/{}/utxo", address))?;

        let blockstream_response = reqwest::get(url).await?.text().await?;
        let blockstream_utxos =
            serde_json::from_str::<Vec<BlockstreamUtxo>>(&blockstream_response)?;

        Ok(blockstream_utxos)
    }

    async fn get_input_txns_and_change_amount(
        &self,
        transaction_params: CreateTransactionParams,
    ) -> Result<(Vec<TxIn>, u64), BtcApiError> {
        let total_expenditure = transaction_params.amount + transaction_params.fee;
        let mut total_utxo_value = 0;
        let mut inputs = vec![];

        //1. Get the utxos for the from address
        let mut utxos = self
            .find_spendable_utxos(transaction_params.from_address)
            .await?;

        //2. Sort the UTXOs by value in ascending order
        utxos.sort_by_key(|utxo| utxo.value);

        for utxo in utxos {
            if utxo.is_confirmed() && total_utxo_value < total_expenditure {
                let txid = Txid::from_str(&utxo.get_txid())?;
                let outpoint = OutPoint::new(txid, utxo.get_vout());

                inputs.push(TxIn {
                    previous_output: outpoint,
                    script_sig: Script::new().into(),
                    //Should be 0xFFFFFFFF (ignored)
                    sequence: bitcoin::Sequence(0xFFFFFFFF),
                    witness: Witness::new(),
                });

                debug!("Added UTXO: {:#?}", utxo);
                total_utxo_value += utxo.value;
            }
        }

        Ok((inputs.to_vec(), total_utxo_value - total_expenditure))
    }
}

#[tokio::test]
async fn test_get_raw_transaction() {
    // All mainnet txn hashes
    // Comment out pending txn hash if needed. It might be confirmed by the time you run the test.
    let pending_txn_hash = "52d54371b3564f7beb9d352abc001f28fef8ff18d499f7b02909f752d3bf732f";
    let confirmed_txn_hash = "ce593556a4868d9ac26a860505a1c732aa38aea51d942505afc0b491c3b35f87";
    let cancelled_txn_hash = "69f8ab2bf2d82b3e5fd7626736d040d9c11d4ea3c31fb0c30bb0d72e8c5a6238";

    let bitcoin = Bitcoin::new("https://xxx.xxx.xx", &ChainVariant::Mainnet);

    let pending_txn_result = bitcoin
        .get_raw_transaction(pending_txn_hash.to_string())
        .await;
    let confirmed_txn_result = bitcoin
        .get_raw_transaction(confirmed_txn_hash.to_string())
        .await;

    let cancelled_txn_result = bitcoin
        .get_raw_transaction(cancelled_txn_hash.to_string())
        .await;

    assert_eq!(pending_txn_result.unwrap().txn_status, TxnStatus::Pending);
    assert_eq!(
        confirmed_txn_result.unwrap().txn_status,
        TxnStatus::Confirmed
    );
    assert_eq!(
        cancelled_txn_result.unwrap().txn_status,
        TxnStatus::Cancelled
    );
}

#[tokio::test]
async fn test_find_spendable_utxos() {
    // All mainnet txn hashes
    // Comment out pending txn hash if needed. It might be confirmed by the time you run the test.
    let wallet_address = "mrZ8L1SgPERaUXbrLrT2dxkfKBYk5RzmB9";

    let bitcoin = Bitcoin::new("https://xxx.xxx.xx", &ChainVariant::Testnet);

    let available_utxos = bitcoin
        .find_spendable_utxos(wallet_address.to_string())
        .await;

    println!(
        "Available UTXOs for {} are {:#?}",
        wallet_address, available_utxos
    );

    assert!(available_utxos.is_ok());
}
