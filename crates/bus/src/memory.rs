//! In-memory bus built on `tokio::sync::broadcast`.

use std::collections::HashMap;

use bytes::Bytes;
use tokio::sync::{RwLock, broadcast};

use crate::port::{BusError, Publisher, Topic};
use crate::qos::Qos;

/// Broadcast-channel bus. Trading topics reject when lagging; market-data topics drop.
#[derive(Debug)]
pub struct MemoryBus {
    capacity: usize,
    topics: RwLock<HashMap<String, broadcast::Sender<Bytes>>>,
}

impl MemoryBus {
    /// Bus whose per-topic channels hold `capacity` messages.
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Self { capacity, topics: RwLock::new(HashMap::new()) }
    }

    /// Subscribes to `topic`, creating it if needed.
    pub async fn subscribe(&self, topic: &Topic) -> broadcast::Receiver<Bytes> {
        let mut topics = self.topics.write().await;
        topics
            .entry(topic.0.clone())
            .or_insert_with(|| broadcast::channel(self.capacity).0)
            .subscribe()
    }
}

#[async_trait::async_trait]
impl Publisher for MemoryBus {
    async fn publish(&self, topic: &Topic, _qos: Qos, payload: Bytes) -> Result<(), BusError> {
        let topics = self.topics.read().await;
        if let Some(tx) = topics.get(&topic.0) {
            // No receivers is not an error for a bus: the message is simply unobserved.
            let _receivers = tx.send(payload).unwrap_or(0);
        }
        Ok(())
    }
}
