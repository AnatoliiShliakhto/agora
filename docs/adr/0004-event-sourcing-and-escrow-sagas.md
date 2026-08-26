# 0004. Event sourcing per shard; escrow sagas across shards

Date: 2026-08-26
Status: accepted (refined by ADR-0009: the ledger is a separate actor, so the saga has no
compensation path after matching)

## Context

SYS-101 requires ACID-level consistency of funds in an asynchronous, lock-free system. A
database transaction per order is exactly the bottleneck we are removing.

## Decision

- Aggregates (engine, ledger, contracts) are pure: `decide(state, command) -> Vec<Event>` and
  `apply(state, event) -> state`. Nothing mutates until the event batch of one command is
  appended atomically to the shard log. That append is the transaction.
- Events are the only persisted data. State is snapshot + replay. Envelopes are versioned,
  sequenced and hash-chained; upcasters migrate old versions on read.
- Time is an input event (`Tick`); identifiers come from the shard sequence. Replay is
  therefore byte-identical.
- Across shards, money moves through the escrow protocol: `reserve` → `commit | release`,
  idempotent by `CommandId`, with compensation on failure. This is a saga, not 2PC.
- A bounded command journal persisted with the log makes retries idempotent across restarts.

## Consequences

No database on the hot path; durability cost is one append (group-committed). Debugging is
replay. Schema discipline is mandatory from day one (snapshot tests, upcasters). Cross-shard
atomicity is eventual; the escrow protocol makes the intermediate states safe (funds are locked,
never double-spent).

## Alternatives considered

- Postgres row locks / serializable transactions: correct, but tens of microseconds per order
  and a single point of contention.
- CRDT ledger: unnecessary, we have a single writer per shard.
