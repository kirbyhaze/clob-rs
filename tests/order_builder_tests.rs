use clob_rs::{
    ApiCreds, ClobClient, CreateOrderOptions, OrderArgs, OrderBuilder, Side, Signer, TickSize,
    POLY_PROXY,
};

const ZERO_ADDRESS: &str = "0x0000000000000000000000000000000000000000";

const TEST_PRIVATE_KEY: &str = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
const EXPECTED_ADDRESS: &str = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266";
const POLYGON_CHAIN_ID: u64 = 137;

fn create_test_signer() -> Signer {
    Signer::new(TEST_PRIVATE_KEY, POLYGON_CHAIN_ID).unwrap()
}

#[tokio::test]
async fn test_create_client() {
    let client = ClobClient::polygon()
        .with_signer(TEST_PRIVATE_KEY)
        .unwrap()
        .with_funder(EXPECTED_ADDRESS)
        .unwrap()
        .with_signature_type(POLY_PROXY)
        .with_creds(ApiCreds {
            api_key: "api_key".to_string(),
            api_secret: "api_secret".to_string(),
            api_passphrase: "api_passphrase".to_string(),
        });
    assert_eq!(client.address(), Some(EXPECTED_ADDRESS.to_string()));
}

#[tokio::test]
async fn test_create_order_decimal_accuracy_buy() {
    let signer = create_test_signer();
    let builder = OrderBuilder::new(signer);

    // BUY price=0.24, size=15
    let order_args = OrderArgs {
        token_id: "123".to_string(),
        price: 0.24,
        size: 15.0,
        side: Side::Buy,
        fee_rate_bps: 0,
        nonce: 0,
        expiration: 0,
        taker: ZERO_ADDRESS.to_string(),
    };
    let options = CreateOrderOptions {
        tick_size: TickSize::Size0_01,
        neg_risk: false,
    };

    let signed_order = builder.create_order(&order_args, &options).await.unwrap();
    assert_eq!(signed_order.maker_amount, "3600000");
    assert_eq!(signed_order.taker_amount, "15000000");
}

#[tokio::test]
async fn test_create_order_decimal_accuracy_sell() {
    let signer = create_test_signer();
    let builder = OrderBuilder::new(signer);

    // SELL price=0.24, size=15
    let order_args = OrderArgs {
        token_id: "123".to_string(),
        price: 0.24,
        size: 15.0,
        side: Side::Sell,
        fee_rate_bps: 0,
        nonce: 0,
        expiration: 0,
        taker: ZERO_ADDRESS.to_string(),
    };
    let options = CreateOrderOptions {
        tick_size: TickSize::Size0_01,
        neg_risk: false,
    };

    let signed_order = builder.create_order(&order_args, &options).await.unwrap();
    assert_eq!(signed_order.maker_amount, "15000000");
    assert_eq!(signed_order.taker_amount, "3600000");
}

#[tokio::test]
async fn test_create_order_decimal_accuracy_buy_82() {
    let signer = create_test_signer();
    let builder = OrderBuilder::new(signer);

    // BUY price=0.82, size=101
    let order_args = OrderArgs {
        token_id: "123".to_string(),
        price: 0.82,
        size: 101.0,
        side: Side::Buy,
        fee_rate_bps: 0,
        nonce: 0,
        expiration: 0,
        taker: ZERO_ADDRESS.to_string(),
    };
    let options = CreateOrderOptions {
        tick_size: TickSize::Size0_01,
        neg_risk: false,
    };

    let signed_order = builder.create_order(&order_args, &options).await.unwrap();
    assert_eq!(signed_order.maker_amount, "82820000");
    assert_eq!(signed_order.taker_amount, "101000000");
}

#[tokio::test]
async fn test_create_order_decimal_accuracy_sell_82() {
    let signer = create_test_signer();
    let builder = OrderBuilder::new(signer);

    // SELL price=0.82, size=101
    let order_args = OrderArgs {
        token_id: "123".to_string(),
        price: 0.82,
        size: 101.0,
        side: Side::Sell,
        fee_rate_bps: 0,
        nonce: 0,
        expiration: 0,
        taker: ZERO_ADDRESS.to_string(),
    };
    let options = CreateOrderOptions {
        tick_size: TickSize::Size0_01,
        neg_risk: false,
    };

    let signed_order = builder.create_order(&order_args, &options).await.unwrap();
    assert_eq!(signed_order.maker_amount, "101000000");
    assert_eq!(signed_order.taker_amount, "82820000");
}

