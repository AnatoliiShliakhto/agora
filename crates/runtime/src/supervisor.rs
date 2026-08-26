//! Supervision and restart policy (Phase 5).

use core::time::Duration;

use agora_resilience::backoff::Backoff;

/// Restart policy applied by the supervisor to a failed shard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RestartPolicy {
    /// Backoff between restart attempts.
    pub backoff: Backoff,
    /// Give up (escalate to `Halted`) after this many consecutive failures.
    pub max_restarts: u32,
}

impl Default for RestartPolicy {
    fn default() -> Self {
        Self {
            backoff: Backoff { base: Duration::from_millis(50), cap: Duration::from_secs(5) },
            max_restarts: 10,
        }
    }
}
