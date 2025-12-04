// https://github.com/Polymarket/py-clob-client/blob/main/py_clob_client/headers/headers.py

use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::Result;
use crate::signer::Signer;
use crate::signing::{build_hmac_signature, sign_clob_auth_message};
use crate::types::ApiCreds;

pub const POLY_ADDRESS: &str = "POLY_ADDRESS";
pub const POLY_SIGNATURE: &str = "POLY_SIGNATURE";
pub const POLY_TIMESTAMP: &str = "POLY_TIMESTAMP";
pub const POLY_NONCE: &str = "POLY_NONCE";
pub const POLY_API_KEY: &str = "POLY_API_KEY";
pub const POLY_PASSPHRASE: &str = "POLY_PASSPHRASE";

pub struct L1Headers {
    pub address: String,
    pub signature: String,
    pub timestamp: String,
    pub nonce: String,
}

pub struct L2Headers {
    pub address: String,
    pub signature: String,
    pub timestamp: String,
    pub api_key: String,
    pub passphrase: String,
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time went backwards")
        .as_secs()
}

pub async fn create_level_1_headers(signer: &Signer, nonce: Option<u64>) -> Result<L1Headers> {
    let timestamp = current_timestamp();
    let n = nonce.unwrap_or(0);
    let signature = sign_clob_auth_message(signer, timestamp, n).await?;

    Ok(L1Headers {
        address: signer.address_string(),
        signature,
        timestamp: timestamp.to_string(),
        nonce: n.to_string(),
    })
}

pub fn create_level_2_headers(
    signer: &Signer,
    creds: &ApiCreds,
    method: &str,
    request_path: &str,
    body: Option<&str>,
) -> L2Headers {
    let timestamp = current_timestamp();
    let signature = build_hmac_signature(&creds.api_secret, timestamp, method, request_path, body);

    L2Headers {
        address: signer.address_string(),
        signature,
        timestamp: timestamp.to_string(),
        api_key: creds.api_key.clone(),
        passphrase: creds.api_passphrase.clone(),
    }
}
