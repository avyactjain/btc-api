use serde::{Deserialize, Serialize};

use crate::btc_api_error::BtcApiError;

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

#[derive(Serialize, Deserialize, Debug, Clone)]
// All fee in Satoshis
pub struct CreateTransactionParams {
    pub from_address: String,
    pub to_address: String,
    pub amount: u64,
    pub fee: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateTransactionResponse {
    pub is_error: bool,
    pub data: Option<CreateTransactionResponseData>,
    pub error_msg: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateTransactionResponseData {
    pub unsigned_raw_txn: String,
}

impl CreateTransactionParams {
    pub fn validate(self) -> Result<CreateTransactionParams, BtcApiError> {
        if self.fee >= self.amount {
            Err(BtcApiError::InvalidFee(format!(
                "Fee {} is greater than amount {}",
                self.fee, self.amount
            )))
        } else {
            Ok(self)
        }
    }
}

mod test {
    use crate::models::CreateTransactionParams;


    #[test]
    fn test_deserialize_create_transaction_params() {
        let json = r#"{"from_address": "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa", "to_address": "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa", "amount": 100000000, "fee": 100000000}"#;
        let params: CreateTransactionParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.from_address, "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa");
        assert_eq!(params.to_address, "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa");
        assert_eq!(params.amount, 100000000);
        assert_eq!(params.fee, 100000000);
    }
}
