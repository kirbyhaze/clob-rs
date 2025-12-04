use alloy_primitives::{keccak256, Address, B256, U256};
use alloy_sol_types::sol;

use crate::config::get_contract_config;
use crate::error::{ClobError, Result};
use crate::signer::Signer;
use crate::types::{CreateOrderOptions, MarketOrderArgs, OrderArgs, RoundConfig, Side, TickSize};

pub const EOA: u8 = 0;
#[allow(dead_code)]
pub const POLY_PROXY: u8 = 1;
#[allow(dead_code)]
pub const POLY_GNOSIS_SAFE: u8 = 2;

sol! {
    struct Order {
        uint256 salt;
        address maker;
        address signer;
        address taker;
        uint256 tokenId;
        uint256 makerAmount;
        uint256 takerAmount;
        uint256 expiration;
        uint256 nonce;
        uint256 feeRateBps;
        uint8 side;
        uint8 signatureType;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedOrder {
    #[serde(serialize_with = "serialize_salt_as_int")]
    pub salt: String,
    pub maker: String,
    pub signer: String,
    pub taker: String,
    #[serde(rename = "tokenId")]
    pub token_id: String,
    #[serde(rename = "makerAmount")]
    pub maker_amount: String,
    #[serde(rename = "takerAmount")]
    pub taker_amount: String,
    pub expiration: String,
    pub nonce: String,
    #[serde(rename = "feeRateBps")]
    pub fee_rate_bps: String,
    #[serde(serialize_with = "serialize_side")]
    pub side: u8,
    #[serde(rename = "signatureType")]
    pub signature_type: u8,
    pub signature: String,
}

fn serialize_side<S>(value: &u8, serializer: S) -> std::result::Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match value {
        0 => serializer.serialize_str("BUY"),
        _ => serializer.serialize_str("SELL"),
    }
}

fn serialize_salt_as_int<S>(value: &str, serializer: S) -> std::result::Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    // Parse the string as u128 and serialize as an integer
    match value.parse::<u128>() {
        Ok(n) => serializer.serialize_u128(n),
        Err(_) => serializer.serialize_str(value),
    }
}

use serde::{Deserialize, Serialize};

const ROUNDING_CONFIG: [(TickSize, RoundConfig); 4] = [
    (
        TickSize::Size0_1,
        RoundConfig {
            price: 1,
            size: 2,
            amount: 3,
        },
    ),
    (
        TickSize::Size0_01,
        RoundConfig {
            price: 2,
            size: 2,
            amount: 4,
        },
    ),
    (
        TickSize::Size0_001,
        RoundConfig {
            price: 3,
            size: 2,
            amount: 5,
        },
    ),
    (
        TickSize::Size0_0001,
        RoundConfig {
            price: 4,
            size: 2,
            amount: 6,
        },
    ),
];

fn get_round_config(tick_size: TickSize) -> RoundConfig {
    ROUNDING_CONFIG
        .iter()
        .find(|(ts, _)| *ts == tick_size)
        .map(|(_, rc)| *rc)
        .unwrap_or(RoundConfig {
            price: 2,
            size: 2,
            amount: 4,
        })
}

fn round_down(x: f64, decimals: u32) -> f64 {
    let factor = 10f64.powi(decimals as i32);
    (x * factor).floor() / factor
}

fn round_normal(x: f64, decimals: u32) -> f64 {
    let factor = 10f64.powi(decimals as i32);
    (x * factor).round() / factor
}

fn round_up(x: f64, decimals: u32) -> f64 {
    let factor = 10f64.powi(decimals as i32);
    (x * factor).ceil() / factor
}

fn to_token_decimals(x: f64) -> u64 {
    let scaled = x * 1_000_000.0;
    scaled.round() as u64
}

// TODO: this is in the order path as well which is called everytime
// there might be optimizations here since it allocates a string
fn decimal_places(x: f64) -> u32 {
    let s = format!("{}", x);
    if let Some(pos) = s.find('.') {
        (s.len() - pos - 1) as u32
    } else {
        0
    }
}

pub struct OrderBuilder {
    signer: Signer,
    sig_type: u8,
    funder: Address,
}

impl OrderBuilder {
    pub fn new(signer: Signer) -> Self {
        let funder = signer.address();
        Self {
            signer,
            sig_type: EOA,
            funder,
        }
    }

    pub fn with_sig_type(mut self, sig_type: u8) -> Self {
        self.sig_type = sig_type;
        self
    }

    pub fn with_funder(mut self, funder: Address) -> Self {
        self.funder = funder;
        self
    }

