# BTC-API

A REST API service for interacting with Bitcoin and other blockchain networks.

## Features

- Network fee estimation endpoint
- Configurable blockchain support
- Async runtime with Tokio
- Error handling with custom error types
- Tracing for logging and diagnostics

## Getting Started

### Prerequisites

- Rust (latest stable version)
- Cargo

### Docker (Don't modify `container-config.json` file)

1. Build the Docker image:
    ```bash
    docker build --no-cache -t btc-api .
    ```

2. Run the Docker container:
    ```bash
    docker run -d -p 3005:3005 btc-api
    ```

3. Access the API:
    ```bash
    curl http://localhost:3005/networkFee
    ```

### Installation

1. Clone the repository:        
    ```bash
    git clone https://github.com/yourusername/btc-api.git
    cd btc-api
    ```

2. Build the project:
    ```bash
    cargo build --release
    ```

3. Run the project:
    ```bash
    cargo run --release 
    ```

4. Access the API:
    ```bash
    curl http://localhost:3002/api/v1/network-fee
    ``` 

## API Endpoints

### Get Network Fee

    ```bash
    curl http://localhost:3005/networkFee
    ``` 

    Response:
    ```json
    {
        "isError": false,
        "data": {
            "fastestFee": 1000,
            "halfHourFee": 500,
            "hourFee": 250,
            "economyFee": 100,
            "minimumFee": 1
        }
    }
    ```

    Error Response:
    ```json
    {
        "isError": true,
        "errorMsg": "Error message"
    }
    ```                 

### Validate Transaction Hash

    ```bash
    curl "http://localhost:3005/validateTransactionHash?transaction_hash=69f8ab2bf2d82b3e5fd7626736d040d9c11d4ea3c31fb0c30bb0d72e8c5a6238"
    ```  

      Response:
    ```json
   {
    "isError": false,
    "data": {
        "txnHash": "69f8ab2bf2d82b3e5fd7626736d040d9c11d4ea3c31fb0c30bb0d72e8c5a6238",
        "txnStatus": "cancelled",
        "txnData": {
            "block_index": null,
            "block_height": null,
            "consumed_fees_in_satoshis": 5220,
            "txn_input_amount_in_satoshis": 123497,
            "txn_output_amount_in_satoshis": 118277,
            "input_txns": [
                {
                    "address": "3LPwjGtU2gfY5kSAAj44Y62pjTFvAHp9L2",
                    "amount": 104000
                },
                {
                    "address": "3LPwjGtU2gfY5kSAAj44Y62pjTFvAHp9L2",
                    "amount": 10443
                },
                {
                    "address": "3LPwjGtU2gfY5kSAAj44Y62pjTFvAHp9L2",
                    "amount": 9054
                }
            ],
            "output_txns": [
                {
                    "address": "32SSfvCfRaSB8XzBLTHx8XHRxnZdJTBdVQ",
                    "amount": 115000
                },
                {
                    "address": "3LPwjGtU2gfY5kSAAj44Y62pjTFvAHp9L2",
                    "amount": 3277
                }
            ]
        }
    },
    "errorMsg": null
    }
    ```

    Error Response:
    ```json
   {
    "isError": true,
    "data": null,
    "errorMsg": "SerdeJsonError: missing field `hash` at line 1 column 83"
    }   
    ```                 


## Configuration

The API is configured using a JSON file. The configuration file is located in the `src/config` directory.

The configuration file is as follows:

```json
{
    "listenAddress": "127.0.0.1:3002",
    "blockchainConfig": {
        "chain": "bitcoin",
        "chainId": 1,
        "rpcUrl": "http://127.0.0.1:8332"
    }
}
```

## Error Handling

The API uses custom error types to handle errors. The error types are defined in the `src/error.rs` file.

The error types are as follows:

```rust
pub enum BtcApiError {
    ConfigLoadError(String),
}
```
