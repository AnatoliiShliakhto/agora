//! Event store port.

use agora_domain::ids::Seq;

use crate::envelope::Envelope;

/// Errors of an event store adapter.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum StoreError {
    /// Batch's first sequence does not follow the last stored one.
    #[error("sequence gap: expected {expected}, got {got}")]
    SequenceGap {
        /// Expected next sequence.
        expected: Seq,
        /// Sequence the batch started with.
        got: Seq,
    },
    /// Hash chain or CRC mismatch on read.
    #[error("integrity failure at {0}")]
    Integrity(Seq),
    /// Underlying storage failed.
    #[error("storage: {0}")]
    Storage(#[source] Box<dyn std::error::Error + Send + Sync>),
}

/// Append-only event log of one shard.
#[async_trait::async_trait]
pub trait EventStore: Send + Sync {
    /// Appends `batch` atomically; all or nothing.
    ///
    /// # Errors
    ///
    /// See [`StoreError`].
    async fn append(&self, batch: &[Envelope]) -> Result<(), StoreError>;

    /// Reads envelopes from `from` (inclusive), at most `limit`.
    ///
    /// # Errors
    ///
    /// See [`StoreError`].
    async fn read(&self, from: Seq, limit: usize) -> Result<Vec<Envelope>, StoreError>;

    /// Last stored sequence, if any.
    ///
    /// # Errors
    ///
    /// See [`StoreError`].
    async fn head(&self) -> Result<Option<Seq>, StoreError>;
}
