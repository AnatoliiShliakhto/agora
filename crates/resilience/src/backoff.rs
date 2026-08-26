//! Backoff with jitter.

use core::time::Duration;

use rand::RngExt;

/// Exponential backoff policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Backoff {
    /// First delay.
    pub base: Duration,
    /// Upper bound of any delay.
    pub cap: Duration,
}

impl Backoff {
    /// Delay for `attempt` (0-based) with full jitter: `rand(0, min(cap, base × 2^attempt))`.
    pub fn full_jitter<R: RngExt>(&self, attempt: u32, rng: &mut R) -> Duration {
        let exp = self
            .base
            .saturating_mul(1_u32.checked_shl(attempt.min(31)).unwrap_or(u32::MAX))
            .min(self.cap);
        rng.random_range(Duration::ZERO..=exp)
    }

    /// Decorrelated jitter: `min(cap, rand(base, prev × 3))`.
    pub fn decorrelated<R: RngExt>(&self, prev: Duration, rng: &mut R) -> Duration {
        let upper = prev.saturating_mul(3).max(self.base);
        rng.random_range(self.base..=upper).min(self.cap)
    }
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    use super::*;

    #[test]
    fn full_jitter_never_exceeds_cap() {
        let policy = Backoff { base: Duration::from_millis(10), cap: Duration::from_millis(500) };
        let mut rng = StdRng::seed_from_u64(7);
        for attempt in 0..40 {
            assert!(
                policy.full_jitter(attempt, &mut rng) <= policy.cap,
                "attempt {attempt} exceeded cap"
            );
        }
    }
}
