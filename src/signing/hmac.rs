use base64::{engine::general_purpose::URL_SAFE, Engine};
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

pub fn build_hmac_signature(
    secret: &str,
    timestamp: u64,
    method: &str,
    request_path: &str,
    body: Option<&str>,
) -> String {
    let decoded_secret = URL_SAFE.decode(secret).expect("invalid base64 secret");

    let mut message = format!("{}{}{}", timestamp, method, request_path);
    if let Some(b) = body {
        message.push_str(b);
    }

    let mut mac =
        HmacSha256::new_from_slice(&decoded_secret).expect("HMAC can take key of any size");
    mac.update(message.as_bytes());

    let result = mac.finalize();
    URL_SAFE.encode(result.into_bytes())
}
