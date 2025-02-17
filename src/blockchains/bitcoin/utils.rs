use super::request_models::{GetRawTransaction, Param};

const JSON_RPC_VERSION: &str = "1.0";
const GET_RAW_TRANSACTION_METHOD: &str = "getrawtransaction";


pub fn create_validate_transaction_request_body(transaction_hash: String, id: u64) -> GetRawTransaction {
    GetRawTransaction{
        jsonrpc: JSON_RPC_VERSION.to_string(),
        id: id.to_string(),
        method: GET_RAW_TRANSACTION_METHOD.to_string(),
        params: vec![Param::String(transaction_hash), Param::Bool(true)],
    }
}
