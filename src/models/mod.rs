use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkFeeResponse {
    pub is_error: bool,
    pub data: Option<NetworkFeeResponseData>,
    pub error_msg: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkFeeResponseData {
    fastest_fee: i64,
    half_hour_fee: i64,
    hour_fee: i64,
    economy_fee: i64,
    minimum_fee: i64,
}

#[derive(Debug, Deserialize)]
pub struct ValidateTransactionHashParams {
    pub transaction_hash: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidateTransactionHashResponse {
    pub is_error: bool,
    pub data: Option<ValidateTransactionHashResponseData>,
    pub error_msg: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidateTransactionHashResponseData {
    pub txn_hash: String,
    pub txn_status: TxnStatus,
    pub txn_data: Option<TxnData>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub enum TxnStatus {
    Confirmed,
    Cancelled,
    Pending,
}
#[derive(Serialize, Deserialize)]
pub struct TxnData {
    pub block_index: Option<u64>,
    pub block_height: Option<u64>,
    pub consumed_fees_in_satoshis: u64,
    pub txn_input_amount_in_satoshis: u64,
    pub txn_output_amount_in_satoshis: u64,
    pub input_txns: Vec<AddressSpent>,
    pub output_txns: Vec<AddressSpent>,
}

#[derive(Serialize, Deserialize)]
pub struct AddressSpent {
    pub address: String,
    // Amount in satoshis
    pub amount: u64,
}
