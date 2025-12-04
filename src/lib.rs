//! Polymarket CLOB Client for Rust
//!
//! This crate provides a client for interacting with the Polymarket Central Limit Order Book (CLOB) API.
//!
//! # Example
//!
//! ```rust,no_run
//! use clob_rs::{ClobClient, Side};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a client for Polygon mainnet
//!     let client = ClobClient::polygon();
//!
//!     // Get an order book
//!     let token_id = "your_token_id";
//!     let book = client.get_order_book(token_id).await?;
//!     println!("Best bid: {:?}", book.best_bid());
//!     println!("Best ask: {:?}", book.best_ask());
//!
//!     // Get the midpoint price
//!     let mid = client.get_midpoint(token_id).await?;
//!     println!("Midpoint: {}", mid);
//!
//!     // Get price for a specific side
//!     let buy_price = client.get_price(token_id, Side::Buy).await?;
//!     println!("Buy price: {}", buy_price);
//!
//!     Ok(())
//! }
//! ```

mod client;
pub mod config;
pub mod endpoints;
mod error;
mod types;

pub use client::ClobClient;
pub use error::{ClobError, Result};
pub use types::{
    BatchLastTradePriceResponse, BatchMidpointResponse, BatchPriceResponse, BatchSpreadResponse,
    BookParams, FeeRateResponse, LastTradePriceResponse, Market, MarketTradeEvent,
    MarketsResponse, MidpointResponse, NegRiskResponse, OrderBook, OrderSummary, PriceResponse,
    ServerTime, Side, SimplifiedMarket, SimplifiedMarketsResponse, SpreadResponse,
    TickSize, TickSizeResponse, Token,
};
