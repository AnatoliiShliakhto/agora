//! Matching engine: interprets [`ValidOrder`] commands against an [`OrderBook`].
//!
//! Pipeline for one command (all steps pure, all outputs are [`BookEvent`]s):
//!
//! 1. validate (tick/lot alignment, price band) → `OrderRejected`
//! 2. stop orders → park in the trigger set → `OrderAccepted`
//! 3. FOK pre-check: total fillable quantity at acceptable prices ≥ order qty, else reject
//! 4. walk opposite levels best-first; per resting order apply STP, then fill → `Trade`
//! 5. remainder: GTC/GTD rests → `OrderAccepted`; IOC → `OrderDone(Cancelled)`
//! 6. after fills: re-evaluate stop triggers against the last trade price
//!
//! Implemented incrementally in Phase 2 of `docs/PLAN.md`.

use agora_domain::error::RejectReason;
use agora_domain::event::BookEvent;
use agora_domain::ids::OrderId;
use agora_domain::instrument::Instrument;
use agora_domain::validation::ValidOrder;

use crate::book::OrderBook;

/// Result of interpreting one command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    /// Events to persist and publish, in order.
    pub events: Vec<BookEvent>,
}

/// Matching engine for one instrument.
///
/// Single-writer: exactly one shard actor owns an `Engine`; there is no interior mutability.
#[derive(Debug)]
pub struct Engine {
    instrument: Instrument,
    book: OrderBook,
    next_order_id: u64,
    next_trade_id: u64,
}

impl Engine {
    /// Empty engine for `instrument`.
    #[must_use]
    pub const fn new(instrument: Instrument) -> Self {
        Self { instrument, book: OrderBook::new(), next_order_id: 1, next_trade_id: 1 }
    }

    /// Instrument this engine trades.
    #[must_use]
    pub const fn instrument(&self) -> &Instrument {
        &self.instrument
    }

    /// Read-only view of the book.
    #[must_use]
    pub const fn book(&self) -> &OrderBook {
        &self.book
    }

    /// Interprets a new-order command.
    ///
    /// The order must already have passed validation (see [`ValidOrder`]), so stateless
    /// rejections cannot happen here; what remains are the rejections that need the book's
    /// state (fillability, self-trade, deviation).
    ///
    /// # Errors
    ///
    /// See [`RejectReason`]; the caller turns it into an `OrderRejected` event.
    pub fn submit(&mut self, order: &ValidOrder) -> Result<Outcome, RejectReason> {
        let new = order.as_new();
        debug_assert_eq!(new.instrument, self.instrument.id, "order routed to the wrong shard");
        // Phase 2 replaces this with the full pipeline documented at the module level.
        let id = OrderId::new(self.next_order_id);
        self.next_order_id = self.next_order_id.checked_add(1).ok_or(RejectReason::Duplicate)?;
        Ok(Outcome {
            events: vec![BookEvent::OrderAccepted {
                order: id,
                command: new.command_id,
                account: new.account,
            }],
        })
    }

    /// Allocates the next trade identifier.
    #[expect(dead_code, reason = "used by the fill loop in Phase 2")]
    const fn next_trade_id(&mut self) -> Option<u64> {
        let id = self.next_trade_id;
        self.next_trade_id = match self.next_trade_id.checked_add(1) {
            Some(n) => n,
            None => return None,
        };
        Some(id)
    }
}
