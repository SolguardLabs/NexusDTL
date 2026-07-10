use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use crate::{
    AccountId, AccountState, Amount, AssetConfig, AssetId, AuctionBook, AuctionLane, Bps,
    CreditBook, CreditFacility, Digest, ExitTicket, FacilityId, FeePolicy, GovernanceConfig,
    InsuranceFund, IntentId, JournalEntry, JournalOp, LiquidityVault, NettingEngine, NexusError,
    NexusResult, OperatorRegistry, OperatorRole, OracleBook, OracleObservation, PublicIdentity,
    RiskEngine, RiskLimits, SettlementObligation, SignedIntent, SignedSettlement, SolverBid,
    TreasuryBook, TxId, VaultConfig, VaultId,
};

#[derive(Clone, Debug, Serialize)]
pub struct NexusLedger {
    network_id: u32,
    assets: BTreeMap<AssetId, AssetConfig>,
    accounts: BTreeMap<AccountId, AccountState>,
    total_supply: BTreeMap<AssetId, Amount>,
    vaults: BTreeMap<VaultId, LiquidityVault>,
    credit_book: CreditBook,
    risk_engine: RiskEngine,
    operator_registry: OperatorRegistry,
    oracle_book: OracleBook,
    auction_book: AuctionBook,
    treasury_book: TreasuryBook,
    netting_engine: NettingEngine,
    settled_intents: BTreeSet<IntentId>,
    executed_batches: BTreeSet<crate::BatchId>,
    seen_transactions: BTreeSet<TxId>,
    journal: Vec<JournalEntry>,
}

#[derive(Serialize)]
struct LedgerDigestView<'a> {
    network_id: u32,
    assets: &'a BTreeMap<AssetId, AssetConfig>,
    accounts: &'a BTreeMap<AccountId, AccountState>,
    total_supply: &'a BTreeMap<AssetId, Amount>,
    vaults: &'a BTreeMap<VaultId, LiquidityVault>,
    credit_book: &'a CreditBook,
    risk_engine: &'a RiskEngine,
    operator_registry: &'a OperatorRegistry,
    oracle_book: &'a OracleBook,
    auction_book: &'a AuctionBook,
    treasury_book: &'a TreasuryBook,
    netting_engine: &'a NettingEngine,
    settled_intents: &'a BTreeSet<IntentId>,
    executed_batches: &'a BTreeSet<crate::BatchId>,
    seen_transactions: &'a BTreeSet<TxId>,
    journal_len: usize,
}

impl NexusLedger {
    pub fn new(network_id: u32) -> Self {
        Self {
            network_id,
            assets: BTreeMap::new(),
            accounts: BTreeMap::new(),
            total_supply: BTreeMap::new(),
            vaults: BTreeMap::new(),
            credit_book: CreditBook::default(),
            risk_engine: RiskEngine::default(),
            operator_registry: OperatorRegistry::default(),
            oracle_book: OracleBook::default(),
            auction_book: AuctionBook::default(),
            treasury_book: TreasuryBook::default(),
            netting_engine: NettingEngine::default(),
            settled_intents: BTreeSet::new(),
            executed_batches: BTreeSet::new(),
            seen_transactions: BTreeSet::new(),
            journal: Vec::new(),
        }
    }

    pub const fn network_id(&self) -> u32 {
        self.network_id
    }

    pub fn set_risk_limits(&mut self, limits: RiskLimits) {
        self.risk_engine.set_limits(limits);
    }

    pub fn set_governance_config(&mut self, config: GovernanceConfig) -> NexusResult<TxId> {
        self.account(config.emergency_council)?;
        self.operator_registry.configure(config);
        let tx_id = TxId::from_serializable(
            "nexus-governance-config-tx-v1",
            &(self.network_id, config, self.journal.len() as u64),
        )?;
        self.append_journal(tx_id, JournalOp::GovernanceConfigured { config })?;
        Ok(tx_id)
    }

    pub fn grant_operator_role(
        &mut self,
        account: AccountId,
        role: OperatorRole,
    ) -> NexusResult<TxId> {
        self.account(account)?;
        self.operator_registry.grant_role(account, role);
        let tx_id = TxId::from_serializable(
            "nexus-operator-role-tx-v1",
            &(self.network_id, account, role, self.journal.len() as u64),
        )?;
        self.append_journal(tx_id, JournalOp::OperatorRoleGranted { account, role })?;
        Ok(tx_id)
    }

