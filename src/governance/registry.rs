use std::collections::BTreeMap;

use serde::Serialize;

use crate::{AccountId, Digest, NexusError, NexusResult};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OperatorRole {
    Coordinator,
    Solver,
    OraclePublisher,
    RiskAdmin,
    Treasurer,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct GovernanceConfig {
    pub current_epoch: u64,
    pub min_delay_epochs: u64,
    pub emergency_council: AccountId,
    pub config_digest: Digest,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct OperatorRegistry {
    roles: BTreeMap<AccountId, Vec<OperatorRole>>,
    config: Option<GovernanceConfig>,
    paused: bool,
}

impl GovernanceConfig {
    pub fn new(
        current_epoch: u64,
        min_delay_epochs: u64,
        emergency_council: AccountId,
        salt: Digest,
    ) -> NexusResult<Self> {
        let config_digest = Digest::from_serializable(
            "nexus-governance-config-v1",
            &(current_epoch, min_delay_epochs, emergency_council, salt),
        )?;
        Ok(Self {
            current_epoch,
            min_delay_epochs,
            emergency_council,
            config_digest,
        })
    }
}

impl OperatorRegistry {
    pub fn configure(&mut self, config: GovernanceConfig) {
        self.config = Some(config);
    }

    pub fn grant_role(&mut self, account: AccountId, role: OperatorRole) {
        let roles = self.roles.entry(account).or_default();
        if !roles.contains(&role) {
            roles.push(role);
            roles.sort();
        }
    }

    pub fn require_role(&self, account: AccountId, role: OperatorRole) -> NexusResult<()> {
        if self.has_role(account, role) {
            return Ok(());
        }
        Err(NexusError::Policy("operator role not assigned".to_owned()))
    }

    pub fn has_role(&self, account: AccountId, role: OperatorRole) -> bool {
        self.roles
            .get(&account)
            .is_some_and(|roles| roles.contains(&role))
    }

    pub fn set_paused(&mut self, paused: bool) {
        self.paused = paused;
    }

    pub fn ensure_not_paused(&self) -> NexusResult<()> {
        if self.paused {
            return Err(NexusError::Policy("protocol is paused".to_owned()));
        }
        Ok(())
    }

    pub fn operator_count(&self) -> usize {
        self.roles.len()
    }
}
