//! Event envelope.
//!
//! The envelope owns everything the payload does not: position, time, schema identity and the
//! integrity chain. Domain events carry none of that (ADR-0004).

pub use agora_domain::event::{DomainEvent, SchemaId, SchemaVersion};
use agora_domain::ids::Seq;
use agora_domain::time::LogicalTime;

/// Persisted record: one domain event plus the metadata needed for replay and integrity.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Envelope {
    /// Position in the shard log; strictly increasing by one.
    pub seq: Seq,
    /// Family the payload belongs to; decides which type it decodes into.
    pub schema: SchemaId,
    /// Payload schema version, for upcasting.
    pub version: SchemaVersion,
    /// Logical time assigned by the shard when it produced the event.
    pub logical_time: LogicalTime,
    /// `blake3(prev_hash || payload)`; detects truncation and tampering on replay.
    pub hash: [u8; 32],
    /// Postcard-encoded domain event.
    pub payload: Vec<u8>,
}

impl Envelope {
    /// Returns `true` if the payload was written by a newer build than this one understands.
    ///
    /// Such a payload must not be decoded: replay stops instead of guessing.
    #[must_use]
    pub const fn is_from_the_future<E: DomainEvent>(&self) -> bool {
        self.version.get() > E::VERSION.get()
    }
}
