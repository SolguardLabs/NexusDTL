use serde::Serialize;

use crate::{Amount, BatchId, Bps, LiquidityVault, NexusError, NexusResult, RfqQuote, RiskLimits};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RiskSnapshot {
    pub batch_id: BatchId,
    pub gross_input: Amount,
    pub gross_output: Amount,
    pub projected_reserve: Amount,
    pub utilization_bps: Bps,
    pub facility_count: usize,
    pub accepted: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct RiskEngine {
    limits: RiskLimits,
}

impl RiskEngine {
    pub fn new(limits: RiskLimits) -> Self {
        Self { limits }
    }

    pub const fn limits(&self) -> RiskLimits {
        self.limits
    }

    pub fn set_limits(&mut self, limits: RiskLimits) {
        self.limits = limits;
    }

    pub fn evaluate_batch(
        &self,
        batch_id: BatchId,
        vault: &LiquidityVault,
        gross_input: Amount,
        gross_output: Amount,
        quote: RfqQuote,
        facility_count: usize,
    ) -> NexusResult<RiskSnapshot> {
        if gross_input > self.limits.max_batch_notional {
            return Err(NexusError::Policy("batch input exceeds limit".to_owned()));
        }
        if gross_output > self.limits.max_batch_notional {
            return Err(NexusError::Policy("batch output exceeds limit".to_owned()));
        }
        if quote.solver_fee_bps > self.limits.max_solver_fee_bps {
            return Err(NexusError::Policy("solver fee exceeds limit".to_owned()));
        }
        if facility_count > self.limits.max_open_facilities {
            return Err(NexusError::Policy(
                "facility count exceeds limit".to_owned(),
            ));
        }
        let projected_reserve = vault.reserve_balance.checked_sub(gross_output)?;
        if projected_reserve < self.limits.min_reserve_after_batch {
            return Err(NexusError::Policy(
                "projected reserve below limit".to_owned(),
            ));
        }
        let utilized = vault.reserve_balance.checked_sub(projected_reserve)?;
        let utilization_bps = if vault.reserve_balance.is_zero() {
            Bps::new(0)?
        } else {
            Bps::new(
                utilized
                    .checked_mul(10_000)?
                    .checked_div(vault.reserve_balance.units())?
                    .units()
                    .min(10_000) as u16,
            )?
        };
        if utilization_bps > self.limits.max_vault_utilization_bps {
            return Err(NexusError::Policy(
                "vault utilization exceeds limit".to_owned(),
            ));
        }
        Ok(RiskSnapshot {
            batch_id,
            gross_input,
            gross_output,
            projected_reserve,
            utilization_bps,
            facility_count,
            accepted: true,
        })
    }
}

impl Default for RiskEngine {
    fn default() -> Self {
        Self::new(RiskLimits::default())
    }
}
