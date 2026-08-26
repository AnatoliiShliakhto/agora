//! Order validation.
//!
//! [`NewOrder::validate`] is the only way to build a [`ValidOrder`], and the matching engine
//! accepts nothing else — an unvalidated order cannot reach the book by construction.
//!
//! Checks run in a fixed order so that a given command always produces the same
//! [`RejectReason`]:
//!
//! 1. the session accepts this kind of command → [`RejectReason::NotTrading`]
//! 2. the order targets this instrument → [`RejectReason::UnknownInstrument`]
//! 3. quantity is non-zero → [`RejectReason::InvalidQty`]
//! 4. every price lies inside the instrument's band → [`RejectReason::InvalidPrice`]
//! 5. a stop-limit's limit is reachable from its trigger → [`RejectReason::InconsistentStopLimit`]
//! 6. the time-in-force suits the order type → [`RejectReason::IncompatibleTimeInForce`]
//! 7. a GTD expiry is in the future → [`RejectReason::ExpiryInThePast`]
//! 8. the worst-case notional does not overflow → [`RejectReason::Arithmetic`]
//!
//! Everything that needs state beyond the order itself — funds, self-trade prevention,
//! fillability, deviation from the reference price, duplicate command ids — is checked later in
//! the shard pipeline, not here.

use crate::error::RejectReason;
use crate::instrument::Instrument;
use crate::money::{Price, RoundingMode};
use crate::order::{NewOrder, OrderType, Side, TimeInForce};
use crate::session::SessionState;
use crate::time::LogicalTime;

/// Order that passed [`NewOrder::validate`].
///
/// The inner order is immutable: any change means re-validating.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ValidOrder(NewOrder);

impl ValidOrder {
    /// Validated order.
    #[must_use]
    pub const fn as_new(&self) -> &NewOrder {
        &self.0
    }

    /// Unwraps the validated order.
    #[must_use]
    pub const fn into_inner(self) -> NewOrder {
        self.0
    }

    /// Limit price, for the order types that carry one.
    #[must_use]
    pub const fn limit(&self) -> Option<Price> {
        match self.0.kind {
            OrderType::Limit { limit } | OrderType::StopLimit { limit, .. } => Some(limit),
            OrderType::Market | OrderType::StopMarket { .. } => None,
        }
    }

    /// Trigger price, for stop orders.
    #[must_use]
    pub const fn trigger(&self) -> Option<Price> {
        match self.0.kind {
            OrderType::StopMarket { trigger } | OrderType::StopLimit { trigger, .. } => {
                Some(trigger)
            },
            OrderType::Limit { .. } | OrderType::Market => None,
        }
    }

    /// Returns `true` if the order waits for a trigger before it can trade.
    #[must_use]
    pub const fn is_stop(&self) -> bool {
        self.trigger().is_some()
    }
}

impl NewOrder {
    /// Validates this order against `instrument` and `session` at `now`.
    ///
    /// See the module documentation for the order of the checks.
    ///
    /// # Errors
    ///
    /// See [`RejectReason`].
    pub fn validate(
        self,
        instrument: &Instrument,
        session: SessionState,
        now: LogicalTime,
    ) -> Result<ValidOrder, RejectReason> {
        session.accepts(self.kind.command_kind())?;

        if self.instrument != instrument.id {
            return Err(RejectReason::UnknownInstrument);
        }
        if self.qty.is_zero() {
            return Err(RejectReason::InvalidQty);
        }

        match self.kind {
            OrderType::Limit { limit } => instrument.check_price(limit)?,
            OrderType::Market => {},
            OrderType::StopMarket { trigger } => instrument.check_price(trigger)?,
            OrderType::StopLimit { trigger, limit } => {
                instrument.check_price(trigger)?;
                instrument.check_price(limit)?;
                // After the trigger fires the order sweeps away from `trigger`, so a limit on the
                // near side of it could never fill and would rest for ever.
                let reachable = match self.side {
                    Side::Buy => limit >= trigger,
                    Side::Sell => limit <= trigger,
                };
                if !reachable {
                    return Err(RejectReason::InconsistentStopLimit);
                }
            },
        }

        // A market order never rests, so it cannot be good-till-cancelled or good-till-date.
        if matches!(self.kind, OrderType::Market)
            && matches!(self.tif, TimeInForce::Gtc | TimeInForce::Gtd { .. })
        {
            return Err(RejectReason::IncompatibleTimeInForce);
        }

        if let TimeInForce::Gtd { expires_at } = self.tif
            && expires_at <= now
        {
            return Err(RejectReason::ExpiryInThePast);
        }

        // Rounded up: the escrow reservation of P3.4 must not overflow either.
        if let Some(limit) = worst_case_price(&self) {
            instrument.notional(limit, self.qty, RoundingMode::Ceil)?;
        }

        Ok(ValidOrder(self))
    }
}

