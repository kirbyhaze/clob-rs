use alloy_primitives::{Address, B256};
use alloy_signer::Signer as AlloySigner;
use alloy_signer_local::PrivateKeySigner;

use crate::error::{ClobError, Result};

pub struct Signer {
    inner: PrivateKeySigner,
    chain_id: u64,
}

impl Signer {
    pub fn new(private_key: &str, chain_id: u64) -> Result<Self> {
        let key = private_key.strip_prefix("0x").unwrap_or(private_key);
        let inner: PrivateKeySigner = key.parse().map_err(|e| ClobError::Signing {
            message: format!("invalid private key: {}", e),
        })?;

        Ok(Self { inner, chain_id })
    }

    pub fn address(&self) -> Address {
        self.inner.address()
    }

    pub fn address_string(&self) -> String {
        self.inner.address().to_checksum(None)
    }

    pub fn chain_id(&self) -> u64 {
        self.chain_id
    }

    pub async fn sign_hash(&self, hash: B256) -> Result<String> {
        let sig = self
            .inner
            .sign_hash(&hash)
            .await
            .map_err(|e| ClobError::Signing {
                message: format!("failed to sign: {}", e),
            })?;

        Ok(format!("0x{}", hex::encode(sig.as_bytes())))
    }
}
