//! `LoanAgreement` contract (COMP-201).

use agora_domain::ids::{AccountId, AssetId};
use agora_domain::money::Amount;

use crate::scheduler::LogicalTime;

/// Terms of a loan.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct LoanTerms {
    /// Creditor.
    pub lender: AccountId,
    /// Debtor.
    pub borrower: AccountId,
    /// Asset lent.
    pub asset: AssetId,
    /// Principal.
    pub principal: Amount,
    /// Interest per period, basis points.
    pub rate_bps: u32,
    /// Period length.
    pub period: LogicalTime,
    /// Number of instalments.
    pub instalments: u32,
}
