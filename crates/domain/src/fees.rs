//! Fee rates and schedules (ADR-0010).

use crate::error::ArithmeticError;
use crate::money::{Amount, RoundingMode};

/// Basis points in one unit (100 %).
pub const BPS_PER_UNIT: u32 = 10_000;

/// Fee rate in basis points (1 bp = 0.01 %), at most [`BPS_PER_UNIT`].
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
#[serde(try_from = "u32", into = "u32")]
pub struct FeeBps(u32);

impl FeeBps {
    /// 100 %.
    pub const MAX: Self = Self(BPS_PER_UNIT);
    /// No fee.
    pub const ZERO: Self = Self(0);

    /// Rate of `bps` basis points; `None` above 100 %.
    #[must_use]
    pub const fn new(bps: u32) -> Option<Self> {
        if bps <= BPS_PER_UNIT { Some(Self(bps)) } else { None }
    }

    /// Raw basis points.
    #[must_use]
    pub const fn bps(self) -> u32 {
        self.0
    }

    /// Computes `amount × rate`, rounded with `mode`.
    ///
    /// Charging a participant uses [`RoundingMode::Ceil`]; paying out uses
    /// [`RoundingMode::Floor`]. The result never exceeds `amount` in magnitude.
    ///
    /// # Errors
    ///
    /// See [`ArithmeticError`].
    pub fn of(self, amount: Amount, mode: RoundingMode) -> Result<Amount, ArithmeticError> {
        amount.checked_mul(i128::from(self.0))?.checked_div(i128::from(BPS_PER_UNIT), mode)
    }
}

impl TryFrom<u32> for FeeBps {
    type Error = ArithmeticError;

    fn try_from(bps: u32) -> Result<Self, Self::Error> {
        Self::new(bps).ok_or(ArithmeticError::Overflow)
    }
}

impl From<FeeBps> for u32 {
    fn from(rate: FeeBps) -> Self {
        rate.0
    }
}

/// Maker and taker rates of an instrument. Fees are charged, so they round up.
#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct FeeSchedule {
    /// Charged to the resting side of a trade.
    pub maker: FeeBps,
    /// Charged to the aggressing side of a trade.
    pub taker: FeeBps,
}

impl FeeSchedule {
    /// Maker fee on `notional`, rounded up.
    ///
    /// # Errors
    ///
    /// See [`ArithmeticError`].
    pub fn maker_fee(self, notional: Amount) -> Result<Amount, ArithmeticError> {
        self.maker.of(notional, RoundingMode::Ceil)
    }

    /// Taker fee on `notional`, rounded up.
    ///
    /// # Errors
    ///
    /// See [`ArithmeticError`].
    pub fn taker_fee(self, notional: Amount) -> Result<Amount, ArithmeticError> {
        self.taker.of(notional, RoundingMode::Ceil)
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    fn bps(v: u32) -> FeeBps {
        FeeBps::new(v).expect("valid rate")
    }

    #[test]
    fn rate_is_capped_at_one_hundred_percent() {
        assert_eq!(FeeBps::new(10_000), Some(FeeBps::MAX));
        assert_eq!(FeeBps::new(10_001), None);
        assert!(serde_json::from_str::<FeeBps>("10001").is_err(), "serde must reject > 100 %");
    }

    #[test]
    fn taker_fee_example_from_domain_doc() {
        // 10 bps on 2_911_570 USD = 2_911.57 USD → 291_157 cents.
        let schedule = FeeSchedule { maker: bps(2), taker: bps(10) };
        assert_eq!(
            schedule.taker_fee(Amount::from_minor(291_157_000)),
            Ok(Amount::from_minor(291_157))
        );
        // 7 bps on 12_345 cents = 8.6415 → charged 9.
        assert_eq!(
            bps(7).of(Amount::from_minor(12_345), RoundingMode::Ceil),
            Ok(Amount::from_minor(9))
        );
        assert_eq!(
            bps(7).of(Amount::from_minor(12_345), RoundingMode::Floor),
            Ok(Amount::from_minor(8))
        );
    }

    proptest! {
        #[test]
        fn fee_never_exceeds_the_amount(
            minor in -1_000_000_000_000_i128..=1_000_000_000_000,
            rate in 0..=BPS_PER_UNIT,
        ) {
            let amount = Amount::from_minor(minor);
            for mode in [RoundingMode::Floor, RoundingMode::Ceil, RoundingMode::HalfEven] {
                let fee = bps(rate).of(amount, mode).expect("no overflow in range");
                prop_assert!(fee.minor().abs() <= minor.abs(), "|fee| ≤ |amount| for {mode:?}");
                if rate == BPS_PER_UNIT {
                    prop_assert_eq!(fee, amount, "100 % is the whole amount");
                }
            }
        }

        #[test]
        fn charged_fee_is_at_least_the_paid_out_fee(
            minor in 0_i128..=1_000_000_000_000,
            rate in 0..=BPS_PER_UNIT,
        ) {
            let amount = Amount::from_minor(minor);
            let charged = bps(rate).of(amount, RoundingMode::Ceil).expect("no overflow");
            let paid = bps(rate).of(amount, RoundingMode::Floor).expect("no overflow");
            prop_assert!(charged >= paid, "rounding must favour the exchange");
        }
    }
}