/// Highest price this order could trade at, if it is known before matching.
const fn worst_case_price(order: &NewOrder) -> Option<Price> {
    match order.kind {
        OrderType::Limit { limit } | OrderType::StopLimit { limit, .. } => Some(limit),
        // Market and stop-market orders are bounded by the deviation guard at execution time.
        OrderType::Market | OrderType::StopMarket { .. } => None,
    }
}

#[cfg(test)]
mod tests {
    use core::num::NonZeroU32;

    use rstest::rstest;

    use super::*;
    use crate::error::ArithmeticError;
    use crate::ids::{AccountId, AssetId, CommandId, InstrumentId};
    use crate::instrument::{NotionalScale, PriceBounds};
    use crate::money::Qty;
    use crate::order::SelfTradePrevention;
    use crate::registry::Symbol;

    const NOW: LogicalTime = LogicalTime::from_nanos(1_000);

    fn instrument() -> Instrument {
        Instrument {
            id: InstrumentId::new(1),
            symbol: Symbol::new("EURUSD").expect("valid symbol"),
            base: AssetId::new(1),
            quote: AssetId::new(2),
            scale: NotionalScale::new(NonZeroU32::MIN, NonZeroU32::MIN),
            price_bounds: PriceBounds::new(Price::from_ticks(1), Price::from_ticks(1_000_000))
                .expect("min ≤ max"),
            max_deviation: None,
        }
    }

    fn order(kind: OrderType, tif: TimeInForce, side: Side, qty: u64) -> NewOrder {
        NewOrder {
            command_id: CommandId::new(1),
            account: AccountId::new(1),
            instrument: InstrumentId::new(1),
            side,
            kind,
            tif,
            stp: SelfTradePrevention::CancelNewest,
            qty: Qty::from_lots(qty),
        }
    }

    fn limit(ticks: i64) -> OrderType {
        OrderType::Limit { limit: Price::from_ticks(ticks) }
    }

    fn validate(order: NewOrder) -> Result<ValidOrder, RejectReason> {
        order.validate(&instrument(), SessionState::Open, NOW)
    }

    #[test]
    fn accepts_a_plain_limit_order() {
        let valid = validate(order(limit(100), TimeInForce::Gtc, Side::Buy, 5)).expect("valid");
        assert_eq!(valid.limit(), Some(Price::from_ticks(100)));
        assert_eq!(valid.trigger(), None);
        assert!(!valid.is_stop(), "a limit order is not a stop");
        assert_eq!(valid.as_new().qty, Qty::from_lots(5));
    }

    #[test]
    fn rejects_when_the_session_does_not_accept_the_command() {
        let rejected = order(OrderType::Market, TimeInForce::Ioc, Side::Buy, 1).validate(
            &instrument(),
            SessionState::PreOpen,
            NOW,
        );
        assert_eq!(rejected, Err(RejectReason::NotTrading { state: SessionState::PreOpen }));
    }

    #[test]
    fn rejects_another_instrument() {
        let mut o = order(limit(100), TimeInForce::Gtc, Side::Buy, 1);
        o.instrument = InstrumentId::new(2);
        assert_eq!(validate(o), Err(RejectReason::UnknownInstrument));
    }

    #[test]
    fn rejects_zero_quantity() {
        assert_eq!(
            validate(order(limit(100), TimeInForce::Gtc, Side::Buy, 0)),
            Err(RejectReason::InvalidQty)
        );
    }

    #[rstest]
    #[case::below_band(0)]
    #[case::negative(-1)]
    #[case::above_band(1_000_001)]
    fn rejects_prices_outside_the_band(#[case] ticks: i64) {
        assert_eq!(
            validate(order(limit(ticks), TimeInForce::Gtc, Side::Buy, 1)),
            Err(RejectReason::InvalidPrice)
        );
    }

