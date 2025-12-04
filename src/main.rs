use clob_rs::{BookParams, ClobClient, Side};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a client for Polygon mainnet
    let mut client = ClobClient::polygon();

    // Health check
    println!("Checking server health...");
    let ok = client.get_ok().await?;
    println!("Server OK: {:?}\n", ok);

    // Get server time
    let time = client.get_server_time().await?;
    println!("Server time: {}\n", time);

    // Example token ID (replace with a real one for testing)
    let token_id = "74018646712472971445258547247048869505144598783748525202442089895996249694683";

    // Get order book
    println!("Fetching order book for token: {}...", &token_id[..20]);
    match client.get_order_book(token_id).await {
        Ok(book) => {
            println!("Order Book:");
            println!("  Market: {}", book.market);
            println!("  Timestamp: {}", book.timestamp);
            if let Some(bid) = book.best_bid() {
                println!("  Best Bid: {} @ {}", bid.price, bid.size);
            }
            if let Some(ask) = book.best_ask() {
                println!("  Best Ask: {} @ {}", ask.price, ask.size);
            }
            if let Some(spread) = book.spread() {
                println!("  Spread: {:.4}", spread);
            }
            if let Some(mid) = book.midpoint() {
                println!("  Midpoint: {:.4}", mid);
            }
            println!("  Bid levels: {}", book.bids.len());
            println!("  Ask levels: {}", book.asks.len());
        }
        Err(e) => println!("  Error: {}", e),
    }
    println!();

    // Get midpoint price
    println!("Fetching midpoint...");
    match client.get_midpoint(token_id).await {
        Ok(mid) => println!("  Midpoint: {}\n", mid),
        Err(e) => println!("  Error: {}\n", e),
    }

    // Get price for BUY side
    println!("Fetching BUY price...");
    match client.get_price(token_id, Side::Buy).await {
        Ok(price) => println!("  Buy price: {}\n", price),
        Err(e) => println!("  Error: {}\n", e),
    }

    // Get spread
    println!("Fetching spread...");
    match client.get_spread(token_id).await {
        Ok(spread) => println!("  Spread: {}\n", spread),
        Err(e) => println!("  Error: {}\n", e),
    }

    // Get tick size (cached after first call)
    println!("Fetching tick size...");
    match client.get_tick_size(token_id).await {
        Ok(tick_size) => println!("  Tick size: {}\n", tick_size),
        Err(e) => println!("  Error: {}\n", e),
    }

    // Get neg risk flag
    println!("Fetching neg risk...");
    match client.get_neg_risk(token_id).await {
        Ok(neg_risk) => println!("  Neg risk: {}\n", neg_risk),
        Err(e) => println!("  Error: {}\n", e),
    }

    // Batch request example: get midpoints for multiple tokens
    println!("Fetching batch midpoints...");
    let params = vec![BookParams::new(token_id)];
    match client.get_midpoints(&params).await {
        Ok(mids) => {
            for mid in mids {
                println!("  Token {}...: {:?}", &mid.token_id[..20], mid.mid);
            }
        }
        Err(e) => println!("  Error: {}", e),
    }
    println!();

    // Batch prices
    println!("Fetching batch prices...");
    let params = vec![BookParams::with_side(token_id, Side::Buy)];
    match client.get_prices(&params).await {
        Ok(prices) => {
            for p in prices {
                println!("  Token {}...: BUY={:?}, SELL={:?}", &p.token_id[..20], p.buy, p.sell);
            }
        }
        Err(e) => println!("  Error: {}", e),
    }
    println!();

    // Batch spreads
    println!("Fetching batch spreads...");
    let params = vec![BookParams::new(token_id)];
    match client.get_spreads(&params).await {
        Ok(spreads) => {
            for s in spreads {
                println!("  Token {}...: {:?}", &s.token_id[..20], s.spread);
            }
        }
        Err(e) => println!("  Error: {}", e),
    }
    println!();

    // Last trades prices
    println!("Fetching last trades prices...");
    let params = vec![BookParams::new(token_id)];
    match client.get_last_trades_prices(&params).await {
        Ok(prices) => {
            for p in prices {
                println!("  Token {}...: {} @ {}", &p.token_id[..20], p.side, p.price);
            }
        }
        Err(e) => println!("  Error: {}", e),
    }
    println!();

    // Get first page of markets
    println!("Fetching markets (first page)...");
    match client.get_markets_page(None).await {
        Ok(resp) => {
            println!("  Found {} markets", resp.data.len());
            println!("  Next cursor: {}", resp.next_cursor);
            if let Some(market) = resp.data.first() {
                println!("  First market:");
                println!("    Condition ID: {}...", &market.condition_id[..20]);
                println!("    Question: {:?}", market.question.as_ref().map(|s| if s.len() > 50 { format!("{}...", &s[..50]) } else { s.clone() }));
                println!("    Tokens: {}", market.tokens.len());
                println!("    Active: {}, Closed: {}", market.active, market.closed);
                println!("    Tick size: {}", market.minimum_tick_size);
            }
        }
        Err(e) => println!("  Error: {}", e),
    }

    println!("\nAll tests completed!");

    Ok(())
}
