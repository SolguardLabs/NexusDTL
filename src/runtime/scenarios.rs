use serde::Serialize;

use crate::{
    Amount, AssetConfig, AssetId, AuctionLane, Bps, CreditFacility, Digest, ExecutionIntent,
    FeePolicy, GovernanceConfig, InsuranceFund, KeyPair, NettingEngine, NexusLedger, NexusResult,
    OperatorRole, OracleObservation, RfqQuote, RiskLimits, SettlementBatch, SettlementObligation,
    SignedIntent, SignedSettlement, SolverBid, TxId, VaultConfig, VaultId,
};

const NETWORK_ID: u32 = 72_900;

struct Fixture {
    ledger: NexusLedger,
    lp: KeyPair,
    trader: KeyPair,
    receiver: KeyPair,
    solver: KeyPair,
    coordinator: KeyPair,
    borrower: KeyPair,
    reserve_asset: AssetConfig,
    collateral_asset: AssetConfig,
    vault_id: VaultId,
    lane_id: Digest,
}

#[derive(Debug, Serialize)]
pub struct ScenarioReport {
    pub scenario: String,
    pub network_id: u32,
    pub reserve_asset: AssetReport,
    pub collateral_asset: AssetReport,
    pub vault: VaultReport,
    pub balances: BalanceReport,
    pub surface: SurfaceReport,
    pub transactions: Vec<TxId>,
    pub journal_entries: usize,
    pub state_digest: Digest,
    pub conservation_ok: bool,
}

#[derive(Debug, Serialize)]
pub struct AssetReport {
    pub id: AssetId,
    pub symbol: &'static str,
    pub decimals: u8,
}

#[derive(Debug, Serialize)]
pub struct VaultReport {
    pub vault_id: VaultId,
    pub reserve_balance: Amount,
    pub total_shares: Amount,
    pub locked_shares: Amount,
    pub pending_fees: Amount,
}

#[derive(Debug, Serialize)]
pub struct BalanceReport {
    pub lp_reserve: Amount,
    pub trader_reserve: Amount,
    pub receiver_reserve: Amount,
    pub solver_reserve: Amount,
    pub borrower_reserve: Amount,
}

#[derive(Debug, Serialize)]
pub struct SurfaceReport {
    pub operators: usize,
    pub oracle_publishers: usize,
    pub oracle_markets: usize,
    pub auction_lanes: usize,
    pub solver_bids: usize,
    pub netting_reports: usize,
    pub treasury_assets: usize,
}

pub fn run_named(name: &str) -> NexusResult<ScenarioReport> {
    match name {
        "snapshot" => run_snapshot(),
        "credit" => run_credit(),
        "exit" => run_exit(),
        "routed" => run_routed(),
        _ => run_routed(),
    }
}

fn run_snapshot() -> NexusResult<ScenarioReport> {
    report(fixture()?, "snapshot", Vec::new())
}

fn run_routed() -> NexusResult<ScenarioReport> {
    let mut fixture = fixture()?;
    let tx = execute_intent_batch(
        &mut fixture,
        IntentInput {
            source_amount: 2_000_000_000,
            min_output: 1_980_000_000,
            price_numerator: 995,
            price_denominator: 1_000,
            solver_fee_bps: 30,
            salt_label: "routed-primary",
        },
    )?;
    report(fixture, "routed", vec![tx])
}

fn run_credit() -> NexusResult<ScenarioReport> {
    let mut fixture = fixture()?;
    let facility = CreditFacility::new(
        fixture.vault_id,
        fixture.borrower.public_identity().account,
        fixture.collateral_asset.id,
        fixture.reserve_asset.id,
        Amount::new(12_000_000_000)?,
        Amount::new(4_000_000_000)?,
        Bps::new(450)?,
        2_700,
    )?;
    let open_tx = fixture
        .ledger
        .open_facility(facility, fixture.ledger.network_id() as u64)?;
    let draw_tx = fixture
        .ledger
        .draw_facility(facility.facility_id, Amount::new(1_500_000_000)?)?;
    report(fixture, "credit", vec![open_tx, draw_tx])
}

fn run_exit() -> NexusResult<ScenarioReport> {
    let mut fixture = fixture()?;
    let shares = Amount::new(5_000_000_000)?;
    let ticket = fixture.ledger.request_vault_exit(
        fixture.lp.public_identity().account,
        fixture.vault_id,
        shares,
    )?;
    let tx = TxId::from_serializable("nexus-runtime-exit-ticket-v1", &ticket)?;
    report(fixture, "exit", vec![tx])
}

