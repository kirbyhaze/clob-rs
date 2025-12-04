use clob_rs::headers::{create_level_1_headers, create_level_2_headers};
use clob_rs::{ApiCreds, Signer};

const TEST_PRIVATE_KEY: &str =
    "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
const EXPECTED_ADDRESS: &str = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266";
const POLYGON_CHAIN_ID: u64 = 137;

fn get_test_creds() -> ApiCreds {
    ApiCreds {
        api_key: "000000000-0000-0000-0000-000000000000".to_string(),
        api_passphrase: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            .to_string(),
        api_secret: "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=".to_string(),
    }
}

#[tokio::test]
async fn test_create_level_1_headers_no_nonce() {
    let signer = Signer::new(TEST_PRIVATE_KEY, POLYGON_CHAIN_ID).unwrap();
    let headers = create_level_1_headers(&signer, None).await.unwrap();

    assert!(!headers.address.is_empty());
    assert_eq!(headers.address, EXPECTED_ADDRESS);
    assert!(!headers.signature.is_empty());
    assert!(!headers.timestamp.is_empty());
    assert!(headers.timestamp.parse::<u64>().is_ok());
    assert_eq!(headers.nonce, "0");
}

#[tokio::test]
async fn test_create_level_1_headers_with_nonce() {
    let signer = Signer::new(TEST_PRIVATE_KEY, POLYGON_CHAIN_ID).unwrap();
    let headers = create_level_1_headers(&signer, Some(1012)).await.unwrap();

    assert!(!headers.address.is_empty());
    assert_eq!(headers.address, EXPECTED_ADDRESS);
    assert!(!headers.signature.is_empty());
    assert!(!headers.timestamp.is_empty());
    assert!(headers.timestamp.parse::<u64>().is_ok());
    assert_eq!(headers.nonce, "1012");
}

#[test]
fn test_create_level_2_headers_no_body() {
    let signer = Signer::new(TEST_PRIVATE_KEY, POLYGON_CHAIN_ID).unwrap();
    let creds = get_test_creds();

    let headers = create_level_2_headers(&signer, &creds, "GET", "/order", None);

    assert!(!headers.address.is_empty());
    assert_eq!(headers.address, EXPECTED_ADDRESS);
    assert!(!headers.signature.is_empty());
    assert!(!headers.timestamp.is_empty());
    assert!(headers.timestamp.parse::<u64>().is_ok());
    assert_eq!(headers.api_key, creds.api_key);
    assert_eq!(headers.passphrase, creds.api_passphrase);
}

#[test]
fn test_create_level_2_headers_with_body() {
    let signer = Signer::new(TEST_PRIVATE_KEY, POLYGON_CHAIN_ID).unwrap();
    let creds = get_test_creds();

    let headers =
        create_level_2_headers(&signer, &creds, "POST", "/order", Some(r#"{"hash": "0x123"}"#));

    assert!(!headers.address.is_empty());
    assert_eq!(headers.address, EXPECTED_ADDRESS);
    assert!(!headers.signature.is_empty());
    assert!(!headers.timestamp.is_empty());
    assert!(headers.timestamp.parse::<u64>().is_ok());
    assert_eq!(headers.api_key, creds.api_key);
    assert_eq!(headers.passphrase, creds.api_passphrase);
}
