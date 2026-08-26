//! Double-entry postings.

use agora_domain::ids::{AccountId, AssetId};
use agora_domain::money::Amount;

/// One leg of a balanced posting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Leg {
    /// Account.
    pub account: AccountId,
    /// Asset.
    pub asset: AssetId,
    /// Positive for credit, negative for debit.
    pub delta: Amount,
}

/// Set of legs that must sum to zero per asset. Built by settlement, validated before apply.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Posting {
    /// Legs.
    pub legs: Vec<Leg>,
}
