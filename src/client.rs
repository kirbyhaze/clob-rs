use reqwest::Client;
use serde::Serialize;
use std::collections::HashMap;

use crate::config::{END_CURSOR, FIRST_CURSOR};
use crate::endpoints;
use crate::error::{ClobError, Result};
use crate::types::{
    BatchLastTradePriceResponse, BatchMidpointResponse, BatchPriceResponse, BatchSpreadResponse,
    BookParams, FeeRateResponse, LastTradePriceResponse, Market, MarketTradeEvent,
    MarketsResponse, MidpointResponse, NegRiskResponse, OrderBook, PriceResponse, 
    ServerTime, Side, SimplifiedMarketsResponse, SpreadResponse, TickSize, 
    TickSizeResponse,
};

/// Polymarket CLOB API Client
///
/// This client provides access to Level 0 (unauthenticated) market data endpoints.
/// For authenticated operations (L1/L2), additional signing capabilities would be needed.
pub struct ClobClient {
    host: String,
    http: Client,
    // Local caches for market metadata
    tick_sizes: HashMap<String, TickSize>,
    neg_risk: HashMap<String, bool>,
    fee_rates: HashMap<String, i32>,
}

impl ClobClient {
    /// Create a new CLOB client for the given host URL
    ///
    /// # Example
    /// ```
    /// use clob_rs::ClobClient;
    /// use clob_rs::config::POLYGON_HOST;
    ///
    /// let client = ClobClient::new(POLYGON_HOST);
    /// ```
    pub fn new(host: impl Into<String>) -> Self {
        let host = host.into();
        let host = if host.ends_with('/') {
            host[..host.len() - 1].to_string()
        } else {
            host
        };

        Self {
            host,
            http: Client::new(),
            tick_sizes: HashMap::new(),
            neg_risk: HashMap::new(),
            fee_rates: HashMap::new(),
        }
    }

    /// Create a new client for Polygon mainnet
    pub fn polygon() -> Self {
        Self::new(crate::config::POLYGON_HOST)
    }

    /// Create a new client for Amoy testnet
    pub fn amoy() -> Self {
        Self::new(crate::config::AMOY_HOST)
    }

    // =========================================================================
    // Health & Server
    // =========================================================================

    /// Health check - confirms the server is up
    pub async fn get_ok(&self) -> Result<serde_json::Value> {
        self.get("/").await
    }

    /// Get the current server timestamp
    pub async fn get_server_time(&self) -> Result<ServerTime> {
        self.get(endpoints::TIME).await
    }

    // =========================================================================
    // Order Book
    // =========================================================================

    /// Fetch the order book for a single token
    pub async fn get_order_book(&self, token_id: &str) -> Result<OrderBook> {
        let url = format!("{}?token_id={}", endpoints::GET_ORDER_BOOK, token_id);
        self.get(&url).await
    }

    /// Fetch order books for multiple tokens
    pub async fn get_order_books(&self, params: &[BookParams]) -> Result<Vec<OrderBook>> {
        let body: Vec<_> = params
            .iter()
            .map(|p| serde_json::json!({"token_id": p.token_id}))
            .collect();
        self.post(endpoints::GET_ORDER_BOOKS, &body).await
    }

    // =========================================================================
    // Price Data
    // =========================================================================

    /// Get the midpoint price for a token
    pub async fn get_midpoint(&self, token_id: &str) -> Result<f64> {
        let url = format!("{}?token_id={}", endpoints::MID_POINT, token_id);
        let resp: MidpointResponse = self.get(&url).await?;
        Ok(resp.mid)
    }

    /// Get midpoint prices for multiple tokens
    pub async fn get_midpoints(&self, params: &[BookParams]) -> Result<Vec<BatchMidpointResponse>> {
        let body: Vec<_> = params
            .iter()
            .map(|p| serde_json::json!({"token_id": p.token_id}))
            .collect();
        self.post(endpoints::MID_POINTS, &body).await
    }

    /// Get the market price for a token on a specific side
    pub async fn get_price(&self, token_id: &str, side: Side) -> Result<f64> {
        let url = format!("{}?token_id={}&side={}", endpoints::PRICE, token_id, side);
        let resp: PriceResponse = self.get(&url).await?;
        Ok(resp.price)
    }

    /// Get prices for multiple token/side combinations
    pub async fn get_prices(&self, params: &[BookParams]) -> Result<Vec<BatchPriceResponse>> {
        let body: Vec<_> = params
            .iter()
            .map(|p| {
                serde_json::json!({
                    "token_id": p.token_id,
                    "side": p.side
                })
            })
            .collect();
        self.post(endpoints::PRICES, &body).await
    }

    /// Get the spread for a token
    pub async fn get_spread(&self, token_id: &str) -> Result<f64> {
        let url = format!("{}?token_id={}", endpoints::SPREAD, token_id);
        let resp: SpreadResponse = self.get(&url).await?;
        Ok(resp.spread)
    }

    /// Get spreads for multiple tokens
    pub async fn get_spreads(&self, params: &[BookParams]) -> Result<Vec<BatchSpreadResponse>> {
        let body: Vec<_> = params
            .iter()
            .map(|p| serde_json::json!({"token_id": p.token_id}))
            .collect();
        self.post(endpoints::SPREADS, &body).await
    }

    /// Get the last trade price for a token
    pub async fn get_last_trade_price(&self, token_id: &str) -> Result<f64> {
        let url = format!("{}?token_id={}", endpoints::LAST_TRADE_PRICE, token_id);
        let resp: LastTradePriceResponse = self.get(&url).await?;
        Ok(resp.price)
    }

