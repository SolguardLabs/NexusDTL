use std::collections::BTreeMap;

use serde::Serialize;

use crate::{AccountId, Amount, AssetId, NexusError, NexusResult, PublicIdentity};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct AccountState {
    pub identity: PublicIdentity,
    pub balances: BTreeMap<AssetId, Amount>,
    pub next_intent_nonce: u64,
}

impl AccountState {
    pub fn new(identity: PublicIdentity) -> Self {
        Self {
            identity,
            balances: BTreeMap::new(),
            next_intent_nonce: 0,
        }
    }

    pub fn balance_of(&self, asset: AssetId) -> Amount {
        self.balances
            .get(&asset)
            .copied()
            .unwrap_or_else(Amount::zero)
    }

    pub(crate) fn credit(&mut self, asset: AssetId, amount: Amount) -> NexusResult<()> {
        let next = self.balance_of(asset).checked_add(amount)?;
        self.set_balance(asset, next);
        Ok(())
    }

    pub(crate) fn debit(
        &mut self,
        account: AccountId,
        asset: AssetId,
        amount: Amount,
    ) -> NexusResult<()> {
        let available = self.balance_of(asset);
        if available < amount {
            return Err(NexusError::InsufficientFunds {
                account,
                asset,
                available,
                required: amount,
            });
        }
        self.set_balance(asset, available.checked_sub(amount)?);
        Ok(())
    }

    pub(crate) fn advance_intent_nonce(&mut self) -> NexusResult<()> {
        self.next_intent_nonce = self
            .next_intent_nonce
            .checked_add(1)
            .ok_or(NexusError::AmountOverflow)?;
        Ok(())
    }

    fn set_balance(&mut self, asset: AssetId, amount: Amount) {
        if amount.is_zero() {
            self.balances.remove(&asset);
        } else {
            self.balances.insert(asset, amount);
        }
    }
}
