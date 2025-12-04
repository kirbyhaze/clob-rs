use serde::{Deserialize, Deserializer, Serialize};

fn deserialize_string_to_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    s.parse::<f64>().map_err(serde::de::Error::custom)
}

#[allow(dead_code)]
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

fn deserialize_null_to_empty_vec<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    let opt: Option<Vec<T>> = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

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
    #[serde(deserialize_with = "deserialize_tick_size")]
    pub minimum_tick_size: TickSize,
}

fn deserialize_tick_size<'de, D>(deserializer: D) -> Result<TickSize, D::Error>
where
    D: Deserializer<'de>,
{
    let value: serde_json::Value = Deserialize::deserialize(deserializer)?;
    let s = match value {
        serde_json::Value::String(s) => s,
        serde_json::Value::Number(n) => n.to_string(),
        _ => return Err(serde::de::Error::custom("expected string or number")),
    };
    TickSize::from_str(&s).ok_or_else(|| serde::de::Error::custom(format!("unknown tick size: {}", s)))
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

// Server time is returned as a raw integer timestamp
pub type ServerTime = u64;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookParams {
    pub token_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub side: Option<Side>,
}

impl BookParams {
    pub fn new(token_id: impl Into<String>) -> Self {
        Self {
            token_id: token_id.into(),
            side: None,
        }
    }

    pub fn with_side(token_id: impl Into<String>, side: Side) -> Self {
        Self {
            token_id: token_id.into(),
            side: Some(side),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BatchMidpointResponse {
    pub token_id: String,
    pub mid: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct BatchPriceResponse {
    pub token_id: String,
    pub buy: Option<f64>,
    pub sell: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct BatchSpreadResponse {
    pub token_id: String,
    pub spread: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LastTradesPriceEntry {
    pub token_id: String,
    pub side: String,
    #[serde(deserialize_with = "deserialize_string_to_f64")]
    pub price: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    pub token_id: String,
    pub outcome: String,
    pub price: f64,
    #[serde(default)]
    pub winner: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketRewards {
    #[serde(default)]
    pub rates: Option<serde_json::Value>,
    #[serde(default)]
    pub min_size: f64,
    #[serde(default)]
    pub max_spread: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Market {
    pub condition_id: String,
    pub question_id: String,
    pub tokens: Vec<Token>,
    #[serde(default)]
    pub enable_order_book: bool,
    pub active: bool,
    pub closed: bool,
    #[serde(default)]
    pub archived: bool,
    #[serde(default)]
    pub accepting_orders: bool,
    #[serde(default)]
    pub accepting_order_timestamp: Option<String>,
    #[serde(default)]
    pub minimum_order_size: f64,
    #[serde(default)]
    pub minimum_tick_size: f64,
    #[serde(default)]
    pub seconds_delay: i32,
    #[serde(default)]
    pub question: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub market_slug: Option<String>,
    #[serde(default)]
    pub end_date_iso: Option<String>,
    #[serde(default)]
    pub game_start_time: Option<String>,
    #[serde(default)]
    pub fpmm: Option<String>,
    #[serde(default)]
    pub maker_base_fee: i32,
    #[serde(default)]
    pub taker_base_fee: i32,
    #[serde(default)]
    pub neg_risk: bool,
    #[serde(default)]
    pub neg_risk_market_id: Option<String>,
    #[serde(default)]
    pub neg_risk_request_id: Option<String>,
    #[serde(default)]
    pub notifications_enabled: bool,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub image: Option<String>,
    #[serde(default)]
    pub rewards: Option<MarketRewards>,
    #[serde(default)]
    pub is_50_50_outcome: bool,
    #[serde(default, deserialize_with = "deserialize_null_to_empty_vec")]
    pub tags: Vec<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketTradeEvent {
    pub id: String,
    #[serde(rename = "type")]
    pub event_type: String,
    pub timestamp: String,
    pub token_id: String,
    pub side: String,
    pub price: f64,
    pub size: f64,
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
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
