use std::collections::BTreeMap;

use serde::Serialize;

use crate::{AccountId, Amount, AssetId, Bps, NexusError, NexusResult};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct FeePolicy {
    pub recipient: AccountId,
    pub solver_rebate_bps: Bps,
    pub protocol_fee_bps: Bps,
    pub insurance_take_bps: Bps,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct InsuranceFund {
    pub reserve_asset: AssetId,
    pub balance: Amount,
    pub floor: Amount,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct TreasuryBook {
    policy: Option<FeePolicy>,
    insurance: Option<InsuranceFund>,
    accrued: BTreeMap<AssetId, Amount>,
}

impl FeePolicy {
    pub fn new(
        recipient: AccountId,
        solver_rebate_bps: Bps,
        protocol_fee_bps: Bps,
        insurance_take_bps: Bps,
    ) -> NexusResult<Self> {
        let combined = u32::from(solver_rebate_bps.units())
            + u32::from(protocol_fee_bps.units())
            + u32::from(insurance_take_bps.units());
        if combined > 10_000 {
            return Err(NexusError::Policy(
                "fee policy exceeds basis point range".to_owned(),
            ));
        }
        Ok(Self {
            recipient,
            solver_rebate_bps,
            protocol_fee_bps,
            insurance_take_bps,
        })
    }
}

impl InsuranceFund {
    pub fn new(reserve_asset: AssetId, balance: Amount, floor: Amount) -> NexusResult<Self> {
        if floor > balance {
            return Err(NexusError::Policy(
                "insurance floor exceeds balance".to_owned(),
            ));
        }
        Ok(Self {
            reserve_asset,
            balance,
            floor,
        })
    }
}

impl TreasuryBook {
    pub fn configure_policy(&mut self, policy: FeePolicy) {
        self.policy = Some(policy);
    }

    pub fn configure_insurance(&mut self, fund: InsuranceFund) {
        self.insurance = Some(fund);
    }

    pub fn protocol_fee_claim(&self, notional: Amount) -> NexusResult<Amount> {
        self.policy
            .map(|policy| notional.checked_mul_bps(policy.protocol_fee_bps))
            .unwrap_or_else(|| Ok(Amount::zero()))
    }

    pub fn insurance_claim(&self, notional: Amount) -> NexusResult<Amount> {
        self.policy
            .map(|policy| notional.checked_mul_bps(policy.insurance_take_bps))
            .unwrap_or_else(|| Ok(Amount::zero()))
    }

    pub fn record_accrual(&mut self, asset: AssetId, amount: Amount) -> NexusResult<()> {
        if amount.is_zero() {
            return Ok(());
        }
        let current = self
            .accrued
            .get(&asset)
            .copied()
            .unwrap_or_else(Amount::zero);
        self.accrued.insert(asset, current.checked_add(amount)?);
        Ok(())
    }

    pub fn absorb_insurance(&mut self, asset: AssetId, amount: Amount) -> NexusResult<()> {
        if amount.is_zero() {
            return Ok(());
        }
        let Some(mut fund) = self.insurance else {
            return Ok(());
        };
        if fund.reserve_asset == asset {
            fund.balance = fund.balance.checked_add(amount)?;
            self.insurance = Some(fund);
        }
        Ok(())
    }

    pub fn accrued_asset_count(&self) -> usize {
        self.accrued.len()
    }
}
