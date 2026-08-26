# 0003. Single-writer shards on actix arbiters

Date: 2026-08-26
Status: accepted (refined by ADR-0012: instruments share a fixed worker pool instead of one
thread each; the single-writer rule itself is unchanged)

## Context

The matching engine must be lock-free on the command path and deterministic. Mutexes serialize
anyway and add contention, priority inversion and the risk of holding a lock across an await.

## Decision

- One shard per instrument. The shard actor is the only writer of its engine, ledger slice and
  log. There is no interior mutability on the hot path.
- Shards are actix actors, each on its own `Arbiter` (a dedicated OS thread), optionally pinned
  to a core. I/O (axum, bus, stores) runs on a tokio multi-thread runtime.
- Hand-off into a shard is a bounded `rtrb` SPSC ring (sequencer → shard); replies are
  `tokio::sync::oneshot`. A full ring is back-pressure, not blocking.
- Read-mostly shared state uses `arc_swap`; counters use atomics.
- `std::sync::Mutex`/`RwLock` are disallowed types (clippy); `parking_lot` is allowed off the
  hot path with an `#[expect]` and a reason.

## Consequences

Parallelism scales with the number of instruments, not within one instrument (that is inherent
to price-time priority). Cross-instrument operations need explicit coordination (ADR-0004).
actix gives supervision and mailbox semantics for free; if arbiter/tokio interplay proves
awkward, the fallback is plain threads + the same rings, which the design already isolates in
`runtime`.

## Alternatives considered

- `ractor`/`kameo`: Erlang-style supervision trees, attractive; actix chosen for maturity and
  team familiarity, and because the actor surface we use is small.
- Shared book behind `parking_lot::RwLock`: simpler, but serializes readers and writers and
  makes determinism harder.
- Disruptor-style single global sequencer thread: considered for later; per-instrument shards
  are the first step towards it.
