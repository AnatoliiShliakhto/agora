//! Latency histograms for the hot path.

use hdrhistogram::Histogram;

/// Fixed-range nanosecond histogram (1 ns … 60 s, 3 significant digits).
#[derive(Debug, Clone)]
pub struct Latency {
    hist: Histogram<u64>,
}

impl Default for Latency {
    fn default() -> Self {
        Self::new()
    }
}

impl Latency {
    /// Empty histogram.
    #[must_use]
    #[expect(
        clippy::missing_panics_doc,
        reason = "bounds are constants accepted by hdrhistogram; never panics"
    )]
    pub fn new() -> Self {
        #[expect(clippy::expect_used, reason = "constant arguments cannot fail")]
        let hist =
            Histogram::new_with_bounds(1, 60_000_000_000, 3).expect("valid histogram bounds");
        Self { hist }
    }

    /// Records one sample in nanoseconds; out-of-range samples are clamped.
    pub fn record(&mut self, nanos: u64) {
        self.hist.saturating_record(nanos);
    }

    /// Value at `quantile` (0.0..=1.0) in nanoseconds.
    #[must_use]
    pub fn quantile(&self, quantile: f64) -> u64 {
        self.hist.value_at_quantile(quantile)
    }

    /// Number of samples.
    #[must_use]
    pub fn len(&self) -> u64 {
        self.hist.len()
    }

    /// Returns `true` if no samples were recorded.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.hist.is_empty()
    }
}