    pub fn register_oracle_publisher(
        &mut self,
        publisher: AccountId,
        weight: u16,
    ) -> NexusResult<TxId> {
        self.account(publisher)?;
        self.operator_registry
            .require_role(publisher, OperatorRole::OraclePublisher)?;
        self.oracle_book.register_publisher(publisher, weight)?;
        let tx_id = TxId::from_serializable(
            "nexus-oracle-publisher-tx-v1",
            &(
                self.network_id,
                publisher,
                weight,
                self.journal.len() as u64,
            ),
        )?;
        self.append_journal(
            tx_id,
            JournalOp::OraclePublisherRegistered { publisher, weight },
        )?;
        Ok(tx_id)
    }

    pub fn publish_oracle(&mut self, observation: OracleObservation) -> NexusResult<TxId> {
        self.account(observation.publisher)?;
        self.asset_config(observation.base_asset)?;
        self.asset_config(observation.quote_asset)?;
        self.operator_registry
            .require_role(observation.publisher, OperatorRole::OraclePublisher)?;
        let feed_digest = self.oracle_book.publish(observation)?;
        let tx_id = TxId::from_serializable(
            "nexus-oracle-publish-tx-v1",
            &(self.network_id, observation, self.journal.len() as u64),
        )?;
        self.append_journal(
            tx_id,
            JournalOp::OraclePublished {
                publisher: observation.publisher,
                base_asset: observation.base_asset,
                quote_asset: observation.quote_asset,
                feed_digest,
            },
        )?;
        Ok(tx_id)
    }

    pub fn register_auction_lane(&mut self, lane: AuctionLane) -> NexusResult<TxId> {
        self.vault(lane.vault_id)?;
        self.asset_config(lane.source_asset)?;
        self.asset_config(lane.target_asset)?;
        let lane_id = self.auction_book.register_lane(lane)?;
        let tx_id = TxId::from_serializable(
            "nexus-auction-lane-tx-v1",
            &(self.network_id, lane, self.journal.len() as u64),
        )?;
        self.append_journal(
            tx_id,
            JournalOp::AuctionLaneRegistered {
                lane_id,
                vault_id: lane.vault_id,
                source_asset: lane.source_asset,
                target_asset: lane.target_asset,
            },
        )?;
        Ok(tx_id)
    }

    pub fn submit_solver_bid(&mut self, bid: SolverBid) -> NexusResult<TxId> {
        self.account(bid.solver)?;
        self.operator_registry
            .require_role(bid.solver, OperatorRole::Solver)?;
        let bid_digest = self.auction_book.submit_bid(bid)?;
        let tx_id = TxId::from_serializable(
            "nexus-solver-bid-tx-v1",
            &(self.network_id, bid, self.journal.len() as u64),
        )?;
        self.append_journal(
            tx_id,
            JournalOp::SolverBidSubmitted {
                lane_id: bid.lane_id,
                solver: bid.solver,
                bid_digest,
                solver_fee_bps: bid.solver_fee_bps,
            },
        )?;
        Ok(tx_id)
    }

    pub fn configure_fee_policy(&mut self, policy: FeePolicy) -> NexusResult<TxId> {
        self.account(policy.recipient)?;
        self.operator_registry
            .require_role(policy.recipient, OperatorRole::Treasurer)?;
        self.treasury_book.configure_policy(policy);
        let tx_id = TxId::from_serializable(
            "nexus-fee-policy-tx-v1",
            &(self.network_id, policy, self.journal.len() as u64),
        )?;
        self.append_journal(
            tx_id,
            JournalOp::FeePolicyConfigured {
                recipient: policy.recipient,
                protocol_fee_bps: policy.protocol_fee_bps,
                insurance_take_bps: policy.insurance_take_bps,
            },
        )?;
        Ok(tx_id)
    }