#[tokio::test]
async fn test_create_order_decimal_accuracy_buy_78() {
    let signer = create_test_signer();
    let builder = OrderBuilder::new(signer);

    // BUY price=0.78, size=12.8205
    let order_args = OrderArgs {
        token_id: "123".to_string(),
        price: 0.78,
        size: 12.8205,
        side: Side::Buy,
        fee_rate_bps: 0,
        nonce: 0,
        expiration: 0,
        taker: ZERO_ADDRESS.to_string(),
    };
    let options = CreateOrderOptions {
        tick_size: TickSize::Size0_01,
        neg_risk: false,
    };

    let signed_order = builder.create_order(&order_args, &options).await.unwrap();
    assert_eq!(signed_order.maker_amount, "9999600");
    assert_eq!(signed_order.taker_amount, "12820000");
}

#[tokio::test]
async fn test_create_order_decimal_accuracy_sell_78() {
    let signer = create_test_signer();
    let builder = OrderBuilder::new(signer);

    // SELL price=0.78, size=12.8205
    let order_args = OrderArgs {
        token_id: "123".to_string(),
        price: 0.78,
        size: 12.8205,
        side: Side::Sell,
        fee_rate_bps: 0,
        nonce: 0,
        expiration: 0,
        taker: ZERO_ADDRESS.to_string(),
    };
    let options = CreateOrderOptions {
        tick_size: TickSize::Size0_01,
        neg_risk: false,
    };

    let signed_order = builder.create_order(&order_args, &options).await.unwrap();
    assert_eq!(signed_order.maker_amount, "12820000");
    assert_eq!(signed_order.taker_amount, "9999600");
}

#[tokio::test]
async fn test_create_order_sell_39() {
    let signer = create_test_signer();
    let builder = OrderBuilder::new(signer);

    // SELL price=0.39, size=2435.89
    let order_args = OrderArgs {
        token_id: "123".to_string(),
        price: 0.39,
        size: 2435.89,
        side: Side::Sell,
        fee_rate_bps: 0,
        nonce: 0,
        expiration: 0,
        taker: ZERO_ADDRESS.to_string(),
    };
    let options = CreateOrderOptions {
        tick_size: TickSize::Size0_01,
        neg_risk: false,
    };

    let signed_order = builder.create_order(&order_args, &options).await.unwrap();
    assert_eq!(signed_order.maker_amount, "2435890000");
    assert_eq!(signed_order.taker_amount, "949997100");
}

#[tokio::test]
async fn test_create_order_sell_43() {
    let signer = create_test_signer();
    let builder = OrderBuilder::new(signer);

    // SELL price=0.43, size=19.1
    let order_args = OrderArgs {
        token_id: "123".to_string(),
        price: 0.43,
        size: 19.1,
        side: Side::Sell,
        fee_rate_bps: 0,
        nonce: 0,
        expiration: 0,
        taker: ZERO_ADDRESS.to_string(),
    };
    let options = CreateOrderOptions {
        tick_size: TickSize::Size0_01,
        neg_risk: false,
    };

    let signed_order = builder.create_order(&order_args, &options).await.unwrap();
    assert_eq!(signed_order.maker_amount, "19100000");
    assert_eq!(signed_order.taker_amount, "8213000");
}

#[tokio::test]
async fn test_create_order_buy_58() {
    let signer = create_test_signer();
    let builder = OrderBuilder::new(signer);

    // BUY price=0.58, size=18233.33
    let order_args = OrderArgs {
        token_id: "123".to_string(),
        price: 0.58,
        size: 18233.33,
        side: Side::Buy,
        fee_rate_bps: 0,
        nonce: 0,
        expiration: 0,
        taker: ZERO_ADDRESS.to_string(),
    };
    let options = CreateOrderOptions {
        tick_size: TickSize::Size0_01,
        neg_risk: false,
    };

    let signed_order = builder.create_order(&order_args, &options).await.unwrap();
    assert_eq!(signed_order.maker_amount, "10575331400");
    assert_eq!(signed_order.taker_amount, "18233330000");

    // Check ratio equals 0.58
    let maker: f64 = signed_order.maker_amount.parse().unwrap();
    let taker: f64 = signed_order.taker_amount.parse().unwrap();
    assert_eq!(maker / taker, 0.58);
}

