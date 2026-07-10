use serde::Serialize;

use crate::{AccountId, Amount, AssetId, Bps, Digest, NexusError, NexusResult, VaultId};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RfqQuote {
    pub solver: AccountId,
    pub vault_id: VaultId,
    pub source_asset: AssetId,
    pub target_asset: AssetId,
    pub price_numerator: u128,
    pub price_denominator: u128,
    pub solver_fee_bps: Bps,
    pub quote_nonce: u64,
    pub expires_at_epoch: u64,
    pub route_digest: Digest,
}

impl RfqQuote {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        solver: AccountId,
        vault_id: VaultId,
        source_asset: AssetId,
        target_asset: AssetId,
        price_numerator: u128,
        price_denominator: u128,
        solver_fee_bps: Bps,
        quote_nonce: u64,
        expires_at_epoch: u64,
        route_digest: Digest,
    ) -> NexusResult<Self> {
        if price_numerator == 0 || price_denominator == 0 {
            return Err(NexusError::DivisionByZero);
        }
        Ok(Self {
            solver,
            vault_id,
            source_asset,
            target_asset,
            price_numerator,
            price_denominator,
            solver_fee_bps,
            quote_nonce,
            expires_at_epoch,
            route_digest,
        })
    }

    pub fn digest(self) -> NexusResult<Digest> {
        Digest::from_serializable("nexus-rfq-quote-v1", &self)
    }

    pub fn output_amount(self, source_amount: Amount) -> NexusResult<Amount> {
        source_amount.checked_mul_ratio(self.price_numerator, self.price_denominator)
    }

    pub fn solver_fee(self, output_amount: Amount) -> NexusResult<Amount> {
        output_amount.checked_mul_bps(self.solver_fee_bps)
    }
}
