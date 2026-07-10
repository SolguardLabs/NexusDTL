use std::collections::BTreeMap;

use serde::Serialize;

use crate::{AccountId, Amount, Digest, ExitTicket, NexusError, NexusResult, VaultConfig};

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct VaultPosition {
    pub active_shares: Amount,
    pub locked_shares: Amount,
    pub rewards_accrued: Amount,
}

#[derive(Clone, Debug, Serialize)]
pub struct LiquidityVault {
    pub config: VaultConfig,
    pub reserve_balance: Amount,
    pub total_shares: Amount,
    pub locked_shares: Amount,
    pub pending_fees: Amount,
    pub positions: BTreeMap<AccountId, VaultPosition>,
    pub exit_tickets: BTreeMap<Digest, ExitTicket>,
}

impl LiquidityVault {
    pub fn new(config: VaultConfig) -> Self {
        Self {
            config,
            reserve_balance: Amount::zero(),
            total_shares: Amount::zero(),
            locked_shares: Amount::zero(),
            pending_fees: Amount::zero(),
            positions: BTreeMap::new(),
            exit_tickets: BTreeMap::new(),
        }
    }

    pub fn position_of(&self, owner: AccountId) -> VaultPosition {
        self.positions.get(&owner).copied().unwrap_or_default()
    }

    pub fn active_share_supply(&self) -> NexusResult<Amount> {
        self.total_shares.checked_sub(self.locked_shares)
    }

    pub fn deposit(&mut self, owner: AccountId, assets: Amount) -> NexusResult<Amount> {
        if assets.is_zero() {
            return Err(NexusError::ZeroAmount);
        }
        let shares = if self.total_shares.is_zero() || self.reserve_balance.is_zero() {
            assets
        } else {
            assets.checked_mul_ratio(self.total_shares.units(), self.reserve_balance.units())?
        };
        self.reserve_balance = self.reserve_balance.checked_add(assets)?;
        self.total_shares = self.total_shares.checked_add(shares)?;
        let mut position = self.position_of(owner);
        position.active_shares = position.active_shares.checked_add(shares)?;
        self.positions.insert(owner, position);
        Ok(shares)
    }

    pub fn accrue_yield(&mut self, assets: Amount, fee_bps: crate::Bps) -> NexusResult<Amount> {
        if assets.is_zero() {
            return Ok(Amount::zero());
        }
        let fee = assets.checked_mul_bps(fee_bps)?;
        let net = assets.checked_sub(fee)?;
        self.reserve_balance = self.reserve_balance.checked_add(net)?;
        self.pending_fees = self.pending_fees.checked_add(fee)?;
        Ok(net)
    }

    pub fn request_exit(
        &mut self,
        owner: AccountId,
        shares: Amount,
        epoch: u64,
    ) -> NexusResult<ExitTicket> {
        if shares.is_zero() {
            return Err(NexusError::ZeroAmount);
        }
        let mut position = self.position_of(owner);
        if position.active_shares < shares {
            return Err(NexusError::InsufficientShares {
                vault: self.config.vault_id,
                owner,
                available: position.active_shares,
                required: shares,
            });
        }
        position.active_shares = position.active_shares.checked_sub(shares)?;
        position.locked_shares = position.locked_shares.checked_add(shares)?;
        self.locked_shares = self.locked_shares.checked_add(shares)?;
        self.positions.insert(owner, position);

        let ticket = ExitTicket::new(
            self.config.vault_id,
            owner,
            shares,
            epoch,
            epoch.saturating_add(self.config.exit_delay_epochs),
        )?;
        self.exit_tickets.insert(ticket.ticket_id, ticket);
        Ok(ticket)
    }

    pub fn preview_redeem_active(&self, shares: Amount) -> NexusResult<Amount> {
        if shares.is_zero() {
            return Err(NexusError::ZeroAmount);
        }
        let active_supply = self.active_share_supply()?;
        if active_supply.is_zero() {
            return Err(NexusError::Policy("vault active supply is zero".to_owned()));
        }
        shares.checked_mul_ratio(self.reserve_balance.units(), active_supply.units())
    }

    pub fn redeem_active(&mut self, owner: AccountId, shares: Amount) -> NexusResult<Amount> {
        let assets = self.preview_redeem_active(shares)?;
        let mut position = self.position_of(owner);
        if position.active_shares < shares {
            return Err(NexusError::InsufficientShares {
                vault: self.config.vault_id,
                owner,
                available: position.active_shares,
                required: shares,
            });
        }
        let remaining = self.reserve_balance.checked_sub(assets)?;
        if remaining < self.config.reserve_floor {
            return Err(NexusError::Policy("vault reserve floor reached".to_owned()));
        }
        position.active_shares = position.active_shares.checked_sub(shares)?;
        self.total_shares = self.total_shares.checked_sub(shares)?;
        self.reserve_balance = remaining;
        self.positions.insert(owner, position);
        Ok(assets)
    }

    pub fn settle_exit_ticket(&mut self, ticket_id: Digest, epoch: u64) -> NexusResult<Amount> {
        let mut ticket = *self
            .exit_tickets
            .get(&ticket_id)
            .ok_or_else(|| NexusError::Policy("exit ticket not found".to_owned()))?;
        if ticket.settled {
            return Err(NexusError::Policy("exit ticket already settled".to_owned()));
        }
        if epoch < ticket.executable_epoch {
            return Err(NexusError::Policy("exit ticket not executable".to_owned()));
        }
        let assets = ticket
            .shares
            .checked_mul_ratio(self.reserve_balance.units(), self.total_shares.units())?;
        let remaining = self.reserve_balance.checked_sub(assets)?;
        if remaining < self.config.reserve_floor {
            return Err(NexusError::Policy("vault reserve floor reached".to_owned()));
        }
        let mut position = self.position_of(ticket.owner);
        position.locked_shares = position.locked_shares.checked_sub(ticket.shares)?;
        self.locked_shares = self.locked_shares.checked_sub(ticket.shares)?;
        self.total_shares = self.total_shares.checked_sub(ticket.shares)?;
        self.reserve_balance = remaining;
        ticket.settled = true;
        self.positions.insert(ticket.owner, position);
        self.exit_tickets.insert(ticket_id, ticket);
        Ok(assets)
    }

    pub fn liquidity_digest(&self) -> NexusResult<Digest> {
        Digest::from_serializable(
            "nexus-vault-liquidity-v1",
            &(
                self.config.vault_id,
                self.reserve_balance,
                self.total_shares,
                self.locked_shares,
                self.pending_fees,
            ),
        )
    }
}
