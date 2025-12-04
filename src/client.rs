use reqwest::Client;
use serde::Serialize;
use std::collections::HashMap;

use crate::config::{END_CURSOR, FIRST_CURSOR};
use crate::endpoints;
use crate::error::{ClobError, Result};
use crate::headers::{
    create_level_1_headers, create_level_2_headers, L2Headers, POLY_ADDRESS, POLY_API_KEY,
    POLY_NONCE, POLY_PASSPHRASE, POLY_SIGNATURE, POLY_TIMESTAMP,
};
use crate::order_builder::{OrderBuilder, SignedOrder};
use crate::signer::Signer;
use crate::types::{
    ApiCreds, BalanceAllowanceParams, BalanceAllowanceResponse, BatchMidpointResponse,
    BatchPriceResponse, BatchSpreadResponse, BookParams, CreateOrderOptions, FeeRateResponse,
    LastTradePriceResponse, LastTradesPriceEntry, Market, MarketOrderArgs, MarketTradeEvent,
    MarketsResponse, MidpointResponse, NegRiskResponse, OpenOrderParams, OrderArgs, OrderBook,
    OrderType, PartialCreateOrderOptions, PriceResponse, ServerTime, Side, SimplifiedMarketsResponse,
    SpreadResponse, TickSize, TickSizeResponse, TradeParams,
};

const L0: u8 = 0;
const L1: u8 = 1;
const L2: u8 = 2;

pub struct ClobClient {
    host: String,
    chain_id: u64,
    http: Client,
    signer: Option<Signer>,
    creds: Option<ApiCreds>,
    order_builder: Option<OrderBuilder>,
    mode: u8,
    tick_sizes: HashMap<String, TickSize>,
    neg_risk: HashMap<String, bool>,
    fee_rates: HashMap<String, i32>,
}

impl ClobClient {
    pub fn new(host: impl Into<String>) -> Self {
        let host = host.into();
        let host = if host.ends_with('/') {
            host[..host.len() - 1].to_string()
        } else {
            host
        };

        Self {
            host,
            chain_id: crate::config::CHAIN_ID,
            http: Client::new(),
            signer: None,
            creds: None,
            order_builder: None,
            mode: L0,
            tick_sizes: HashMap::new(),
            neg_risk: HashMap::new(),
            fee_rates: HashMap::new(),
        }
    }

    pub fn polygon() -> Self {
        Self::new(crate::config::HOST)
    }

    pub fn with_signer(mut self, private_key: &str) -> Result<Self> {
        let signer = Signer::new(private_key, self.chain_id)?;
        let order_builder = OrderBuilder::new(Signer::new(private_key, self.chain_id)?);
        self.signer = Some(signer);
        self.order_builder = Some(order_builder);
        self.mode = L1;
        Ok(self)
    }

    pub fn with_creds(mut self, creds: ApiCreds) -> Self {
        self.creds = Some(creds);
        if self.signer.is_some() {
            self.mode = L2;
        }
        self
    }

    pub fn set_creds(&mut self, creds: ApiCreds) {
        self.creds = Some(creds);
        if self.signer.is_some() {
            self.mode = L2;
        }
    }

    pub fn with_funder(mut self, funder: &str) -> Result<Self> {
        let funder_addr: alloy_primitives::Address = funder
            .parse()
            .map_err(|_| ClobError::InvalidParameter("invalid funder address".to_string()))?;
        if let Some(builder) = self.order_builder.take() {
            self.order_builder = Some(builder.with_funder(funder_addr));
        }
        Ok(self)
    }

    pub fn set_funder(&mut self, funder: &str) -> Result<()> {
        let funder_addr: alloy_primitives::Address = funder
            .parse()
            .map_err(|_| ClobError::InvalidParameter("invalid funder address".to_string()))?;
        if let Some(builder) = self.order_builder.take() {
            self.order_builder = Some(builder.with_funder(funder_addr));
        }
        Ok(())
    }

    pub fn with_signature_type(mut self, sig_type: u8) -> Self {
        if let Some(builder) = self.order_builder.take() {
            self.order_builder = Some(builder.with_sig_type(sig_type));
        }
        self
    }

    pub fn set_signature_type(&mut self, sig_type: u8) {
        if let Some(builder) = self.order_builder.take() {
            self.order_builder = Some(builder.with_sig_type(sig_type));
        }
    }

    pub fn address(&self) -> Option<String> {
        self.signer.as_ref().map(|s| s.address_string())
    }

