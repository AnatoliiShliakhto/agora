//! Deterministic clock.

use core::cell::Cell;

/// Logical clock advanced explicitly by the test.
#[derive(Debug, Default)]
pub struct TestClock {
    now: Cell<u64>,
}

impl TestClock {
    /// Clock at time zero.
    #[must_use]
    pub const fn new() -> Self {
        Self { now: Cell::new(0) }
    }

    /// Current logical time, nanoseconds.
    #[must_use]
    pub const fn now(&self) -> u64 {
        self.now.get()
    }

    /// Advances by `nanos`, saturating.
    pub fn advance(&self, nanos: u64) {
        self.now.set(self.now.get().saturating_add(nanos));
    }
}
