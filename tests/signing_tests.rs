use clob_rs::signing::eip712::sign_clob_auth_message;
use clob_rs::Signer;

const TEST_PRIVATE_KEY: &str =
    "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
const EXPECTED_ADDRESS: &str = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266";
const POLYGON_CHAIN_ID: u64 = 137;

#[test]
fn test_signer_address() {
    let signer = Signer::new(TEST_PRIVATE_KEY, POLYGON_CHAIN_ID).unwrap();
    assert_eq!(signer.address_string(), EXPECTED_ADDRESS);
}

#[tokio::test]
async fn test_sign_clob_auth_message() {
    let signer = Signer::new(TEST_PRIVATE_KEY, POLYGON_CHAIN_ID).unwrap();
    let timestamp = 1000000u64;
    let nonce = 0u64;

    let signature = sign_clob_auth_message(&signer, timestamp, nonce)
        .await
        .unwrap();

    assert!(signature.starts_with("0x"));
    assert_eq!(signature.len(), 132);

    let signature2 = sign_clob_auth_message(&signer, timestamp, nonce)
        .await
        .unwrap();
    assert_eq!(signature, signature2);
}

#[tokio::test]
async fn test_sign_clob_auth_message_with_nonce() {
    let signer = Signer::new(TEST_PRIVATE_KEY, POLYGON_CHAIN_ID).unwrap();
    let timestamp = 1000000u64;
    let nonce = 1u64;

    let signature = sign_clob_auth_message(&signer, timestamp, nonce)
        .await
        .unwrap();
    assert!(signature.starts_with("0x"));
    assert_eq!(signature.len(), 132); // 0x + 65 bytes * 2
}

#[tokio::test]
async fn test_sign_clob_auth_message_different_timestamps() {
    let signer = Signer::new(TEST_PRIVATE_KEY, POLYGON_CHAIN_ID).unwrap();

    let sig1 = sign_clob_auth_message(&signer, 1000000, 0).await.unwrap();
    let sig2 = sign_clob_auth_message(&signer, 2000000, 0).await.unwrap();

    assert_ne!(sig1, sig2);
}

#[tokio::test]
async fn test_sign_clob_auth_message_different_nonces() {
    let signer = Signer::new(TEST_PRIVATE_KEY, POLYGON_CHAIN_ID).unwrap();

    let sig1 = sign_clob_auth_message(&signer, 1000000, 0).await.unwrap();
    let sig2 = sign_clob_auth_message(&signer, 1000000, 1).await.unwrap();

    assert_ne!(sig1, sig2);
}
