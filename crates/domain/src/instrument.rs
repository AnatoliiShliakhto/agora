//! Tradable instrument specification.

use core::num::NonZeroU32;

use crate::error::{ArithmeticError, RejectReason};
use crate::ids::{AssetId, InstrumentId};
use crate::money::{Amount, Price, Qty, RoundingMode};
use crate::registry::Symbol;

/// Conversion factor from `ticks × lots` to quote minor units, as a rational `num / den`.
///
/// Example: EUR/USD with tick `0.0001 USD`, lot `1000 EUR`, USD minor unit = cent gives
/// `1 tick × 1 lot = 0.1 USD = 10 cents`, i.e. `num = 10, den = 1`. Tick `0.00001` with lot `1`
/// gives `0.001 cent`, i.e. `num = 1, den = 1000`. Both parts are non-zero by construction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct NotionalScale {
    num: NonZeroU32,
    den: NonZeroU32,
}

impl NotionalScale {
    /// Scale `num / den`.
    #[must_use]
    pub const fn new(num: NonZeroU32, den: NonZeroU32) -> Self {
        Self { num, den }
    }

    /// Numerator, in quote minor units.
    #[must_use]
    pub const fn num(self) -> i128 {
        self.num.get() as i128
    }

    /// Denominator.
    #[must_use]
    pub const fn den(self) -> i128 {
        self.den.get() as i128
    }
}

/// Inclusive price band an order's prices must lie in.
///
/// Signed, so an instrument that legitimately trades below zero (a spread, some futures) sets a
/// negative `min`; a spot pair sets `min` to one tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "PriceBoundsRepr", into = "PriceBoundsRepr")]
pub struct PriceBounds {
    min: Price,
    max: Price,
}

/// Serialized form of [`PriceBounds`], validated on the way in.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
struct PriceBoundsRepr {
    min: Price,
    max: Price,
}

/// Error of building [`PriceBounds`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[error("price band is inverted: min {min} > max {max}")]
pub struct InvertedBounds {
    /// Lower bound.
    pub min: Price,
    /// Upper bound.
    pub max: Price,
}

impl PriceBounds {
    /// Band `min..=max`.
    ///
    /// # Errors
    ///
    /// [`InvertedBounds`] if `min > max`.
    pub const fn new(min: Price, max: Price) -> Result<Self, InvertedBounds> {
        if min.ticks() > max.ticks() {
            Err(InvertedBounds { min, max })
        } else {
            Ok(Self { min, max })
        }
    }

    /// Lower bound.
    #[must_use]
    pub const fn min(self) -> Price {
        self.min
    }

    /// Upper bound.
    #[must_use]
    pub const fn max(self) -> Price {
        self.max
    }

    /// Returns `true` if `price` lies in the band.
    #[must_use]
    pub const fn contains(self, price: Price) -> bool {
        self.min.ticks() <= price.ticks() && price.ticks() <= self.max.ticks()
    }
}

impl TryFrom<PriceBoundsRepr> for PriceBounds {
    type Error = InvertedBounds;

    fn try_from(repr: PriceBoundsRepr) -> Result<Self, Self::Error> {
        Self::new(repr.min, repr.max)
    }
}

impl From<PriceBounds> for PriceBoundsRepr {
    fn from(bounds: PriceBounds) -> Self {
        Self { min: bounds.min, max: bounds.max }
    }
}

/// Static specification of a tradable instrument.
///
/// Immutable for the lifetime of a shard; changing tick or lot size requires a new instrument
/// (all resting orders are expressed in these units).
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Instrument {
    /// Instrument identifier.
    pub id: InstrumentId,
    /// Human-readable symbol, e.g. `EURUSD`.
    pub symbol: Symbol,
    /// Asset being bought or sold.
    pub base: AssetId,
    /// Asset prices are quoted in.
    pub quote: AssetId,
    /// Conversion of `ticks × lots` to quote minor units.
    pub scale: NotionalScale,
    /// Band every order price must lie in.
    pub price_bounds: PriceBounds,
    /// Maximum allowed deviation of an execution price from the reference price, in ticks.
    ///
    /// `None` disables the deviation guard (only sensible in tests).
    pub max_deviation: Option<Price>,
}

