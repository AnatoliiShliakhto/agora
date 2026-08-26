//! Canonical fixtures.

use core::num::NonZeroU32;

use agora_domain::ids::{AccountId, AssetId, InstrumentId};
use agora_domain::instrument::{Instrument, NotionalScale, PriceBounds};
use agora_domain::money::Price;
use agora_domain::registry::Symbol;

/// `1 tick × 1 lot` of [`eurusd`] in cents.
const EURUSD_NUM: NonZeroU32 = NonZeroU32::new(10).expect("non-zero literal");

/// Price band of [`eurusd`]: `0.0001`…`100.0000`.
const EURUSD_BOUNDS: PriceBounds =
    match PriceBounds::new(Price::from_ticks(1), Price::from_ticks(1_000_000)) {
        Ok(bounds) => bounds,
        Err(_) => panic!("min ≤ max"),
    };

/// EUR/USD: tick `0.0001`, lot `1000 EUR`, USD cents; deviation guard 50 ticks.
#[must_use]
#[expect(clippy::missing_panics_doc, reason = "symbol literal is valid")]
pub fn eurusd() -> Instrument {
    Instrument {
        id: InstrumentId::new(1),
        symbol: Symbol::new("EURUSD").expect("valid symbol"),
        base: AssetId::new(1),
        quote: AssetId::new(2),
        scale: NotionalScale::new(EURUSD_NUM, NonZeroU32::MIN),
        price_bounds: EURUSD_BOUNDS,
        max_deviation: Some(Price::from_ticks(50)),
    }
}

/// First test account.
#[must_use]
pub const fn alice() -> AccountId {
    AccountId::new(1)
}

/// Second test account.
#[must_use]
pub const fn bob() -> AccountId {
    AccountId::new(2)
}