    pub fn configure_insurance_fund(&mut self, fund: InsuranceFund) -> NexusResult<TxId> {
        self.asset_config(fund.reserve_asset)?;
        self.treasury_book.configure_insurance(fund);
        let tx_id = TxId::from_serializable(
            "nexus-insurance-fund-tx-v1",
            &(self.network_id, fund, self.journal.len() as u64),
        )?;
        self.append_journal(
            tx_id,
            JournalOp::InsuranceConfigured {
                reserve_asset: fund.reserve_asset,
                balance: fund.balance,
                floor: fund.floor,
            },
        )?;
        Ok(tx_id)
    }

    pub fn register_asset(&mut self, config: AssetConfig) -> NexusResult<()> {
        if self.assets.contains_key(&config.id) {
            return Err(NexusError::AssetAlreadyExists(config.id));
        }
        self.total_supply
            .entry(config.id)
            .or_insert_with(Amount::zero);
        self.assets.insert(config.id, config);
        Ok(())
    }

    pub fn register_account(&mut self, identity: PublicIdentity) -> NexusResult<()> {
        identity.verify_consistency()?;
        if self.accounts.contains_key(&identity.account) {
            return Err(NexusError::AccountAlreadyExists(identity.account));
        }
        self.accounts
            .insert(identity.account, AccountState::new(identity));
        Ok(())
    }

    pub fn credit_genesis(
        &mut self,
        account: AccountId,
        asset: AssetId,
        amount: Amount,
    ) -> NexusResult<TxId> {
        if amount.is_zero() {
            return Err(NexusError::ZeroAmount);
        }
        self.asset_config(asset)?;
        self.credit(account, asset, amount)?;
        let supply = self.total_supply_of(asset).checked_add(amount)?;
        self.total_supply.insert(asset, supply);
        self.verify_conservation(asset)?;
        let tx_id = TxId::from_serializable(
            "nexus-genesis-credit-v1",
            &(
                self.network_id,
                account,
                asset,
                amount,
                self.journal.len() as u64,
            ),
        )?;
        self.append_journal(
            tx_id,
            JournalOp::GenesisCredit {
                account,
                asset,
                amount,
            },
        )?;
        Ok(tx_id)
    }

    pub fn register_vault(&mut self, config: VaultConfig) -> NexusResult<TxId> {
        self.account(config.controller)?;
        self.asset_config(config.reserve_asset)?;
        if self.vaults.contains_key(&config.vault_id) {
            return Err(NexusError::Policy("vault already registered".to_owned()));
        }
        self.vaults
            .insert(config.vault_id, LiquidityVault::new(config));
        let tx_id = TxId::from_serializable(
            "nexus-vault-register-tx-v1",
            &(self.network_id, config, self.journal.len() as u64),
        )?;
        self.append_journal(
            tx_id,
            JournalOp::VaultRegistered {
                vault_id: config.vault_id,
                reserve_asset: config.reserve_asset,
                controller: config.controller,
            },
        )?;
        Ok(tx_id)
    }

    pub fn deposit_to_vault(
        &mut self,
        owner: AccountId,
        vault_id: VaultId,
        assets: Amount,
    ) -> NexusResult<TxId> {
        let reserve_asset = self.vault(vault_id)?.config.reserve_asset;
        self.debit(owner, reserve_asset, assets)?;
        let shares = self.vault_mut(vault_id)?.deposit(owner, assets)?;
        self.verify_conservation(reserve_asset)?;
        let tx_id = TxId::from_serializable(
            "nexus-vault-deposit-tx-v1",
            &(self.network_id, owner, vault_id, assets, shares),
        )?;
        self.append_journal(
            tx_id,
            JournalOp::VaultDeposit {
                vault_id,
                owner,
                assets,
                shares,
            },
        )?;
        Ok(tx_id)
    }

    pub fn request_vault_exit(
        &mut self,
        owner: AccountId,
        vault_id: VaultId,
        shares: Amount,
    ) -> NexusResult<ExitTicket> {
        let epoch = self.risk_engine.limits().current_epoch;
        let ticket = self
            .vault_mut(vault_id)?
            .request_exit(owner, shares, epoch)?;
        let tx_id =
            TxId::from_serializable("nexus-vault-exit-request-tx-v1", &(self.network_id, ticket))?;
        self.append_journal(
            tx_id,
            JournalOp::VaultExitRequested {
                vault_id,
                owner,
                shares,
                ticket_id: ticket.ticket_id,
            },
        )?;
        Ok(ticket)
    }

