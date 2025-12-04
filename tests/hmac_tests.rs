use clob_rs::signing::hmac::build_hmac_signature;

#[test]
fn test_build_hmac_signature() {
    let signature = build_hmac_signature(
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=",
        1000000,
        "test-sign",
        "/orders",
        Some(r#"{"hash": "0x123"}"#),
    );
    assert!(!signature.is_empty());
    assert_eq!(signature, "ZwAdJKvoYRlEKDkNMwd5BuwNNtg93kNaR_oU2HrfVvc=");
}

#[test]
fn test_build_hmac_signature_no_body() {
    let signature = build_hmac_signature(
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=",
        1000000,
        "GET",
        "/orders",
        None,
    );
    assert!(!signature.is_empty());
}
