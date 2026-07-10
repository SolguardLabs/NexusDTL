use serde::Serialize;

use crate::{
    AccountId, Amount, AssetId, Digest, IntentId, KeyPair, NexusError, NexusResult, PublicIdentity,
    RfqQuote, SignatureBytes, VaultId, verify_signature,
};

pub const INTENT_DOMAIN: &str = "nexus-execution-intent-v1";

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ExecutionIntent {
    pub network_id: u32,
    pub intent_id: IntentId,
    pub owner: AccountId,
    pub receiver: AccountId,
    pub vault_id: VaultId,
    pub source_asset: AssetId,
    pub target_asset: AssetId,
    pub source_amount: Amount,
    pub min_output: Amount,
    pub owner_nonce: u64,
    pub quote: RfqQuote,
    pub salt: Digest,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IntentAuthorizationView {
    network_id: u32,
    intent_id: IntentId,
    owner: AccountId,
    receiver: AccountId,
    vault_id: VaultId,
    source_asset: AssetId,
    target_asset: AssetId,
    source_amount: Amount,
    min_output: Amount,
    owner_nonce: u64,
    quote_digest: Digest,
    salt: Digest,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SignedIntent {
    pub signer: PublicIdentity,
    pub intent: ExecutionIntent,
    pub signature: SignatureBytes,
}

impl ExecutionIntent {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        network_id: u32,
        owner: AccountId,
        receiver: AccountId,
        vault_id: VaultId,
        source_asset: AssetId,
        target_asset: AssetId,
        source_amount: Amount,
        min_output: Amount,
        owner_nonce: u64,
        quote: RfqQuote,
        salt: Digest,
    ) -> NexusResult<Self> {
        if source_amount.is_zero() {
            return Err(NexusError::ZeroAmount);
        }
        if quote.vault_id != vault_id
            || quote.source_asset != source_asset
            || quote.target_asset != target_asset
        {
            return Err(NexusError::Policy("quote/intent mismatch".to_owned()));
        }
        let intent_id = IntentId::from_serializable(
            "nexus-intent-id-v1",
            &(
                network_id,
                owner,
                receiver,
                vault_id,
                source_asset,
                target_asset,
                owner_nonce,
                salt,
            ),
        )?;
        Ok(Self {
            network_id,
            intent_id,
            owner,
            receiver,
            vault_id,
            source_asset,
            target_asset,
            source_amount,
            min_output,
            owner_nonce,
            quote,
            salt,
        })
    }

    pub fn authorization_view(self) -> NexusResult<IntentAuthorizationView> {
        Ok(IntentAuthorizationView {
            network_id: self.network_id,
            intent_id: self.intent_id,
            owner: self.owner,
            receiver: self.receiver,
            vault_id: self.vault_id,
            source_asset: self.source_asset,
            target_asset: self.target_asset,
            source_amount: self.source_amount,
            min_output: self.min_output,
            owner_nonce: self.owner_nonce,
            quote_digest: self.quote.digest()?,
            salt: self.salt,
        })
    }
}

impl SignedIntent {
    pub fn sign(intent: ExecutionIntent, key_pair: &KeyPair) -> NexusResult<Self> {
        let signer = key_pair.public_identity();
        if signer.account != intent.owner {
            return Err(NexusError::UnauthorizedSigner {
                expected: intent.owner,
                received: signer.account,
            });
        }
        let signature = key_pair.sign(INTENT_DOMAIN, &intent.authorization_view()?)?;
        Ok(Self {
            signer,
            intent,
            signature,
        })
    }

    pub fn verify(&self) -> NexusResult<()> {
        if self.signer.account != self.intent.owner {
            return Err(NexusError::UnauthorizedSigner {
                expected: self.intent.owner,
                received: self.signer.account,
            });
        }
        verify_signature(
            self.signer,
            self.signature,
            INTENT_DOMAIN,
            &self.intent.authorization_view()?,
        )
    }
}
