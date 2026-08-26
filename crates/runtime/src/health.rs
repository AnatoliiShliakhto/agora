//! Health state.

use agora_resilience::degrade::Level;

/// Aggregated health of the exchange as reported on `/readyz`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Health {
    /// Current degradation level.
    pub level: Level,
    /// Number of shards that are running.
    pub shards_up: u32,
    /// Total shards.
    pub shards_total: u32,
}

impl Health {
    /// Ready to serve trading traffic.
    #[must_use]
    pub const fn is_ready(&self) -> bool {
        matches!(self.level, Level::Normal | Level::Throttled)
            && self.shards_up == self.shards_total
    }
}