    /// Get last trade prices for multiple tokens
    pub async fn get_last_trades_prices(
        &self,
        params: &[BookParams],
    ) -> Result<Vec<BatchLastTradePriceResponse>> {
        let body: Vec<_> = params
            .iter()
            .map(|p| serde_json::json!({"token_id": p.token_id}))
            .collect();
        self.post(endpoints::LAST_TRADES_PRICES, &body).await
    }

    // =========================================================================
    // Market Metadata
    // =========================================================================

    /// Get the minimum tick size for a token
    pub async fn get_tick_size(&mut self, token_id: &str) -> Result<TickSize> {
        // Check cache first
        if let Some(&tick_size) = self.tick_sizes.get(token_id) {
            return Ok(tick_size);
        }

        let url = format!("{}?token_id={}", endpoints::TICK_SIZE, token_id);
        let resp: TickSizeResponse = self.get(&url).await?;

        let tick_size = TickSize::from_str(&resp.minimum_tick_size).ok_or_else(|| {
            ClobError::InvalidParameter(format!(
                "Unknown tick size: {}",
                resp.minimum_tick_size
            ))
        })?;

        self.tick_sizes.insert(token_id.to_string(), tick_size);
        Ok(tick_size)
    }

    /// Get whether a token is in a negative risk market
    pub async fn get_neg_risk(&mut self, token_id: &str) -> Result<bool> {
        // Check cache first
        if let Some(&neg_risk) = self.neg_risk.get(token_id) {
            return Ok(neg_risk);
        }

        let url = format!("{}?token_id={}", endpoints::NEG_RISK, token_id);
        let resp: NegRiskResponse = self.get(&url).await?;

        self.neg_risk.insert(token_id.to_string(), resp.neg_risk);
        Ok(resp.neg_risk)
    }

    /// Get the fee rate in basis points for a token
    pub async fn get_fee_rate_bps(&mut self, token_id: &str) -> Result<i32> {
        // Check cache first
        if let Some(&fee_rate) = self.fee_rates.get(token_id) {
            return Ok(fee_rate);
        }

        let url = format!("{}?token_id={}", endpoints::FEE_RATE, token_id);
        let resp: FeeRateResponse = self.get(&url).await?;
        let fee_rate = resp.base_fee.unwrap_or(0);

        self.fee_rates.insert(token_id.to_string(), fee_rate);
        Ok(fee_rate)
    }

    // =========================================================================
    // Markets
    // =========================================================================

    /// Get a single page of markets
    pub async fn get_markets_page(&self, cursor: Option<&str>) -> Result<MarketsResponse> {
        let cursor = cursor.unwrap_or(FIRST_CURSOR);
        let url = format!("{}?next_cursor={}", endpoints::MARKETS, cursor);
        self.get(&url).await
    }

    /// Get all markets (auto-paginates)
    pub async fn get_markets(&self) -> Result<Vec<Market>> {
        let mut results = Vec::new();
        let mut cursor = FIRST_CURSOR.to_string();

        loop {
            let response = self.get_markets_page(Some(&cursor)).await?;
            results.extend(response.data);

            if response.next_cursor == END_CURSOR {
                break;
            }
            cursor = response.next_cursor;
        }

        Ok(results)
    }

    /// Get a single page of simplified markets
    pub async fn get_simplified_markets_page(
        &self,
        cursor: Option<&str>,
    ) -> Result<SimplifiedMarketsResponse> {
        let cursor = cursor.unwrap_or(FIRST_CURSOR);
        let url = format!("{}?next_cursor={}", endpoints::SIMPLIFIED_MARKETS, cursor);
        self.get(&url).await
    }

    /// Get a single page of sampling markets
    pub async fn get_sampling_markets_page(&self, cursor: Option<&str>) -> Result<MarketsResponse> {
        let cursor = cursor.unwrap_or(FIRST_CURSOR);
        let url = format!("{}?next_cursor={}", endpoints::SAMPLING_MARKETS, cursor);
        self.get(&url).await
    }

    /// Get a single page of sampling simplified markets
    pub async fn get_sampling_simplified_markets_page(
        &self,
        cursor: Option<&str>,
    ) -> Result<SimplifiedMarketsResponse> {
        let cursor = cursor.unwrap_or(FIRST_CURSOR);
        let url = format!(
            "{}?next_cursor={}",
            endpoints::SAMPLING_SIMPLIFIED_MARKETS,
            cursor
        );
        self.get(&url).await
    }

    /// Get a market by condition ID
    pub async fn get_market(&self, condition_id: &str) -> Result<Market> {
        let url = format!("{}{}", endpoints::MARKET, condition_id);
        self.get(&url).await
    }

    /// Get trade events for a market by condition ID
    pub async fn get_market_trades_events(
        &self,
        condition_id: &str,
    ) -> Result<Vec<MarketTradeEvent>> {
        let url = format!("{}{}", endpoints::MARKET_TRADES_EVENTS, condition_id);
        self.get(&url).await
    }

    // =========================================================================
    // HTTP Helpers
    // =========================================================================

    async fn get<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = format!("{}{}", self.host, path);
        let response = self
            .http
            .get(&url)
            .header("User-Agent", "clob-rs")
            .header("Accept", "application/json")
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ClobError::Api {
                message: format!("HTTP {}: {}", status, body),
            });
        }

        let body = response.json().await?;
        Ok(body)
    }

    async fn post<T: serde::de::DeserializeOwned, B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let url = format!("{}{}", self.host, path);
        let response = self
            .http
            .post(&url)
            .header("User-Agent", "clob-rs")
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ClobError::Api {
                message: format!("HTTP {}: {}", status, body),
            });
        }

        let body = response.json().await?;
        Ok(body)
    }
}
