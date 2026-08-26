//! Degradation ladder.

/// Operating level of the exchange or of one shard. Higher levels shed more work.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Level {
    /// Everything accepted.
    Normal,
    /// Per-account limits tightened; low-priority traffic shed.
    Throttled,
    /// New orders rejected; cancels and market data continue.
    CancelOnly,
    /// Only market data is served.
    MarketDataOnly,
    /// Nothing served; waiting for recovery.
    Halted,
}