    fn get_order_amounts(
        &self,
        side: Side,
        size: f64,
        price: f64,
        round_config: RoundConfig,
    ) -> (u8, u64, u64) {
        let raw_price = round_normal(price, round_config.price);

        match side {
            Side::Buy => {
                let raw_taker_amt = round_down(size, round_config.size);
                let mut raw_maker_amt = raw_taker_amt * raw_price;

                if decimal_places(raw_maker_amt) > round_config.amount {
                    raw_maker_amt = round_up(raw_maker_amt, round_config.amount + 4);
                    if decimal_places(raw_maker_amt) > round_config.amount {
                        raw_maker_amt = round_down(raw_maker_amt, round_config.amount);
                    }
                }

                let maker_amount = to_token_decimals(raw_maker_amt);
                let taker_amount = to_token_decimals(raw_taker_amt);
                (0, maker_amount, taker_amount) // 0 = BUY
            }
            Side::Sell => {
                let raw_maker_amt = round_down(size, round_config.size);
                let mut raw_taker_amt = raw_maker_amt * raw_price;

                if decimal_places(raw_taker_amt) > round_config.amount {
                    raw_taker_amt = round_up(raw_taker_amt, round_config.amount + 4);
                    if decimal_places(raw_taker_amt) > round_config.amount {
                        raw_taker_amt = round_down(raw_taker_amt, round_config.amount);
                    }
                }

                let maker_amount = to_token_decimals(raw_maker_amt);
                let taker_amount = to_token_decimals(raw_taker_amt);
                (1, maker_amount, taker_amount) // 1 = SELL
            }
        }
    }

    fn get_market_order_amounts(
        &self,
        side: Side,
        amount: f64,
        price: f64,
        round_config: RoundConfig,
    ) -> (u8, u64, u64) {
        let raw_price = round_normal(price, round_config.price);

        match side {
            Side::Buy => {
                let raw_maker_amt = round_down(amount, round_config.size);
                let mut raw_taker_amt = raw_maker_amt / raw_price;

                if decimal_places(raw_taker_amt) > round_config.amount {
                    raw_taker_amt = round_up(raw_taker_amt, round_config.amount + 4);
                    if decimal_places(raw_taker_amt) > round_config.amount {
                        raw_taker_amt = round_down(raw_taker_amt, round_config.amount);
                    }
                }

                let maker_amount = to_token_decimals(raw_maker_amt);
                let taker_amount = to_token_decimals(raw_taker_amt);
                (0, maker_amount, taker_amount)
            }
            Side::Sell => {
                let raw_maker_amt = round_down(amount, round_config.size);
                let mut raw_taker_amt = raw_maker_amt * raw_price;

                if decimal_places(raw_taker_amt) > round_config.amount {
                    raw_taker_amt = round_up(raw_taker_amt, round_config.amount + 4);
                    if decimal_places(raw_taker_amt) > round_config.amount {
                        raw_taker_amt = round_down(raw_taker_amt, round_config.amount);
                    }
                }

                let maker_amount = to_token_decimals(raw_maker_amt);
                let taker_amount = to_token_decimals(raw_taker_amt);
                (1, maker_amount, taker_amount)
            }
        }
    }

    //TODO: domain separators are computed on every order/signature
    //these are computed on every order creation and uncessary maybe some type of lazylock
    fn domain_separator(&self, exchange: &str, chain_id: u64) -> B256 {
        let type_hash = keccak256(
            "EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)",
        );

        let name_hash = keccak256("Polymarket CTF Exchange");
        let version_hash = keccak256("1");
        let chain_id_bytes = U256::from(chain_id);

        let exchange_addr: Address = exchange.parse().expect("invalid exchange address");
        let mut exchange_padded = [0u8; 32];
        exchange_padded[12..].copy_from_slice(exchange_addr.as_slice());

        let encoded = [
            type_hash.as_slice(),
            name_hash.as_slice(),
            version_hash.as_slice(),
            &chain_id_bytes.to_be_bytes::<32>(),
            &exchange_padded,
        ]
        .concat();

        keccak256(&encoded)
    }

    #[allow(clippy::too_many_arguments)]
    fn order_struct_hash(
        &self,
        salt: U256,
        maker: Address,
        signer_addr: Address,
        taker: Address,
        token_id: U256,
        maker_amount: U256,
        taker_amount: U256,
        expiration: U256,
        nonce: U256,
        fee_rate_bps: U256,
        side: u8,
        signature_type: u8,
    ) -> B256 {
        let type_hash = keccak256(
            "Order(uint256 salt,address maker,address signer,address taker,uint256 tokenId,uint256 makerAmount,uint256 takerAmount,uint256 expiration,uint256 nonce,uint256 feeRateBps,uint8 side,uint8 signatureType)",
        );

        fn pad_address(addr: Address) -> [u8; 32] {
            let mut padded = [0u8; 32];
            padded[12..].copy_from_slice(addr.as_slice());
            padded
        }

        let encoded = [
            type_hash.as_slice(),
            &salt.to_be_bytes::<32>(),
            &pad_address(maker),
            &pad_address(signer_addr),
            &pad_address(taker),
            &token_id.to_be_bytes::<32>(),
            &maker_amount.to_be_bytes::<32>(),
            &taker_amount.to_be_bytes::<32>(),
            &expiration.to_be_bytes::<32>(),
            &nonce.to_be_bytes::<32>(),
            &fee_rate_bps.to_be_bytes::<32>(),
            &U256::from(side).to_be_bytes::<32>(),
            &U256::from(signature_type).to_be_bytes::<32>(),
        ]
        .concat();

        keccak256(&encoded)
    }

