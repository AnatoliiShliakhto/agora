//! Domain errors.

use crate::session::SessionState;

/// Arithmetic failure on a fixed-point value.
///
/// Never wraps silently: any overflow or division by zero surfaces as this error and the caller
/// decides whether to reject the command.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error, serde::Serialize, serde::Deserialize,
)]
#[non_exhaustive]
pub enum ArithmeticError {
    /// Result does not fit the target type.
    #[error("arithmetic overflow")]
    Overflow,
    /// Division by zero.
    #[error("division by zero")]
    DivisionByZero,
}

/// Reason an order (or another command) was rejected before touching the book.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error, serde::Serialize, serde::Deserialize,
)]
#[non_exhaustive]
pub enum RejectReason {
    /// Quantity is zero or not a whole number of lots.
    #[error("invalid quantity")]
    InvalidQty,
    /// Price is outside the instrument's allowed band.
    #[error("invalid price")]
    InvalidPrice,
    /// Stop-limit whose limit is on the near side of its trigger: it could never fill.
    #[error("stop-limit prices are inconsistent")]
    InconsistentStopLimit,
    /// Time-in-force cannot be used with this order type (a resting TIF on a market order).
    #[error("incompatible time-in-force")]
    IncompatibleTimeInForce,
    /// Good-till-date expiry is not in the future.
    #[error("expiry is in the past")]
    ExpiryInThePast,
    /// Instrument is unknown.
    #[error("unknown instrument")]
    UnknownInstrument,
    /// Session state does not accept this command (see [`SessionState::accepts`]).
    #[error("not trading: session is {state}")]
    NotTrading {
        /// Current session state.
        state: SessionState,
    },
    /// Account has insufficient available balance for the escrow reservation.
    #[error("insufficient funds")]
    InsufficientFunds,
    /// Order would trade against the same account (self-trade prevention).
    #[error("self-trade prevented")]
    SelfTrade,
    /// Fill-or-kill could not be filled in full.
    #[error("fill-or-kill not fillable")]
    FokUnfillable,
    /// Execution would exceed the instrument's price deviation limit.
    #[error("price deviation limit exceeded")]
    DeviationExceeded,
    /// Command carries an identifier that was already processed.
    #[error("duplicate command")]
    Duplicate,
    /// Arithmetic failed while evaluating the command.
    #[error(transparent)]
    Arithmetic(#[from] ArithmeticError),
}