#[tokio::test]
async fn test_create_order_buy_0_1() {
    let signer = create_test_signer();
    let builder = OrderBuilder::new(signer);

    let order_args = OrderArgs {
        token_id: "123".to_string(),
        price: 0.5,
        size: 21.04,
        side: Side::Buy,
        fee_rate_bps: 111,
        nonce: 123,
        expiration: 50000,
        taker: ZERO_ADDRESS.to_string(),
    };
    let options = CreateOrderOptions {
        tick_size: TickSize::Size0_1,
        neg_risk: false,
    };

    let signed_order = builder.create_order(&order_args, &options).await.unwrap();

    assert!(!signed_order.salt.is_empty());
    assert_eq!(signed_order.maker, EXPECTED_ADDRESS);
    assert_eq!(signed_order.signer, EXPECTED_ADDRESS);
    assert_eq!(signed_order.taker, ZERO_ADDRESS);
    assert_eq!(signed_order.token_id, "123");
    assert_eq!(signed_order.maker_amount, "10520000");
    assert_eq!(signed_order.taker_amount, "21040000");
    assert_eq!(signed_order.side, 0); // BUY
    assert_eq!(signed_order.expiration, "50000");
    assert_eq!(signed_order.nonce, "123");
    assert_eq!(signed_order.fee_rate_bps, "111");
    assert_eq!(signed_order.signature_type, 0); // EOA
    assert!(!signed_order.signature.is_empty());

    // Verify ratio
    let maker: f64 = signed_order.maker_amount.parse().unwrap();
    let taker: f64 = signed_order.taker_amount.parse().unwrap();
    assert_eq!(maker / taker, 0.5);
}

#[tokio::test]
async fn test_create_order_buy_0_01() {
    let signer = create_test_signer();
    let builder = OrderBuilder::new(signer);

    let order_args = OrderArgs {
        token_id: "123".to_string(),
        price: 0.56,
        size: 21.04,
        side: Side::Buy,
        fee_rate_bps: 111,
        nonce: 123,
        expiration: 50000,
        taker: ZERO_ADDRESS.to_string(),
    };
    let options = CreateOrderOptions {
        tick_size: TickSize::Size0_01,
        neg_risk: false,
    };

    let signed_order = builder.create_order(&order_args, &options).await.unwrap();

    assert_eq!(signed_order.maker_amount, "11782400");
    assert_eq!(signed_order.taker_amount, "21040000");
    assert_eq!(signed_order.side, 0);

    let maker: f64 = signed_order.maker_amount.parse().unwrap();
    let taker: f64 = signed_order.taker_amount.parse().unwrap();
    assert_eq!(maker / taker, 0.56);
}

#[tokio::test]
async fn test_create_order_buy_0_001() {
    let signer = create_test_signer();
    let builder = OrderBuilder::new(signer);

    let order_args = OrderArgs {
        token_id: "123".to_string(),
        price: 0.056,
        size: 21.04,
        side: Side::Buy,
        fee_rate_bps: 111,
        nonce: 123,
        expiration: 50000,
        taker: ZERO_ADDRESS.to_string(),
    };
    let options = CreateOrderOptions {
        tick_size: TickSize::Size0_001,
        neg_risk: false,
    };

    let signed_order = builder.create_order(&order_args, &options).await.unwrap();

    assert_eq!(signed_order.maker_amount, "1178240");
    assert_eq!(signed_order.taker_amount, "21040000");
    assert_eq!(signed_order.side, 0);

    let maker: f64 = signed_order.maker_amount.parse().unwrap();
    let taker: f64 = signed_order.taker_amount.parse().unwrap();
    assert_eq!(maker / taker, 0.056);
}

