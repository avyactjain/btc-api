# BTC-API

A REST API service for interacting with Bitcoin blockchain network.

## Features

- Fetching wallet balances
- Estimating network fees
- Validating transactions
- Creating new transactions
- Broadcasting new transactions 
- Error handling with custom error types
- Tracing for logging and diagnostics
- Swagger OpenApi documentation out of the box for testing

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
    curl http://127.0.0.1:3002/network-fee
    ``` 

## API Endpoints

Refer to the Swagger OpenApi documentation hosted at `http://127.0.0.1:3002/docs`

## Configuration

The API is configured using a JSON file. The configuration file is located in the `src/config` directory.

The configuration file is as follows:

```json
{
    "listen_address": "127.0.0.1:3002",
    "chain_config": {
        "chain": "bitcoin",
        "variant": "Testnet",
        "rpc_url": "https://blockstream.info/testnet/api/"
    },
    "rust_log_level": "info", 
    "sign_txn": true
}
```

| Config Field  | Meaning  | Possible value(s)|
|----------|---------|------------|
| listen_address   | ✅ The address on which the API will be listening.  | 127.0.0.1:3002 |
| chain   | The related chain. Only bitcoin is supported as of now | bitcoin  |
| variant   | Vairant of the chain | mainnet, testnet  |
| rpc_url   | The RPC URL of the underlying chain. only blockstream is supported as of now. | https://blockstream.info/testnet/api/
| sign_txn   | Whether to sign the txn or not using the wallet defined in `src/blockchains/bitcoin/utils.rs` | 



## Error Handling

The API uses custom error types to handle errors. The error types are defined in the `src/error.rs` file.

For suggestions/feedbacks, contact : https://t.me/avyact_jain
