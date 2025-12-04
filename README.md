# clob-rs

Rust client for the Polymarket CLOB API.

## Usage

### L0 - Public Endpoints (No Auth)

```rust
use clob_rs::{BookParams, ClobClient, Side};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = ClobClient::polygon();
    let token_id = "71321045679252212594626385532706912750332728571942532289631379312455583992563";

    // health check
    client.get_ok().await?;
    let time = client.get_server_time().await?;

    // order book
    let book = client.get_order_book(token_id).await?;
    println!("{:?} {:?}", book.best_bid(), book.best_ask());

    // prices
    let mid = client.get_midpoint(token_id).await?;
    let price = client.get_price(token_id, Side::Buy).await?;
    let spread = client.get_spread(token_id).await?;
    let last = client.get_last_trade_price(token_id).await?;

    // market info (cached)
    let tick_size = client.get_tick_size(token_id).await?;
    let neg_risk = client.get_neg_risk(token_id).await?;
    let fee_bps = client.get_fee_rate_bps(token_id).await?;

    // batch requests
    let params = vec![BookParams::new(token_id)];
    let books = client.get_order_books(&params).await?;
    let mids = client.get_midpoints(&params).await?;
    let spreads = client.get_spreads(&params).await?;
    let lasts = client.get_last_trades_prices(&params).await?;

    let params = vec![BookParams::with_side(token_id, Side::Buy)];
    let prices = client.get_prices(&params).await?;

    // markets
    let page = client.get_markets_page(None).await?;
    let all = client.get_markets().await?; // paginate all
    let market = client.get_market("0x123...").await?;
    let trades = client.get_market_trades_events("0x123...").await?;

    // simplified/sampling variants
    let simplified = client.get_simplified_markets_page(None).await?;
    let sampling = client.get_sampling_markets_page(None).await?;

    Ok(())
}
```

### L1 - Wallet Auth (Create/Derive API Keys, Sign Orders)

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

### L2 - Full Auth (Trading, Order Management)

```rust
use clob_rs::{ClobClient, OrderArgs, OrderType, Side};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let private_key = "0x...";

    // Create client with signer, then add API credentials
    let mut client = ClobClient::polygon()
        .with_signer(private_key)?;

    // Get API credentials
    let creds = client.derive_api_key(None).await?;
    client.set_creds(creds);

    // Now you have L2 auth for trading
    let token_id = "71321045679252212594626385532706912750332728571942532289631379312455583992563";

    // Create and post an order
    let order_args = OrderArgs::new(token_id.to_string(), 0.50, 10.0, Side::Buy);
    let signed_order = client.create_order(&order_args, None).await?;
    let result = client.post_order(&signed_order, OrderType::GTC).await?;

    // Get your orders
    let orders = client.get_orders(None).await?;
    let order = client.get_order("order-id").await?;

    // Get your trades
    let trades = client.get_trades(None).await?;

    // Cancel orders
    client.cancel("order-id").await?;
    client.cancel_orders(&["id1".to_string(), "id2".to_string()]).await?;
    client.cancel_all().await?;

    // API key management
    let keys = client.get_api_keys().await?;
    client.delete_api_key().await?;

    Ok(())
}
```
