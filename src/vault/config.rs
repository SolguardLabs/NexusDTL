use serde::Serialize;

use crate::{AccountId, Amount, AssetId, Bps, Digest, NexusError, NexusResult, VaultId};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct VaultConfig {
    pub vault_id: VaultId,
    pub controller: AccountId,
    pub reserve_asset: AssetId,
    pub exit_delay_epochs: u64,
    pub management_fee_bps: Bps,
    pub reserve_floor: Amount,
    pub enabled: bool,
}

impl VaultConfig {
    pub fn new(
        controller: AccountId,
        reserve_asset: AssetId,
        exit_delay_epochs: u64,
        management_fee_bps: Bps,
        reserve_floor: Amount,
        salt: Digest,
    ) -> NexusResult<Self> {
        if reserve_floor.is_zero() {
            return Err(NexusError::Policy("vault reserve floor is zero".to_owned()));
        }
        Ok(Self {
            vault_id: VaultId::derive(controller, reserve_asset, salt),
            controller,
            reserve_asset,
            exit_delay_epochs,
            management_fee_bps,
            reserve_floor,
            enabled: true,
        })
    }
}
