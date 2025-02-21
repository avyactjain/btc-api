// Type of response from blockchain.info/rawtx/<transaction_hash>

use serde::{Deserialize, Serialize};

use crate::models::AddressSpent;

#[derive(Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
pub struct Input {
    sequence: u64,
    witness: String,
    script: String,
    index: u64,
    pub prev_out: Out,
}

#[derive(Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
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
}
