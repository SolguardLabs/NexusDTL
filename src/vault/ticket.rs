use serde::Serialize;

use crate::{AccountId, Amount, Digest, NexusResult, VaultId};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ExitTicket {
    pub ticket_id: Digest,
    pub vault_id: VaultId,
    pub owner: AccountId,
    pub shares: Amount,
    pub requested_epoch: u64,
    pub executable_epoch: u64,
    pub settled: bool,
}

impl ExitTicket {
    pub fn new(
        vault_id: VaultId,
        owner: AccountId,
        shares: Amount,
        requested_epoch: u64,
        executable_epoch: u64,
    ) -> NexusResult<Self> {
        let ticket_id = Digest::from_serializable(
            "nexus-exit-ticket-v1",
            &(vault_id, owner, shares, requested_epoch, executable_epoch),
        )?;
        Ok(Self {
            ticket_id,
            vault_id,
            owner,
            shares,
            requested_epoch,
            executable_epoch,
            settled: false,
        })
    }
}
