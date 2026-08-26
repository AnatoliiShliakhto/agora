# 0011. Sandbox integration contract: opaque external identifiers, eventual settlement delivery

Date: 2026-08-26
Status: accepted

## Context

This engine is one module (CL-02) of a larger game-world sandbox. Two integration questions were
open in `docs/PLAN.md` and blocked P1.5:

1. What identifier format do the other modules use for accounts and assets? The engine does not
   create either: players, NPCs and companies are created by the world, currencies and goods by
   the macroeconomy module (MOD-08).
2. Does a trade need a synchronous confirmation from MOD-08, or is delivery over the bus enough?

Both were decided here rather than deferred, so Phase 1 can close.

## Decision

### External identifiers are opaque; internal identifiers are dense

- An identifier that comes from outside is an **`ExternalId`**: an opaque string of 1..=32
  characters from `[A-Za-z0-9._:-]`, compared byte for byte. The engine never parses it, never
  derives meaning from it and never generates one. ULIDs are recommended, but `player:4211` or a
  bare number is equally acceptable — that is the integrator's choice, not ours.
- Internally every entity is a **dense integer** newtype (`AccountId(u64)`, `AssetId(u32)`, …),
  because the hot path hashes, sorts and array-indexes them.
- The mapping `ExternalId → internal id` is assigned once, on first sight, by the owning
  aggregate, and is **recorded as an event** so it is deterministic across replay and restart.
- Identifiers the engine owns — `OrderId`, `TradeId`, `InstrumentId`, `ContractId`, `Seq` — are
  allocated by the shard sequencer and exposed outward as decimal numbers, not ULIDs.
- `CommandId` stays a client-supplied `u128` idempotency key; the API accepts a ULID in the
  `Idempotency-Key` header and stores its 128 bits verbatim.

### Settlement is delivered eventually, and the engine never blocks on an external module

- A trade is confirmed to the client as soon as its event batch is **durable**, not when any
  external module acknowledges it. Execution reports, ledger postings and market data go out
  over the bus afterwards.
- Every published message carries `(instrument, seq)`. A consumer that sees a gap resyncs from
  the log; delivery is therefore at-least-once with detectable loss, and every consumer-side
  effect must be idempotent by `(instrument, seq)`.
- When MOD-08 must *authorise* something rather than observe it — minting, burning, tax rates,
  interest — that is a **command into** the engine, and it travels the ordinary synchronous
  path (validate → reserve → apply). Authorisation before the fact, never confirmation after it.

## Consequences

The engine has no compile-time or runtime dependency on any other module's identifier scheme,
and integration cannot be blocked by a naming disagreement. The cost is one indirection
(`ExternalId → dense id`) at the boundary and one registry per entity kind, both cheap and both
event-sourced.

Refusing synchronous confirmation keeps a network round-trip out of the matching path and keeps
the exchange trading while MOD-08 is down or slow (ADR-0007) — at the price of a millisecond or
so before the rest of the world sees a trade, and the requirement that every consumer be
idempotent. For a game economy, whose aggregates are computed over windows rather than per
trade, that is the right trade.

## Alternatives considered

- **Mandating ULIDs everywhere**: simpler on paper, but forces every other module to change, and
  buys the engine nothing — it never interprets the identifier.
- **Using external identifiers directly as internal keys**: strings on the hot path, no dense
  arrays, worse cache behaviour, and hash-order non-determinism to guard against.
- **Synchronous confirmation to MOD-08**: puts an external RTT inside the matching path, makes
  liveness depend on another module, and contradicts ADR-0007.
