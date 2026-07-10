use thiserror::Error;

use crate::{AccountId, Amount, AssetId, BatchId, Digest, FacilityId, IntentId, TxId, VaultId};

pub type NexusResult<T> = Result<T, NexusError>;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum NexusError {
    #[error("amount overflow")]
    AmountOverflow,
    #[error("amount underflow")]
    AmountUnderflow,
    #[error("division by zero")]
    DivisionByZero,
    #[error("zero amount")]
    ZeroAmount,
    #[error("basis points out of range: {0}")]
    BpsOutOfRange(u16),
    #[error("serialization error: {0}")]
    Serialization(String),
    #[error("signature error: {0}")]
    Signature(String),
    #[error("account already exists: {0}")]
    AccountAlreadyExists(AccountId),
    #[error("account not found: {0}")]
    AccountNotFound(AccountId),
    #[error("asset already exists: {0}")]
    AssetAlreadyExists(AssetId),
    #[error("asset not found: {0}")]
    AssetNotFound(AssetId),
    #[error("vault not found: {0}")]
    VaultNotFound(VaultId),
    #[error("facility not found: {0}")]
    FacilityNotFound(FacilityId),
    #[error("intent already settled: {0}")]
    IntentSettled(IntentId),
    #[error("batch already executed: {0}")]
    BatchExecuted(BatchId),
    #[error("duplicate transaction: {0}")]
    DuplicateTransaction(TxId),
    #[error(
        "insufficient funds for {account} on {asset}: available {available}, required {required}"
    )]
    InsufficientFunds {
        account: AccountId,
        asset: AssetId,
        available: Amount,
        required: Amount,
    },
    #[error(
        "insufficient shares in vault {vault}: owner {owner}, available {available}, required {required}"
    )]
    InsufficientShares {
        vault: VaultId,
        owner: AccountId,
        available: Amount,
        required: Amount,
    },
    #[error("nonce mismatch for {account}: expected {expected}, received {received}")]
    NonceMismatch {
        account: AccountId,
        expected: u64,
        received: u64,
    },
    #[error("unauthorized signer: expected {expected}, received {received}")]
    UnauthorizedSigner {
        expected: AccountId,
        received: AccountId,
    },
    #[error("digest mismatch: expected {expected}, received {received}")]
    DigestMismatch { expected: Digest, received: Digest },
    #[error("policy violation: {0}")]
    Policy(String),
    #[error("conservation error for {asset}: expected {expected}, observed {observed}")]
    Conservation {
        asset: AssetId,
        expected: Amount,
        observed: Amount,
    },
}