fn fixture() -> NexusResult<Fixture> {
    let lp = keyed(11);
    let trader = keyed(22);
    let receiver = keyed(33);
    let solver = keyed(44);
    let coordinator = keyed(55);
    let borrower = keyed(66);
    let oracle = keyed(77);
    let treasurer = keyed(88);
    let reserve_asset = AssetConfig::new("NXUSD", 6)?;
    let collateral_asset = AssetConfig::new("NXBOND", 6)?;
    let mut ledger = NexusLedger::new(NETWORK_ID);
    ledger.set_risk_limits(RiskLimits::new(
        2_100,
        Amount::new(25_000_000_000)?,
        Bps::new(7_500)?,
        Amount::new(10_000_000_000)?,
        Bps::new(150)?,
        12,
    )?);
    ledger.register_asset(reserve_asset)?;
    ledger.register_asset(collateral_asset)?;
    for identity in [
        lp.public_identity(),
        trader.public_identity(),
        receiver.public_identity(),
        solver.public_identity(),
        coordinator.public_identity(),
        borrower.public_identity(),
        oracle.public_identity(),
        treasurer.public_identity(),
    ] {
        ledger.register_account(identity)?;
    }
    ledger.set_governance_config(GovernanceConfig::new(
        2_100,
        2,
        coordinator.public_identity().account,
        Digest::from_parts("nexus-runtime-governance-salt-v1", &[b"primary"]),
    )?)?;
    ledger.grant_operator_role(
        coordinator.public_identity().account,
        OperatorRole::Coordinator,
    )?;
    ledger.grant_operator_role(
        coordinator.public_identity().account,
        OperatorRole::RiskAdmin,
    )?;
    ledger.grant_operator_role(solver.public_identity().account, OperatorRole::Solver)?;
    ledger.grant_operator_role(
        oracle.public_identity().account,
        OperatorRole::OraclePublisher,
    )?;
    ledger.grant_operator_role(treasurer.public_identity().account, OperatorRole::Treasurer)?;
    ledger.register_oracle_publisher(oracle.public_identity().account, 100)?;
    ledger.publish_oracle(OracleObservation::new(
        oracle.public_identity().account,
        reserve_asset.id,
        reserve_asset.id,
        995,
        1_000,
        Bps::new(15)?,
        2_100,
    )?)?;
    ledger.configure_fee_policy(FeePolicy::new(
        treasurer.public_identity().account,
        Bps::new(250)?,
        Bps::new(8)?,
        Bps::new(12)?,
    )?)?;
    ledger.configure_insurance_fund(InsuranceFund::new(
        reserve_asset.id,
        Amount::new(3_000_000_000)?,
        Amount::new(1_000_000_000)?,
    )?)?;
    ledger.credit_genesis(
        lp.public_identity().account,
        reserve_asset.id,
        Amount::new(120_000_000_000)?,
    )?;
    ledger.credit_genesis(
        trader.public_identity().account,
        reserve_asset.id,
        Amount::new(18_000_000_000)?,
    )?;
    ledger.credit_genesis(
        borrower.public_identity().account,
        collateral_asset.id,
        Amount::new(30_000_000_000)?,
    )?;
    let vault_config = VaultConfig::new(
        coordinator.public_identity().account,
        reserve_asset.id,
        3,
        Bps::new(20)?,
        Amount::new(8_000_000_000)?,
        Digest::from_parts("nexus-runtime-vault-salt-v1", &[b"primary"]),
    )?;
    let vault_id = vault_config.vault_id;
    ledger.register_vault(vault_config)?;
    ledger.deposit_to_vault(
        lp.public_identity().account,
        vault_id,
        Amount::new(90_000_000_000)?,
    )?;
    let lane = AuctionLane::new(
        vault_id,
        reserve_asset.id,
        reserve_asset.id,
        Amount::new(1_000_000_000)?,
        Amount::new(20_000_000_000)?,
        Bps::new(50)?,
        Digest::from_parts("nexus-runtime-auction-lane-salt-v1", &[b"nxusd-primary"]),
    )?;
    let lane_id = lane.lane_id;
    ledger.register_auction_lane(lane)?;
    ledger.submit_solver_bid(SolverBid::new(
        lane_id,
        solver.public_identity().account,
        995,
        1_000,
        Bps::new(30)?,
        Amount::new(10_000_000_000)?,
        9_999_999,
        Digest::from_parts("nexus-runtime-solver-bid-salt-v1", &[b"solver-a"]),
    )?)?;
    ledger.vault(vault_id)?.liquidity_digest()?;

    Ok(Fixture {
        ledger,
        lp,
        trader,
        receiver,
        solver,
        coordinator,
        borrower,
        reserve_asset,
        collateral_asset,
        vault_id,
        lane_id,
    })
}

struct IntentInput {
    source_amount: u128,
    min_output: u128,
    price_numerator: u128,
    price_denominator: u128,
    solver_fee_bps: u16,
    salt_label: &'static str,
}

