use clob_rs::{
    ApiCreds, ClobClient, CreateOrderOptions, OrderArgs, OrderBuilder, OrderType, Side, Signer,
    TickSize,
};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const TEST_PRIVATE_KEY: &str =
    "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
const POLYGON_CHAIN_ID: u64 = 137;

fn create_test_signer() -> Signer {
    Signer::new(TEST_PRIVATE_KEY, POLYGON_CHAIN_ID).unwrap()
}

fn create_test_creds() -> ApiCreds {
    ApiCreds {
        api_key: "test-api-key".to_string(),
        api_secret: "dGVzdC1hcGktc2VjcmV0".to_string(), // base64 encoded "test-api-secret"
        api_passphrase: "test-passphrase".to_string(),
    }
}

async fn create_signed_order() -> clob_rs::SignedOrder {
    let signer = create_test_signer();
    let builder = OrderBuilder::new(signer);

    let order_args = OrderArgs::new("123456", 0.5, 100.0, Side::Buy);
    let options = CreateOrderOptions {
        tick_size: TickSize::Size0_01,
        neg_risk: false,
    };

    builder.create_order(&order_args, &options).await.unwrap()
}

#[tokio::test]
async fn test_post_order_success() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/order"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true,
            "errorMsg": "",
            "orderID": "0x1234567890abcdef",
            "makingAmount": "50000000",
            "takingAmount": "100000000",
            "status": "LIVE",
            "order_hashes": ["0xabc123"]
        })))
        .mount(&mock_server)
        .await;

    let client = ClobClient::new(&mock_server.uri())
        .with_signer(TEST_PRIVATE_KEY)
        .unwrap()
        .with_creds(create_test_creds());

    let signed_order = create_signed_order().await;
    let result = client.post_order(&signed_order, OrderType::GTC).await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.success);
    assert_eq!(response.order_id, "0x1234567890abcdef");
    assert_eq!(response.status, "LIVE");
    assert_eq!(response.making_amount, "50000000");
    assert_eq!(response.taking_amount, "100000000");
    assert_eq!(response.order_hashes, vec!["0xabc123"]);
}

#[tokio::test]
async fn test_post_order_error_response() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/order"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": false,
            "errorMsg": "Insufficient balance",
            "orderID": "",
            "makingAmount": "0",
            "takingAmount": "0",
            "status": "",
            "order_hashes": []
        })))
        .mount(&mock_server)
        .await;

    let client = ClobClient::new(&mock_server.uri())
        .with_signer(TEST_PRIVATE_KEY)
        .unwrap()
        .with_creds(create_test_creds());

    let signed_order = create_signed_order().await;
    let result = client.post_order(&signed_order, OrderType::GTC).await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(!response.success);
    assert_eq!(response.error_msg, "Insufficient balance");
}

#[tokio::test]
async fn test_post_order_fok() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/order"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true,
            "errorMsg": "",
            "orderID": "0xfok-order-id",
            "makingAmount": "50000000",
            "takingAmount": "100000000",
            "status": "MATCHED",
            "order_hashes": []
        })))
        .mount(&mock_server)
        .await;

    let client = ClobClient::new(&mock_server.uri())
        .with_signer(TEST_PRIVATE_KEY)
        .unwrap()
        .with_creds(create_test_creds());

    let signed_order = create_signed_order().await;
    let result = client.post_order(&signed_order, OrderType::FOK).await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.success);
    assert_eq!(response.order_id, "0xfok-order-id");
    assert_eq!(response.status, "MATCHED");
}

#[tokio::test]
async fn test_post_order_http_error() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/order"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&mock_server)
        .await;

    let client = ClobClient::new(&mock_server.uri())
        .with_signer(TEST_PRIVATE_KEY)
        .unwrap()
        .with_creds(create_test_creds());

    let signed_order = create_signed_order().await;
    let result = client.post_order(&signed_order, OrderType::GTC).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_post_order_requires_credentials() {
    let client = ClobClient::new("http://localhost:8080")
        .with_signer(TEST_PRIVATE_KEY)
        .unwrap();

    let signed_order = create_signed_order().await;
    let result = client.post_order(&signed_order, OrderType::GTC).await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("credentials"));
}
