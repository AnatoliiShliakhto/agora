//! Per-call resilience policies (Phase 8).

use core::time::Duration;

/// How the client behaves once the server is unreachable or degraded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DegradationStrategy {
    /// Fail every call immediately.
    FailFast,
    /// Keep cancelling and reading, stop placing.
    CancelOnly,
    /// Serve the last known market data; refuse trading.
    CachedMarketData,
}

/// Policy applied to one API method.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CallPolicy {
    /// Overall deadline.
    pub deadline: Duration,
    /// Retries, only honoured for idempotent calls.
    pub max_retries: u32,
    /// Strategy once the breaker opens.
    pub degradation: DegradationStrategy,
}
