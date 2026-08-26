//! Proptest strategies.

use agora_domain::money::{Price, Qty};
use agora_domain::order::Side;
use proptest::prelude::*;

/// Prices within a realistic band around a mid of `mid` ± `band` ticks.
pub fn price_near(mid: i64, band: i64) -> impl Strategy<Value = Price> {
    (mid.saturating_sub(band)..=mid.saturating_add(band)).prop_map(Price::from_ticks)
}

/// Non-zero quantities up to `max` lots.
pub fn qty_up_to(max: u64) -> impl Strategy<Value = Qty> {
    (1..=max.max(1)).prop_map(Qty::from_lots)
}

/// Either side.
pub fn side() -> impl Strategy<Value = Side> {
    prop_oneof![Just(Side::Buy), Just(Side::Sell)]
}