    fn assert_l1(&self) -> Result<&Signer> {
        if self.mode < L1 {
            return Err(ClobError::AuthRequired(
                "L1 auth required (provide private key)".to_string(),
            ));
        }
        self.signer.as_ref().ok_or_else(|| {
            ClobError::AuthRequired("L1 auth required (provide private key)".to_string())
        })
    }

    fn assert_l2(&self) -> Result<(&Signer, &ApiCreds)> {
        if self.mode < L2 {
            return Err(ClobError::AuthRequired(
                "L2 auth required (provide API credentials)".to_string(),
            ));
        }
        let signer = self
            .signer
            .as_ref()
            .ok_or_else(|| ClobError::AuthRequired("L2 auth required".to_string()))?;
        let creds = self.creds.as_ref().ok_or_else(|| {
            ClobError::AuthRequired("L2 auth required (provide API credentials)".to_string())
        })?;
        Ok((signer, creds))
    }

    // ========== L0 Endpoints (public) ==========

    pub async fn get_ok(&self) -> Result<serde_json::Value> {
        self.get("/").await
    }

    pub async fn get_server_time(&self) -> Result<ServerTime> {
        self.get(endpoints::TIME).await
    }

    pub async fn get_order_book(&self, token_id: &str) -> Result<OrderBook> {
        let url = format!("{}?token_id={}", endpoints::GET_ORDER_BOOK, token_id);
        self.get(&url).await
    }

    pub async fn get_order_books(&self, params: &[BookParams]) -> Result<Vec<OrderBook>> {
        let body: Vec<_> = params
            .iter()
            .map(|p| serde_json::json!({"token_id": p.token_id}))
            .collect();
        self.post(endpoints::GET_ORDER_BOOKS, &body).await
    }

    pub async fn get_midpoint(&self, token_id: &str) -> Result<f64> {
        let url = format!("{}?token_id={}", endpoints::MID_POINT, token_id);
        let resp: MidpointResponse = self.get(&url).await?;
        Ok(resp.mid)
    }

    pub async fn get_midpoints(&self, params: &[BookParams]) -> Result<Vec<BatchMidpointResponse>> {
        let body: Vec<_> = params
            .iter()
            .map(|p| serde_json::json!({"token_id": p.token_id}))
            .collect();

        let raw: HashMap<String, String> = self.post(endpoints::MID_POINTS, &body).await?;

        Ok(raw
            .into_iter()
            .map(|(token_id, mid)| BatchMidpointResponse {
                token_id,
                mid: mid.parse().ok(),
            })
            .collect())
    }

    pub async fn get_price(&self, token_id: &str, side: Side) -> Result<f64> {
        let url = format!("{}?token_id={}&side={}", endpoints::PRICE, token_id, side);
        let resp: PriceResponse = self.get(&url).await?;
        Ok(resp.price)
    }

    pub async fn get_prices(&self, params: &[BookParams]) -> Result<Vec<BatchPriceResponse>> {
        let body: Vec<_> = params
            .iter()
            .map(|p| {
                serde_json::json!({
                    "token_id": p.token_id,
                    "side": p.side.map(|s| s.to_string())
                })
            })
            .collect();

        let raw: HashMap<String, HashMap<String, String>> =
            self.post(endpoints::PRICES, &body).await?;

        Ok(raw
            .into_iter()
            .map(|(token_id, sides)| BatchPriceResponse {
                token_id,
                buy: sides.get("BUY").and_then(|s| s.parse().ok()),
                sell: sides.get("SELL").and_then(|s| s.parse().ok()),
            })
            .collect())
    }

    pub async fn get_spread(&self, token_id: &str) -> Result<f64> {
        let url = format!("{}?token_id={}", endpoints::SPREAD, token_id);
        let resp: SpreadResponse = self.get(&url).await?;
        Ok(resp.spread)
    }

    pub async fn get_spreads(&self, params: &[BookParams]) -> Result<Vec<BatchSpreadResponse>> {
        let body: Vec<_> = params
            .iter()
            .map(|p| serde_json::json!({"token_id": p.token_id}))
            .collect();

        let raw: HashMap<String, String> = self.post(endpoints::SPREADS, &body).await?;

        Ok(raw
            .into_iter()
            .map(|(token_id, spread)| BatchSpreadResponse {
                token_id,
                spread: spread.parse().ok(),
            })
            .collect())
    }

    pub async fn get_last_trade_price(&self, token_id: &str) -> Result<f64> {
        let url = format!("{}?token_id={}", endpoints::LAST_TRADE_PRICE, token_id);
        let resp: LastTradePriceResponse = self.get(&url).await?;
        Ok(resp.price)
    }

