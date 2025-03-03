use std::str::FromStr;

use axum::Json;
use bitcoin::{
    absolute::LockTime, consensus::encode::serialize_hex, key::Secp256k1, secp256k1::Message,
    sighash::SighashCache, transaction::Version, Address, Amount, EcdsaSighashType, Network,
    OutPoint, Script, ScriptBuf, TxIn, TxOut, Txid, Witness,
};

use bitcoin::blockdata::transaction::Transaction;
use regex::Regex;
use reqwest::{Client, Url};
use response_models::{BlockchaincomResponse, BlockstreamUtxo, BlockstreamWalletBalance};
use tracing::{debug, error, info};
use utils::senders_keys;
pub(crate) mod response_models;

use crate::models::{TransactionData, WalletBalanceResponse, WalletBalanceResponseData};
use crate::{
    btc_api_error::BtcApiError,
    chain::Chain,
    config::ChainVariant,
    models::{
        BroadcastTransactionResponse, BroadcastTransactionResponseData, CreateTransactionParams,
        CreateTransactionResponse, CreateTransactionResponseData, NetworkFeeResponse,
        NetworkFeeResponseData, TxnStatus, ValidateTransactionHashResponse,
        ValidateTransactionHashResponseData,
    },
};
mod utils;

// Mempool API for network fee
const MEMPOOL_API_NETWORK_FEE_URL: &str = "https://mempool.space/api/v1/fees/recommended";
// Blockchain API for raw transaction, always mainnet
const BLOCKCHAIN_API_RAW_TRANSACTION_URL: &str = "https://blockchain.info/rawtx/";
// Bitcoin txid regex
const BITCOIN_TXID_REGEX: &str = r"^[a-fA-F0-9]{64}$";
// Blockstream Testnet Explorer URL
const BLOCKSTREAM_TESTNET_EXPLORER_URL: &str = "https://blockstream.info/testnet/";
// Blockstream Mainnet Explorer URL
const BLOCKSTREAM_MAINNET_EXPLORER_URL: &str = "https://blockstream.info/";

#[derive(Debug, Clone)]
pub struct Bitcoin {
    pub rpc_url: Url,
    pub network: Network,
    pub bitcoin_txid_regex: Regex,
    pub explorer_url: Url,
    pub sign_txn: bool,
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
            Ok((transaction, used_utxos)) => {
                if self.sign_txn {
                    let signed_txn_hash = self
                        .sign_transaction(transaction.clone(), used_utxos.clone())
                        .await;
                }

                // self.broadcast_transaction(signed_txn_hash).await.unwrap();

                result.is_error = false;
                result.data = Some(CreateTransactionResponseData {
                    unsigned_raw_txn: transaction,
                    used_utxos,
                });
            }
            Err(err) => {
                result.error_msg = Some(err.to_string());
            }
        }

        Json(result)
    }

    async fn broadcast_transaction(
        &self,
        transaction: crate::models::BroadcastTransactionParams,
    ) -> Json<crate::models::BroadcastTransactionResponse> {
        let mut result = BroadcastTransactionResponse {
            is_error: false,
            data: None,
            error_msg: None,
        };

        match self.broadcast_transaction(transaction.signed_raw_txn).await {
            Ok(broadcase_api_response) => {
                result.data = Some(broadcase_api_response);
            }
            Err(err) => {
                result.is_error = true;
                result.error_msg = Some(err.to_string());
            }
        }

        Json(result)
    }

    async fn get_wallet_balance(
        &self,
        address: String,
    ) -> Json<crate::models::WalletBalanceResponse> {
        let mut result = WalletBalanceResponse {
            is_error: true,
            data: None,
            error_msg: None,
        };

        match self.get_wallet_balance(address).await {
            Ok(wallet_balance) => {
                result.is_error = false;
                result.data = Some(wallet_balance);
            }
            Err(err) => {
                result.error_msg = Some(err.to_string());
            }
        }

        Json(result)
    }
}

