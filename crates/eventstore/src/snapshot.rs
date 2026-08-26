//! Snapshots.

use agora_domain::ids::Seq;

/// Serialized aggregate state as of `seq`; replay resumes from `seq + 1`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Snapshot {
    /// Last event included.
    pub seq: Seq,
    /// Postcard-encoded aggregate state.
    pub state: Vec<u8>,
    /// `blake3(state)`.
    pub hash: [u8; 32],
}
