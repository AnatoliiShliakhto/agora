//! Instrument shard actor (Phase 5).

use actix::prelude::*;
use agora_domain::instrument::Instrument;
use agora_matching::engine::Engine;

/// Actor owning one instrument's engine and ledger slice.
#[derive(Debug)]
pub struct Shard {
    engine: Engine,
}

impl Shard {
    /// Shard for `instrument` with an empty book.
    #[must_use]
    pub const fn new(instrument: Instrument) -> Self {
        Self { engine: Engine::new(instrument) }
    }

    /// Engine owned by this shard.
    #[must_use]
    pub const fn engine(&self) -> &Engine {
        &self.engine
    }
}

impl Actor for Shard {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        tracing::info!(instrument = %self.engine.instrument().symbol, "shard started");
    }
}
