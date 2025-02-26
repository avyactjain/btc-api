use std::{error::Error, str::FromStr};

use axum::Json;
use bitcoin::{
    absolute::LockTime,
    consensus::encode::serialize_hex,
    hashes::{sha256d, Hash},
    key::{rand::rngs::OsRng, Secp256k1},
    secp256k1::{Message, SecretKey},
    sighash::{Prevouts, SighashCache},
    transaction::Version,
    Address, Amount, CompressedPublicKey, EcdsaSighashType, KnownHrp, Network, NetworkKind,
    OutPoint, PrivateKey, PublicKey, Script, ScriptBuf, Sequence, TxIn, TxOut, Txid, Witness,
};

use bitcoin::blockdata::transaction::Transaction;
use bitcoincore_rpc::json::SigHashType;
use reqwest::{Client, Url};
use response_models::{BlockchaincomResponse, BlockstreamUtxo};
use tracing::{debug, error, info};
use utils::{
    dummy_unspent_transaction_output, receivers_address, senders_keys, CHANGE_AMOUNT,
    DUMMY_UTXO_AMOUNT, SPEND_AMOUNT,
};

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
            Ok((transaction, used_utxos)) => {
                let raw_txn = serialize_hex(&transaction);

                // Convert the raw transaction to bytes
                let raw_tx_bytes = hex::decode(raw_txn).expect("Invalid hex transaction");

                // Compute the double SHA-256 hash (32 bytes)
                let tx_hash = sha256d::Hash::hash(&raw_tx_bytes).to_string();

                // let signed_txn = self
                //     .sign_transaction(transaction.clone(), used_utxos)
                //     .await
                //     .unwrap();

                // self.broadcast_transaction(signed_txn).await.unwrap();

                let x = self.sign_txn_test().await;
                self.broadcast_transaction(x).await.unwrap();

                result.is_error = false;
                result.data = Some(CreateTransactionResponseData {
                    unsigned_raw_txn: tx_hash,
                })
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
    ) -> Result<(Transaction, Vec<BlockstreamUtxo>), BtcApiError> {
        let send_amount = transaction_params.amount;

        let receiver_address =
            Address::from_str(&transaction_params.to_address)?.require_network(self.network)?;

        //This will be the change address as well
        let sender_address =
            Address::from_str(&transaction_params.from_address)?.require_network(self.network)?;

        //1. Get the Txn inputs based on the UTXOs and the change amount
        let (inputs, used_utxos, change_amount) = self
            .get_input_txns_utxos_amounts(transaction_params)
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

        Ok((txn, used_utxos))
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

        if blockstream_utxos.is_empty() {
            Err(BtcApiError::NoUtxosFound(address))
        } else {
            println!("Blockstream UTXOs: {:#?}", blockstream_utxos);
            Ok(blockstream_utxos)
        }
    }

    async fn get_input_txns_utxos_amounts(
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
                used_utxos.push(utxo);
            }
        }

        Ok((
            inputs.to_vec(),
            used_utxos,
            total_utxo_value - total_expenditure,
        ))
    }

    async fn sign_txn_test(&self) -> String {
        let secp = Secp256k1::new();

        // Get a secret key we control and the pubkeyhash of the associated pubkey.
        // In a real application these would come from a stored secret.
        let (sk, wpkh) = senders_keys(&secp);

        // Get an address to send to.
        let address = receivers_address();

        // Get an unspent output that is locked to the key above that we control.
        // In a real application these would come from the chain.
        let (dummy_out_point, dummy_utxo) = dummy_unspent_transaction_output(wpkh);

        // The input for the transaction we are constructing.
        let input = TxIn {
            previous_output: dummy_out_point, // The dummy output we are spending.
            script_sig: ScriptBuf::default(), // For a p2wpkh script_sig is empty.
            sequence: Sequence::ZERO,
            witness: Witness::default(), // Filled in after signing.
        };

        // The spend output is locked to a key controlled by the receiver.
        let spend = TxOut {
            value: SPEND_AMOUNT,
            script_pubkey: address.script_pubkey(),
        };

        // The change output is locked to a key controlled by us.
        let change = TxOut {
            value: CHANGE_AMOUNT,
            script_pubkey: ScriptBuf::new_p2wpkh(&wpkh), // Change comes back to us.
        };

        // The transaction we want to sign and broadcast.
        let mut unsigned_tx = Transaction {
            version: Version::TWO,       // Post BIP-68.
            lock_time: LockTime::ZERO,   // Ignore the locktime.
            input: vec![input],          // Input goes into index 0.
            output: vec![spend, change], // Outputs, order does not matter.
        };
        let input_index = 0;

        // Get the sighash to sign.
        let sighash_type = EcdsaSighashType::All;
        let mut sighasher = SighashCache::new(&mut unsigned_tx);
        let sighash = sighasher
            .p2wpkh_signature_hash(
                input_index,
                &dummy_utxo.script_pubkey,
                DUMMY_UTXO_AMOUNT,
                sighash_type,
            )
            .expect("failed to create sighash");

        // Sign the sighash using the secp256k1 library (exported by rust-bitcoin).
        let msg = Message::from(sighash);
        let signature = secp.sign_ecdsa(&msg, &sk);

        // Update the witness stack.
        let signature = bitcoin::ecdsa::Signature {
            signature,
            sighash_type,
        };
        let pk = sk.public_key(&secp);
        *sighasher.witness_mut(input_index).unwrap() = Witness::p2wpkh(&signature, &pk);

        // Get the signed transaction.
        let tx = sighasher.into_transaction();

        // BOOM! Transaction signed and ready to broadcast.
        println!("{:#?}", tx);

        let x = serialize_hex(&tx);
        println!("Signed Transaction: {}", x);
        x
    }

    // async fn sign_transaction(
    //     &self,
    //     mut unsigned_txn: Transaction,
    //     utxos: Vec<BlockstreamUtxo>,
    // ) -> Result<String, BtcApiError> {
    //     let secp = Secp256k1::new();

    //     let wif = "cSjgVro2xkCVat8fjye1jNfozoaC8XASd3UuvLXF49ugaZx1MHsg"; // Replace with your private key
    //     let private_key = PrivateKey::from_wif(wif).expect("Invalid WIF key");
    //     let public_key = private_key.public_key(&secp);

    //     let mut signatures = vec![];

    //     {
    //         let mut sighash_cache = SighashCache::new(&unsigned_txn);

    //         for (i, utxo) in utxos.iter().enumerate() {
    //             let pubkey = CompressedPublicKey::from_private_key(&secp, &private_key).unwrap();
    //             println!("Pubkey: {:#?}", pubkey);
    //             let address = Address::p2wpkh(&pubkey, Network::Testnet);
    //             let script_pubkey = address.script_pubkey();
    //             println!("Script Pubkey: {:#?}", script_pubkey);
    //             println!("Address: {:#?}", address);

    //             let sighash_msg = sighash_cache
    //                 .p2wpkh_signature_hash(
    //                     0,
    //                     script_pubkey.as_script(),
    //                     Amount::from_sat(utxo.value),
    //                     bitcoin::EcdsaSighashType::All,
    //                 )
    //                 .unwrap();

    //             let msg =
    //                 Message::from_digest_slice(&sighash_msg[..]).expect("Failed to create message");

    //             let sig = secp.sign_ecdsa(&msg, &private_key.inner);

    //             let mut sig_with_sighash = sig.serialize_der().to_vec();
    //             sig_with_sighash.push(bitcoin::EcdsaSighashType::All as u8);
    //             signatures.push((i, sig_with_sighash, pubkey.to_bytes()));
    //         }
    //     }

    //     println!("Signatures: {:#?}", signatures);
    //     // 🔹 Apply Signatures to Transaction
    //     for (i, sig, pubkey) in signatures {
    //         unsigned_txn.input[i].witness.push(sig);
    //         unsigned_txn.input[i].witness.push(pubkey);
    //     }

    //     println!("Signed Transaction: {:#?}", unsigned_txn);
    //     let x = unsigned_txn.clone();
    //     println!("Signed Transaction: {:#?}", serialize_hex(&x));
    //     // 🔹 Serialize & Print Signed Transaction
    //     let raw_tx_hex = serialize_hex(&unsigned_txn);
    //     println!("Signed Transaction: {}", raw_tx_hex);

    //     Ok(raw_tx_hex)
    // }

    async fn broadcast_transaction(&self, signed_txn_hash: String) -> Result<(), Box<dyn Error>> {
        println!("Broadcasting transaction");
        // 2️⃣ Blockstream API endpoint for Testnet
        let url = "https://blockstream.info/testnet/api/tx";

        // 3️⃣ Send the transaction using an HTTP POST request
        let client = Client::new();
        let response = client.post(url).body(signed_txn_hash).send().await?;

        // 4️⃣ Print the Transaction ID (TxID)
        println!(
            "✅ Transaction broadcasted! TxID: {}",
            response.text().await?
        );

        Ok(())
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
