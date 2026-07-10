use std::collections::BTreeMap;

use serde::Serialize;

use crate::{AccountId, Amount, AssetId, Bps, Digest, NexusError, NexusResult, VaultId};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct AuctionLane {
    pub lane_id: Digest,
    pub vault_id: VaultId,
    pub source_asset: AssetId,
    pub target_asset: AssetId,
    pub min_size: Amount,
    pub max_size: Amount,
    pub max_solver_fee_bps: Bps,
    pub enabled: bool,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SolverBid {
    pub lane_id: Digest,
    pub solver: AccountId,
    pub price_numerator: u128,
    pub price_denominator: u128,
    pub solver_fee_bps: Bps,
    pub capacity: Amount,
    pub expires_at_epoch: u64,
    pub bid_digest: Digest,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct AuctionBook {
    lanes: BTreeMap<Digest, AuctionLane>,
    bids: BTreeMap<Digest, SolverBid>,
    best_by_lane: BTreeMap<Digest, Digest>,
}

impl AuctionLane {
    pub fn new(
        vault_id: VaultId,
        source_asset: AssetId,
        target_asset: AssetId,
        min_size: Amount,
        max_size: Amount,
        max_solver_fee_bps: Bps,
        salt: Digest,
    ) -> NexusResult<Self> {
        if min_size.is_zero() || max_size.is_zero() {
            return Err(NexusError::ZeroAmount);
        }
        if min_size > max_size {
            return Err(NexusError::Policy(
                "auction lane size range is invalid".to_owned(),
            ));
        }
        let lane_id = Digest::from_serializable(
            "nexus-auction-lane-v1",
            &(
                vault_id,
                source_asset,
                target_asset,
                min_size,
                max_size,
                max_solver_fee_bps,
                salt,
            ),
        )?;
        Ok(Self {
            lane_id,
            vault_id,
            source_asset,
            target_asset,
            min_size,
            max_size,
            max_solver_fee_bps,
            enabled: true,
        })
    }
}

impl SolverBid {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        lane_id: Digest,
        solver: AccountId,
        price_numerator: u128,
        price_denominator: u128,
        solver_fee_bps: Bps,
        capacity: Amount,
        expires_at_epoch: u64,
        salt: Digest,
    ) -> NexusResult<Self> {
        if price_numerator == 0 || price_denominator == 0 {
            return Err(NexusError::DivisionByZero);
        }
        if capacity.is_zero() {
            return Err(NexusError::ZeroAmount);
        }
        let bid_digest = Digest::from_serializable(
            "nexus-solver-bid-v1",
            &(
                lane_id,
                solver,
                price_numerator,
                price_denominator,
                solver_fee_bps,
                capacity,
                expires_at_epoch,
                salt,
            ),
        )?;
        Ok(Self {
            lane_id,
            solver,
            price_numerator,
            price_denominator,
            solver_fee_bps,
            capacity,
            expires_at_epoch,
            bid_digest,
        })
    }
}

impl AuctionBook {
    pub fn register_lane(&mut self, lane: AuctionLane) -> NexusResult<Digest> {
        if self.lanes.contains_key(&lane.lane_id) {
            return Err(NexusError::Policy("auction lane already exists".to_owned()));
        }
        self.lanes.insert(lane.lane_id, lane);
        Ok(lane.lane_id)
    }

    pub fn submit_bid(&mut self, bid: SolverBid) -> NexusResult<Digest> {
        let lane = self
            .lanes
            .get(&bid.lane_id)
            .ok_or_else(|| NexusError::Policy("auction lane not found".to_owned()))?;
        if !lane.enabled {
            return Err(NexusError::Policy("auction lane is disabled".to_owned()));
        }
        if bid.solver_fee_bps > lane.max_solver_fee_bps {
            return Err(NexusError::Policy("bid fee exceeds lane limit".to_owned()));
        }
        self.bids.insert(bid.bid_digest, bid);
        let current_best = self
            .best_by_lane
            .get(&bid.lane_id)
            .and_then(|bid_digest| self.bids.get(bid_digest))
            .copied();
        if current_best.is_none_or(|current| bid_is_better(bid, current)) {
            self.best_by_lane.insert(bid.lane_id, bid.bid_digest);
        }
        Ok(bid.bid_digest)
    }

    pub fn best_bid_for(
        &self,
        lane_id: Digest,
        amount: Amount,
        current_epoch: u64,
    ) -> NexusResult<&SolverBid> {
        let lane = self
            .lanes
            .get(&lane_id)
            .ok_or_else(|| NexusError::Policy("auction lane not found".to_owned()))?;
        if !lane.enabled {
            return Err(NexusError::Policy("auction lane is disabled".to_owned()));
        }
        if amount < lane.min_size || amount > lane.max_size {
            return Err(NexusError::Policy(
                "auction lane size not accepted".to_owned(),
            ));
        }
        let mut best: Option<&SolverBid> = None;
        for bid in self.bids.values() {
            if bid.lane_id != lane_id
                || bid.capacity < amount
                || bid.expires_at_epoch < current_epoch
            {
                continue;
            }
            if best.is_none_or(|current| bid_is_better(*bid, *current)) {
                best = Some(bid);
            }
        }
        best.ok_or_else(|| NexusError::Policy("solver bid not available".to_owned()))
    }

    pub fn lane(&self, lane_id: Digest) -> NexusResult<&AuctionLane> {
        self.lanes
            .get(&lane_id)
            .ok_or_else(|| NexusError::Policy("auction lane not found".to_owned()))
    }

    pub fn lane_count(&self) -> usize {
        self.lanes.len()
    }

    pub fn bid_count(&self) -> usize {
        self.bids.len()
    }
}

fn bid_is_better(candidate: SolverBid, current: SolverBid) -> bool {
    let candidate_output = candidate
        .price_numerator
        .saturating_mul(current.price_denominator);
    let current_output = current
        .price_numerator
        .saturating_mul(candidate.price_denominator);
    candidate_output > current_output
        || (candidate_output == current_output && candidate.solver_fee_bps < current.solver_fee_bps)
}
