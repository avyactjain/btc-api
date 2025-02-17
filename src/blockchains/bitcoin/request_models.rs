use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct GetRawTransaction {
    pub jsonrpc: String,
    pub id: String,
    pub method: String,
    pub params: Vec<Param>,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum Param {
    String(String),
    Bool(bool),
}