    pub fn redeem_vault_active(
        &mut self,
        owner: AccountId,
        vault_id: VaultId,
        shares: Amount,
    ) -> NexusResult<TxId> {
        let reserve_asset = self.vault(vault_id)?.config.reserve_asset;
        let assets = self.vault_mut(vault_id)?.redeem_active(owner, shares)?;
        self.credit(owner, reserve_asset, assets)?;
        self.verify_conservation(reserve_asset)?;
        let tx_id = TxId::from_serializable(
            "nexus-vault-redeem-tx-v1",
            &(self.network_id, owner, vault_id, shares, assets),
        )?;
        self.append_journal(
            tx_id,
            JournalOp::VaultRedeem {
                vault_id,
                owner,
                shares,
                assets,
            },
        )?;
        Ok(tx_id)
    }

    pub fn open_facility(
        &mut self,
        facility: CreditFacility,
        current_epoch: u64,
    ) -> NexusResult<TxId> {
        self.vault(facility.vault_id)?;
        self.account(facility.borrower)?;
        self.asset_config(facility.collateral_asset)?;
        self.asset_config(facility.borrow_asset)?;
        self.credit_book.open_facility(facility, current_epoch)?;
        let tx_id = TxId::from_serializable(
            "nexus-facility-open-tx-v1",
            &(self.network_id, facility, self.journal.len() as u64),
        )?;
        self.append_journal(
            tx_id,
            JournalOp::FacilityOpened {
                facility_id: facility.facility_id,
                vault_id: facility.vault_id,
                borrower: facility.borrower,
                debt_ceiling: facility.debt_ceiling,
            },
        )?;
        Ok(tx_id)
    }

    pub fn draw_facility(&mut self, facility_id: FacilityId, amount: Amount) -> NexusResult<TxId> {
        let facility = self.credit_book.facility(facility_id)?;
        let reserve_asset = self.vault(facility.vault_id)?.config.reserve_asset;
        if facility.borrow_asset != reserve_asset {
            return Err(NexusError::Policy(
                "facility borrow asset mismatch".to_owned(),
            ));
        }
        let reserve_after = self
            .vault(facility.vault_id)?
            .reserve_balance
            .checked_sub(amount)?;
        if reserve_after < self.vault(facility.vault_id)?.config.reserve_floor {
            return Err(NexusError::Policy("vault reserve floor reached".to_owned()));
        }
        self.credit_book.draw(facility_id, amount)?;
        self.vault_mut(facility.vault_id)?.reserve_balance = reserve_after;
        self.credit(facility.borrower, reserve_asset, amount)?;
        self.verify_conservation(reserve_asset)?;
        let tx_id = TxId::from_serializable(
            "nexus-facility-draw-tx-v1",
            &(
                self.network_id,
                facility_id,
                amount,
                self.journal.len() as u64,
            ),
        )?;
        self.append_journal(
            tx_id,
            JournalOp::FacilityDraw {
                facility_id,
                borrower: facility.borrower,
                amount,
            },
        )?;
        Ok(tx_id)
    }

    pub fn execute_batch(
        &mut self,
        intents: &[SignedIntent],
        settlement: &SignedSettlement,
    ) -> NexusResult<TxId> {
        let mut candidate = self.clone();
        let tx_id = candidate.apply_batch(intents, settlement)?;
        *self = candidate;
        Ok(tx_id)
    }

    pub fn balance_of(&self, account: AccountId, asset: AssetId) -> NexusResult<Amount> {
        Ok(self.account(account)?.balance_of(asset))
    }

    pub fn intent_nonce(&self, account: AccountId) -> NexusResult<u64> {
        Ok(self.account(account)?.next_intent_nonce)
    }

    pub fn total_supply_of(&self, asset: AssetId) -> Amount {
        self.total_supply
            .get(&asset)
            .copied()
            .unwrap_or_else(Amount::zero)
    }

    pub fn vault(&self, vault_id: VaultId) -> NexusResult<&LiquidityVault> {
        self.vaults
            .get(&vault_id)
            .ok_or(NexusError::VaultNotFound(vault_id))
    }

    pub fn journal(&self) -> &[JournalEntry] {
        &self.journal
    }

