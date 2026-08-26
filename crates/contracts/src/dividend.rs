//! Dividend distribution contract.

use agora_domain::ids::{AccountId, AssetId};
use agora_domain::money::Amount;

/// Declared dividend: `per_share` of `cash_asset` for each unit of `share_asset` held at the
/// record time, paid from `payer`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct DividendDeclaration {
    /// Paying account (the issuer).
    pub payer: AccountId,
    /// Asset whose holders are paid.
    pub share_asset: AssetId,
    /// Asset paid out.
    pub cash_asset: AssetId,
    /// Amount per share unit.
    pub per_share: Amount,
}
