//! Feeds arbitrary order streams to the engine; any panic is a bug (Phase 2 adds invariants).
#![no_main]

use libfuzzer_sys::fuzz_target;
use agora_domain::ids::{AccountId, CommandId, InstrumentId};
use agora_domain::instrument::{Instrument, NotionalScale, PriceBounds};
use agora_domain::money::{Price, Qty};
use agora_domain::order::{NewOrder, OrderType, SelfTradePrevention, Side, TimeInForce};
use agora_domain::session::SessionState;
use agora_domain::time::LogicalTime;
use agora_matching::engine::Engine;

#[derive(arbitrary::Arbitrary, Debug)]
struct Cmd {
    buy: bool,
    price: i32,
    qty: u16,
    account: u8,
}

fuzz_target!(|cmds: Vec<Cmd>| {
    let instrument = Instrument {
        id: InstrumentId::new(1),
        symbol: agora_domain::registry::Symbol::new("FUZZ").expect("valid symbol"),
        base: agora_domain::ids::AssetId::new(1),
        quote: agora_domain::ids::AssetId::new(2),
        scale: NotionalScale::new(core::num::NonZeroU32::MIN, core::num::NonZeroU32::MIN),
        price_bounds: PriceBounds::new(Price::from_ticks(i32::MIN.into()), Price::from_ticks(i32::MAX.into()))
            .expect("min ≤ max"),
        max_deviation: None,
    };
    let mut engine = Engine::new(instrument.clone());
    for (i, cmd) in cmds.into_iter().enumerate() {
        let order = NewOrder {
            command_id: CommandId::new(i as u128),
            account: AccountId::new(u64::from(cmd.account)),
            instrument: InstrumentId::new(1),
            side: if cmd.buy { Side::Buy } else { Side::Sell },
            kind: OrderType::Limit { limit: Price::from_ticks(i64::from(cmd.price)) },
            tif: TimeInForce::Gtc,
            stp: SelfTradePrevention::CancelNewest,
            qty: Qty::from_lots(u64::from(cmd.qty)),
        };
        // Invalid orders never reach the engine, exactly as in the shard pipeline.
        if let Ok(valid) = order.validate(&instrument, SessionState::Open, LogicalTime::ZERO) {
            let _ = engine.submit(&valid);
        }
    }
});