    #[rstest]
    #[case::buy_limit_below_trigger(Side::Buy, 100, 99)]
    #[case::sell_limit_above_trigger(Side::Sell, 100, 101)]
    fn rejects_unreachable_stop_limits(
        #[case] side: Side,
        #[case] trigger: i64,
        #[case] limit_ticks: i64,
    ) {
        let kind = OrderType::StopLimit {
            trigger: Price::from_ticks(trigger),
            limit: Price::from_ticks(limit_ticks),
        };
        assert_eq!(
            validate(order(kind, TimeInForce::Gtc, side, 1)),
            Err(RejectReason::InconsistentStopLimit)
        );
    }

    #[rstest]
    #[case::buy_limit_above_trigger(Side::Buy, 100, 101)]
    #[case::buy_limit_at_trigger(Side::Buy, 100, 100)]
    #[case::sell_limit_below_trigger(Side::Sell, 100, 99)]
    #[case::sell_limit_at_trigger(Side::Sell, 100, 100)]
    fn accepts_reachable_stop_limits(
        #[case] side: Side,
        #[case] trigger: i64,
        #[case] limit_ticks: i64,
    ) {
        let kind = OrderType::StopLimit {
            trigger: Price::from_ticks(trigger),
            limit: Price::from_ticks(limit_ticks),
        };
        let valid = validate(order(kind, TimeInForce::Gtc, side, 1)).expect("reachable");
        assert!(valid.is_stop(), "a stop-limit is a stop");
    }

    #[rstest]
    #[case::gtc(TimeInForce::Gtc)]
    #[case::gtd(TimeInForce::Gtd { expires_at: LogicalTime::MAX })]
    fn rejects_resting_time_in_force_on_market_orders(#[case] tif: TimeInForce) {
        assert_eq!(
            validate(order(OrderType::Market, tif, Side::Buy, 1)),
            Err(RejectReason::IncompatibleTimeInForce)
        );
    }

    #[rstest]
    #[case::ioc(TimeInForce::Ioc)]
    #[case::fok(TimeInForce::Fok)]
    fn accepts_immediate_time_in_force_on_market_orders(#[case] tif: TimeInForce) {
        let valid = validate(order(OrderType::Market, tif, Side::Buy, 1)).expect("valid");
        assert_eq!(valid.limit(), None, "a market order carries no price");
    }

    #[rstest]
    #[case::in_the_past(999)]
    #[case::now(1_000)]
    fn rejects_expiry_that_is_not_in_the_future(#[case] nanos: u64) {
        let tif = TimeInForce::Gtd { expires_at: LogicalTime::from_nanos(nanos) };
        assert_eq!(
            validate(order(limit(100), tif, Side::Buy, 1)),
            Err(RejectReason::ExpiryInThePast)
        );
    }

    #[test]
    fn accepts_expiry_in_the_future() {
        let tif = TimeInForce::Gtd { expires_at: LogicalTime::from_nanos(1_001) };
        assert!(
            validate(order(limit(100), tif, Side::Buy, 1)).is_ok(),
            "one nanosecond ahead is enough"
        );
    }

    #[test]
    fn rejects_an_order_whose_notional_overflows() {
        // `i64::MAX × u64::MAX` still fits `i128`; the instrument's scale is what tips it over.
        let mut wide = instrument();
        wide.price_bounds = PriceBounds::new(Price::from_ticks(1), Price::MAX).expect("min ≤ max");
        wide.scale = NotionalScale::new(NonZeroU32::new(10).expect("non-zero"), NonZeroU32::MIN);
        let o = order(limit(i64::MAX), TimeInForce::Gtc, Side::Buy, u64::MAX);
        assert_eq!(
            o.validate(&wide, SessionState::Open, NOW),
            Err(RejectReason::Arithmetic(ArithmeticError::Overflow))
        );
    }

    #[test]
    fn accepts_the_largest_notional_that_still_fits() {
        let mut wide = instrument();
        wide.price_bounds = PriceBounds::new(Price::from_ticks(1), Price::MAX).expect("min ≤ max");
        let o = order(limit(i64::MAX), TimeInForce::Gtc, Side::Buy, u64::MAX);
        assert!(o.validate(&wide, SessionState::Open, NOW).is_ok(), "scale 1/1 keeps it in range");
    }

    #[test]
    fn validation_is_deterministic() {
        let o = order(limit(100), TimeInForce::Gtc, Side::Buy, 5);
        assert_eq!(validate(o.clone()), validate(o), "same input, same verdict");
    }
}
