use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct GetRawTransactionResponse {
    result: GetRawTransactionResponseData,
    error: Option<serde_json::Value>,
    id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetRawTransactionResponseData {
    txid: String,
    hash: String,
    version: i64,
    size: i64,
    vsize: i64,
    weight: i64,
    locktime: i64,
    vin: Vec<Vin>,
    vout: Vec<Vout>,
    hex: String,
    blockhash: String,
    confirmations: i64,
    time: i64,
    blocktime: i64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Vin {
    txid: String,
    vout: i64,
    script_sig: ScriptSig,
    txinwitness: Vec<String>,
    sequence: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ScriptSig {
    asm: String,
    hex: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Vout {
    value: f64,
    n: i64,
    script_pub_key: ScriptPubKey,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ScriptPubKey {
    asm: String,
    desc: String,
    hex: String,
    address: String,
    #[serde(rename = "type")]
    script_pub_key_type: String,
}
