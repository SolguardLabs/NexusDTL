use serde::Serialize;

use crate::{
    AccountId, Amount, AssetId, BatchId, Bps, Digest, FacilityId, GovernanceConfig, IntentId,
    NettingReport, OperatorRole, PriceBand, RiskSnapshot, TxId, VaultId,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct JournalEntry {
    pub sequence: u64,
    pub tx_id: TxId,
    pub op: JournalOp,
    pub state_digest: Digest,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum JournalOp {
    GenesisCredit {
        account: AccountId,
        asset: AssetId,
        amount: Amount,
    },
    VaultRegistered {
        vault_id: VaultId,
        reserve_asset: AssetId,
        controller: AccountId,
    },
    VaultDeposit {
        vault_id: VaultId,
        owner: AccountId,
        assets: Amount,
        shares: Amount,
    },
    VaultExitRequested {
        vault_id: VaultId,
        owner: AccountId,
        shares: Amount,
        ticket_id: Digest,
    },
    VaultRedeem {
        vault_id: VaultId,
        owner: AccountId,
        shares: Amount,
        assets: Amount,
    },
    GovernanceConfigured {
        config: GovernanceConfig,
    },
    OperatorRoleGranted {
        account: AccountId,
        role: OperatorRole,
    },
    OraclePublisherRegistered {
        publisher: AccountId,
        weight: u16,
    },
    OraclePublished {
        publisher: AccountId,
        base_asset: AssetId,
        quote_asset: AssetId,
        feed_digest: Digest,
    },
    AuctionLaneRegistered {
        lane_id: Digest,
        vault_id: VaultId,
        source_asset: AssetId,
        target_asset: AssetId,
    },
    SolverBidSubmitted {
        lane_id: Digest,
        solver: AccountId,
        bid_digest: Digest,
        solver_fee_bps: Bps,
    },
    FeePolicyConfigured {
        recipient: AccountId,
        protocol_fee_bps: Bps,
        insurance_take_bps: Bps,
    },
    InsuranceConfigured {
        reserve_asset: AssetId,
        balance: Amount,
        floor: Amount,
    },
    FacilityOpened {
        facility_id: FacilityId,
        vault_id: VaultId,
        borrower: AccountId,
        debt_ceiling: Amount,
    },
    FacilityDraw {
        facility_id: FacilityId,
        borrower: AccountId,
        amount: Amount,
    },
    SettlementBatch {
        batch_id: BatchId,
        vault_id: VaultId,
        solver: AccountId,
        intents: Vec<IntentId>,
        gross_input: Amount,
        gross_output: Amount,
        solver_fee: Amount,
        price_band: PriceBand,
        netting_report: Box<NettingReport>,
        risk: Box<RiskSnapshot>,
    },
}
