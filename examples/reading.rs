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