    pub fn state_digest(&self) -> NexusResult<Digest> {
        Digest::from_serializable(
            "nexus-ledger-state-v1",
            &LedgerDigestView {
                network_id: self.network_id,
                assets: &self.assets,
                accounts: &self.accounts,
                total_supply: &self.total_supply,
                vaults: &self.vaults,
                credit_book: &self.credit_book,
                risk_engine: &self.risk_engine,
                operator_registry: &self.operator_registry,
                oracle_book: &self.oracle_book,
                auction_book: &self.auction_book,
                treasury_book: &self.treasury_book,
                netting_engine: &self.netting_engine,
                settled_intents: &self.settled_intents,
                executed_batches: &self.executed_batches,
                seen_transactions: &self.seen_transactions,
                journal_len: self.journal.len(),
            },
        )
    }

    pub fn is_conserved(&self, asset: AssetId) -> NexusResult<bool> {
        self.verify_conservation(asset)?;
        Ok(true)
    }

    pub fn operator_count(&self) -> usize {
        self.operator_registry.operator_count()
    }

    pub fn oracle_market_count(&self) -> usize {
        self.oracle_book.market_count()
    }

    pub fn oracle_publisher_count(&self) -> usize {
        self.oracle_book.publisher_count()
    }

    pub fn auction_lane_count(&self) -> usize {
        self.auction_book.lane_count()
    }

    pub fn solver_bid_count(&self) -> usize {
        self.auction_book.bid_count()
    }

    pub fn netting_report_count(&self) -> usize {
        self.netting_engine.report_count()
    }

    pub fn treasury_accrued_asset_count(&self) -> usize {
        self.treasury_book.accrued_asset_count()
    }