fn execute_intent_batch(fixture: &mut Fixture, input: IntentInput) -> NexusResult<TxId> {
    let owner = fixture.trader.public_identity().account;
    let receiver = fixture.receiver.public_identity().account;
    let solver = fixture.solver.public_identity().account;
    let coordinator = fixture.coordinator.public_identity().account;
    let nonce = fixture.ledger.intent_nonce(owner)?;
    let source_amount = Amount::new(input.source_amount)?;
    let min_output = Amount::new(input.min_output)?;
    let quote = RfqQuote::new(
        solver,
        fixture.vault_id,
        fixture.reserve_asset.id,
        fixture.reserve_asset.id,
        input.price_numerator,
        input.price_denominator,
        Bps::new(input.solver_fee_bps)?,
        nonce,
        9_999_999,
        fixture.lane_id,
    )?;
    let intent = ExecutionIntent::new(
        fixture.ledger.network_id(),
        owner,
        receiver,
        fixture.vault_id,
        fixture.reserve_asset.id,
        fixture.reserve_asset.id,
        source_amount,
        min_output,
        nonce,
        quote,
        Digest::from_parts("nexus-intent-salt-v1", &[input.salt_label.as_bytes()]),
    )?;
    let signed_intent = SignedIntent::sign(intent, &fixture.trader)?;
    let intents = vec![signed_intent];
    let output = quote.output_amount(source_amount)?;
    let fee = quote.solver_fee(output)?;
    let receiver_amount = output.checked_sub(fee)?;
    let obligations = vec![
        SettlementObligation::debit(owner, fixture.reserve_asset.id, source_amount),
        SettlementObligation::credit(solver, fixture.reserve_asset.id, source_amount),
        SettlementObligation::debit(coordinator, fixture.reserve_asset.id, output),
        SettlementObligation::credit(receiver, fixture.reserve_asset.id, receiver_amount),
        SettlementObligation::credit(solver, fixture.reserve_asset.id, fee),
    ];
    let batch = SettlementBatch::new(
        fixture.ledger.network_id(),
        coordinator,
        solver,
        fixture.vault_id,
        2_100,
        &intents,
        fixture.ledger.vault(fixture.vault_id)?.liquidity_digest()?,
        NettingEngine::digest_for(&obligations)?,
    )?;
    let signed_batch = SignedSettlement::sign(batch, &fixture.coordinator)?;
    fixture.ledger.execute_batch(&intents, &signed_batch)
}

fn report(
    fixture: Fixture,
    scenario: &'static str,
    transactions: Vec<TxId>,
) -> NexusResult<ScenarioReport> {
    let lp = fixture.lp.public_identity().account;
    let trader = fixture.trader.public_identity().account;
    let receiver = fixture.receiver.public_identity().account;
    let solver = fixture.solver.public_identity().account;
    let borrower = fixture.borrower.public_identity().account;
    let reserve = fixture.reserve_asset.id;
    let vault = fixture.ledger.vault(fixture.vault_id)?;
    Ok(ScenarioReport {
        scenario: scenario.to_owned(),
        network_id: fixture.ledger.network_id(),
        reserve_asset: asset_report(fixture.reserve_asset),
        collateral_asset: asset_report(fixture.collateral_asset),
        vault: VaultReport {
            vault_id: fixture.vault_id,
            reserve_balance: vault.reserve_balance,
            total_shares: vault.total_shares,
            locked_shares: vault.locked_shares,
            pending_fees: vault.pending_fees,
        },
        balances: BalanceReport {
            lp_reserve: fixture.ledger.balance_of(lp, reserve)?,
            trader_reserve: fixture.ledger.balance_of(trader, reserve)?,
            receiver_reserve: fixture.ledger.balance_of(receiver, reserve)?,
            solver_reserve: fixture.ledger.balance_of(solver, reserve)?,
            borrower_reserve: fixture.ledger.balance_of(borrower, reserve)?,
        },
        surface: SurfaceReport {
            operators: fixture.ledger.operator_count(),
            oracle_publishers: fixture.ledger.oracle_publisher_count(),
            oracle_markets: fixture.ledger.oracle_market_count(),
            auction_lanes: fixture.ledger.auction_lane_count(),
            solver_bids: fixture.ledger.solver_bid_count(),
            netting_reports: fixture.ledger.netting_report_count(),
            treasury_assets: fixture.ledger.treasury_accrued_asset_count(),
        },
        transactions,
        journal_entries: fixture.ledger.journal().len(),
        state_digest: fixture.ledger.state_digest()?,
        conservation_ok: fixture.ledger.is_conserved(reserve)?,
    })
}

fn asset_report(config: AssetConfig) -> AssetReport {
    AssetReport {
        id: config.id,
        symbol: config.symbol,
        decimals: config.decimals,
    }
}

fn keyed(byte: u8) -> KeyPair {
    KeyPair::from_seed([byte; 32])
}
