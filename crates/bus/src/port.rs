//! Bus port.

use bytes::Bytes;

use crate::qos::Qos;

/// Topic name.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Topic(pub String);

/// Errors of a bus adapter.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum BusError {
    /// Trading queue is full; the caller must apply its degradation strategy.
    #[error("back-pressure on {0:?}")]
    BackPressure(Topic),
    /// Adapter is disconnected.
    #[error("disconnected")]
    Disconnected,
}

/// Publishing side of the bus.
#[async_trait::async_trait]
pub trait Publisher: Send + Sync {
    /// Publishes `payload` on `topic` with the given QoS.
    ///
    /// # Errors
    ///
    /// See [`BusError`].
    async fn publish(&self, topic: &Topic, qos: Qos, payload: Bytes) -> Result<(), BusError>;
}
