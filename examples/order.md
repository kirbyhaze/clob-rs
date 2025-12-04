```rust
use clob_rs::{
    ApiCreds, ClobClient, OrderArgs, OrderType, Side, POLY_PROXY,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token_id = "45343480653694577807177505914664405669209636932459044719445554137639656106379";

    dotenvy::dotenv().ok();
    let proxy_wallet = std::env::var("proxy_wallet").expect("load priv");
    let private_key = std::env::var("private_key").expect("load priv");
    let api_key = std::env::var("api_key").expect("load api key");
    let api_secret = std::env::var("api_secret").expect("load secret");
    let api_passphrase = std::env::var("api_passphrase").expect("load pass");

    let mut client = ClobClient::polygon()
        .with_signer(&private_key)?
        .with_funder(&proxy_wallet)?
        .with_signature_type(POLY_PROXY)
        .with_creds(ApiCreds {
            api_key: api_key.clone(),
            api_secret,
            api_passphrase,
        });

    let order_args = OrderArgs::new(token_id, 0.01, 100.0, Side::Buy);
    let signed_order = client.create_order(&order_args, None).await?;
    match client.post_order(&signed_order, OrderType::GTC).await {
        Ok(response) => println!("Order posted: {}", response),
        Err(e) => println!("Error posting order: {}", e),
    }

    Ok(())
}
```
