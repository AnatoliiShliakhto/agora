//! Circuit breaker.

/// Breaker state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    /// Requests flow.
    Closed,
    /// Requests are rejected until the cool-down elapses.
    Open,
    /// A limited number of probe requests are allowed through.
    HalfOpen,
}
