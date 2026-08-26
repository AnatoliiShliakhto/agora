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

pub mod policy;

/// Client errors.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ClientError {
    /// Transport failure.
    #[error("transport: {0}")]
    Transport(#[from] reqwest::Error),
    /// Server returned an API error.
    #[error("api {code}: {message}")]
    Api {
        /// Machine-readable code.
        code: String,
        /// Human-readable message.
        message: String,
    },
    /// Local policy refused the call (breaker open, limiter exhausted, degraded).
    #[error("refused locally: {0}")]
    Refused(&'static str),
}

/// HTTP client for the v1 API.
#[derive(Debug, Clone)]
pub struct Client {
    base_url: String,
    http: reqwest::Client,
}

impl Client {
    /// Client for the server at `base_url`.
    #[must_use]
    pub fn new(base_url: impl Into<String>) -> Self {
        Self { base_url: base_url.into(), http: reqwest::Client::new() }
    }

    /// Base URL.
    #[must_use]
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Liveness probe.
    ///
    /// # Errors
    ///
    /// See [`ClientError`].
    pub async fn healthz(&self) -> Result<(), ClientError> {
        self.http.get(format!("{}/healthz", self.base_url)).send().await?.error_for_status()?;
        Ok(())
    }
}