    pub async fn get_last_trades_prices(
        &self,
        params: &[BookParams],
    ) -> Result<Vec<LastTradesPriceEntry>> {
        let body: Vec<_> = params
            .iter()
            .map(|p| serde_json::json!({"token_id": p.token_id}))
            .collect();
        self.post(endpoints::LAST_TRADES_PRICES, &body).await
    }

    pub async fn get_tick_size(&mut self, token_id: &str) -> Result<TickSize> {
        if let Some(&tick_size) = self.tick_sizes.get(token_id) {
            return Ok(tick_size);
        }

        let url = format!("{}?token_id={}", endpoints::TICK_SIZE, token_id);
        let resp: TickSizeResponse = self.get(&url).await?;

        self.tick_sizes
            .insert(token_id.to_string(), resp.minimum_tick_size);
        Ok(resp.minimum_tick_size)
    }

    pub async fn get_neg_risk(&mut self, token_id: &str) -> Result<bool> {
        if let Some(&neg_risk) = self.neg_risk.get(token_id) {
            return Ok(neg_risk);
        }

        let url = format!("{}?token_id={}", endpoints::NEG_RISK, token_id);
        let resp: NegRiskResponse = self.get(&url).await?;

        self.neg_risk.insert(token_id.to_string(), resp.neg_risk);
        Ok(resp.neg_risk)
    }

    pub async fn get_fee_rate_bps(&mut self, token_id: &str) -> Result<i32> {
        if let Some(&fee_rate) = self.fee_rates.get(token_id) {
            return Ok(fee_rate);
        }

        let url = format!("{}?token_id={}", endpoints::FEE_RATE, token_id);
        let resp: FeeRateResponse = self.get(&url).await?;
        let fee_rate = resp.base_fee.unwrap_or(0);

        self.fee_rates.insert(token_id.to_string(), fee_rate);
        Ok(fee_rate)
    }

    pub async fn get_markets_page(&self, cursor: Option<&str>) -> Result<MarketsResponse> {
        let cursor = cursor.unwrap_or(FIRST_CURSOR);
        let url = format!("{}?next_cursor={}", endpoints::MARKETS, cursor);
        self.get(&url).await
    }

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

    pub async fn get_simplified_markets_page(
        &self,
        cursor: Option<&str>,
    ) -> Result<SimplifiedMarketsResponse> {
        let cursor = cursor.unwrap_or(FIRST_CURSOR);
        let url = format!("{}?next_cursor={}", endpoints::SIMPLIFIED_MARKETS, cursor);
        self.get(&url).await
    }

    pub async fn get_sampling_markets_page(&self, cursor: Option<&str>) -> Result<MarketsResponse> {
        let cursor = cursor.unwrap_or(FIRST_CURSOR);
        let url = format!("{}?next_cursor={}", endpoints::SAMPLING_MARKETS, cursor);
        self.get(&url).await
    }

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

    pub async fn get_market(&self, condition_id: &str) -> Result<Market> {
        let url = format!("{}{}", endpoints::MARKET, condition_id);
        self.get(&url).await
    }

    pub async fn get_market_trades_events(
        &self,
        condition_id: &str,
    ) -> Result<Vec<MarketTradeEvent>> {
        let url = format!("{}{}", endpoints::MARKET_TRADES_EVENTS, condition_id);
        self.get(&url).await
    }

    // ========== L1 Endpoints (requires signer) ==========

