use axum::{routing::get, Router};
use blockchains::bitcoin::Bitcoin;
use blockchains::blockchain_wrapper::BlockchainWrapper;
use btc_api_error::BtcApiError;
use chain::ChainName;
use handlers::{bitcoin_network_fee_handler, bitcoin_validate_transaction_hash_handler};

use tracing::{info, Level};
mod blockchains;
mod btc_api_error;
mod chain;
mod config;
mod handlers;
mod models;

#[tokio::main]
async fn main() -> Result<(), BtcApiError> {
    // Initialize tracing
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    // Load config
    let config = config::Config::load()?;

    // Create shared state of the blockchain instance
    let blockchain = match config.chain_config.chain {
        //Should inject the required config into the blockchain instance here.
        ChainName::Bitcoin => BlockchainWrapper::new(Bitcoin::new(config.chain_config.rpc_url)),
    };

    let app = Router::new()
        .route("/networkFee", get(bitcoin_network_fee_handler))
        .route(
            "/validateTransactionHash",
            get(bitcoin_validate_transaction_hash_handler),
        )
        .with_state(blockchain);

    let addr = config.listen_address;
    info!("BTC-API listening on http://{:?}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");

    axum::serve(listener, app)
        .await
        .expect("Unable to serve the app");
    Ok(())
}
