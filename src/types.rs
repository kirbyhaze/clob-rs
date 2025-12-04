use serde::{Deserialize, Deserializer, Serialize};
use serde_aux::field_attributes::deserialize_number_from_string;

/// Deserialize a string that may be a number into f64
fn deserialize_string_to_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    s.parse::<f64>().map_err(serde::de::Error::custom)
}

/// Deserialize optional string to Option<f64>
fn deserialize_optional_string_to_f64<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt: Option<String> = Option::deserialize(deserializer)?;
    match opt {
        Some(s) if !s.is_empty() => s.parse::<f64>().map(Some).map_err(serde::de::Error::custom),
        _ => Ok(None),
    }
}

// ============================================================================
// Order Book Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderSummary {
    pub price: String,
    pub size: String,
}

impl OrderSummary {
    pub fn price_f64(&self) -> f64 {
        self.price.parse().unwrap_or(0.0)
    }

    pub fn size_f64(&self) -> f64 {
        self.size.parse().unwrap_or(0.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBook {
    pub market: String,
    pub asset_id: String,
    pub timestamp: String,
    pub hash: String,
    #[serde(default)]
    pub bids: Vec<OrderSummary>,
    #[serde(default)]
    pub asks: Vec<OrderSummary>,
    #[serde(default)]
    pub min_order_size: Option<String>,
    #[serde(default)]
    pub tick_size: Option<String>,
    #[serde(default)]
    pub neg_risk: Option<bool>,
}

impl OrderBook {
    pub fn best_bid(&self) -> Option<&OrderSummary> {
        self.bids.first()
    }

    pub fn best_ask(&self) -> Option<&OrderSummary> {
        self.asks.first()
    }

    pub fn spread(&self) -> Option<f64> {
        match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) => Some(ask.price_f64() - bid.price_f64()),
            _ => None,
        }
    }

    pub fn midpoint(&self) -> Option<f64> {
        match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) => Some((bid.price_f64() + ask.price_f64()) / 2.0),
            _ => None,
        }
    }
}

// ============================================================================
// Price/Market Data Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MidpointResponse {
    #[serde(deserialize_with = "deserialize_string_to_f64")]
    pub mid: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceResponse {
    #[serde(deserialize_with = "deserialize_string_to_f64")]
    pub price: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpreadResponse {
    #[serde(deserialize_with = "deserialize_string_to_f64")]
    pub spread: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickSizeResponse {
    pub minimum_tick_size: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NegRiskResponse {
    pub neg_risk: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeRateResponse {
    #[serde(default)]
    pub base_fee: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LastTradePriceResponse {
    #[serde(deserialize_with = "deserialize_string_to_f64")]
    pub price: f64,
}

/// Server time is returned as a raw integer timestamp
pub type ServerTime = u64;

// ============================================================================
// Batch Request/Response Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookParams {
    pub token_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub side: Option<String>,
}

impl BookParams {
    pub fn new(token_id: impl Into<String>) -> Self {
        Self {
            token_id: token_id.into(),
            side: None,
        }
    }

    pub fn with_side(token_id: impl Into<String>, side: impl Into<String>) -> Self {
        Self {
            token_id: token_id.into(),
            side: Some(side.into()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchMidpointResponse {
    pub token_id: String,
    #[serde(deserialize_with = "deserialize_optional_string_to_f64", default)]
    pub mid: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchPriceResponse {
    pub token_id: String,
    #[serde(deserialize_with = "deserialize_optional_string_to_f64", default)]
    pub price: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchSpreadResponse {
    pub token_id: String,
    #[serde(deserialize_with = "deserialize_optional_string_to_f64", default)]
    pub spread: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchLastTradePriceResponse {
    pub token_id: String,
    #[serde(deserialize_with = "deserialize_optional_string_to_f64", default)]
    pub price: Option<f64>,
}

// ============================================================================
// Market Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    pub token_id: String,
    pub outcome: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub price: f64,
    #[serde(default)]
    pub winner: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Market {
    pub condition_id: String,
    pub question_id: String,
    pub tokens: Vec<Token>,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub min_incentive_size: f64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub max_incentive_spread: f64,
    pub active: bool,
    pub closed: bool,
    #[serde(default)]
    pub accepting_orders: bool,
    #[serde(default)]
    pub neg_risk: bool,
    #[serde(default)]
    pub min_tick_size: Option<String>,
    #[serde(default)]
    pub min_order_size: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimplifiedMarket {
    pub condition_id: String,
    pub tokens: Vec<Token>,
    #[serde(default)]
    pub neg_risk: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketsResponse {
    pub data: Vec<Market>,
    pub next_cursor: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimplifiedMarketsResponse {
    pub data: Vec<SimplifiedMarket>,
    pub next_cursor: String,
}

// ============================================================================
// Trade Event Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketTradeEvent {
    pub id: String,
    #[serde(rename = "type")]
    pub event_type: String,
    pub timestamp: String,
    pub token_id: String,
    pub side: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub price: f64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub size: f64,
}

// ============================================================================
// Side enum for order operations
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Side {
    Buy,
    Sell,
}

impl std::fmt::Display for Side {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Side::Buy => write!(f, "BUY"),
            Side::Sell => write!(f, "SELL"),
        }
    }
}

// ============================================================================
// Tick Size
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TickSize {
    Size0_1,
    Size0_01,
    Size0_001,
    Size0_0001,
}

impl TickSize {
    pub fn as_str(&self) -> &'static str {
        match self {
            TickSize::Size0_1 => "0.1",
            TickSize::Size0_01 => "0.01",
            TickSize::Size0_001 => "0.001",
            TickSize::Size0_0001 => "0.0001",
        }
    }

    pub fn as_f64(&self) -> f64 {
        match self {
            TickSize::Size0_1 => 0.1,
            TickSize::Size0_01 => 0.01,
            TickSize::Size0_001 => 0.001,
            TickSize::Size0_0001 => 0.0001,
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "0.1" => Some(TickSize::Size0_1),
            "0.01" => Some(TickSize::Size0_01),
            "0.001" => Some(TickSize::Size0_001),
            "0.0001" => Some(TickSize::Size0_0001),
            _ => None,
        }
    }
}

impl std::fmt::Display for TickSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