    pub async fn create_order(
        &self,
        order_args: &OrderArgs,
        options: &CreateOrderOptions,
    ) -> Result<SignedOrder> {
        let round_config = get_round_config(options.tick_size);
        let (side, maker_amount, taker_amount) = self.get_order_amounts(
            order_args.side,
            order_args.size,
            order_args.price,
            round_config,
        );

        let contract_config = get_contract_config(self.signer.chain_id(), options.neg_risk)
            .ok_or_else(|| ClobError::InvalidParameter("invalid chain_id".to_string()))?;

        // Generate salt similar to Python: round(timestamp * random())
        // This gives a value in the range of 0 to ~current_timestamp
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
        let salt = U256::from((now * rand::random::<f64>()).round() as u64);
        let taker_addr: Address = order_args.taker.parse().unwrap_or_default();
        let token_id = U256::from_str_radix(
            order_args
                .token_id
                .strip_prefix("0x")
                .unwrap_or(&order_args.token_id),
            if order_args.token_id.starts_with("0x") {
                16
            } else {
                10
            },
        )
        .map_err(|_| ClobError::InvalidParameter("invalid token_id".to_string()))?;

        let domain_sep = self.domain_separator(contract_config.exchange, self.signer.chain_id());
        let struct_hash = self.order_struct_hash(
            salt,
            self.funder,
            self.signer.address(),
            taker_addr,
            token_id,
            U256::from(maker_amount),
            U256::from(taker_amount),
            U256::from(order_args.expiration),
            U256::from(order_args.nonce),
            U256::from(order_args.fee_rate_bps as u64),
            side,
            self.sig_type,
        );

        let mut message = Vec::with_capacity(66);
        message.extend_from_slice(&[0x19, 0x01]);
        message.extend_from_slice(domain_sep.as_slice());
        message.extend_from_slice(struct_hash.as_slice());

        let hash = keccak256(&message);
        let signature = self.signer.sign_hash(hash).await?;

        //TODO: there are to many to_string or even clone calls here, could be room to optimize
        // this is in the order path
        Ok(SignedOrder {
            salt: salt.to_string(),
            maker: self.funder.to_checksum(None),
            signer: self.signer.address_string(),
            taker: order_args.taker.clone(),
            token_id: order_args.token_id.clone(),
            maker_amount: maker_amount.to_string(),
            taker_amount: taker_amount.to_string(),
            expiration: order_args.expiration.to_string(),
            nonce: order_args.nonce.to_string(),
            fee_rate_bps: order_args.fee_rate_bps.to_string(),
            side,
            signature_type: self.sig_type,
            signature,
        })
    }

    pub async fn create_market_order(
        &self,
        order_args: &MarketOrderArgs,
        options: &CreateOrderOptions,
    ) -> Result<SignedOrder> {
        let round_config = get_round_config(options.tick_size);
        let (side, maker_amount, taker_amount) = self.get_market_order_amounts(
            order_args.side,
            order_args.amount,
            order_args.price,
            round_config,
        );

        let contract_config = get_contract_config(self.signer.chain_id(), options.neg_risk)
            .ok_or_else(|| ClobError::InvalidParameter("invalid chain_id".to_string()))?;

        // Generate salt similar to Python: round(timestamp * random())
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
        let salt = U256::from((now * rand::random::<f64>()).round() as u64);
        let taker_addr: Address = order_args.taker.parse().unwrap_or_default();
        let token_id = U256::from_str_radix(
            order_args
                .token_id
                .strip_prefix("0x")
                .unwrap_or(&order_args.token_id),
            if order_args.token_id.starts_with("0x") {
                16
            } else {
                10
            },
        )
        .map_err(|_| ClobError::InvalidParameter("invalid token_id".to_string()))?;

        let domain_sep = self.domain_separator(contract_config.exchange, self.signer.chain_id());
        let struct_hash = self.order_struct_hash(
            salt,
            self.funder,
            self.signer.address(),
            taker_addr,
            token_id,
            U256::from(maker_amount),
            U256::from(taker_amount),
            U256::from(0u64), // market orders have no expiration
            U256::from(order_args.nonce),
            U256::from(order_args.fee_rate_bps as u64),
            side,
            self.sig_type,
        );

        let mut message = Vec::with_capacity(66);
        message.extend_from_slice(&[0x19, 0x01]);
        message.extend_from_slice(domain_sep.as_slice());
        message.extend_from_slice(struct_hash.as_slice());

        let hash = keccak256(&message);
        let signature = self.signer.sign_hash(hash).await?;

        Ok(SignedOrder {
            salt: salt.to_string(),
            maker: self.funder.to_checksum(None),
            signer: self.signer.address_string(),
            taker: order_args.taker.clone(),
            token_id: order_args.token_id.clone(),
            maker_amount: maker_amount.to_string(),
            taker_amount: taker_amount.to_string(),
            expiration: "0".to_string(),
            nonce: order_args.nonce.to_string(),
            fee_rate_bps: order_args.fee_rate_bps.to_string(),
            side,
            signature_type: self.sig_type,
            signature,
        })
    }
}
