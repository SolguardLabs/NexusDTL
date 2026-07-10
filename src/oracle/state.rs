use std::collections::BTreeMap;

use serde::Serialize;

use crate::{AccountId, AssetId, Bps, Digest, NexusError, NexusResult};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OracleObservation {
    pub publisher: AccountId,
    pub base_asset: AssetId,
    pub quote_asset: AssetId,
    pub price_numerator: u128,
    pub price_denominator: u128,
    pub confidence_bps: Bps,
    pub epoch: u64,
    pub feed_digest: Digest,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct PriceBand {
    pub market_digest: Digest,
    pub observed_deviation_bps: Bps,
    pub max_deviation_bps: Bps,
    pub observation_epoch: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OracleBook {
    publishers: BTreeMap<AccountId, u16>,
    latest: BTreeMap<Digest, OracleObservation>,
    stale_after_epochs: u64,
}

impl OracleObservation {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        publisher: AccountId,
        base_asset: AssetId,
        quote_asset: AssetId,
        price_numerator: u128,
        price_denominator: u128,
        confidence_bps: Bps,
        epoch: u64,
    ) -> NexusResult<Self> {
        if price_numerator == 0 || price_denominator == 0 {
            return Err(NexusError::DivisionByZero);
        }
        let feed_digest = Digest::from_serializable(
            "nexus-oracle-observation-v1",
            &(
                publisher,
                base_asset,
                quote_asset,
                price_numerator,
                price_denominator,
                confidence_bps,
                epoch,
            ),
        )?;
        Ok(Self {
            publisher,
            base_asset,
            quote_asset,
            price_numerator,
            price_denominator,
            confidence_bps,
            epoch,
            feed_digest,
        })
    }

    pub fn market_digest(self) -> Digest {
        market_digest(self.base_asset, self.quote_asset)
    }
}

impl OracleBook {
    pub fn new(stale_after_epochs: u64) -> Self {
        Self {
            publishers: BTreeMap::new(),
            latest: BTreeMap::new(),
            stale_after_epochs,
        }
    }

    pub fn register_publisher(&mut self, publisher: AccountId, weight: u16) -> NexusResult<()> {
        if weight == 0 {
            return Err(NexusError::Policy(
                "oracle publisher weight is zero".to_owned(),
            ));
        }
        self.publishers.insert(publisher, weight);
        Ok(())
    }

    pub fn publish(&mut self, observation: OracleObservation) -> NexusResult<Digest> {
        if !self.publishers.contains_key(&observation.publisher) {
            return Err(NexusError::Policy(
                "oracle publisher is not registered".to_owned(),
            ));
        }
        let market_digest = observation.market_digest();
        self.latest.insert(market_digest, observation);
        Ok(observation.feed_digest)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn check_quote(
        &self,
        base_asset: AssetId,
        quote_asset: AssetId,
        price_numerator: u128,
        price_denominator: u128,
        max_deviation_bps: Bps,
        current_epoch: u64,
    ) -> NexusResult<PriceBand> {
        if price_numerator == 0 || price_denominator == 0 {
            return Err(NexusError::DivisionByZero);
        }
        let market_digest = market_digest(base_asset, quote_asset);
        let observation = self
            .latest
            .get(&market_digest)
            .ok_or_else(|| NexusError::Policy("oracle observation not available".to_owned()))?;
        if current_epoch.saturating_sub(observation.epoch) > self.stale_after_epochs {
            return Err(NexusError::Policy("oracle observation is stale".to_owned()));
        }
        let deviation = ratio_deviation_bps(
            price_numerator,
            price_denominator,
            observation.price_numerator,
            observation.price_denominator,
        )?;
        if deviation > max_deviation_bps {
            return Err(NexusError::Policy("quote outside oracle band".to_owned()));
        }
        Ok(PriceBand {
            market_digest,
            observed_deviation_bps: deviation,
            max_deviation_bps,
            observation_epoch: observation.epoch,
        })
    }

    pub fn publisher_count(&self) -> usize {
        self.publishers.len()
    }

    pub fn market_count(&self) -> usize {
        self.latest.len()
    }
}

impl Default for OracleBook {
    fn default() -> Self {
        Self::new(64)
    }
}

pub fn market_digest(base_asset: AssetId, quote_asset: AssetId) -> Digest {
    Digest::from_parts(
        "nexus-oracle-market-v1",
        &[&base_asset.bytes(), &quote_asset.bytes()],
    )
}

fn ratio_deviation_bps(
    quoted_numerator: u128,
    quoted_denominator: u128,
    observed_numerator: u128,
    observed_denominator: u128,
) -> NexusResult<Bps> {
    let quoted = quoted_numerator
        .checked_mul(observed_denominator)
        .ok_or(NexusError::AmountOverflow)?;
    let observed = observed_numerator
        .checked_mul(quoted_denominator)
        .ok_or(NexusError::AmountOverflow)?;
    if observed == 0 {
        return Err(NexusError::DivisionByZero);
    }
    let distance = quoted.abs_diff(observed);
    let raw_bps = distance
        .checked_mul(10_000)
        .ok_or(NexusError::AmountOverflow)?
        .checked_div(observed)
        .ok_or(NexusError::DivisionByZero)?;
    Bps::new(raw_bps.min(10_000) as u16)
}
