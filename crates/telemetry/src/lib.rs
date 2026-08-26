#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        reason = "tests may panic"
    )
)]

pub mod latency;
pub mod names;

use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt as _;
use tracing_subscriber::util::SubscriberInitExt as _;

/// Output format of the tracing subscriber.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// Human-readable, for local runs.
    Pretty,
    /// One JSON object per line, for production.
    Json,
}

/// Installs the global tracing subscriber.
///
/// Filter comes from `RUST_LOG`, defaulting to `info`.
///
/// # Errors
///
/// If a global subscriber is already installed.
pub fn init(format: Format) -> Result<(), tracing_subscriber::util::TryInitError> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let registry =
        tracing_subscriber::registry().with(filter).with(tracing_error::ErrorLayer::default());
    match format {
        Format::Pretty => registry.with(tracing_subscriber::fmt::layer().pretty()).try_init(),
        Format::Json => registry.with(tracing_subscriber::fmt::layer().json()).try_init(),
    }
}
