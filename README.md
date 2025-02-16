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
    curl http://localhost:3002/api/v1/network-fee
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
