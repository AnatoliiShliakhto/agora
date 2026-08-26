# agora-eventstore

Event store port and adapters (ADR-0004).

- `envelope` — versioned, sequenced, hash-chained record wrapping a domain event. It owns the
  position, the logical time and the schema identity; the payload owns none of them.
- `port` — `EventStore` trait: append a batch atomically, read from a sequence, snapshot.
- `memory` — in-memory adapter for tests and the deterministic simulator.
- `wal` *(planned, P4.2)* — segmented append-only log with CRC framing and an fsync policy.
- `snapshot` — snapshot format; the cadence and store land with P4.3.

An append is atomic per shard: either the whole batch produced by one command is durable or
none of it. That is the ACID boundary of the system.

## Schema evolution

Every event family (`SchemaId`) has its own `SchemaVersion`, declared by the `DomainEvent` impl
in `agora-domain`. `postcard` encodes an enum variant as its index, so:

| Change | Compatible? | What it needs |
|--------|-------------|---------------|
| append a variant at the end | yes | nothing; old readers refuse the new index loudly |
| append a field to a struct variant | no | new version + upcaster |
| insert, reorder or remove a variant | no | new version + upcaster |
| change a field's type or order | no | new version + upcaster |

`Envelope::is_from_the_future` guards the other direction: a payload written by a newer build is
never decoded — replay stops instead of guessing. The `schema_encoding_is_frozen` snapshots in
`agora-domain` fail the build when an incompatible change slips in unnoticed.
