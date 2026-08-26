//! Fixed-point money types.
//!
//! All values are integers scaled by the instrument (see
//! [`Instrument`](crate::instrument::Instrument)). Floating point is forbidden across the
//! workspace (`clippy::float_arithmetic = deny`); decimals exist only at the API boundary.

use core::fmt;

use crate::error::ArithmeticError;

macro_rules! checked_ops {
    ($name:ident, $inner:ty) => {
        impl $name {
            /// Maximum representable value.
            pub const MAX: Self = Self(<$inner>::MAX);
            /// Zero value.
            pub const ZERO: Self = Self(0);

            /// Checked addition.
            ///
            /// # Errors
            ///
            /// [`ArithmeticError::Overflow`] if the result does not fit.
            pub const fn checked_add(self, rhs: Self) -> Result<Self, ArithmeticError> {
                match self.0.checked_add(rhs.0) {
                    Some(v) => Ok(Self(v)),
                    None => Err(ArithmeticError::Overflow),
                }
            }

            /// Checked subtraction.
            ///
            /// # Errors
            ///
            /// [`ArithmeticError::Overflow`] if the result does not fit (including going below
            /// zero for unsigned types).
            pub const fn checked_sub(self, rhs: Self) -> Result<Self, ArithmeticError> {
                match self.0.checked_sub(rhs.0) {
                    Some(v) => Ok(Self(v)),
                    None => Err(ArithmeticError::Overflow),
                }
            }

            /// Returns `true` if the value is zero.
            #[must_use]
            pub const fn is_zero(self) -> bool {
                self.0 == 0
            }
        }
    };
}

/// Rounding direction of a division. Always a parameter, never a default (ADR-0002).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum RoundingMode {
    /// Toward negative infinity: paying out to a participant.
    Floor,
    /// Toward positive infinity: charging a participant.
    Ceil,
    /// Nearest, ties to even: reporting and pro-rata splits where bias matters.
    HalfEven,
}

/// Price in integer ticks of the instrument's tick size.
///
/// Signed: some instruments (spreads, certain futures) legitimately trade below zero.
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
pub struct Price(i64);

impl Price {
    /// Builds a price from raw ticks.
    #[must_use]
    pub const fn from_ticks(ticks: i64) -> Self {
        Self(ticks)
    }

    /// Raw tick count.
    #[must_use]
    pub const fn ticks(self) -> i64 {
        self.0
    }
}

checked_ops!(Price, i64);

/// Quantity in integer lots of the instrument's lot size.
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
pub struct Qty(u64);

impl Qty {
    /// Builds a quantity from raw lots.
    #[must_use]
    pub const fn from_lots(lots: u64) -> Self {
        Self(lots)
    }

    /// Raw lot count.
    #[must_use]
    pub const fn lots(self) -> u64 {
        self.0
    }
}

checked_ops!(Qty, u64);

/// Monetary amount in minor units of an asset (e.g. cents).
///
/// `i128` so that `price × qty` for any realistic instrument cannot overflow.
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
pub struct Amount(i128);

impl Amount {
    /// Builds an amount from raw minor units.
    #[must_use]
    pub const fn from_minor(minor: i128) -> Self {
        Self(minor)
    }

    /// Raw minor units.
    #[must_use]
    pub const fn minor(self) -> i128 {
        self.0
    }

    /// Checked multiplication by an integer factor.
    ///
    /// # Errors
    ///
    /// [`ArithmeticError::Overflow`] if the result does not fit.
    pub const fn checked_mul(self, factor: i128) -> Result<Self, ArithmeticError> {
        match self.0.checked_mul(factor) {
            Some(v) => Ok(Self(v)),
            None => Err(ArithmeticError::Overflow),
        }
    }

    /// Checked division with an explicit [`RoundingMode`].
    ///
    /// # Errors
    ///
    /// [`ArithmeticError::DivisionByZero`] if `divisor` is zero,
    /// [`ArithmeticError::Overflow`] on `i128::MIN / -1`.
    pub const fn checked_div(
        self,
        divisor: i128,
        mode: RoundingMode,
    ) -> Result<Self, ArithmeticError> {
        if divisor == 0 {
            return Err(ArithmeticError::DivisionByZero);
        }
        let (Some(quotient), Some(remainder)) =
            (self.0.checked_div(divisor), self.0.checked_rem(divisor))
        else {
            return Err(ArithmeticError::Overflow);
        };
        if remainder == 0 {
            return Ok(Self(quotient));
        }
        // `quotient` is truncated toward zero; `negative` says which way the exact value lies.
        let negative = (self.0 < 0) != (divisor < 0);
        let away_from_zero: i128 = if negative { -1 } else { 1 };
        let step = match mode {
            RoundingMode::Floor => {
                if negative {
                    -1
                } else {
                    0
                }
            },
            RoundingMode::Ceil => {
                if negative {
                    0
                } else {
                    1
                }
            },
            RoundingMode::HalfEven => {
                // |remainder| < |divisor| ≤ 2^127, so doubling cannot overflow u128.
                let twice_remainder = remainder.unsigned_abs().saturating_mul(2);
                let abs_divisor = divisor.unsigned_abs();
                let is_tie = twice_remainder == abs_divisor;
                if twice_remainder > abs_divisor || (is_tie && quotient & 1 == 1) {
                    away_from_zero
                } else {
                    0
                }
            },
        };
        match quotient.checked_add(step) {
            Some(v) => Ok(Self(v)),
            None => Err(ArithmeticError::Overflow),
        }
    }