#[tokio::test]
async fn test_create_order_buy_0_0001() {
    let signer = create_test_signer();
    let builder = OrderBuilder::new(signer);

    let order_args = OrderArgs {
        token_id: "123".to_string(),
        price: 0.0056,
        size: 21.04,
        side: Side::Buy,
        fee_rate_bps: 111,
        nonce: 123,
        expiration: 50000,
        taker: ZERO_ADDRESS.to_string(),
    };
    let options = CreateOrderOptions {
        tick_size: TickSize::Size0_0001,
        neg_risk: false,
    };

    let signed_order = builder.create_order(&order_args, &options).await.unwrap();

    assert_eq!(signed_order.maker_amount, "117824");
    assert_eq!(signed_order.taker_amount, "21040000");
    assert_eq!(signed_order.side, 0);

    let maker: f64 = signed_order.maker_amount.parse().unwrap();
    let taker: f64 = signed_order.taker_amount.parse().unwrap();
    assert_eq!(maker / taker, 0.0056);
}

#[tokio::test]
async fn test_create_order_sell_0_1() {
    let signer = create_test_signer();
    let builder = OrderBuilder::new(signer);

    let order_args = OrderArgs {
        token_id: "123".to_string(),
        price: 0.5,
        size: 21.04,
        side: Side::Sell,
        fee_rate_bps: 111,
        nonce: 123,
        expiration: 50000,
        taker: ZERO_ADDRESS.to_string(),
    };
    let options = CreateOrderOptions {
        tick_size: TickSize::Size0_1,
        neg_risk: false,
    };

    let signed_order = builder.create_order(&order_args, &options).await.unwrap();

    assert_eq!(signed_order.maker, EXPECTED_ADDRESS);
    assert_eq!(signed_order.maker_amount, "21040000");
    assert_eq!(signed_order.taker_amount, "10520000");
    assert_eq!(signed_order.side, 1); // SELL

    let maker: f64 = signed_order.maker_amount.parse().unwrap();
    let taker: f64 = signed_order.taker_amount.parse().unwrap();
    assert_eq!(taker / maker, 0.5);
}

#[tokio::test]
async fn test_create_order_sell_0_01() {
    let signer = create_test_signer();
    let builder = OrderBuilder::new(signer);

    let order_args = OrderArgs {
        token_id: "123".to_string(),
        price: 0.56,
        size: 21.04,
        side: Side::Sell,
        fee_rate_bps: 111,
        nonce: 123,
        expiration: 50000,
        taker: ZERO_ADDRESS.to_string(),
    };
    let options = CreateOrderOptions {
        tick_size: TickSize::Size0_01,
        neg_risk: false,
    };

    let signed_order = builder.create_order(&order_args, &options).await.unwrap();

    assert_eq!(signed_order.maker_amount, "21040000");
    assert_eq!(signed_order.taker_amount, "11782400");
    assert_eq!(signed_order.side, 1);

    let maker: f64 = signed_order.maker_amount.parse().unwrap();
    let taker: f64 = signed_order.taker_amount.parse().unwrap();
    assert_eq!(taker / maker, 0.56);
}

#[tokio::test]
async fn test_create_order_sell_0_001() {
    let signer = create_test_signer();
    let builder = OrderBuilder::new(signer);

    let order_args = OrderArgs {
        token_id: "123".to_string(),
        price: 0.056,
        size: 21.04,
        side: Side::Sell,
        fee_rate_bps: 111,
        nonce: 123,
        expiration: 50000,
        taker: ZERO_ADDRESS.to_string(),
    };
    let options = CreateOrderOptions {
        tick_size: TickSize::Size0_001,
        neg_risk: false,
    };

    let signed_order = builder.create_order(&order_args, &options).await.unwrap();

    assert_eq!(signed_order.maker_amount, "21040000");
    assert_eq!(signed_order.taker_amount, "1178240");
    assert_eq!(signed_order.side, 1);

    let maker: f64 = signed_order.maker_amount.parse().unwrap();
    let taker: f64 = signed_order.taker_amount.parse().unwrap();
    assert_eq!(taker / maker, 0.056);
}

#[tokio::test]
async fn test_create_order_sell_0_0001() {
    let signer = create_test_signer();
    let builder = OrderBuilder::new(signer);

    let order_args = OrderArgs {
        token_id: "123".to_string(),
        price: 0.0056,
        size: 21.04,
        side: Side::Sell,
        fee_rate_bps: 111,
        nonce: 123,
        expiration: 50000,
        taker: ZERO_ADDRESS.to_string(),
    };
    let options = CreateOrderOptions {
        tick_size: TickSize::Size0_0001,
        neg_risk: false,
    };

    let signed_order = builder.create_order(&order_args, &options).await.unwrap();

    assert_eq!(signed_order.maker_amount, "21040000");
    assert_eq!(signed_order.taker_amount, "117824");
    assert_eq!(signed_order.side, 1);

    let maker: f64 = signed_order.maker_amount.parse().unwrap();
    let taker: f64 = signed_order.taker_amount.parse().unwrap();
    assert_eq!(taker / maker, 0.0056);
}