    pub async fn create_api_key(&self, nonce: Option<u64>) -> Result<ApiCreds> {
        let signer = self.assert_l1()?;
        let headers = create_level_1_headers(signer, nonce).await?;

        let url = format!("{}{}", self.host, endpoints::CREATE_API_KEY);
        let response = self
            .http
            .post(&url)
            .header("User-Agent", "clob-rs")
            .header("Accept", "application/json")
            .header(POLY_ADDRESS, &headers.address)
            .header(POLY_SIGNATURE, &headers.signature)
            .header(POLY_TIMESTAMP, &headers.timestamp)
            .header(POLY_NONCE, &headers.nonce)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ClobError::Api {
                message: format!("HTTP {}: {}", status, body),
            });
        }

        response.json().await.map_err(|e| ClobError::Json {
            message: e.to_string(),
        })
    }

    pub async fn derive_api_key(&self, nonce: Option<u64>) -> Result<ApiCreds> {
        let signer = self.assert_l1()?;
        let headers = create_level_1_headers(signer, nonce).await?;

        let url = format!("{}{}", self.host, endpoints::DERIVE_API_KEY);
        let response = self
            .http
            .get(&url)
            .header("User-Agent", "clob-rs")
            .header("Accept", "application/json")
            .header(POLY_ADDRESS, &headers.address)
            .header(POLY_SIGNATURE, &headers.signature)
            .header(POLY_TIMESTAMP, &headers.timestamp)
            .header(POLY_NONCE, &headers.nonce)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ClobError::Api {
                message: format!("HTTP {}: {}", status, body),
            });
        }

        response.json().await.map_err(|e| ClobError::Json {
            message: e.to_string(),
        })
    }

    pub async fn create_or_derive_api_key(&self, nonce: Option<u64>) -> Result<ApiCreds> {
        match self.create_api_key(nonce).await {
            Ok(creds) => Ok(creds),
            Err(_) => self.derive_api_key(nonce).await,
        }
    }

    pub async fn create_order(
        &mut self,
        order_args: &OrderArgs,
        options: Option<PartialCreateOrderOptions>,
    ) -> Result<SignedOrder> {
        self.assert_l1()?;

        let tick_size = match options.as_ref().and_then(|o| o.tick_size) {
            Some(ts) => ts,
            None => self.get_tick_size(&order_args.token_id).await?,
        };

        let neg_risk = match options.as_ref().and_then(|o| o.neg_risk) {
            Some(nr) => nr,
            None => self.get_neg_risk(&order_args.token_id).await?,
        };

        let create_options = CreateOrderOptions {
            tick_size,
            neg_risk,
        };

        self.order_builder
            .as_ref()
            .ok_or_else(|| ClobError::AuthRequired("order builder not initialized".to_string()))?
            .create_order(order_args, &create_options)
            .await
    }

    pub async fn create_market_order(
        &mut self,
        order_args: &MarketOrderArgs,
        options: Option<PartialCreateOrderOptions>,
    ) -> Result<SignedOrder> {
        self.assert_l1()?;

        let tick_size = match options.as_ref().and_then(|o| o.tick_size) {
            Some(ts) => ts,
            None => self.get_tick_size(&order_args.token_id).await?,
        };

        let neg_risk = match options.as_ref().and_then(|o| o.neg_risk) {
            Some(nr) => nr,
            None => self.get_neg_risk(&order_args.token_id).await?,
        };

        let create_options = CreateOrderOptions {
            tick_size,
            neg_risk,
        };

        self.order_builder
            .as_ref()
            .ok_or_else(|| ClobError::AuthRequired("order builder not initialized".to_string()))?
            .create_market_order(order_args, &create_options)
            .await
    }

    // ========== L2 Endpoints (requires API credentials) ==========

    pub async fn get_api_keys(&self) -> Result<serde_json::Value> {
        let (signer, creds) = self.assert_l2()?;
        let headers = create_level_2_headers(signer, creds, "GET", endpoints::GET_API_KEYS, None);

        let url = format!("{}{}", self.host, endpoints::GET_API_KEYS);
        self.get_with_l2_headers(&url, &headers).await
    }

    pub async fn delete_api_key(&self) -> Result<serde_json::Value> {
        let (signer, creds) = self.assert_l2()?;
        let headers =
            create_level_2_headers(signer, creds, "DELETE", endpoints::DELETE_API_KEY, None);

        let url = format!("{}{}", self.host, endpoints::DELETE_API_KEY);
        self.delete_with_l2_headers(&url, &headers).await
    }

    pub async fn post_order(
        &self,
        order: &SignedOrder,
        order_type: OrderType,
    ) -> Result<serde_json::Value> {
        let (signer, creds) = self.assert_l2()?;

        let body = serde_json::json!({
            "order": order,
            "owner": creds.api_key,
            "orderType": order_type.to_string()
        });
        let body_str = serde_json::to_string(&body).unwrap();

        let headers = create_level_2_headers(
            signer,
            creds,
            "POST",
            endpoints::POST_ORDER,
            Some(&body_str),
        );

        let url = format!("{}{}", self.host, endpoints::POST_ORDER);
        self.post_with_l2_headers(&url, &headers, &body).await
    }

    pub async fn cancel(&self, order_id: &str) -> Result<serde_json::Value> {
        let (signer, creds) = self.assert_l2()?;

        let body = serde_json::json!({"orderID": order_id});
        let body_str = serde_json::to_string(&body).unwrap();

        let headers =
            create_level_2_headers(signer, creds, "DELETE", endpoints::CANCEL, Some(&body_str));

        let url = format!("{}{}", self.host, endpoints::CANCEL);
        self.delete_with_l2_headers_and_body(&url, &headers, &body)
            .await
    }

    pub async fn cancel_orders(&self, order_ids: &[String]) -> Result<serde_json::Value> {
        let (signer, creds) = self.assert_l2()?;

        let body = serde_json::json!(order_ids);
        let body_str = serde_json::to_string(&body).unwrap();

        let headers = create_level_2_headers(
            signer,
            creds,
            "DELETE",
            endpoints::CANCEL_ORDERS,
            Some(&body_str),
        );

        let url = format!("{}{}", self.host, endpoints::CANCEL_ORDERS);
        self.delete_with_l2_headers_and_body(&url, &headers, &body)
            .await
    }

    pub async fn cancel_all(&self) -> Result<serde_json::Value> {
        let (signer, creds) = self.assert_l2()?;
        let headers = create_level_2_headers(signer, creds, "DELETE", endpoints::CANCEL_ALL, None);

        let url = format!("{}{}", self.host, endpoints::CANCEL_ALL);
        self.delete_with_l2_headers(&url, &headers).await
    }

    pub async fn get_orders(&self, params: Option<&OpenOrderParams>) -> Result<serde_json::Value> {
        let (signer, creds) = self.assert_l2()?;
        let headers = create_level_2_headers(signer, creds, "GET", endpoints::ORDERS, None);

        let mut url = format!("{}{}", self.host, endpoints::ORDERS);

        // Add query params
        let mut query_parts = Vec::new();
        if let Some(p) = params {
            if let Some(id) = &p.id {
                query_parts.push(format!("id={}", id));
            }
            if let Some(market) = &p.market {
                query_parts.push(format!("market={}", market));
            }
            if let Some(asset_id) = &p.asset_id {
                query_parts.push(format!("asset_id={}", asset_id));
            }
        }
        if !query_parts.is_empty() {
            url = format!("{}?{}", url, query_parts.join("&"));
        }

        self.get_with_l2_headers(&url, &headers).await
    }

    pub async fn get_order(&self, order_id: &str) -> Result<serde_json::Value> {
        let (signer, creds) = self.assert_l2()?;
        let path = format!("{}{}", endpoints::GET_ORDER, order_id);
        let headers = create_level_2_headers(signer, creds, "GET", &path, None);

        let url = format!("{}{}", self.host, path);
        self.get_with_l2_headers(&url, &headers).await
    }

    pub async fn get_trades(&self, params: Option<&TradeParams>) -> Result<serde_json::Value> {
        let (signer, creds) = self.assert_l2()?;
        let headers = create_level_2_headers(signer, creds, "GET", endpoints::TRADES, None);

        let mut url = format!("{}{}", self.host, endpoints::TRADES);

        let mut query_parts = Vec::new();
        if let Some(p) = params {
            if let Some(id) = &p.id {
                query_parts.push(format!("id={}", id));
            }
            if let Some(maker) = &p.maker_address {
                query_parts.push(format!("maker_address={}", maker));
            }
            if let Some(market) = &p.market {
                query_parts.push(format!("market={}", market));
            }
            if let Some(asset_id) = &p.asset_id {
                query_parts.push(format!("asset_id={}", asset_id));
            }
            if let Some(before) = p.before {
                query_parts.push(format!("before={}", before));
            }
            if let Some(after) = p.after {
                query_parts.push(format!("after={}", after));
            }
        }
        if !query_parts.is_empty() {
            url = format!("{}?{}", url, query_parts.join("&"));
        }

        self.get_with_l2_headers(&url, &headers).await
    }

    pub async fn get_balance_allowance(
        &self,
        params: &BalanceAllowanceParams,
    ) -> Result<BalanceAllowanceResponse> {
        let (signer, creds) = self.assert_l2()?;
        let headers =
            create_level_2_headers(signer, creds, "GET", endpoints::GET_BALANCE_ALLOWANCE, None);

        let mut url = format!("{}{}", self.host, endpoints::GET_BALANCE_ALLOWANCE);

        // Add query params
        let mut query_parts = Vec::new();
        if let Some(asset_type) = &params.asset_type {
            query_parts.push(format!("asset_type={}", asset_type));
        }
        if let Some(token_id) = &params.token_id {
            query_parts.push(format!("token_id={}", token_id));
        }
        if let Some(sig_type) = params.signature_type {
            query_parts.push(format!("signature_type={}", sig_type));
        }
        if !query_parts.is_empty() {
            url = format!("{}?{}", url, query_parts.join("&"));
        }

        self.get_with_l2_headers(&url, &headers).await
    }

    pub async fn update_balance_allowance(
        &self,
        params: &BalanceAllowanceParams,
    ) -> Result<BalanceAllowanceResponse> {
        let (signer, creds) = self.assert_l2()?;
        let headers = create_level_2_headers(
            signer,
            creds,
            "GET",
            endpoints::UPDATE_BALANCE_ALLOWANCE,
            None,
        );

        let mut url = format!("{}{}", self.host, endpoints::UPDATE_BALANCE_ALLOWANCE);

        // Add query params
        let mut query_parts = Vec::new();
        if let Some(asset_type) = &params.asset_type {
            query_parts.push(format!("asset_type={}", asset_type));
        }
        if let Some(token_id) = &params.token_id {
            query_parts.push(format!("token_id={}", token_id));
        }
        if let Some(sig_type) = params.signature_type {
            query_parts.push(format!("signature_type={}", sig_type));
        }
        if !query_parts.is_empty() {
            url = format!("{}?{}", url, query_parts.join("&"));
        }

        self.get_with_l2_headers(&url, &headers).await
    }

    // ========== Internal HTTP helpers ==========

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

        let text = response.text().await?;
        serde_json::from_str(&text).map_err(|e| ClobError::Json {
            message: e.to_string(),
        })
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

    async fn get_with_l2_headers<T: serde::de::DeserializeOwned>(
        &self,
        url: &str,
        headers: &L2Headers,
    ) -> Result<T> {
        let response = self
            .http
            .get(url)
            .header("User-Agent", "clob-rs")
            .header("Accept", "application/json")
            .header(POLY_ADDRESS, &headers.address)
            .header(POLY_SIGNATURE, &headers.signature)
            .header(POLY_TIMESTAMP, &headers.timestamp)
            .header(POLY_API_KEY, &headers.api_key)
            .header(POLY_PASSPHRASE, &headers.passphrase)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ClobError::Api {
                message: format!("HTTP {}: {}", status, body),
            });
        }

        response.json().await.map_err(|e| ClobError::Json {
            message: e.to_string(),
        })
    }

    async fn post_with_l2_headers<T: serde::de::DeserializeOwned, B: Serialize>(
        &self,
        url: &str,
        headers: &L2Headers,
        body: &B,
    ) -> Result<T> {
        let response = self
            .http
            .post(url)
            .header("User-Agent", "clob-rs")
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .header(POLY_ADDRESS, &headers.address)
            .header(POLY_SIGNATURE, &headers.signature)
            .header(POLY_TIMESTAMP, &headers.timestamp)
            .header(POLY_API_KEY, &headers.api_key)
            .header(POLY_PASSPHRASE, &headers.passphrase)
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

        response.json().await.map_err(|e| ClobError::Json {
            message: e.to_string(),
        })
    }

    async fn delete_with_l2_headers<T: serde::de::DeserializeOwned>(
        &self,
        url: &str,
        headers: &L2Headers,
    ) -> Result<T> {
        let response = self
            .http
            .delete(url)
            .header("User-Agent", "clob-rs")
            .header("Accept", "application/json")
            .header(POLY_ADDRESS, &headers.address)
            .header(POLY_SIGNATURE, &headers.signature)
            .header(POLY_TIMESTAMP, &headers.timestamp)
            .header(POLY_API_KEY, &headers.api_key)
            .header(POLY_PASSPHRASE, &headers.passphrase)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ClobError::Api {
                message: format!("HTTP {}: {}", status, body),
            });
        }

        response.json().await.map_err(|e| ClobError::Json {
            message: e.to_string(),
        })
    }

    async fn delete_with_l2_headers_and_body<T: serde::de::DeserializeOwned, B: Serialize>(
        &self,
        url: &str,
        headers: &L2Headers,
        body: &B,
    ) -> Result<T> {
        let response = self
            .http
            .delete(url)
            .header("User-Agent", "clob-rs")
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .header(POLY_ADDRESS, &headers.address)
            .header(POLY_SIGNATURE, &headers.signature)
            .header(POLY_TIMESTAMP, &headers.timestamp)
            .header(POLY_API_KEY, &headers.api_key)
            .header(POLY_PASSPHRASE, &headers.passphrase)
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

        response.json().await.map_err(|e| ClobError::Json {
            message: e.to_string(),
        })
    }
}
