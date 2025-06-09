use axum::{
    routing::{get, post},
    Router,
};
use blockchains::bitcoin::Bitcoin;
use btc_api_error::BtcApiError;
use chain::ChainName;
use handlers::{
    bitcoin_broadcast_transaction_handler, bitcoin_create_transaction_handler,
    bitcoin_network_fee_handler, bitcoin_validate_transaction_hash_handler,
    bitcoin_wallet_balance_handler, method_not_allowed_handler,
};

use tower_http::services::{ServeDir, ServeFile};
use tracing::info;

use crate::blockchains::btc_api_state::BtcApiState;
mod blockchains;
mod btc_api_error;
mod chain;
mod config;
mod handlers;
mod models;

#[tokio::main]
async fn main() -> Result<(), BtcApiError> {
    // Load config
    let config = config::Config::load()?;

    // 1. Serve OpenAPI JSON
    let openapi_json_path = "openapi/openapi.json";
    let openapi_service = ServeFile::new(openapi_json_path);

    // 2. Serve Swagger UI - use a relative path instead of absolute
    let swagger_ui_dir = "swagger-ui";
    let swagger_ui_service = ServeDir::new(swagger_ui_dir)
        .fallback(ServeFile::new(format!("{}/index.html", swagger_ui_dir)));

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(config.rust_log_level)
        .init();

    // Create shared state of the blockchain instance
    let blockchain = match config.chain_config.chain {
        //Should inject the required config into the blockchain instance here.
        ChainName::Bitcoin => BtcApiState::new(Bitcoin::new(
            &config.chain_config.rpc_url,
            &config.chain_config.variant,
            config.sign_txn,
        )?),
    };

    let app = Router::new()
        .route("/networkFee", get(bitcoin_network_fee_handler))
        .route(
            "/validateTransactionHash",
            get(bitcoin_validate_transaction_hash_handler),
        )
        .route(
            "/createTransaction",
            post(bitcoin_create_transaction_handler),
        )
        .route(
            "/broadcastTransaction",
            post(bitcoin_broadcast_transaction_handler),
        )
        .route("/walletBalance", get(bitcoin_wallet_balance_handler))
        .route_service("/docs/openapi.json", openapi_service) // Serve JSON file
        .nest_service("/docs", swagger_ui_service) // Serve Swagger UI
        .method_not_allowed_fallback(method_not_allowed_handler)
        .with_state(blockchain);

    let addr = config.listen_address;

    info!(
        "BTC-API listening on http://{:?} with config: {:#?}",
        addr, config
    );

    info!("Swagger OpenApi docs available on http://{:?}/docs", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");

    axum::serve(listener, app)
        .await
        .expect("Unable to serve the app");
    Ok(())
}
