use axum::{
    extract::{Query, State},
    Json,
};
use tracing::{debug, error};

use crate::{
    blockchains::btc_api_state::BtcApiState,
    chain::Chain,
    models::{
        BroadcastTransactionParams, BroadcastTransactionResponse, CreateTransactionParams,
        CreateTransactionResponse, MethodNotAllowedResponse, NetworkFeeResponse,
        ValidateTransactionHashParams, ValidateTransactionHashResponse, WalletBalanceParams,
        WalletBalanceResponse,
    },
};

pub(crate) async fn method_not_allowed_handler<T: Chain>(
    State(_): State<BtcApiState<T>>,
) -> Json<MethodNotAllowedResponse> {
    error!("Method not allowed");
    Json(MethodNotAllowedResponse {
        is_error: true,
        error_msg: "Method not allowed".to_string(),
    })
}

pub(crate) async fn bitcoin_network_fee_handler<T: Chain>(
    State(blockchain): State<BtcApiState<T>>,
) -> Json<NetworkFeeResponse> {
    debug!("Received request to get network fee");
    Json(blockchain.get_network_fee().await)
}

pub(crate) async fn bitcoin_validate_transaction_hash_handler<T: Chain>(
    Query(params): Query<ValidateTransactionHashParams>,
    State(blockchain): State<BtcApiState<T>>,
) -> Json<ValidateTransactionHashResponse> {
    debug!(
        "Received request to validate transaction hash: {:#?}",
        params
    );

    Json(
        blockchain
            .validate_transaction_hash(params.transaction_hash)
            .await,
    )
}

pub(crate) async fn bitcoin_wallet_balance_handler<T: Chain>(
    Query(params): Query<WalletBalanceParams>,
    State(blockchain): State<BtcApiState<T>>,
) -> Json<WalletBalanceResponse> {
    debug!("Received request to fetch wallet balance: {:#?}", params);

    Json(blockchain.get_wallet_balance(params.wallet_address).await)
}

pub(crate) async fn bitcoin_create_transaction_handler<T: Chain>(
    State(blockchain): State<BtcApiState<T>>,
    Json(params): Json<CreateTransactionParams>,
) -> Json<CreateTransactionResponse> {
    debug!("Received request to create transaction: {:#?}", params);

    match params.validate() {
        Ok(params) => Json(blockchain.create_transaction(params).await),
        Err(e) => Json(CreateTransactionResponse {
            is_error: true,
            data: None,
            error_msg: Some(e.to_string()),
        }),
    }
}

pub(crate) async fn bitcoin_broadcast_transaction_handler<T: Chain>(
    State(blockchain): State<BtcApiState<T>>,
    Json(params): Json<BroadcastTransactionParams>,
) -> Json<BroadcastTransactionResponse> {
    debug!("Received request to broadcast transaction: {:#?}", params);

    Json(blockchain.broadcast_transaction(params).await)
}

#[cfg(test)]
mod tests {
    use http_body_util::BodyExt;

    use super::*;

    // To Satisfy Axum
    impl Clone for crate::chain::MockChain {
        fn clone(&self) -> Self {
            crate::chain::MockChain::new()
        }
    }

    /// Tests the network fee handler by mocking a Bitcoin chain instance and verifying
    /// that it correctly returns the expected fee data structure.
    #[tokio::test]
    async fn test_network_fee_handler() {
        use crate::chain::MockChain;
        use crate::models::{NetworkFeeResponse, NetworkFeeResponseData};
        use axum::routing::get;
        use axum::{
            body::Body,
            http::{Request, StatusCode},
            Router,
        };
        use tower::ServiceExt; // for `oneshot`

        let expected_network_fee_response = NetworkFeeResponse {
            is_error: false,
            data: Some(NetworkFeeResponseData {
                fastest_fee: 100,
                half_hour_fee: 90,
                hour_fee: 80,
                economy_fee: 70,
                minimum_fee: 60,
            }),
            error_msg: None,
        };

        let expected_network_fee_response_clone = expected_network_fee_response.clone();

        let mut mock_bitcoin = MockChain::new();
        mock_bitcoin.expect_get_network_fee().returning(move || {
            let expected_network_fee_response = expected_network_fee_response.clone();
            Box::pin(async { expected_network_fee_response })
        });

        let app = Router::new()
            .route("/networkFee", get(bitcoin_network_fee_handler))
            .with_state(BtcApiState::new(mock_bitcoin));

        let request = Request::builder()
            .uri("/networkFee")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap();
        let bytes = body.to_bytes();
        let body_str = String::from_utf8(bytes.to_vec()).unwrap();

        let de_body: NetworkFeeResponse = serde_json::from_str(&body_str).unwrap();

        assert_eq!(de_body, expected_network_fee_response_clone)
    }
}
