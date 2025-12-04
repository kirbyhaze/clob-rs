// https://github.com/Polymarket/py-clob-client/blob/main/py_clob_client/config.py
use crate::types::ContractConfig;

pub const HOST: &str = "https://clob.polymarket.com";
pub const CHAIN_ID: u64 = 137;

pub const END_CURSOR: &str = "LTE=";
pub const FIRST_CURSOR: &str = "MA==";

const POLYGON_CONFIG: ContractConfig = ContractConfig {
    exchange: "0x4bFb41d5B3570DeFd03C39a9A4D8dE6Bd8B8982E",
    collateral: "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174",
    conditional_tokens: "0x4D97DCd97eC945f40cF65F87097ACe5EA0476045",
};

const POLYGON_NEG_RISK_CONFIG: ContractConfig = ContractConfig {
    exchange: "0xC5d563A36AE78145C45a50134d48A1215220f80a",
    collateral: "0x2791bca1f2de4661ed88a30c99a7a9449aa84174",
    conditional_tokens: "0x4D97DCd97eC945f40cF65F87097ACe5EA0476045",
};

pub fn get_contract_config(chain_id: u64, neg_risk: bool) -> Option<&'static ContractConfig> {
    match (chain_id, neg_risk) {
        (137, false) => Some(&POLYGON_CONFIG),
        (137, true) => Some(&POLYGON_NEG_RISK_CONFIG),
        _ => None,
    }
}
