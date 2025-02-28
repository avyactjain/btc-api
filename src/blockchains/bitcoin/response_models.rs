// Type of response from blockchain.info/rawtx/<transaction_hash>

use serde::{Deserialize, Serialize};

use crate::models::AddressSpent;

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum BlockchaincomResponse {
    RawTxn(BlockchaincomRawTxn),
    ApiError(BlockchaincomApiError),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BlockchaincomApiError {
    pub error: String,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BlockchaincomRawTxn {
    hash: String,
    ver: u64,
    vin_sz: u64,
    vout_sz: u64,
    size: u64,
    weight: u64,
    //Transaction fee in satoshis
    pub fee: u64,
    relayed_by: String,
    lock_time: u64,
    tx_index: u64,
    pub double_spend: bool,
    time: u64,
    pub block_index: Option<u64>,
    pub block_height: Option<u64>,
    pub inputs: Vec<Input>,
    out: Vec<Out>,
    pub rbf: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Input {
    sequence: u64,
    witness: String,
    script: String,
    index: u64,
    pub prev_out: Out,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Out {
    #[serde(rename = "type")]
    out_type: u64,
    pub spent: bool,
    pub value: u64,
    spending_outpoints: Vec<SpendingOutpoint>,
    n: u64,
    tx_index: u64,
    pub script: String,
    pub addr: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SpendingOutpoint {
    tx_index: u64,
    n: u64,
}

impl BlockchaincomRawTxn {
    pub fn get_input_txns(&self) -> Vec<AddressSpent> {
        self.inputs
            .iter()
            .map(|input| AddressSpent {
                address: input.prev_out.addr.clone().unwrap_or("Unknown".to_string()),
                amount: input.prev_out.value,
            })
            .collect::<Vec<AddressSpent>>()
    }

    pub fn get_output_txns(&self) -> Vec<AddressSpent> {
        self.out
            .iter()
            .map(|out| AddressSpent {
                address: out.addr.clone().unwrap_or("Unknown".to_string()),
                amount: out.value,
            })
            .collect::<Vec<AddressSpent>>()
    }

    pub fn get_total_input_amount(&self) -> u64 {
        self.inputs
            .iter()
            .map(|input| input.prev_out.value)
            .sum::<u64>()
    }

    pub fn get_total_output_amount(&self) -> u64 {
        self.out.iter().map(|out| out.value).sum::<u64>()
    }

    pub fn get_total_fee(&self) -> u64 {
        self.fee
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Ord, PartialOrd, PartialEq, Eq)]
pub struct BlockstreamUtxo {
    txid: String,
    vout: u32,
    status: Status,
    pub value: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Ord, Eq, PartialEq, PartialOrd)]
pub struct Status {
    confirmed: bool,
    block_height: Option<u64>,
    block_hash: Option<String>,
    block_time: Option<u64>,
}

impl BlockstreamUtxo {
    pub fn get_txid(&self) -> String {
        self.txid.clone()
    }

    pub fn get_vout(&self) -> u32 {
        self.vout
    }

    pub fn is_confirmed(&self) -> bool {
        self.status.confirmed
    }
}

#[derive(Serialize, Deserialize)]
pub struct BlockstreamWalletBalance {
    address: String,
    chain_stats: Stats,
    mempool_stats: Stats,
}

#[derive(Serialize, Deserialize)]
pub struct Stats {
    funded_txo_count: u64,
    funded_txo_sum: i64,
    spent_txo_count: u64,
    spent_txo_sum: i64,
    tx_count: u64,
}

impl BlockstreamWalletBalance {
    pub fn get_confirmed_balance(&self) -> i64 {
        self.chain_stats.funded_txo_sum - self.chain_stats.spent_txo_sum
    }

    pub fn get_unconfirmed_balance(&self) -> i64 {
        self.mempool_stats.funded_txo_sum - self.mempool_stats.spent_txo_sum
    }
}

#[cfg(test)]
mod tests {
    use super::*; // Import functions from parent module

    #[test]
    fn test_de_blockchaincom_raw_txn_response() {
        let json = r#"{
                            "hash": "69f8ab2bf2d82b3e5fd7626736d040d9c11d4ea3c31fb0c30bb0d72e8c5a6238",
                            "ver": 2,
                            "vin_sz": 3,
                            "vout_sz": 2,
                            "size": 590,
                            "weight": 1388,
                            "fee": 5220,
                            "relayed_by": "0.0.0.0",
                            "lock_time": 0,
                            "tx_index": 1983842466781942,
                            "double_spend": false,
                            "time": 1740082068,
                            "block_index": null,
                            "block_height": null,
                            "inputs": [
                                {
                                "sequence": 4294967293,
                                "witness": "0247304402205f0afddeed93c941c4285411763cb36fd94d1a66b206e886dddf6840c369151902202a1fb3c558f95a6edd7167fe2cc3b8f28f09239284efed30fa9875ca824f1f8701210368db5fff504f996f887c23200967e17eccdca3dd1956427f39513c410ea4c86f",
                                "script": "160014af5fcdda823022f56922022804997da4b01ae9d0",
                                "index": 0,
                                "prev_out": {
                                    "type": 0,
                                    "spent": true,
                                    "value": 104000,
                                    "spending_outpoints": [
                                    {
                                        "tx_index": 1983842466781942,
                                        "n": 0
                                    }
                                    ],
                                    "n": 19,
                                    "tx_index": 6450264201823941,
                                    "script": "a914cd2fd13bfc172b4684355643c32f0ffea44c8db887",
                                    "addr": "3LPwjGtU2gfY5kSAAj44Y62pjTFvAHp9L2"
                                }
                                },
                                {
                                "sequence": 4294967293,
                                "witness": "0248304502210083bee9952cb6bde948caec39f1b707d731a391a3cf3c6e380f9326165d9f734b02205fce66840866c6145c13e0045f895acfaaa7e6a72fd4836c8edd69de6b23f7a501210368db5fff504f996f887c23200967e17eccdca3dd1956427f39513c410ea4c86f",
                                "script": "160014af5fcdda823022f56922022804997da4b01ae9d0",
                                "index": 1,
                                "prev_out": {
                                    "type": 0,
                                    "spent": true,
                                    "value": 10443,
                                    "spending_outpoints": [
                                    {
                                        "tx_index": 1983842466781942,
                                        "n": 1
                                    }
                                    ],
                                    "n": 0,
                                    "tx_index": 7505855718760187,
                                    "script": "a914cd2fd13bfc172b4684355643c32f0ffea44c8db887",
                                    "addr": "3LPwjGtU2gfY5kSAAj44Y62pjTFvAHp9L2"
                                }
                                },
                                {
                                "sequence": 4294967293,
                                "witness": "0247304402206322d6c54231449bb72675e877ae39fa1af43f52411a80a7d03d9296b9d6c7b802202127c1b9f5a3e5eee33cd833de47a155caf58aebe7c461b9014f73a441cb705701210368db5fff504f996f887c23200967e17eccdca3dd1956427f39513c410ea4c86f",
                                "script": "160014af5fcdda823022f56922022804997da4b01ae9d0",
                                "index": 2,
                                "prev_out": {
                                    "type": 0,
                                    "spent": true,
                                    "value": 9054,
                                    "spending_outpoints": [
                                    {
                                        "tx_index": 1983842466781942,
                                        "n": 2
                                    }
                                    ],
                                    "n": 0,
                                    "tx_index": 6631934295867329,
                                    "script": "a914cd2fd13bfc172b4684355643c32f0ffea44c8db887",
                                    "addr": "3LPwjGtU2gfY5kSAAj44Y62pjTFvAHp9L2"
                                }
                                }
                            ],
                            "out": [
                                {
                                "type": 0,
                                "spent": false,
                                "value": 115000,
                                "spending_outpoints": [],
                                "n": 0,
                                "tx_index": 1983842466781942,
                                "script": "a91408368b78847d0f42552eea496044af9d3331b09f87",
                                "addr": "32SSfvCfRaSB8XzBLTHx8XHRxnZdJTBdVQ"
                                },
                                {
                                "type": 0,
                                "spent": false,
                                "value": 3277,
                                "spending_outpoints": [],
                                "n": 1,
                                "tx_index": 1983842466781942,
                                "script": "a914cd2fd13bfc172b4684355643c32f0ffea44c8db887",
                                "addr": "3LPwjGtU2gfY5kSAAj44Y62pjTFvAHp9L2"
                                }
                            ],
                            "rbf": true
        }"#;

        let deserialized_json = serde_json::from_str::<BlockchaincomRawTxn>(json);
        assert!(deserialized_json.is_ok()); // ✅ Passes
    }

    #[test]
    fn test_de_blockchaincom_response() {
        let json = r#"{
            "error": "not-found-or-invalid-arg",
            "message": "Item not found or argument invalid"
        }"#;

        let deserialized_json = serde_json::from_str::<BlockchaincomResponse>(json);
        assert!(deserialized_json.is_ok()); // ✅ Passes
    }

    #[test]
    fn test_de_blockstream_response() {
        let json = r#"
       [
            {
                "txid": "cf63765034a06d6afb13ff7bf7bd5c4a6959188cf167c85aa17bb22a4c4b33b2",
                "vout": 0,
                "status": {
                "confirmed": true,
                "block_height": 3659267,
                "block_hash": "00000000000000a54221360b8c9286bfeba1951e7bf3b47e2a5680d982a12c8e",
                "block_time": 1738199336
                },
                "value": 39649
            },
            {
                "txid": "d6db69946d2eece44bcda9e6beb2e859ad627662b53a917679b9ea8e70e1d60f",
                "vout": 0,
                "status": {
                "confirmed": false
                },
                "value": 177791842
            },
            {
                "txid": "a2a9afba41ea32a4c04e8984e84593796de447ac7b8f6caed9265ef332b21223",
                "vout": 0,
                "status": {
                "confirmed": false
                },
                "value": 179929342
            }
        ]
        "#;

        let deserialized_json = serde_json::from_str::<Vec<BlockstreamUtxo>>(json);
        assert!(deserialized_json.is_ok()); // Successfully deserialized response from blockstream ✅
        assert!(deserialized_json.unwrap().first().unwrap().is_confirmed());
    }
}
