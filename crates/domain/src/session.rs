//! Trading session state of one instrument.
//!
//! The state is owned by the instrument's shard and gates which commands reach the engine.
//! See the command matrix in `docs/DOMAIN.md`.

use crate::error::RejectReason;

/// Session state.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, strum::Display,
)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum SessionState {
    /// Book accepts resting orders, no matching happens yet.
    PreOpen,
    /// Normal trading.
    Open,
    /// Trading suspended by a circuit breaker or an operator; cancels only.
    Halted,
    /// Scheduled close; cancels only, resting GTC orders survive to the next session.
    Closed,
}

/// Kind of command as far as session gating is concerned.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, strum::Display, strum::EnumIter)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum CommandKind {
    /// New limit order.
    SubmitLimit,
    /// New market order.
    SubmitMarket,
    /// New stop (market or limit) order.
    SubmitStop,
    /// Cancel of a resting or pending order.
    Cancel,
    /// Amend of a resting order.
    Amend,
}

/// Error of an invalid session transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[error("cannot go from {from} to {to}")]
pub struct TransitionError {
    /// Current state.
    pub from: SessionState,
    /// Requested state.
    pub to: SessionState,
}

impl SessionState {
    /// Checks whether `command` is accepted in this state.
    ///
    /// # Errors
    ///
    /// [`RejectReason::NotTrading`] carrying the current state.
    pub const fn accepts(self, command: CommandKind) -> Result<(), RejectReason> {
        let ok = match self {
            Self::Open => true,
            // No matching yet, so a market order could never execute.
            Self::PreOpen => !matches!(command, CommandKind::SubmitMarket),
            Self::Halted | Self::Closed => matches!(command, CommandKind::Cancel),
        };
        if ok { Ok(()) } else { Err(RejectReason::NotTrading { state: self }) }
    }

    /// Returns `true` if the engine matches orders in this state.
    #[must_use]
    pub const fn is_matching(self) -> bool {
        matches!(self, Self::Open)
    }

    /// Moves to `to` if the transition is allowed:
    /// `PreOpen → Open`, `Open ⇄ Halted`, `Open | Halted → Closed`, `Closed → PreOpen`.
    ///
    /// # Errors
    ///
    /// [`TransitionError`] for any other pair, including `self → self`.
    pub const fn transition(self, to: Self) -> Result<Self, TransitionError> {
        let allowed = matches!(
            (self, to),
            (Self::PreOpen, Self::Open)
                | (Self::Open, Self::Halted | Self::Closed)
                | (Self::Halted, Self::Open | Self::Closed)
                | (Self::Closed, Self::PreOpen)
        );
        if allowed { Ok(to) } else { Err(TransitionError { from: self, to }) }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use strum::IntoEnumIterator as _;

    use super::CommandKind::{Amend, Cancel, SubmitLimit, SubmitMarket, SubmitStop};
    use super::SessionState::{Closed, Halted, Open, PreOpen};
    use super::*;

    #[rstest]
    #[case(PreOpen, SubmitLimit, true)]
    #[case(PreOpen, SubmitMarket, false)]
    #[case(PreOpen, SubmitStop, true)]
    #[case(PreOpen, Cancel, true)]
    #[case(PreOpen, Amend, true)]
    #[case(Open, SubmitLimit, true)]
    #[case(Open, SubmitMarket, true)]
    #[case(Open, SubmitStop, true)]
    #[case(Open, Cancel, true)]
    #[case(Open, Amend, true)]
    #[case(Halted, SubmitLimit, false)]
    #[case(Halted, SubmitMarket, false)]
    #[case(Halted, SubmitStop, false)]
    #[case(Halted, Cancel, true)]
    #[case(Halted, Amend, false)]
    #[case(Closed, SubmitLimit, false)]
    #[case(Closed, SubmitMarket, false)]
    #[case(Closed, SubmitStop, false)]
    #[case(Closed, Cancel, true)]
    #[case(Closed, Amend, false)]
    fn command_matrix(
        #[case] state: SessionState,
        #[case] command: CommandKind,
        #[case] accepted: bool,
    ) {
        let expected = if accepted { Ok(()) } else { Err(RejectReason::NotTrading { state }) };
        assert_eq!(state.accepts(command), expected, "{state} × {command}");
    }

    #[test]
    fn cancel_is_always_accepted() {
        for state in [PreOpen, Open, Halted, Closed] {
            assert_eq!(state.accepts(Cancel), Ok(()), "{state} must accept cancels");
        }
    }

    #[test]
    fn only_open_matches() {
        assert!(Open.is_matching(), "open matches");
        for state in [PreOpen, Halted, Closed] {
            assert!(!state.is_matching(), "{state} must not match");
        }
    }

    #[rstest]
    #[case(PreOpen, Open, true)]
    #[case(PreOpen, Halted, false)]
    #[case(PreOpen, Closed, false)]
    #[case(Open, Halted, true)]
    #[case(Open, Closed, true)]
    #[case(Open, PreOpen, false)]
    #[case(Halted, Open, true)]
    #[case(Halted, Closed, true)]
    #[case(Halted, PreOpen, false)]
    #[case(Closed, PreOpen, true)]
    #[case(Closed, Open, false)]
    #[case(Closed, Halted, false)]
    fn transitions(#[case] from: SessionState, #[case] to: SessionState, #[case] allowed: bool) {
        let expected = if allowed { Ok(to) } else { Err(TransitionError { from, to }) };
        assert_eq!(from.transition(to), expected, "{from} → {to}");
    }

    #[test]
    fn self_transition_is_rejected() {
        for state in [PreOpen, Open, Halted, Closed] {
            assert!(state.transition(state).is_err(), "{state} → {state} must be rejected");
        }
    }

    #[test]
    fn every_command_kind_is_in_the_matrix() {
        assert_eq!(CommandKind::iter().count(), 5, "extend the rstest matrix when adding a kind");
    }
}
