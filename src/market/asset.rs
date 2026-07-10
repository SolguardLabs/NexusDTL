use serde::Serialize;

use crate::{AssetId, NexusError, NexusResult};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct AssetConfig {
    pub id: AssetId,
    pub symbol: &'static str,
    pub decimals: u8,
    pub settlement_enabled: bool,
}

impl AssetConfig {
    pub fn new(symbol: &'static str, decimals: u8) -> NexusResult<Self> {
        if decimals > 18 {
            return Err(NexusError::Policy("asset decimals exceed limit".to_owned()));
        }
        Ok(Self {
            id: AssetId::derive(symbol, decimals),
            symbol,
            decimals,
            settlement_enabled: true,
        })
    }
}