    fn apply_batch(
        &mut self,
        intents: &[SignedIntent],
        settlement: &SignedSettlement,
    ) -> NexusResult<TxId> {
        if intents.is_empty() {
            return Err(NexusError::Policy("empty settlement batch".to_owned()));
        }
        settlement.verify()?;
        let batch = settlement.batch.clone();
        if batch.network_id != self.network_id {
            return Err(NexusError::Policy("network mismatch".to_owned()));
        }
        if self.executed_batches.contains(&batch.batch_id) {
            return Err(NexusError::BatchExecuted(batch.batch_id));
        }
        let observed_intents_digest =
            Digest::from_serializable("nexus-batch-intents-v1", &intents)?;
        if observed_intents_digest != batch.intents_digest {
            return Err(NexusError::DigestMismatch {
                expected: batch.intents_digest,
                received: observed_intents_digest,
            });
        }
        let tx_id = TxId::from_serializable("nexus-batch-execution-tx-v1", &(intents, settlement))?;
        if self.seen_transactions.contains(&tx_id) {
            return Err(NexusError::DuplicateTransaction(tx_id));
        }
        self.operator_registry.ensure_not_paused()?;
        self.operator_registry
            .require_role(batch.coordinator, OperatorRole::Coordinator)?;
        self.operator_registry
            .require_role(batch.solver, OperatorRole::Solver)?;

        let mut gross_input = Amount::zero();
        let mut gross_output = Amount::zero();
        let mut solver_fee = Amount::zero();
        let mut intent_ids = Vec::new();
        let current_epoch = self.risk_engine.limits().current_epoch;
        if batch.epoch != current_epoch {
            return Err(NexusError::Policy("batch epoch mismatch".to_owned()));
        }
        let first_quote = intents[0].intent.quote;

        for signed in intents {
            signed.verify()?;
            let intent = signed.intent;
            if !self.asset_config(intent.source_asset)?.settlement_enabled
                || !self.asset_config(intent.target_asset)?.settlement_enabled
            {
                return Err(NexusError::Policy(
                    "asset settlement is disabled".to_owned(),
                ));
            }
            if intent.network_id != self.network_id {
                return Err(NexusError::Policy("intent network mismatch".to_owned()));
            }
            if intent.vault_id != batch.vault_id || intent.quote.vault_id != batch.vault_id {
                return Err(NexusError::Policy("intent vault mismatch".to_owned()));
            }
            if intent.quote.solver != batch.solver {
                return Err(NexusError::Policy("solver mismatch".to_owned()));
            }
            if intent.quote.expires_at_epoch < current_epoch {
                return Err(NexusError::Policy("quote expired".to_owned()));
            }
            if self.settled_intents.contains(&intent.intent_id) {
                return Err(NexusError::IntentSettled(intent.intent_id));
            }
            if self.intent_nonce(intent.owner)? != intent.owner_nonce {
                return Err(NexusError::NonceMismatch {
                    account: intent.owner,
                    expected: self.intent_nonce(intent.owner)?,
                    received: intent.owner_nonce,
                });
            }
            let expected_quote_digest = intent.quote.digest()?;
            if expected_quote_digest != first_quote.digest()? {
                return Err(NexusError::Policy("mixed quote batch".to_owned()));
            }
            let output = intent.quote.output_amount(intent.source_amount)?;
            if output < intent.min_output {
                return Err(NexusError::Policy("minimum output not reached".to_owned()));
            }
            gross_input = gross_input.checked_add(intent.source_amount)?;
            gross_output = gross_output.checked_add(output)?;
            solver_fee = solver_fee.checked_add(intent.quote.solver_fee(output)?)?;
            intent_ids.push(intent.intent_id);
        }

        let price_band = self.oracle_book.check_quote(
            first_quote.source_asset,
            first_quote.target_asset,
            first_quote.price_numerator,
            first_quote.price_denominator,
            Bps::new(300)?,
            current_epoch,
        )?;
        let lane = self.auction_book.lane(first_quote.route_digest)?;
        if lane.vault_id != batch.vault_id
            || lane.source_asset != first_quote.source_asset
            || lane.target_asset != first_quote.target_asset
        {
            return Err(NexusError::Policy("auction lane mismatch".to_owned()));
        }
        let best_bid = *self.auction_book.best_bid_for(
            first_quote.route_digest,
            gross_input,
            current_epoch,
        )?;
        if best_bid.solver != batch.solver
            || best_bid.price_numerator != first_quote.price_numerator
            || best_bid.price_denominator != first_quote.price_denominator
            || best_bid.solver_fee_bps != first_quote.solver_fee_bps
        {
            return Err(NexusError::Policy(
                "settlement quote not selected".to_owned(),
            ));
        }
        let obligations = self.settlement_obligations(intents, batch.coordinator)?;
        let netting_report = self.netting_engine.record_obligations(obligations)?;
        if netting_report.report_digest != batch.netting_digest {
            return Err(NexusError::DigestMismatch {
                expected: batch.netting_digest,
                received: netting_report.report_digest,
            });
        }

        let liquidity_snapshot = self.vault(batch.vault_id)?.liquidity_digest()?;
        if liquidity_snapshot != batch.liquidity_snapshot {
            return Err(NexusError::DigestMismatch {
                expected: batch.liquidity_snapshot,
                received: liquidity_snapshot,
            });
        }
        let risk = self.risk_engine.evaluate_batch(
            batch.batch_id,
            self.vault(batch.vault_id)?,
            gross_input,
            gross_output,
            first_quote,
            self.credit_book.facility_count(),
        )?;
        let protocol_claim = self.treasury_book.protocol_fee_claim(gross_output)?;
        let insurance_claim = self.treasury_book.insurance_claim(gross_output)?;

        let reserve_asset = self.vault(batch.vault_id)?.config.reserve_asset;
        self.treasury_book
            .record_accrual(reserve_asset, protocol_claim)?;
        self.treasury_book
            .absorb_insurance(reserve_asset, insurance_claim)?;
        let vault_reserve_after = self
            .vault(batch.vault_id)?
            .reserve_balance
            .checked_sub(gross_output)?;
        self.vault_mut(batch.vault_id)?.reserve_balance = vault_reserve_after;

        for signed in intents {
            let intent = signed.intent;
            let output = intent.quote.output_amount(intent.source_amount)?;
            let fee = intent.quote.solver_fee(output)?;
            let receiver_amount = output.checked_sub(fee)?;
            self.debit(intent.owner, intent.source_asset, intent.source_amount)?;
            self.credit(batch.solver, intent.source_asset, intent.source_amount)?;
            self.credit(intent.receiver, intent.target_asset, receiver_amount)?;
            if !fee.is_zero() {
                self.credit(batch.solver, intent.target_asset, fee)?;
            }
            self.account_mut(intent.owner)?.advance_intent_nonce()?;
            self.settled_intents.insert(intent.intent_id);
        }

        self.executed_batches.insert(batch.batch_id);
        self.seen_transactions.insert(tx_id);
        self.verify_conservation(reserve_asset)?;
        for signed in intents {
            self.verify_conservation(signed.intent.source_asset)?;
            self.verify_conservation(signed.intent.target_asset)?;
        }
        self.append_journal(
            tx_id,
            JournalOp::SettlementBatch {
                batch_id: batch.batch_id,
                vault_id: batch.vault_id,
                solver: batch.solver,
                intents: intent_ids,
                gross_input,
                gross_output,
                solver_fee,
                price_band,
                netting_report: Box::new(netting_report),
                risk: Box::new(risk),
            },
        )?;
        Ok(tx_id)
    }

