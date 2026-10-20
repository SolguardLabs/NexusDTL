use serde::Serialize;

use crate::{Amount, Bps, LiquidityVault, NexusResult};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct LiquidityStress {
    pub haircut_bps: Bps,
    pub opening_reserve: Amount,
    pub shock_loss: Amount,
    pub stressed_reserve: Amount,
    pub reserve_floor: Amount,
    pub accrued_fees: Amount,
    pub operating_buffer: Amount,
    pub floor_covered: bool,
}

impl LiquidityStress {
    pub fn evaluate(vault: &LiquidityVault, haircut_bps: Bps) -> NexusResult<Self> {
        let retained_bps = Bps::new(10_000 - haircut_bps.units())?;
        let stressed_reserve = vault.reserve_balance.checked_mul_bps(retained_bps)?;
        let shock_loss = vault.reserve_balance.checked_sub(stressed_reserve)?;
        let reserve_floor = vault.config.reserve_floor;
        let floor_covered = stressed_reserve >= reserve_floor;
        let after_floor = if floor_covered {
            stressed_reserve.checked_sub(reserve_floor)?
        } else {
            Amount::zero()
        };
        let operating_buffer = if after_floor >= vault.pending_fees {
            after_floor.checked_sub(vault.pending_fees)?
        } else {
            Amount::zero()
        };

        Ok(Self {
            haircut_bps,
            opening_reserve: vault.reserve_balance,
            shock_loss,
            stressed_reserve,
            reserve_floor,
            accrued_fees: vault.pending_fees,
            operating_buffer,
            floor_covered,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AccountId, AssetId, Digest, VaultConfig};

    #[test]
    fn stress_window_respects_floor_and_haircut() {
        let owner = AccountId::from_bytes([1; 32]);
        let asset = AssetId::from_bytes([2; 32]);
        let config = VaultConfig::new(
            owner,
            asset,
            3,
            Bps::new(20).unwrap(),
            Amount::new(8_000_000_000).unwrap(),
            Digest::from_bytes([3; 32]),
        )
        .unwrap();
        let mut vault = LiquidityVault::new(config);
        vault
            .deposit(owner, Amount::new(90_000_000_000).unwrap())
            .unwrap();

        let moderate = LiquidityStress::evaluate(&vault, Bps::new(1_000).unwrap()).unwrap();
        assert_eq!(moderate.stressed_reserve.units(), 81_000_000_000);
        assert_eq!(moderate.operating_buffer.units(), 73_000_000_000);
        assert!(moderate.floor_covered);

        let severe = LiquidityStress::evaluate(&vault, Bps::new(9_500).unwrap()).unwrap();
        assert_eq!(severe.stressed_reserve.units(), 4_500_000_000);
        assert_eq!(severe.operating_buffer.units(), 0);
        assert!(!severe.floor_covered);
    }
}
