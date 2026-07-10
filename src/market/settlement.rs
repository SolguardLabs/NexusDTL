use serde::Serialize;

use crate::{
    AccountId, BatchId, Digest, KeyPair, NexusError, NexusResult, PublicIdentity, SignatureBytes,
    SignedIntent, VaultId, verify_signature,
};

pub const SETTLEMENT_DOMAIN: &str = "nexus-settlement-batch-v1";

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SettlementBatch {
    pub network_id: u32,
    pub batch_id: BatchId,
    pub coordinator: AccountId,
    pub solver: AccountId,
    pub vault_id: VaultId,
    pub epoch: u64,
    pub intents_digest: Digest,
    pub liquidity_snapshot: Digest,
    pub netting_digest: Digest,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SignedSettlement {
    pub signer: PublicIdentity,
    pub batch: SettlementBatch,
    pub signature: SignatureBytes,
}

impl SettlementBatch {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        network_id: u32,
        coordinator: AccountId,
        solver: AccountId,
        vault_id: VaultId,
        epoch: u64,
        intents: &[SignedIntent],
        liquidity_snapshot: Digest,
        netting_digest: Digest,
    ) -> NexusResult<Self> {
        let intents_digest = Digest::from_serializable("nexus-batch-intents-v1", &intents)?;
        let batch_id = BatchId::from_serializable(
            "nexus-batch-id-v1",
            &(
                network_id,
                coordinator,
                solver,
                vault_id,
                epoch,
                intents_digest,
                liquidity_snapshot,
                netting_digest,
            ),
        )?;
        Ok(Self {
            network_id,
            batch_id,
            coordinator,
            solver,
            vault_id,
            epoch,
            intents_digest,
            liquidity_snapshot,
            netting_digest,
        })
    }
}

impl SignedSettlement {
    pub fn sign(batch: SettlementBatch, key_pair: &KeyPair) -> NexusResult<Self> {
        let signer = key_pair.public_identity();
        if signer.account != batch.coordinator {
            return Err(NexusError::UnauthorizedSigner {
                expected: batch.coordinator,
                received: signer.account,
            });
        }
        let signature = key_pair.sign(SETTLEMENT_DOMAIN, &batch)?;
        Ok(Self {
            signer,
            batch,
            signature,
        })
    }

    pub fn verify(&self) -> NexusResult<()> {
        if self.signer.account != self.batch.coordinator {
            return Err(NexusError::UnauthorizedSigner {
                expected: self.batch.coordinator,
                received: self.signer.account,
            });
        }
        verify_signature(self.signer, self.signature, SETTLEMENT_DOMAIN, &self.batch)
    }
}