#[tokio::test]
async fn test_create_order_decimal_accuracy_neg_risk_buy() {
    let signer = create_test_signer();
    let builder = OrderBuilder::new(signer);

    let order_args = OrderArgs {
        token_id: "123".to_string(),
        price: 0.24,
        size: 15.0,
        side: Side::Buy,
        fee_rate_bps: 0,
        nonce: 0,
        expiration: 0,
        taker: ZERO_ADDRESS.to_string(),
    };
    let options = CreateOrderOptions {
        tick_size: TickSize::Size0_01,
        neg_risk: true,
    };

    let signed_order = builder.create_order(&order_args, &options).await.unwrap();
    assert_eq!(signed_order.maker_amount, "3600000");
    assert_eq!(signed_order.taker_amount, "15000000");
}

#[tokio::test]
async fn test_create_order_decimal_accuracy_neg_risk_sell() {
    let signer = create_test_signer();
    let builder = OrderBuilder::new(signer);

    let order_args = OrderArgs {
        token_id: "123".to_string(),
        price: 0.24,
        size: 15.0,
        side: Side::Sell,
        fee_rate_bps: 0,
        nonce: 0,
        expiration: 0,
        taker: ZERO_ADDRESS.to_string(),
    };
    let options = CreateOrderOptions {
        tick_size: TickSize::Size0_01,
        neg_risk: true,
    };

    let signed_order = builder.create_order(&order_args, &options).await.unwrap();
    assert_eq!(signed_order.maker_amount, "15000000");
    assert_eq!(signed_order.taker_amount, "3600000");
}

#[tokio::test]
async fn test_create_order_buy_0_1_neg_risk() {
    let signer = create_test_signer();
    let builder = OrderBuilder::new(signer);

    let order_args = OrderArgs {
        token_id: "123".to_string(),
        price: 0.5,
        size: 21.04,
        side: Side::Buy,
        fee_rate_bps: 111,
        nonce: 123,
        expiration: 50000,
        taker: ZERO_ADDRESS.to_string(),
    };
    let options = CreateOrderOptions {
        tick_size: TickSize::Size0_1,
        neg_risk: true,
    };

    let signed_order = builder.create_order(&order_args, &options).await.unwrap();

    assert_eq!(signed_order.maker_amount, "10520000");
    assert_eq!(signed_order.taker_amount, "21040000");
    assert_eq!(signed_order.side, 0);

    let maker: f64 = signed_order.maker_amount.parse().unwrap();
    let taker: f64 = signed_order.taker_amount.parse().unwrap();
    assert_eq!(maker / taker, 0.5);
}

#[tokio::test]
async fn test_create_order_sell_0_1_neg_risk() {
    let signer = create_test_signer();
    let builder = OrderBuilder::new(signer);

    let order_args = OrderArgs {
        token_id: "123".to_string(),
        price: 0.5,
        size: 21.04,
        side: Side::Sell,
        fee_rate_bps: 111,
        nonce: 123,
        expiration: 50000,
        taker: ZERO_ADDRESS.to_string(),
    };
    let options = CreateOrderOptions {
        tick_size: TickSize::Size0_1,
        neg_risk: true,
    };

    let signed_order = builder.create_order(&order_args, &options).await.unwrap();

    assert_eq!(signed_order.maker_amount, "21040000");
    assert_eq!(signed_order.taker_amount, "10520000");
    assert_eq!(signed_order.side, 1);

    let maker: f64 = signed_order.maker_amount.parse().unwrap();
    let taker: f64 = signed_order.taker_amount.parse().unwrap();
    assert_eq!(taker / maker, 0.5);
}