    fn append_journal(&mut self, tx_id: TxId, op: JournalOp) -> NexusResult<()> {
        let entry = JournalEntry {
            sequence: self.journal.len() as u64,
            tx_id,
            op,
            state_digest: self.state_digest()?,
        };
        self.journal.push(entry);
        Ok(())
    }

    fn asset_config(&self, asset: AssetId) -> NexusResult<AssetConfig> {
        self.assets
            .get(&asset)
            .copied()
            .ok_or(NexusError::AssetNotFound(asset))
    }

    fn account(&self, account: AccountId) -> NexusResult<&AccountState> {
        self.accounts
            .get(&account)
            .ok_or(NexusError::AccountNotFound(account))
    }

    fn account_mut(&mut self, account: AccountId) -> NexusResult<&mut AccountState> {
        self.accounts
            .get_mut(&account)
            .ok_or(NexusError::AccountNotFound(account))
    }

    fn vault_mut(&mut self, vault_id: VaultId) -> NexusResult<&mut LiquidityVault> {
        self.vaults
            .get_mut(&vault_id)
            .ok_or(NexusError::VaultNotFound(vault_id))
    }

    fn credit(&mut self, account: AccountId, asset: AssetId, amount: Amount) -> NexusResult<()> {
        self.account_mut(account)?.credit(asset, amount)
    }

    fn debit(&mut self, account: AccountId, asset: AssetId, amount: Amount) -> NexusResult<()> {
        self.account_mut(account)?.debit(account, asset, amount)
    }

    fn verify_conservation(&self, asset: AssetId) -> NexusResult<()> {
        let account_total = self
            .accounts
            .values()
            .try_fold(Amount::zero(), |accumulator, account| {
                accumulator.checked_add(account.balance_of(asset))
            })?;
        let vault_total = self
            .vaults
            .values()
            .filter(|vault| vault.config.reserve_asset == asset)
            .try_fold(Amount::zero(), |accumulator, vault| {
                accumulator.checked_add(vault.reserve_balance)
            })?;
        let observed = account_total.checked_add(vault_total)?;
        let expected = self.total_supply_of(asset);
        if observed != expected {
            return Err(NexusError::Conservation {
                asset,
                expected,
                observed,
            });
        }
        Ok(())
    }

    fn settlement_obligations(
        &self,
        intents: &[SignedIntent],
        vault_account: AccountId,
    ) -> NexusResult<Vec<SettlementObligation>> {
        let mut obligations = Vec::with_capacity(intents.len() * 5);
        for signed in intents {
            let intent = signed.intent;
            let output = intent.quote.output_amount(intent.source_amount)?;
            let fee = intent.quote.solver_fee(output)?;
            let receiver_amount = output.checked_sub(fee)?;
            obligations.push(SettlementObligation::debit(
                intent.owner,
                intent.source_asset,
                intent.source_amount,
            ));
            obligations.push(SettlementObligation::credit(
                intent.quote.solver,
                intent.source_asset,
                intent.source_amount,
            ));
            obligations.push(SettlementObligation::debit(
                vault_account,
                intent.target_asset,
                output,
            ));
            obligations.push(SettlementObligation::credit(
                intent.receiver,
                intent.target_asset,
                receiver_amount,
            ));
            if !fee.is_zero() {
                obligations.push(SettlementObligation::credit(
                    intent.quote.solver,
                    intent.target_asset,
                    fee,
                ));
            }
        }
        Ok(obligations)
    }
}