impl Instrument {
    /// Checks that `price` lies in this instrument's band.
    ///
    /// # Errors
    ///
    /// [`RejectReason::InvalidPrice`].
    pub const fn check_price(&self, price: Price) -> Result<(), RejectReason> {
        if self.price_bounds.contains(price) { Ok(()) } else { Err(RejectReason::InvalidPrice) }
    }

    /// Computes the notional value of `qty` at `price` in quote minor units, rounded with
    /// `mode`.
    ///
    /// # Errors
    ///
    /// See [`ArithmeticError`].
    pub fn notional(
        &self,
        price: Price,
        qty: Qty,
        mode: RoundingMode,
    ) -> Result<Amount, ArithmeticError> {
        Amount::from_minor(i128::from(price.ticks()))
            .checked_mul(i128::from(qty.lots()))?
            .checked_mul(self.scale.num())?
            .checked_div(self.scale.den(), mode)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const fn nz(v: u32) -> NonZeroU32 {
        NonZeroU32::new(v).expect("non-zero")
    }

    fn eurusd() -> Instrument {
        Instrument {
            id: InstrumentId::new(1),
            symbol: Symbol::new("EURUSD").expect("valid symbol"),
            base: AssetId::new(1),
            quote: AssetId::new(2),
            scale: NotionalScale::new(nz(10), nz(1)),
            price_bounds: PriceBounds::new(Price::from_ticks(1), Price::from_ticks(1_000_000))
                .expect("min ≤ max"),
            max_deviation: None,
        }
    }

    #[test]
    fn notional_of_one_lot_at_one_tick_is_ten_cents() {
        let n = eurusd().notional(Price::from_ticks(1), Qty::from_lots(1), RoundingMode::Floor);
        assert_eq!(n, Ok(Amount::from_minor(10)));
    }

    #[test]
    fn notional_example_from_spec() {
        // Buy 2.6M EUR (2600 lots of 1000) at 1.1200 (11_200 ticks) = 2_912_000 USD.
        let n = eurusd().notional(
            Price::from_ticks(11_200),
            Qty::from_lots(2_600),
            RoundingMode::Floor,
        );
        assert_eq!(n, Ok(Amount::from_minor(291_200_000)));
    }

    #[test]
    fn price_band_is_inclusive_and_rejects_outside() {
        let inst = eurusd();
        assert_eq!(inst.check_price(Price::from_ticks(1)), Ok(()));
        assert_eq!(inst.check_price(Price::from_ticks(1_000_000)), Ok(()));
        assert_eq!(inst.check_price(Price::from_ticks(0)), Err(RejectReason::InvalidPrice));
        assert_eq!(inst.check_price(Price::from_ticks(1_000_001)), Err(RejectReason::InvalidPrice));
    }

    #[test]
    fn inverted_price_bounds_are_rejected() {
        let (min, max) = (Price::from_ticks(10), Price::from_ticks(9));
        assert_eq!(PriceBounds::new(min, max), Err(InvertedBounds { min, max }));
        let json = r#"{"min":10,"max":9}"#;
        assert!(
            serde_json::from_str::<PriceBounds>(json).is_err(),
            "serde must reject inverted bounds"
        );
    }

    #[test]
    fn fractional_scale_rounds_as_requested() {
        let mut inst = eurusd();
        inst.scale = NotionalScale::new(nz(1), nz(1000));
        let ticks_lots = (Price::from_ticks(1_500), Qty::from_lots(1)); // 1.5 cents
        assert_eq!(
            inst.notional(ticks_lots.0, ticks_lots.1, RoundingMode::Floor),
            Ok(Amount::from_minor(1))
        );
        assert_eq!(
            inst.notional(ticks_lots.0, ticks_lots.1, RoundingMode::Ceil),
            Ok(Amount::from_minor(2))
        );
        assert_eq!(
            inst.notional(ticks_lots.0, ticks_lots.1, RoundingMode::HalfEven),
            Ok(Amount::from_minor(2))
        );
    }
}
