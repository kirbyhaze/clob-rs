use alloy_primitives::{keccak256, Address, B256, U256};
use alloy_sol_types::sol;

use crate::error::Result;
use crate::signer::Signer;

const CLOB_DOMAIN_NAME: &str = "ClobAuthDomain";
const CLOB_VERSION: &str = "1";
const MSG_TO_SIGN: &str = "This message attests that I control the given wallet";

sol! {
    struct EIP712Domain {
        string name;
        string version;
        uint256 chainId;
    }

    struct ClobAuth {
        address address_;
        string timestamp;
        uint256 nonce;
        string message;
    }
}

fn domain_separator(chain_id: u64) -> B256 {
    let domain_type_hash = keccak256("EIP712Domain(string name,string version,uint256 chainId)");

    let name_hash = keccak256(CLOB_DOMAIN_NAME);
    let version_hash = keccak256(CLOB_VERSION);
    let chain_id_bytes = U256::from(chain_id);

    let encoded = [
        domain_type_hash.as_slice(),
        name_hash.as_slice(),
        version_hash.as_slice(),
        &chain_id_bytes.to_be_bytes::<32>(),
    ]
    .concat();

    keccak256(&encoded)
}

fn struct_hash(address: Address, timestamp: u64, nonce: u64) -> B256 {
    let type_hash =
        keccak256("ClobAuth(address address,string timestamp,uint256 nonce,string message)");

    let timestamp_hash = keccak256(timestamp.to_string());
    let message_hash = keccak256(MSG_TO_SIGN);
    let nonce_bytes = U256::from(nonce);

    let mut address_padded = [0u8; 32];
    address_padded[12..].copy_from_slice(address.as_slice());

    let encoded = [
        type_hash.as_slice(),
        &address_padded,
        timestamp_hash.as_slice(),
        &nonce_bytes.to_be_bytes::<32>(),
        message_hash.as_slice(),
    ]
    .concat();

    keccak256(&encoded)
}

pub async fn sign_clob_auth_message(signer: &Signer, timestamp: u64, nonce: u64) -> Result<String> {
    let domain_sep = domain_separator(signer.chain_id());
    let struct_h = struct_hash(signer.address(), timestamp, nonce);

    // EIP-712: "\x19\x01" + domain_separator + struct_hash
    let mut message = Vec::with_capacity(66);
    message.extend_from_slice(&[0x19, 0x01]);
    message.extend_from_slice(domain_sep.as_slice());
    message.extend_from_slice(struct_h.as_slice());

    let hash = keccak256(&message);
    signer.sign_hash(hash).await
}
