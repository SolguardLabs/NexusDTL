use std::collections::BTreeMap;

use serde::Serialize;

use crate::{Amount, CreditFacility, CreditPosition, FacilityId, NexusError, NexusResult, VaultId};

#[derive(Clone, Debug, Default, Serialize)]
pub struct CreditBook {
    facilities: BTreeMap<FacilityId, CreditFacility>,
    positions: BTreeMap<FacilityId, CreditPosition>,
}

impl CreditBook {
    pub fn open_facility(
        &mut self,
        facility: CreditFacility,
        current_epoch: u64,
    ) -> NexusResult<()> {
        if self.facilities.contains_key(&facility.facility_id) {
            return Err(NexusError::Policy("facility already exists".to_owned()));
        }
        self.positions.insert(
            facility.facility_id,
            CreditPosition::new(facility.facility_id, facility.borrower, current_epoch),
        );
        self.facilities.insert(facility.facility_id, facility);
        Ok(())
    }

    pub fn facility(&self, facility_id: FacilityId) -> NexusResult<CreditFacility> {
        self.facilities
            .get(&facility_id)
            .copied()
            .ok_or(NexusError::FacilityNotFound(facility_id))
    }

    pub fn position(&self, facility_id: FacilityId) -> NexusResult<CreditPosition> {
        self.positions
            .get(&facility_id)
            .copied()
            .ok_or(NexusError::FacilityNotFound(facility_id))
    }

    pub fn draw(&mut self, facility_id: FacilityId, amount: Amount) -> NexusResult<()> {
        if amount.is_zero() {
            return Err(NexusError::ZeroAmount);
        }
        let facility = self.facility(facility_id)?;
        if !facility.enabled {
            return Err(NexusError::Policy("facility disabled".to_owned()));
        }
        let mut position = self.position(facility_id)?;
        let next_principal = position.principal.checked_add(amount)?;
        if next_principal > facility.debt_ceiling {
            return Err(NexusError::Policy(
                "facility debt ceiling exceeded".to_owned(),
            ));
        }
        position.principal = next_principal;
        self.positions.insert(facility_id, position);
        Ok(())
    }

    pub fn accrue(&mut self, facility_id: FacilityId, current_epoch: u64) -> NexusResult<Amount> {
        let facility = self.facility(facility_id)?;
        let mut position = self.position(facility_id)?;
        if current_epoch <= position.last_accrual_epoch || position.principal.is_zero() {
            return Ok(Amount::zero());
        }
        let elapsed = current_epoch - position.last_accrual_epoch;
        let interest = position.principal.checked_mul_ratio(
            u128::from(facility.annual_interest_bps.units()) * u128::from(elapsed),
            10_000 * 365,
        )?;
        position.accrued_interest = position.accrued_interest.checked_add(interest)?;
        position.last_accrual_epoch = current_epoch;
        self.positions.insert(facility_id, position);
        Ok(interest)
    }

    pub fn vault_debt(&self, vault_id: VaultId) -> NexusResult<Amount> {
        self.facilities
            .values()
            .filter(|facility| facility.vault_id == vault_id)
            .try_fold(Amount::zero(), |accumulator, facility| {
                let position = self.position(facility.facility_id)?;
                accumulator.checked_add(position.total_debt()?)
            })
    }

    pub fn facility_count(&self) -> usize {
        self.facilities.len()
    }
}
