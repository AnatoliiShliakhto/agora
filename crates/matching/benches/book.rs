//! Order book micro-benchmarks. Run with `just bench matching`.

use agora_domain::ids::OrderId;
use agora_domain::money::{Price, Qty};
use agora_domain::order::Side;
use agora_matching::book::{BookSide, Resting};
use criterion::{BatchSize, Criterion, criterion_group, criterion_main};

fn push_then_remove(c: &mut Criterion) {
    c.bench_function("book_side/push_remove_1k", |b| {
        b.iter_batched(
            || BookSide::new(Side::Sell),
            |mut side| {
                for i in 0..1_000_u64 {
                    let price = Price::from_ticks(i64::try_from(i % 50).unwrap_or(0));
                    let _ =
                        side.push(price, Resting { id: OrderId::new(i), qty: Qty::from_lots(1) });
                }
                for i in 0..1_000_u64 {
                    let price = Price::from_ticks(i64::try_from(i % 50).unwrap_or(0));
                    side.remove(price, OrderId::new(i));
                }
                side
            },
            BatchSize::SmallInput,
        );
    });
}

criterion_group!(benches, push_then_remove);
criterion_main!(benches);
