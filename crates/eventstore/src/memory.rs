//! In-memory event store for tests and simulation.

use agora_domain::ids::Seq;
use tokio::sync::RwLock;

use crate::envelope::Envelope;
use crate::port::{EventStore, StoreError};

/// Event store backed by a `Vec`.
#[derive(Debug, Default)]
pub struct MemoryStore {
    log: RwLock<Vec<Envelope>>,
}

impl MemoryStore {
    /// Empty store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait::async_trait]
impl EventStore for MemoryStore {
    async fn append(&self, batch: &[Envelope]) -> Result<(), StoreError> {
        let mut log = self.log.write().await;
        let expected = Seq::new(log.len() as u64);
        if let Some(first) = batch.first()
            && first.seq != expected
        {
            return Err(StoreError::SequenceGap { expected, got: first.seq });
        }
        log.extend_from_slice(batch);
        Ok(())
    }

    async fn read(&self, from: Seq, limit: usize) -> Result<Vec<Envelope>, StoreError> {
        let log = self.log.read().await;
        let start = usize::try_from(from.raw()).unwrap_or(usize::MAX);
        Ok(log.iter().skip(start).take(limit).cloned().collect())
    }

    async fn head(&self) -> Result<Option<Seq>, StoreError> {
        let log = self.log.read().await;
        Ok(log.last().map(|e| e.seq))
    }
}

#[cfg(test)]
mod tests {
    use agora_domain::time::LogicalTime;

    use super::*;
    use crate::envelope::{SchemaId, SchemaVersion};

    fn env(seq: u64) -> Envelope {
        Envelope {
            seq: Seq::new(seq),
            schema: SchemaId::Book,
            version: SchemaVersion::new(1),
            logical_time: LogicalTime::from_nanos(seq),
            hash: [0; 32],
            payload: vec![],
        }
    }

    #[tokio::test]
    async fn append_rejects_gaps() {
        let store = MemoryStore::new();
        store.append(&[env(0), env(1)]).await.expect("contiguous append");
        assert!(matches!(store.append(&[env(3)]).await, Err(StoreError::SequenceGap { .. })));
        assert_eq!(store.head().await.expect("head"), Some(Seq::new(1)));
        assert_eq!(store.read(Seq::new(1), 10).await.expect("read").len(), 1);
    }
}
