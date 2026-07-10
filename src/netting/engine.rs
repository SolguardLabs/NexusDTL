use std::collections::BTreeMap;

use serde::Serialize;

use crate::{AccountId, Amount, AssetId, Digest, NexusError, NexusResult};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SettlementObligation {
    pub account: AccountId,
    pub asset: AssetId,
    pub debit: Amount,
    pub credit: Amount,
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct NetAssetFlow {
    pub gross_debit: Amount,
    pub gross_credit: Amount,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct NettingReport {
    pub report_digest: Digest,
    pub flows: BTreeMap<AssetId, NetAssetFlow>,
    pub obligation_count: usize,
    pub batch_count: usize,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct NettingEngine {
    reports: BTreeMap<Digest, NettingReport>,
}

impl SettlementObligation {
    pub fn debit(account: AccountId, asset: AssetId, amount: Amount) -> Self {
        Self {
            account,
            asset,
            debit: amount,
            credit: Amount::zero(),
        }
    }

    pub fn credit(account: AccountId, asset: AssetId, amount: Amount) -> Self {
        Self {
            account,
            asset,
            debit: Amount::zero(),
            credit: amount,
        }
    }
}

impl NettingEngine {
    pub fn digest_for(obligations: &[SettlementObligation]) -> NexusResult<Digest> {
        if obligations.is_empty() {
            return Err(NexusError::Policy(
                "netting obligations are empty".to_owned(),
            ));
        }
        Digest::from_serializable("nexus-netting-report-v1", &obligations)
    }

    pub fn record_obligations(
        &mut self,
        obligations: Vec<SettlementObligation>,
    ) -> NexusResult<NettingReport> {
        let report_digest = Self::digest_for(&obligations)?;
        let mut flows = BTreeMap::new();
        for obligation in &obligations {
            let flow = flows
                .entry(obligation.asset)
                .or_insert_with(NetAssetFlow::default);
            flow.gross_debit = flow.gross_debit.checked_add(obligation.debit)?;
            flow.gross_credit = flow.gross_credit.checked_add(obligation.credit)?;
        }
        for flow in flows.values() {
            if flow.gross_debit != flow.gross_credit {
                return Err(NexusError::Policy("netting flow is unbalanced".to_owned()));
            }
        }
        let report = NettingReport {
            report_digest,
            flows,
            obligation_count: obligations.len(),
            batch_count: 1,
        };
        self.reports.insert(report_digest, report.clone());
        Ok(report)
    }

    pub fn report_count(&self) -> usize {
        self.reports.len()
    }
}
