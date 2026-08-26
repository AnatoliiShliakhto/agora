//! Logical time.
//!
//! The domain never reads a clock: time enters as a `Tick` command carrying a
//! [`LogicalTime`], so replay is deterministic (ADR-0004).

use core::fmt;

use crate::error::ArithmeticError;

/// Monotonic timestamp in nanoseconds since the shard epoch.
#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
#[serde(transparent)]
pub struct LogicalTime(u64);

impl LogicalTime {
    /// Latest representable time.
    pub const MAX: Self = Self(u64::MAX);
    /// Shard epoch.
    pub const ZERO: Self = Self(0);

    /// Time `nanos` after the epoch.
    #[must_use]
    pub const fn from_nanos(nanos: u64) -> Self {
        Self(nanos)
    }

    /// Nanoseconds since the epoch.
    #[must_use]
    pub const fn nanos(self) -> u64 {
        self.0
    }

    /// Adds `nanos`.
    ///
    /// # Errors
    ///
    /// [`ArithmeticError::Overflow`] past [`LogicalTime::MAX`].
    pub const fn checked_add_nanos(self, nanos: u64) -> Result<Self, ArithmeticError> {
        match self.0.checked_add(nanos) {
            Some(v) => Ok(Self(v)),
            None => Err(ArithmeticError::Overflow),
        }
    }
}

impl fmt::Display for LogicalTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "t{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn addition_is_checked() {
        assert_eq!(LogicalTime::ZERO.checked_add_nanos(5), Ok(LogicalTime::from_nanos(5)));
        assert_eq!(LogicalTime::MAX.checked_add_nanos(1), Err(ArithmeticError::Overflow));
    }

    #[test]
    fn ordering_is_by_nanos() {
        assert!(LogicalTime::ZERO < LogicalTime::from_nanos(1), "epoch precedes later times");
    }
}
