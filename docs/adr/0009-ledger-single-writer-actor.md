# 0009. Ledger is a separate single-writer actor

Date: 2026-08-26
Status: accepted

## Context

Matching is sharded by instrument (ADR-0003). An account holds many assets and trades many
instruments at once, so a balance cannot live inside an instrument shard without being split
across shards. SYS-102 requires escrow without race conditions; SYS-101 requires funds to stay
consistent under concurrency. `docs/PLAN.md` task P5.8 asked for this decision.

## Decision

- The ledger (balances, escrow reservations, postings) is its own single-writer actor with its
  own event log, keyed by `AccountId`. It starts as **one partition**; the key is chosen so that
  partitioning by account hash later is a change of partition count, not of design.
- Instrument shards never see balances. The only synchronous cross-actor step on the order
  path is `reserve` (escrow) before matching: `API → ledger.reserve → instrument shard → reply`.
- Everything after matching is applied by the ledger **asynchronously from the instrument
  event logs**, idempotently by `(instrument, seq)`: `commit` of the buyer's and seller's
  reservations, credits to the counterparties, fees to the exchange account, `release` of IOC/FOK/
  cancel remainders. Credits and releases cannot fail, so there is no compensation after
  matching; the saga of ADR-0004 degenerates to "reserve synchronously, settle from the log".
- One trade settles in **one ledger batch** (commit + credits + fees), so per-asset
  conservation holds at every point in time — no money "in flight".

## Consequences

One extra hop per order (microseconds) and a millisecond-scale delay before proceeds become
available; available balance for debits is always exact. Throughput ceiling is one thread of
hash-map updates and log appends (millions of postings per second), far above a game world's
needs; if it is ever reached, partition by account. Cross-shard settlement tests become
simple: replay instrument logs into the ledger and check conservation. Refines ADR-0004.

## Alternatives considered

- Ledger inside the instrument shard: balances split across shards, impossible to keep
  consistent for multi-instrument accounts.
- Ledger sharded by account from day one: settlement of one trade spans two partitions,
  introducing in-flight money and a real saga; deferred until measured need.
- Ledger sharded by asset: a trade touches two assets, same problem, worse locality.
