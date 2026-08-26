//! Reference price port.

use agora_domain::ids::InstrumentId;
use agora_domain::money::Price;

/// Adapter errors.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum AdapterError {
    /// Provider did not answer in time.
    #[error("timeout")]
    Timeout,
    /// Provider answered with an error.
    #[error("upstream: {0}")]
    Upstream(String),
    /// Circuit breaker is open.
    #[error("circuit open")]
    CircuitOpen,
}

/// Source of reference prices.
#[async_trait::async_trait]
pub trait PriceOracle: Send + Sync {
    /// Latest reference price for `instrument`.
    ///
    /// # Errors
    ///
    /// See [`AdapterError`].
    async fn reference_price(&self, instrument: InstrumentId) -> Result<Price, AdapterError>;
}

/// `mockall` mocks of the ports in this module.
#[cfg(feature = "mocks")]
pub mod mocks {
    #![expect(clippy::disallowed_types, reason = "mockall generates `std::sync::Mutex` internally")]

    use agora_domain::ids::InstrumentId;
    use agora_domain::money::Price;

    use super::{AdapterError, PriceOracle};

    mockall::mock! {
        /// Mock [`PriceOracle`].
        pub PriceOracle {}

        #[async_trait::async_trait]
        impl PriceOracle for PriceOracle {
            async fn reference_price(&self, id: InstrumentId) -> Result<Price, AdapterError>;
        }
    }
}