impl Bitcoin {
    pub fn new(rpc_url: &str, variant: &ChainVariant, sign_txn: bool) -> Result<Self, BtcApiError> {
        let (network, explorer_url) = match variant {
            ChainVariant::Mainnet => (Network::Bitcoin, BLOCKSTREAM_MAINNET_EXPLORER_URL),
            ChainVariant::Testnet => (Network::Testnet, BLOCKSTREAM_TESTNET_EXPLORER_URL),
        };

        info!(
            "Creating Bitcoin instance with rpc_url: {} on network: {}",
            rpc_url, network
        );

        Ok(Self {
            rpc_url: rpc_url.parse::<Url>()?,
            network,
            bitcoin_txid_regex: Regex::new(BITCOIN_TXID_REGEX)?,
            explorer_url: explorer_url.parse::<Url>()?,
            sign_txn,
        })
    }

    async fn get_wallet_balance(
        &self,
        address: String,
    ) -> Result<WalletBalanceResponseData, BtcApiError> {
        let url = self.rpc_url.join(&format!("address/{}", address))?;

        let blockstream_response = reqwest::get(url).await?.text().await?;

        let blockstream_wallet_balance =
            serde_json::from_str::<BlockstreamWalletBalance>(&blockstream_response)?;

        // This can be negative also if the wallet has more unconfirmedincoming transactions than outgoing
        let confirmed_balance = blockstream_wallet_balance.get_confirmed_balance();

        // This can be negative also if the wallet has more unconfirmed outgoing transactions than incoming
        let unconfirmed_balance = blockstream_wallet_balance.get_unconfirmed_balance();

        let total_balance = confirmed_balance + unconfirmed_balance;

        Ok(WalletBalanceResponseData {
            confirmed_balance,
            unconfirmed_balance: unconfirmed_balance,
            total_balance,
        })
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
                            txn_status_flag: 1,
                            txn_data: Some(TransactionData {
                                block_index: None,
                                block_height: None,
                                consumed_fees: blockchaincom_raw_txn.get_total_fee(),
                                txn_input_amount: blockchaincom_raw_txn.get_total_input_amount(),
                                txn_output_amount: blockchaincom_raw_txn.get_total_output_amount(),
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
                            txn_status_flag: 0,
                            txn_data: Some(TransactionData {
                                block_index: Some(block_index),
                                block_height: Some(block_height),
                                consumed_fees: blockchaincom_raw_txn.get_total_fee(),
                                txn_input_amount: blockchaincom_raw_txn.get_total_input_amount(),
                                txn_output_amount: blockchaincom_raw_txn.get_total_output_amount(),
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
                            txn_status_flag: 2,
                            txn_data: Some(TransactionData {
                                block_index: None,
                                block_height: None,
                                consumed_fees: blockchaincom_raw_txn.get_total_fee(),
                                txn_input_amount: blockchaincom_raw_txn.get_total_input_amount(),
                                txn_output_amount: blockchaincom_raw_txn.get_total_output_amount(),
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
    ) -> Result<(Transaction, Vec<BlockstreamUtxo>), BtcApiError> {
        let send_amount = transaction_params.amount;

        let receiver_address =
            Address::from_str(&transaction_params.to_address)?.require_network(self.network)?;

        //This will be the change address as well
        let sender_address =
            Address::from_str(&transaction_params.from_address)?.require_network(self.network)?;

        //1. Get the Txn inputs based on the UTXOs and the change amount
        let (inputs, used_utxos, change_amount) = self
            .get_input_txns_utxos_change_amount(transaction_params)
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

        info!("Unsigned transaction created: {:#?}", txn);
        Ok((txn, used_utxos))
    }

    async fn get_input_txns_utxos_change_amount(
        &self,
        transaction_params: CreateTransactionParams,
    ) -> Result<(Vec<TxIn>, Vec<BlockstreamUtxo>, u64), BtcApiError> {
        let total_expenditure = transaction_params.amount + transaction_params.fee;
        let mut total_utxo_value = 0;
        let mut inputs = vec![];
        let mut used_utxos = vec![];

        //1. Get the utxos for the from address
        let mut utxos = self
            .find_spendable_utxos(transaction_params.from_address.clone())
            .await?;

        //2. Sort the UTXOs by value in ascending order, this is to get the smallest utxos first so transaction is split up as much as possible
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
                used_utxos.push(utxo);
            }
        }

        if total_expenditure > total_utxo_value {
            return Err(BtcApiError::InsufficientFunds(
                total_expenditure - total_utxo_value,
            ));
        }

        Ok((
            inputs.to_vec(),
            used_utxos,
            (total_utxo_value - total_expenditure),
        ))
    }

    async fn find_spendable_utxos(
        &self,
        address: String,
    ) -> Result<Vec<BlockstreamUtxo>, BtcApiError> {
        //todo: Do URL parsing here
        let url = self.rpc_url.join(&format!("address/{}/utxo", address))?;

        let blockstream_response = reqwest::get(url).await?.text().await?;
        let blockstream_utxos =
            serde_json::from_str::<Vec<BlockstreamUtxo>>(&blockstream_response)?;

        if blockstream_utxos.is_empty() {
            Err(BtcApiError::NoUtxosFound(address))
        } else {
            Ok(blockstream_utxos)
        }
    }
    async fn sign_transaction(
        &self,
        mut unsigned_txn: Transaction,
        used_utxos: Vec<BlockstreamUtxo>,
    ) -> String {
        info!("Signing transaction with {:#?}", unsigned_txn);

        let secp = Secp256k1::new();

        let (sk, wpkh) = senders_keys(&secp);

        let sighash_type = EcdsaSighashType::All;

        let mut sighasher = SighashCache::new(&mut unsigned_txn);

        //need to sign every input of the unsigned txn
        for (input_index, utxo) in used_utxos.iter().enumerate() {
            // Create SegwitV0Signhash
            let sighash = sighasher
                .p2wpkh_signature_hash(
                    input_index,
                    &ScriptBuf::new_p2wpkh(&wpkh),
                    Amount::from_sat(utxo.value),
                    sighash_type,
                )
                .expect("failed to create sighash");

            // Sign the sighash using the secp256k1
            let msg = Message::from(sighash);
            let signature = secp.sign_ecdsa(&msg, &sk);

            // Update the witness stack.
            let signature = bitcoin::ecdsa::Signature {
                signature,
                sighash_type,
            };
            let pk = sk.public_key(&secp);
            *sighasher.witness_mut(input_index).unwrap() = Witness::p2wpkh(&signature, &pk);
        }

        // Get the signed transaction.
        let signed_txn: &mut Transaction = sighasher.into_transaction();

        let signed_txn_hash = serialize_hex(&signed_txn);

        info!("Signed transaction hash: {}", signed_txn_hash);
        signed_txn_hash
    }

    async fn broadcast_transaction(
        &self,
        signed_txn_hash: String,
    ) -> Result<BroadcastTransactionResponseData, BtcApiError> {
        info!("Broadcasting transaction: {}", signed_txn_hash);

        let url = self.rpc_url.join("tx")?;

        let client = Client::new();
        let response = client.post(url).body(signed_txn_hash).send().await?;

        let response_text = response.text().await?;

        //Check if the response text is a hash
        if self.bitcoin_txid_regex.is_match(&response_text) {
            info!("✅ Transaction broadcast result: {}", response_text);

            //Valid txid, transaction broadcasted successfully
            let explorer_url = self.explorer_url.join(&format!("tx/{}", response_text))?;

            Ok(BroadcastTransactionResponseData {
                txn_hash: response_text,
                txn_hash_url: explorer_url.to_string(),
            })
        } else {
            return Err(BtcApiError::InvalidBroadcastResponse(response_text));
        }
    }
}

#[tokio::test]
async fn test_get_raw_transaction() {
    // All mainnet txn hashes
    // Comment out pending txn hash if needed. It might be confirmed by the time you run the test.
    let pending_txn_hash = "52d54371b3564f7beb9d352abc001f28fef8ff18d499f7b02909f752d3bf732f";
    let confirmed_txn_hash = "ce593556a4868d9ac26a860505a1c732aa38aea51d942505afc0b491c3b35f87";
    let cancelled_txn_hash = "69f8ab2bf2d82b3e5fd7626736d040d9c11d4ea3c31fb0c30bb0d72e8c5a6238";

    let bitcoin = Bitcoin::new("https://xxx.xxx.xx", &ChainVariant::Mainnet, false).unwrap();

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

    let bitcoin = Bitcoin::new("https://xxx.xxx.xx", &ChainVariant::Testnet, false).unwrap();

    let available_utxos = bitcoin
        .find_spendable_utxos(wallet_address.to_string())
        .await;

    println!(
        "Available UTXOs for {} are {:#?}",
        wallet_address, available_utxos
    );

    assert!(available_utxos.is_ok());
}
