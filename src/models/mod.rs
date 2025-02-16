#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkFeeResponse {
    pub is_error: bool,
    pub data: Option<NetworkFeeResponseData>,
    pub error_msg: Option<String>,
}

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkFeeResponseData {
    fastest_fee: i64,
    half_hour_fee: i64,
    hour_fee: i64,
    economy_fee: i64,
    minimum_fee: i64,
}
