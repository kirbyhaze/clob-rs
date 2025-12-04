```rust
use clob_rs::{ClobClient, OrderArgs, Side};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let private_key = "0x..."; // your wallet private key

    let mut client = ClobClient::polygon()
        .with_signer(private_key)?;
    println!("Address: {}", client.address().unwrap());

    // Create or derive API key (L1 auth)
    let creds = client.create_or_derive_api_key(None).await?;
    println!("API Key: {}", creds.api_key);

    // Create a signed order
    let token_id = "71321045679252212594626385532706912750332728571942532289631379312455583992563";
    let order_args = OrderArgs::new(token_id.to_string(), 0.50, 10.0, Side::Buy);
    let signed_order = client.create_order(&order_args, None).await?;

    Ok(())
}
```
