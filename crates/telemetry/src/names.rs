//! Metric names.

/// Commands accepted by the sequencer, by instrument.
pub const COMMANDS_TOTAL: &str = "agora_commands_total";
/// Trades executed, by instrument.
pub const TRADES_TOTAL: &str = "agora_trades_total";
/// Shard pipeline latency, seconds.
pub const SHARD_LATENCY: &str = "agora_shard_latency_seconds";
/// Current degradation level (0 = normal).
pub const DEGRADATION_LEVEL: &str = "agora_degradation_level";