    /// Negation.
    ///
    /// # Errors
    ///
    /// [`ArithmeticError::Overflow`] for `i128::MIN`.
    pub const fn checked_neg(self) -> Result<Self, ArithmeticError> {
        match self.0.checked_neg() {
            Some(v) => Ok(Self(v)),
            None => Err(ArithmeticError::Overflow),
        }
    }
}

checked_ops!(Amount, i128);

impl fmt::Display for Price {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}t", self.0)
    }
}

impl fmt::Display for Qty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}L", self.0)
    }
}

impl fmt::Display for Amount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}m", self.0)
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    #[test]
    fn qty_sub_below_zero_is_overflow() {
        assert_eq!(
            Qty::from_lots(1).checked_sub(Qty::from_lots(2)),
            Err(ArithmeticError::Overflow)
        );
    }

    fn div(a: i128, d: i128, mode: RoundingMode) -> Result<i128, ArithmeticError> {
        Amount::from_minor(a).checked_div(d, mode).map(Amount::minor)
    }

    #[test]
    fn division_examples_from_domain_doc() {
        use RoundingMode::{Ceil, Floor, HalfEven};
        // 8.6415 cents
        assert_eq!(div(12_345 * 7, 10_000, Ceil), Ok(9));
        assert_eq!(div(12_345 * 7, 10_000, Floor), Ok(8));
        assert_eq!(div(12_345 * 7, 10_000, HalfEven), Ok(9));
        // -8.6415 cents
        assert_eq!(div(-12_345 * 7, 10_000, Ceil), Ok(-8));
        assert_eq!(div(-12_345 * 7, 10_000, Floor), Ok(-9));
        assert_eq!(div(-12_345 * 7, 10_000, HalfEven), Ok(-9));
        // ties to even
        assert_eq!(div(5, 2, HalfEven), Ok(2));
        assert_eq!(div(7, 2, HalfEven), Ok(4));
        assert_eq!(div(-5, 2, HalfEven), Ok(-2));
        assert_eq!(div(-7, 2, HalfEven), Ok(-4));
        // negative divisor
        assert_eq!(div(7, -2, Floor), Ok(-4));
        assert_eq!(div(7, -2, Ceil), Ok(-3));
    }

    #[test]
    fn division_errors() {
        assert_eq!(div(7, 0, RoundingMode::Floor), Err(ArithmeticError::DivisionByZero));
        assert_eq!(div(i128::MIN, -1, RoundingMode::Floor), Err(ArithmeticError::Overflow));
    }

    proptest! {
        #[test]
        fn price_add_sub_roundtrip(a in any::<i64>(), b in any::<i64>()) {
            let (a, b) = (Price::from_ticks(a), Price::from_ticks(b));
            if let Ok(sum) = a.checked_add(b) {
                prop_assert_eq!(sum.checked_sub(b), Ok(a));
            }
        }

        #[test]
        fn rounding_modes_are_ordered_and_bracket_the_exact_quotient(
            a in any::<i128>(),
            d in any::<i128>().prop_filter("non-zero", |d| *d != 0),
        ) {
            let (Ok(floor), Ok(half), Ok(ceil)) = (
                div(a, d, RoundingMode::Floor),
                div(a, d, RoundingMode::HalfEven),
                div(a, d, RoundingMode::Ceil),
            ) else {
                // Only `i128::MIN / -1` overflows; every mode must agree on that.
                prop_assert_eq!((a, d), (i128::MIN, -1));
                return Ok(());
            };
            prop_assert!(floor <= half && half <= ceil, "{floor} ≤ {half} ≤ {ceil}");
            let gap = ceil.checked_sub(floor);
            prop_assert!(gap.is_some_and(|g| g == 0 || g == 1), "ceil - floor ∈ {{0, 1}}");
            // Exactness: a = floor × d + r where r is zero or has the sign of d and |r| < |d|
            // (floor is the largest integer not above the exact quotient). An overflowing
            // `floor × d` lies beyond `a` on the correct side, so it satisfies the bound trivially.
            if let Some(lo) = floor.checked_mul(d) {
                let r = a.checked_sub(lo).expect("|a - floor × d| < |d|");
                let sign_ok = r == 0 || ((r < 0) == (d < 0));
                prop_assert!(sign_ok && r.abs() < d.abs(), "remainder {r} invalid for {a} / {d}");
            }
        }

        #[test]
        fn qty_add_never_wraps(a in any::<u64>(), b in any::<u64>()) {
            let sum = Qty::from_lots(a).checked_add(Qty::from_lots(b));
            match sum {
                Ok(s) => prop_assert!(s.lots() >= a && s.lots() >= b),
                Err(e) => prop_assert_eq!(e, ArithmeticError::Overflow),
            }
        }
    }
}