#[test]
fn test_order_args_new() {
    let args = OrderArgs::new("123", 0.5, 100.0, Side::Buy);
    assert_eq!(args.token_id, "123");
    assert_eq!(args.price, 0.5);
    assert_eq!(args.size, 100.0);
    assert_eq!(args.side, Side::Buy);
    assert_eq!(args.fee_rate_bps, 0);
    assert_eq!(args.nonce, 0);
    assert_eq!(args.expiration, 0);
    assert_eq!(args.taker, ZERO_ADDRESS);
}

#[test]
fn test_tick_size_as_str() {
    assert_eq!(TickSize::Size0_1.as_str(), "0.1");
    assert_eq!(TickSize::Size0_01.as_str(), "0.01");
    assert_eq!(TickSize::Size0_001.as_str(), "0.001");
    assert_eq!(TickSize::Size0_0001.as_str(), "0.0001");
}

#[test]
fn test_tick_size_as_f64() {
    assert_eq!(TickSize::Size0_1.as_f64(), 0.1);
    assert_eq!(TickSize::Size0_01.as_f64(), 0.01);
    assert_eq!(TickSize::Size0_001.as_f64(), 0.001);
    assert_eq!(TickSize::Size0_0001.as_f64(), 0.0001);
}

#[test]
fn test_tick_size_from_str() {
    assert_eq!("0.1".parse::<TickSize>().unwrap(), TickSize::Size0_1);
    assert_eq!("0.01".parse::<TickSize>().unwrap(), TickSize::Size0_01);
    assert_eq!("0.001".parse::<TickSize>().unwrap(), TickSize::Size0_001);
    assert_eq!("0.0001".parse::<TickSize>().unwrap(), TickSize::Size0_0001);
    assert!("invalid".parse::<TickSize>().is_err());
}

#[test]
fn test_tick_size_display() {
    assert_eq!(format!("{}", TickSize::Size0_1), "0.1");
    assert_eq!(format!("{}", TickSize::Size0_01), "0.01");
    assert_eq!(format!("{}", TickSize::Size0_001), "0.001");
    assert_eq!(format!("{}", TickSize::Size0_0001), "0.0001");
}

#[test]
fn test_side_display() {
    assert_eq!(format!("{}", Side::Buy), "BUY");
    assert_eq!(format!("{}", Side::Sell), "SELL");
}

#[test]
fn test_order_summary() {
    use clob_rs::OrderSummary;

    let summary = OrderSummary {
        price: "0.5".to_string(),
        size: "100".to_string(),
    };
    assert_eq!(summary.price_f64(), 0.5);
    assert_eq!(summary.size_f64(), 100.0);
}

#[test]
fn test_order_book_helpers() {
    use clob_rs::{OrderBook, OrderSummary};

    let book = OrderBook {
        market: "test".to_string(),
        asset_id: "123".to_string(),
        timestamp: "1234567890".to_string(),
        hash: "abc".to_string(),
        bids: vec![
            OrderSummary {
                price: "0.5".to_string(),
                size: "100".to_string(),
            },
            OrderSummary {
                price: "0.4".to_string(),
                size: "200".to_string(),
            },
        ],
        asks: vec![
            OrderSummary {
                price: "0.6".to_string(),
                size: "150".to_string(),
            },
            OrderSummary {
                price: "0.7".to_string(),
                size: "50".to_string(),
            },
        ],
        min_order_size: Some("10".to_string()),
        tick_size: Some("0.01".to_string()),
        neg_risk: Some(false),
    };

    let best_bid = book.best_bid().unwrap();
    assert_eq!(best_bid.price_f64(), 0.5);

    let best_ask = book.best_ask().unwrap();
    assert_eq!(best_ask.price_f64(), 0.6);

    let spread = book.spread().unwrap();
    assert!((spread - 0.1).abs() < 1e-10);

    let midpoint = book.midpoint().unwrap();
    assert!((midpoint - 0.55).abs() < 1e-10);
}

#[test]
fn test_order_book_empty() {
    use clob_rs::OrderBook;

    let book = OrderBook {
        market: "test".to_string(),
        asset_id: "123".to_string(),
        timestamp: "1234567890".to_string(),
        hash: "abc".to_string(),
        bids: vec![],
        asks: vec![],
        min_order_size: None,
        tick_size: None,
        neg_risk: None,
    };

    assert!(book.best_bid().is_none());
    assert!(book.best_ask().is_none());
    assert!(book.spread().is_none());
    assert!(book.midpoint().is_none());
}
