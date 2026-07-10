use serde::Serialize;

use crate::{Amount, Bps, NexusError, NexusResult};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RiskLimits {
    pub current_epoch: u64,
    pub max_batch_notional: Amount,
    pub max_vault_utilization_bps: Bps,
    pub min_reserve_after_batch: Amount,
    pub max_solver_fee_bps: Bps,
    pub max_open_facilities: usize,
}

impl RiskLimits {
    pub fn new(
        current_epoch: u64,
        max_batch_notional: Amount,
        max_vault_utilization_bps: Bps,
        min_reserve_after_batch: Amount,
        max_solver_fee_bps: Bps,
        max_open_facilities: usize,
    ) -> NexusResult<Self> {
        if max_batch_notional.is_zero() || min_reserve_after_batch.is_zero() {
            return Err(NexusError::ZeroAmount);
        }
        if max_open_facilities == 0 {
            return Err(NexusError::Policy("max open facilities is zero".to_owned()));
        }
        Ok(Self {
            current_epoch,
            max_batch_notional,
            max_vault_utilization_bps,
            min_reserve_after_batch,
            max_solver_fee_bps,
            max_open_facilities,
        })
    }
}

impl Default for RiskLimits {
    fn default() -> Self {
        Self {
            current_epoch: 2_100,
            max_batch_notional: Amount::new(250_000_000_000).expect("valid amount"),
            max_vault_utilization_bps: Bps::new(8_500).expect("valid bps"),
            min_reserve_after_batch: Amount::new(10_000_000_000).expect("valid amount"),
            max_solver_fee_bps: Bps::new(120).expect("valid bps"),
            max_open_facilities: 16,
        }
    }
}
