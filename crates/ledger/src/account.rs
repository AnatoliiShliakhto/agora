//! Accounts and balances.

use agora_domain::error::ArithmeticError;
use agora_domain::money::Amount;

/// Balance of one asset in one account.
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct Balance {
    /// Freely usable.
    pub available: Amount,
    /// Locked by open reservations.
    pub reserved: Amount,
}

impl Balance {
    /// `available + reserved`.
    ///
    /// # Errors
    ///
    /// [`ArithmeticError::Overflow`] if the sum does not fit.
    pub const fn total(self) -> Result<Amount, ArithmeticError> {
        self.available.checked_add(self.reserved)
    }
}
