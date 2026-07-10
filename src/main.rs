mod amount;
mod auction;
mod codec;
mod credit;
mod crypto;
mod error;
mod governance;
mod ids;
mod ledger;
mod market;
mod netting;
mod oracle;
mod risk;
mod runtime;
mod treasury;
mod vault;

pub use amount::{Amount, Bps};
pub use auction::{AuctionBook, AuctionLane, SolverBid};
pub use codec::canonical_bytes;
pub use credit::{CreditBook, CreditFacility, CreditPosition};
pub use crypto::{KeyPair, PublicIdentity, SignatureBytes, verify_signature};
pub use error::{NexusError, NexusResult};
pub use governance::{GovernanceConfig, OperatorRegistry, OperatorRole};
pub use ids::{AccountId, AssetId, BatchId, Digest, FacilityId, IntentId, TxId, VaultId};
pub use ledger::{AccountState, JournalEntry, JournalOp, NexusLedger};
pub use market::{
    AssetConfig, ExecutionIntent, IntentAuthorizationView, RfqQuote, SettlementBatch, SignedIntent,
    SignedSettlement,
};
pub use netting::{NetAssetFlow, NettingEngine, NettingReport, SettlementObligation};
pub use oracle::{OracleBook, OracleObservation, PriceBand};
pub use risk::{RiskEngine, RiskLimits, RiskSnapshot};
pub use runtime::ScenarioReport;
pub use treasury::{FeePolicy, InsuranceFund, TreasuryBook};
pub use vault::{ExitTicket, LiquidityVault, VaultConfig, VaultPosition};

fn main() {
    if let Err(error) = runtime::run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
