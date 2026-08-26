//! Order model.
//!
//! See `docs/DOMAIN.md` for the semantics of every variant; the matching engine
//! (`agora-matching`) is the single interpreter of these rules.

use crate::ids::{AccountId, CommandId, InstrumentId, OrderId};
use crate::money::{Price, Qty};
use crate::session::CommandKind;
use crate::time::LogicalTime;

/// Side of an order.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, strum::Display,
)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum Side {
    /// Buy the base asset.
    Buy,
    /// Sell the base asset.
    Sell,
}

impl Side {
    /// Opposite side.
    #[must_use]
    pub const fn opposite(self) -> Self {
        match self {
            Self::Buy => Self::Sell,
            Self::Sell => Self::Buy,
        }
    }
}

/// Order type.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, strum::Display,
)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum OrderType {
    /// Executes at `limit` or better; the unfilled remainder may rest on the book.
    Limit {
        /// Worst acceptable price.
        limit: Price,
    },
    /// Executes against available liquidity; never rests. Protected by the deviation guard.
    Market,
    /// Becomes a market order once the last trade price crosses `trigger`.
    StopMarket {
        /// Trigger price.
        trigger: Price,
    },
    /// Becomes a limit order once the last trade price crosses `trigger`.
    StopLimit {
        /// Trigger price.
        trigger: Price,
        /// Limit of the resulting order.
        limit: Price,
    },
}

impl OrderType {
    /// Kind of command this order type is, for session gating.
    #[must_use]
    pub const fn command_kind(self) -> CommandKind {
        match self {
            Self::Limit { .. } => CommandKind::SubmitLimit,
            Self::Market => CommandKind::SubmitMarket,
            Self::StopMarket { .. } | Self::StopLimit { .. } => CommandKind::SubmitStop,
        }
    }
}

/// Time-in-force.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, strum::Display,
)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum TimeInForce {
    /// Good-till-cancelled: rests until filled or cancelled.
    Gtc,
    /// Immediate-or-cancel: fills what it can immediately, cancels the rest.
    Ioc,
    /// Fill-or-kill: fills completely and immediately, or is rejected in full.
    Fok,
    /// Good-till-date: rests until the given logical time, then expires.
    Gtd {
        /// Expiry, in shard logical time.
        expires_at: LogicalTime,
    },
}

/// Self-trade prevention mode: what to do when an incoming order would match a resting order
/// from the same account.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, strum::Display,
)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum SelfTradePrevention {
    /// Reject the incoming order.
    CancelNewest,
    /// Cancel the resting order and continue matching.
    CancelOldest,
    /// Cancel both.
    CancelBoth,
    /// Decrement both by the overlapping quantity, cancel the smaller.
    DecrementAndCancel,
}

/// Order lifecycle status.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, strum::Display,
)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum OrderStatus {
    /// Accepted, not yet on the book (stop orders waiting for trigger).
    Pending,
    /// Resting on the book, no fills yet.
    Open,
    /// Resting on the book with some quantity filled.
    PartiallyFilled,
    /// Fully filled.
    Filled,
    /// Cancelled by the owner, STP, or IOC/FOK remainder handling.
    Cancelled,
    /// Expired (GTD).
    Expired,
    /// Rejected before reaching the book.
    Rejected,
}

/// New order command as validated by the domain (prices and quantities already in ticks/lots).
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct NewOrder {
    /// Idempotency key supplied by the client.
    pub command_id: CommandId,
    /// Owning account.
    pub account: AccountId,
    /// Target instrument.
    pub instrument: InstrumentId,
    /// Side.
    pub side: Side,
    /// Type and prices.
    pub kind: OrderType,
    /// Time-in-force.
    pub tif: TimeInForce,
    /// Self-trade prevention mode.
    pub stp: SelfTradePrevention,
    /// Total quantity.
    pub qty: Qty,
}

/// Order as tracked by the engine.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Order {
    /// Engine-assigned identifier.
    pub id: OrderId,
    /// Originating command.
    pub new: NewOrder,
    /// Quantity not yet filled.
    pub remaining: Qty,
    /// Current status.
    pub status: OrderStatus,
    /// Shard sequence number at acceptance; the time component of price-time priority.
    pub accepted_seq: u64,
}
