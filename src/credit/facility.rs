use serde::Serialize;

use crate::{AccountId, Amount, AssetId, Bps, FacilityId, NexusError, NexusResult, VaultId};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CreditFacility {
    pub facility_id: FacilityId,
    pub vault_id: VaultId,
    pub borrower: AccountId,
    pub collateral_asset: AssetId,
    pub borrow_asset: AssetId,
    pub collateral_amount: Amount,
    pub debt_ceiling: Amount,
    pub annual_interest_bps: Bps,
    pub maturity_epoch: u64,
    pub enabled: bool,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CreditPosition {
    pub facility_id: FacilityId,
    pub borrower: AccountId,
    pub principal: Amount,
    pub accrued_interest: Amount,
    pub last_accrual_epoch: u64,
}

impl CreditFacility {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        vault_id: VaultId,
        borrower: AccountId,
        collateral_asset: AssetId,
        borrow_asset: AssetId,
        collateral_amount: Amount,
        debt_ceiling: Amount,
        annual_interest_bps: Bps,
        maturity_epoch: u64,
    ) -> NexusResult<Self> {
        if collateral_amount.is_zero() || debt_ceiling.is_zero() {
            return Err(NexusError::ZeroAmount);
        }
        Ok(Self {
            facility_id: FacilityId::derive(vault_id, borrower, collateral_asset),
            vault_id,
            borrower,
            collateral_asset,
            borrow_asset,
            collateral_amount,
            debt_ceiling,
            annual_interest_bps,
            maturity_epoch,
            enabled: true,
        })
    }
}

impl CreditPosition {
    pub fn new(facility_id: FacilityId, borrower: AccountId, epoch: u64) -> Self {
        Self {
            facility_id,
            borrower,
            principal: Amount::zero(),
            accrued_interest: Amount::zero(),
            last_accrual_epoch: epoch,
        }
    }

    pub fn total_debt(self) -> NexusResult<Amount> {
        self.principal.checked_add(self.accrued_interest)
    }
}
