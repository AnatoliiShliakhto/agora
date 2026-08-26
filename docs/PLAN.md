# Implementation plan

Working plan for building the Economy & Market Engine cluster (CL-02): an event-sourced,
actor-based, lock-free matching engine with escrow and a smart-contract rules engine. This file
is the backlog. It is written for an engineer or an agent that has to pick up the next piece of
work without a conversation history.

- [How to use this plan](#how-to-use-this-plan)
- [Non-negotiables](#non-negotiables)
- [Workspace map](#workspace-map)
- [Milestones](#milestones)
- [Phases and tasks](#phases-and-tasks)
- [Testing strategy](#testing-strategy)
- [Benchmark policy](#benchmark-policy)
- [Risk register](#risk-register)
- [Open questions](#open-questions)

## How to use this plan

1. Pick the first unchecked task whose `depends on` list is fully checked. Prefer finishing a
   phase over starting the next one.
2. Read the linked docs first: `docs/DOMAIN.md` for semantics, `docs/ARCHITECTURE.md` for
   structure, the ADRs for decisions already made. Do not re-litigate an ADR inside a task; write
   a new ADR that supersedes it.
3. Implement the task in one PR. The PR ticks the checkbox here, adds tests and (for hot-path
   tasks) benches, and passes `just verify`.
4. Anything discovered but out of scope becomes a new task in the right phase, not a detour.

Task IDs are `P<phase>.<n>`. Every task lists *deliverables* (what exists afterwards) and
*acceptance* (how we know it is done). "Bench" means a criterion benchmark with a committed
baseline (see [Benchmark policy](#benchmark-policy)).

**Definition of done** for every task:

- `just verify` is green (fmt, clippy `-D warnings` with the workspace lint set, tests, doctests,
  rustdoc `-D warnings`, cargo-deny, typos, benches compile);
- new public items are documented per `CODESTYLE.md`;
- new behaviour has a test that fails without the change;
- hot-path changes have a bench and the numbers are recorded in `docs/PERFORMANCE.md`;
- an architectural choice made along the way has an ADR.

## Non-negotiables

These are the engineering constraints of the whole project. They are enforced by lints where
possible and by review otherwise.

1. **No floating point** in any crate except `telemetry` and `loadgen` (`clippy::float_arithmetic = deny`).
   Prices are integer ticks, quantities are integer lots, money is `i128` minor units
   (ADR-0002).
2. **Checked arithmetic only** in `domain`, `matching`, `ledger`, `contracts`
   (`clippy::arithmetic_side_effects = deny` at crate level). Overflow is an error, never a wrap.
3. **Determinism**: an aggregate is a pure function of its command sequence. No wall clock, no RNG,
   no `HashMap` iteration order inside `domain`, `matching`, `ledger`, `contracts`. Time is an
   input (`Tick`), identifiers come from the shard sequencer.
4. **Single writer per shard** (ADR-0003). No `Mutex`/`RwLock` on the command path; cross-thread
   hand-off is lock-free queues only. Zero allocations per command after warm-up is the target,
   measured, not assumed.
5. **Event sourcing** (ADR-0004): state is derived from the log; the atomic append of one
   command's event batch is the ACID boundary. Order ↔ ledger work crosses actors through
   escrow (ADR-0009): `reserve` is synchronous and is the only step that can fail; commit,
   credit, fee and release are applied by the ledger actor from the instrument log, one batch
   per trade — nothing after matching needs compensation.
6. **Every port has a mock and a fault-injecting fake**; every outbound call is guarded by
   timeout + circuit breaker + bounded retry (ADR-0007).
7. **QoS split** (ADR-0005): trading traffic is lossless with back-pressure; market data is
   conflated and never blocks trading.
8. **Versioned API**: DTOs live in `protocol::v1`, the domain never leaks to the wire.
9. **Self-healing over perfection**: panics unwind into the supervisor, shards restart from
   snapshot, the degradation ladder sheds load before the process dies.

## Workspace map

| Crate        | Layer           | Responsibility                                                      |
|--------------|-----------------|---------------------------------------------------------------------|
| `domain`     | domain          | value objects, ids, instrument, order model, domain events, errors  |
| `matching`   | domain          | order book, matching algorithm, TIF/STP/stop/deviation, fees        |
| `ledger`     | domain          | balances, escrow (reserve/commit/release), double-entry postings    |
| `contracts`  | domain          | rules engine, scheduler, `LoanAgreement`, dividends                 |
| `eventstore` | port + adapters | envelope, hash chain, `EventStore`, memory/WAL adapters, snapshots  |
| `bus`        | port + adapters | `Publisher`/`Subscriber`, QoS, memory/NATS adapters, conflation     |
| `resilience` | infrastructure  | limiter, breaker, backoff, bulkhead, shedding, degradation ladder   |
| `runtime`    | application     | shard actors, ledger actor, sequencer, supervisor, health, pipeline |
| `adapters`   | port + adapters | liquidity provider, price oracle, macroeconomy ports; fakes; mocks  |
| `protocol`   | interface       | versioned DTOs, decimal ⇄ ticks conversion, OpenAPI schemas         |
| `api`        | interface       | axum REST/WS, middleware, health, metrics                           |
| `client`     | interface       | SDK with policies and degradation strategies                        |
| `telemetry`  | infrastructure  | tracing init, metric names, HDR latency                             |
| `testkit`    | test            | clock, fixtures, proptest strategies, reference matcher             |
| `server`     | binary          | composition root                                                    |
| `loadgen`    | binary          | open-loop load generator                                            |
| `fuzz/`      | test            | cargo-fuzz targets (separate nightly workspace)                     |

Dependency direction is strictly downward: `domain` ← `matching`/`ledger`/`contracts` ←
`runtime` ← `api`/`server`. `testkit` is a dev-dependency only.

## Milestones

The capstone frame is three working days for a small team plus agents. The plan is larger than
that on purpose: it defines the target system, and the cut line below defines the demo.

| Milestone | Phases                                                       | Demo-able outcome                                                  |
|-----------|--------------------------------------------------------------|--------------------------------------------------------------------|
| M0 (done) | 0                                                            | workspace, lints, CI, docs, skills                                 |
| M1        | ~~1~~, 2 (P2.1–P2.5, P2.12), 3 (P3.1–P3.4)                   | limit/market/IOC/FOK matching with escrow; property tests; benches |
| M2        | 4 (memory + WAL), 5 (P5.1–P5.5, P5.9, P5.11), 6 (memory bus) | shards restart from snapshot under fault injection, no lost events |
| M3        | 7 (REST subset), 8, 9 (fakes), 11 (metrics)                  | end-to-end via HTTP with rate limiting, degradation and dashboards |
| M4        | 10, 12 (loadgen, DST subset), 13                             | loan + dividend contracts; load report; presentation               |
| Stretch   | 4.7, 6.4, 7.9, 12.4, 13.4                                    | NATS JetStream, Postgres, gRPC, bench regression CI, multi-node    |

**MVP cut line (must exist for the capstone review):** M1 + M2 + REST `POST/DELETE /v1/orders`,
`GET /v1/books/{symbol}`, metrics endpoint, loadgen report, one contract (`LoanAgreement`).

## Phases and tasks

### Phase 0 — Foundation (done)

- [x] P0.1 Workspace layout, 16 crates, strict lint set, profiles (`Cargo.toml`).
- [x] P0.2 Tooling: `rust-toolchain.toml`, `rustfmt.toml` (nightly), `clippy.toml`, `deny.toml`,
      `typos.toml`, `taplo.toml`, `.cargo/config.toml`, `justfile` with `verify` gate.
- [x] P0.3 CI workflow mirroring `just verify`; dependabot; PR/issue templates.
- [x] P0.4 Governance: `README.md`, `LICENSE` (MIT), `CONTRIBUTING.md`, `SECURITY.md`,
      `CHANGELOG.md`.
- [x] P0.5 Engineering docs: `CODESTYLE.md`, `CLAUDE.md`, `AGENTS.md`, `docs/ARCHITECTURE.md`,
      `docs/DOMAIN.md`, ADR-0001…0008, this plan.
- [x] P0.6 Agent skills in `.claude/skills/`.
- [x] P0.7 Skeleton code compiles under the full lint set with first tests and a bench.

### Phase 1 — Domain core (done)

Goal: the ubiquitous language exists as types, with invariants that cannot be violated by
construction. Closed 2026-08-26: 117 tests, `just verify` green, decisions in ADR-0009…0011.

- [x] **P1.1 Rounding policy and fee math.** `RoundingMode { Floor, Ceil, HalfEven }` and
      `Amount::checked_div(divisor, mode)`; `NotionalScale::new(NonZeroU32, NonZeroU32)` (valid
      by construction); `FeeBps` capped at 100 % with `FeeBps::of(amount, mode)` and
      `FeeSchedule { maker, taker }` rounding up. Delivered in `domain::money`, `domain::fees`,
      `domain::instrument`; tests: proptest `floor ≤ half_even ≤ ceil` and exactness of the
      floor remainder, fee never exceeds the amount, charged ≥ paid out, worked examples from
      `docs/DOMAIN.md`. Depends on: —.
- [x] **P1.2 Instrument registry and session state.** `registry::Symbol` (validated,
      `[A-Z0-9_]{1,16}`), `InstrumentRegistry` (id ↔ symbol, id-ordered iteration, no partial
      inserts), `session::SessionState` with `accepts(CommandKind)`, `is_matching`, `transition`;
      `RejectReason::NotTrading { state }`. Tests: `rstest` matrices for commands × states and
      transitions, symbol validation, registry duplicates. Matrix documented in
      `docs/DOMAIN.md` § Session states. Depends on: —.
- [x] **P1.3 Order validation.** `NewOrder::validate(&Instrument, SessionState, LogicalTime)
      -> Result<ValidOrder, RejectReason>` with a documented, fixed check order; `PriceBounds`
      on the instrument; new rejections `InconsistentStopLimit`, `IncompatibleTimeInForce`,
      `ExpiryInThePast`; `LogicalTime` moved into `domain::time` (contracts re-export it);
      `Engine::submit` takes `&ValidOrder`, so an unvalidated order cannot reach the book.
      Tests: one case per rejection, both stop-limit directions, TIF compatibility, the expiry
      boundary, notional overflow and the largest notional that still fits. Documented in
      `docs/DOMAIN.md` § Order validation. Depends on: P1.2.
- [x] **P1.4 Event schema freeze.** `SchemaId` + `SchemaVersion` + the `DomainEvent` trait, one
      version line per event family; `ContractEvent`/`ContractOutcome` added; `LedgerEvent` now
      names the `asset` of every leg; `Envelope` carries `schema`, `LogicalTime` and
      `is_from_the_future`. Events deliberately carry **no** timestamp or sequence — the
      envelope is the single source of truth for both (ADR-0004), so `logical_time` was not
      duplicated into `Trade`; fees landed in P1.1 and spread capture is gone with ADR-0010.
      Tests: postcard round-trip proptests per family, a truncation proptest, and
      `insta` snapshots of every variant's byte encoding with a `strum` discriminant check that
      fails when a variant has no sample. Rules documented in `crates/eventstore/README.md`.
      Depends on: P1.1.
- [x] **P1.5 Glossary sign-off.** `docs/DOMAIN.md` § Sandbox identifiers now maps every brief
      identifier (`CL-02`, `COMP-107/108`, `SYS-101/102/103`, `MOD-08/16/25`, `COMP-201`) to the
      code that implements it. The two integration questions were **decided in ADR-0011** rather
      than deferred: external identifiers are opaque `ExternalId` strings mapped once to dense
      internal ids, and settlement reaches other modules eventually over the bus — the engine
      never blocks on MOD-08. `domain::ids::ExternalId` implements the first half.
      Depends on: —.

### Phase 2 — Matching engine

Goal: a complete, deterministic, allocation-free matcher with proofs by property and model
tests. This is the core of the capstone.

- [ ] **P2.1 Order arena and O(1) cancel.** Replace `VecDeque` levels with an intrusive doubly
      linked FIFO over a `slotmap` arena; `OrderId → slot` index. Bench: insert/cancel constant
      in book depth. Depends on: P1.3.
- [ ] **P2.2 Price-time limit matching (GTC).** Fill loop over opposite levels best-first, FIFO
      within level; partial fills; `Trade`, `OrderDone`. Model-based test against
      `testkit::reference` (naive O(n²) matcher) over random command streams. Invariants as
      proptests: quantity conservation, book never crossed after a command, priority (a resting
      order fills only after every older order at a better-or-equal price). Depends on: P2.1.
- [ ] **P2.3 Market orders and deviation guard.** Reference price = last trade, else mid, else
      oracle; walk stops at `max_deviation`; remainder handling policy `RejectRemainder |
      CancelRemainder`. Tests: thin-book market order does not blow through the band. Depends on: P2.2.
- [ ] **P2.4 IOC and FOK.** FOK is a read-only pre-walk (fillable qty at acceptable prices ≥ qty)
      then the normal fill loop; IOC cancels the remainder. Test: the spec example (buy 10M @
      1.1200 vs 500k@1.1200, 2M@1.1198, 100k@1.1197 → 2.6M filled, three makers fully filled).
      Depends on: P2.2.
- [ ] **P2.5 Self-trade prevention.** All four modes; STP evaluated per resting order before
      the fill; events for the cancelled side. Tests per mode, proptest "no trade has equal
      accounts on both sides". Depends on: P2.2.
- [ ] **P2.6 Stop orders.** Trigger sets keyed by trigger price per side; activation after every
      trade; deterministic activation order (price, then accepted seq); activated stop-limit
      goes through the normal pipeline. Tests: cascade of stops triggers in one command,
      replay-equal. Depends on: P2.3.
- [ ] **P2.7 GTD expiry.** Expiry index; `Tick(logical_time)` command expires due orders with
      `OrderDone(Expired)`; escrow release event follows. Depends on: P2.2.
- [ ] **P2.8 Fees and the money sink.** Apply the instrument's current `FeeSchedule` per trade
      (the one in force at matching time); `Trade` carries `maker_fee` and `taker_fee`. Execution
      is always at the maker's price, one price per trade (ADR-0010). Add `FeeDestination
      { burn_bps, treasury }` and the `max_fee_bps` ceiling to the instrument, and a
      `SetFeeSchedule` command that only the macroeconomy port may issue, applied as an event so
      replay reproduces historical rates (ADR-0013). Tests: fee sums match the `docs/DOMAIN.md`
      worked example, rounding never creates money, a rate change never affects a resting order,
      a schedule above the ceiling is rejected. Depends on: P1.1, P2.2.
- [ ] **P2.9 Market-data views.** L2 snapshot (top-N), L3 snapshot, incremental `BookDelta`
      events suitable for conflation. Tests: applying deltas to a snapshot equals the next
      snapshot. Depends on: P2.1.
- [ ] **P2.10 Cancel and amend.** `Cancel`, `Amend { qty, price }`: qty decrease keeps priority,
      anything else re-queues. Tests: priority rules. Depends on: P2.2.
- [ ] **P2.11 Fuzzing with invariants.** Extend `fuzz/matching_stream` to assert conservation
      and no-crossed-book after each command; 10-minute nightly CI job. Depends on: P2.5.
- [ ] **P2.12 Benchmarks.** `orders/s` on a warm book, p50/p99 per command class (rest, cross
      one level, cross N levels, cancel), depth scaling 10/100/1000 levels; baseline committed.
      Depends on: P2.4.
- [ ] **P2.13 Hot-path audit.** Allocation counter test (zero allocs per command after warm-up),
      `#[repr(align(64))]` on per-shard hot structs, `perf stat` for branch misses and cache
      misses on the bench; results and decisions recorded in `docs/PERFORMANCE.md`.
      Depends on: P2.12.

### Phase 3 — Ledger and escrow

Goal: money cannot be created, lost or double-spent, and every trade settles atomically with
its fills.

- [ ] **P3.1 Ledger aggregate.** Accounts, per-asset `Balance`; commands `Deposit`, `Withdraw`,
      `Mint`, `Burn` (the last two only from the macroeconomy port); `LedgerEvent` apply
      functions including `Burned`, so the money supply is derivable from the log alone
      (ADR-0013). Depends on: P1.4.
- [ ] **P3.2 Escrow implementation.** Reservation table keyed by `CommandId`; reserve / commit
      / release with idempotent retries. Proptests: `available + reserved == total`, no
      negative balances, double commit is a no-op. Depends on: P3.1.
- [ ] **P3.3 Trade settlement.** `Trade → Posting`: quote from buyer to seller, base from seller
      to buyer, maker and taker fees to the exchange account; posting must balance per asset.
      Built as a pure function so the ledger actor can apply it from the instrument log as one
      batch per trade (ADR-0009). Tests: worked example; proptest over random trades that
      per-asset totals are conserved. Depends on: P2.8, P3.2.
- [ ] **P3.4 Reservation sizing.** Limit buy reserves `notional(limit) + fee at max_fee_bps`;
      sell reserves base qty; market buy reserves `notional(ref × (1 + deviation)) + ceiling
      fee`; over-reserve then release the difference. Sizing uses the **ceiling**, never the
      current rate, so a fee change cannot invalidate a resting order (ADR-0013). Tests per
      order class, plus: raise the rate under a resting order and check the reservation still
      covers the fill. Depends on: P3.2, P2.3.
- [ ] **P3.5 Snapshot and replay.** Serialize ledger state; proptest `replay(events) == state`.
      Depends on: P3.3.
- [ ] **P3.6 Multi-asset conservation.** Long random sequence of deposits, orders, trades and
      cancels across several instruments; assert per-asset conservation and zero orphaned
      reservations at the end. Depends on: P3.4.

### Phase 4 — Event sourcing

Goal: durable, verifiable, replayable per-shard logs.

- [ ] **P4.1 Envelope encoding.** `postcard` payloads, `blake3` hash chain, `SchemaVersion`,
      `Upcaster` trait with a registry. Tests: chain verification detects reorder/removal.
      Depends on: P1.4.
- [ ] **P4.2 WAL adapter.** Segmented append-only files, CRC32 frames, group commit with a max
      latency budget, fsync policy enum, recovery that truncates a torn tail. Tests: kill-9
      simulation by truncating at random offsets. Depends on: P4.1.
- [ ] **P4.3 Snapshots.** Snapshot store with atomic rename; cadence by event count and by
      time; retention. Depends on: P4.2.
- [ ] **P4.4 Replay and integrity.** `snapshot + tail → aggregate` for engine and ledger; bit-flip
      and truncation tests; replay is the restart path of the supervisor. Depends on: P4.3, P3.5.
- [ ] **P4.5 Command journal.** Bounded `CommandId → outcome` window persisted with the log so
      idempotency survives a restart. Depends on: P4.2.
- [ ] **P4.6 Bench.** Append throughput with and without fsync; group-commit latency
      distribution. Depends on: P4.2.
- [ ] **P4.7 (stretch) Postgres and JetStream stores.** Same contract test suite against `sqlx`
      and `async-nats` adapters. Depends on: P4.4, P6.4.

### Phase 5 — Runtime

Goal: shards run, fail, and heal without losing or duplicating a single event.

- [ ] **P5.1 Shard pipeline.** `Shard` actor owns the matching engine only (ADR-0009); messages
      `Submit`, `Cancel`, `Amend`, `Tick`, `Snapshot`, `Query`; pipeline
      dedup → `ledger.reserve` (synchronous call to the ledger actor) → match → append batch →
      publish. A failed reservation emits `OrderRejected`; nothing after a successful
      reservation can fail except the append, so there is no in-memory compensation. Tests:
      pipeline as a pure function. Depends on: P3.4, P4.1, P5.11.
- [ ] **P5.2 Sequencer.** Intake from API and bus; global sequence; per-instrument routing;
      `rtrb` SPSC ring per shard; full ring → back-pressure signal to callers. Depends on: P5.1.
- [ ] **P5.3 Worker pool and thread topology.** A fixed pool of worker arbiters sized from the
      cores (minus the I/O runtime and the ledger actor); instruments assigned deterministically
      by `hash(InstrumentId) % workers`, the assignment recorded as an event; round-robin over a
      worker's non-empty queues so a hot book cannot starve its neighbours; tokio multi-thread
      runtime for I/O; startup wiring in `server` (ADR-0012). Tests: 1 000 books on 8 workers
      replay identically to 1 000 books on 3; fairness under a hot instrument. Bench: latency of
      a quiet book while a neighbour on the same worker is saturated. Depends on: P5.2.
- [ ] **P5.4 Supervisor.** actix `Supervisor` + `RestartPolicy`; a panic is caught at the
      **worker boundary** and restarts only the offending book from its snapshot — the worker
      keeps serving its other books (ADR-0012); escalation to `Halted` after `max_restarts`;
      health transitions and metrics. Tests: a poisoned command on one book leaves its
      neighbours trading. Depends on: P5.1, P4.4.
- [ ] **P5.5 Watchdog and fault injection.** Shard heartbeats; stalled shard → restart; `fail`
      points that panic mid-batch; test that after recovery the log has no gaps and no
      duplicates and the ledger is conserved. Depends on: P5.4.
- [ ] **P5.6 Degradation controller.** Ladder driven by queue depth, error rate and p99 latency
      against the SLO; exposes `Level` to API admission. Depends on: P5.2.
- [ ] **P5.7 Graceful shutdown.** Drain rings, snapshot, close WAL, then exit. Depends on: P5.4.
- [x] **P5.8 Cross-shard design.** Decided 2026-08-26 in ADR-0009: the ledger is a **separate
      single-writer actor with one partition**, keyed by `AccountId`; instrument shards hold no
      balances. `reserve` is the only synchronous cross-actor step; commit/credit/fee/release
      are applied by the ledger from the instrument logs, idempotently, one batch per trade, so
      there is no in-flight money and no compensating saga. Implementation is P5.11.
      Depends on: —.
- [ ] **P5.11 Ledger actor.** Single-writer actor owning the ledger aggregate and its own log
      (one partition, keyed by account); messages `Reserve`, `Release`, `Query` (synchronous,
      answered over `oneshot`); a consumer that follows every instrument log in order and
      applies commit + credits + fees per trade and releases per `OrderDone` as one batch,
      idempotent by `(instrument, seq)`. Tests: replay instrument logs → per-asset conservation;
      duplicate delivery is a no-op; a reserve rejection never reaches the engine.
      Depends on: P3.3, P4.2.
- [ ] **P5.9 End-to-end deterministic test.** Command stream → shards → memory store + memory bus;
      assert identical event logs across two runs. Depends on: P5.2.
- [ ] **P5.10 Loom and Miri.** Any hand-written lock-free structure gets a loom model; Miri job
      in CI over `resilience` and `runtime`. Depends on: P5.2.

### Phase 6 — Bus and QoS

- [ ] **P6.1 Topic scheme.** `trading.{instrument}.exec`, `md.{instrument}.l2`,
      `ledger.{account}`, `contracts.{id}`; every message carries the shard `Seq` so consumers
      detect gaps. Depends on: P5.1.
- [ ] **P6.2 Memory bus complete.** `Subscriber` trait; lag handling per QoS (trading: error and
      resync by `Seq`; market data: jump to latest). Depends on: P6.1.
- [ ] **P6.3 Conflation.** Latest-state coalescer per instrument per subscriber with a flush
      interval; test that a slow subscriber sees a consistent latest book. Depends on: P2.9, P6.2.
- [ ] **P6.4 NATS JetStream adapter.** Publish with `Nats-Msg-Id = seq` dedup; pull consumer with
      ack; reconnect with backoff; breaker. Feature `nats`. Depends on: P6.2.
- [ ] **P6.5 Contract tests.** One suite run against memory and NATS (podman container in CI).
      Depends on: P6.4.
- [ ] **P6.6 Back-pressure end-to-end.** Slow market-data consumer must not increase trading
      latency; measured by a test with latency assertions. Depends on: P6.3.

### Phase 7 — API

- [ ] **P7.1 Wiring and conversion.** `AppState` holding the sequencer handle and instrument
      registry; `protocol::convert` decimal → ticks/lots with alignment errors; `RejectReason →
      (status, ApiError)` mapping table. Depends on: P5.2.
- [ ] **P7.2 REST v1.** `POST /v1/orders`, `DELETE /v1/orders/{id}`, `GET /v1/orders/{id}`,
      `GET /v1/books/{symbol}?depth=`, `GET /v1/accounts/{id}/balances`. Depends on: P7.1.
- [ ] **P7.3 Idempotency and request metadata.** `Idempotency-Key` → `CommandId`; request id;
      `X-Deadline-Ms` propagation into the pipeline. Depends on: P7.2.
- [ ] **P7.4 Admission control.** Per-key limiter, global load shedding with `Retry-After`,
      degradation level gates (cancel-only rejects new orders with a stable error code).
      Depends on: P7.2, P8.1, P5.6.
- [ ] **P7.5 WebSocket streams.** `/v1/stream`: market data (conflated) and private executions
      (lossless, resume by `Seq`). Depends on: P6.3.
- [ ] **P7.6 OpenAPI.** `utoipa` document, Swagger UI at `/docs`, snapshot test of the spec.
      Depends on: P7.2.
- [ ] **P7.7 Auth stub.** API keys via `secrecy`, key → account binding; ADR that real identity
      is the sandbox's concern. Depends on: P7.2.
- [ ] **P7.8 Integration tests.** `tower::ServiceExt::oneshot` for REST; `tokio-tungstenite` for
      WS; error mapping table test. Depends on: P7.5.
- [ ] **P7.9 (stretch) gRPC.** `tonic` for module-to-module calls. Depends on: P7.2.

### Phase 8 — Resilience and client SDK

- [ ] **P8.1 GCRA limiter.** Atomic CAS implementation, keyed variant with sharded map; loom
      model; bench vs `governor`. Depends on: —.
- [ ] **P8.2 Circuit breaker.** Sliding-window failure rate, half-open probes, test clock.
      Depends on: —.
- [ ] **P8.3 Bulkhead and shedding.** Semaphore bulkhead; shed by queue age against deadline.
      Depends on: —.
- [ ] **P8.4 Retry policy.** `backon` integration; decorrelated jitter; retries only for
      idempotent calls (`CommandId` present). Depends on: —.
- [ ] **P8.5 Client SDK.** Policy chain per method; `DegradationStrategy` state machine; WS
      resubscribe with `Seq` resume; `wiremock` tests for 429/503/timeouts/partial outages.
      Depends on: P8.1–P8.4, P7.5.
- [ ] **P8.6 Chaos test.** Client fleet vs server with injected faults; assert no duplicate
      orders and bounded retry traffic. Depends on: P8.5.

### Phase 9 — External adapters

- [ ] **P9.1 Ports.** `LiquidityProvider { quote, execute }`, `PriceOracle`, `Macroeconomy {
      mint, burn, rates }`, `ClockSource`. Mocks under feature `mocks`. Depends on: —.
- [ ] **P9.2 Fakes with fault injection.** Scripted scenarios; latency distribution, error rate,
      hangs; `fail` points. Depends on: P9.1.
- [ ] **P9.3 `Guarded<T>` decorator.** Timeout + breaker + retry + metrics around any port.
      Depends on: P8.2, P8.4.
- [ ] **P9.4 Liquidity backstop.** When depth < threshold, source synthetic maker orders from
      the provider on an LP account; ADR on accounting. Depends on: P9.3, P5.1.
- [ ] **P9.5 Macroeconomy integration.** Escrow over the bus for MOD-08 (reserve/commit/release
      messages) and the authorisation path *into* the engine: `Mint`, `Burn`, `SetFeeSchedule`
      (ADR-0011, ADR-0013), so macroeconomics can answer inflation by raising fees without a
      deployment. Event contract documented. Tests: a rate change is visible in the next trade's
      fee and in replay. Depends on: P6.2, P3.2, P2.8.
- [ ] **P9.6 Chaos suite.** Provider down/slow → degradation level and recovery. Depends on: P9.4, P5.6.

### Phase 10 — Smart contracts

- [ ] **P10.1 Contract runtime.** Registry, subscription to bus topics, scheduler ticks, command
      emission via the sequencer, persistence via `ContractEvent`. Depends on: P6.2, P5.2.
- [ ] **P10.2 `LoanAgreement`.** States `Active → Overdue → Defaulted | Closed`; instalment
      schedule; interest in bps with rounding policy; collection through escrow; default
      triggers collateral liquidation as a market sell under the deviation guard. Depends on: P10.1, P3.4.
- [ ] **P10.3 Dividends.** Holdings snapshot at record time; largest-remainder pro-rata
      distribution; test `sum(paid) == declared`. Depends on: P10.1.
- [ ] **P10.4 Properties.** Contracts never create money; replay determinism. Depends on: P10.2, P10.3.
- [ ] **P10.5 (stretch) Contract templates.** Config-driven templates for the sandbox designers.
      Depends on: P10.4.

### Phase 11 — Observability

- [ ] **P11.1 Metrics.** All names in `telemetry::names`; Prometheus exporter; per-shard labels.
      Depends on: P5.1.
- [ ] **P11.2 Tracing.** Spans across API → sequencer → shard keyed by `command_id`; sampling on
      the hot path. Depends on: P7.1.
- [ ] **P11.3 Dashboards and alerts.** Grafana JSON under `deploy/grafana`. Depends on: P11.1.
- [ ] **P11.4 Profiling guide.** `perf`, flamegraph, `perf stat` cache/branch counters,
      false-sharing checklist; `just profile`. Deliverable: `docs/PERFORMANCE.md`. Depends on: P2.13.
- [ ] **P11.5 SLOs.** p99 shard latency, availability, degradation triggers documented and
      wired to the controller. Depends on: P5.6.

### Phase 12 — Deterministic simulation, load and performance

- [ ] **P12.1 DST harness.** Single-threaded scheduler, seeded RNG, `turmoil` network for API
      and bus, fault injection; every failure reproducible from a seed. Depends on: P5.9.
- [ ] **P12.2 Load generator.** Open-loop arrival, HDR histograms, scenario files, JSON/CSV
      report. Depends on: P8.5.
- [ ] **P12.3 Tuning iteration.** Batching per wakeup, busy-poll vs park, arena sizing; results
      in `docs/PERFORMANCE.md`. Depends on: P12.2.
- [ ] **P12.4 Bench regression CI.** Nightly criterion compare against `main`; fail on >10 %
      p99 regression. Depends on: P2.12.
- [ ] **P12.5 Soak.** One hour with chaos enabled: stable memory, no log gaps. Depends on: P12.1.

### Phase 13 — Release and presentation

- [ ] **P13.1 Final architecture doc** with diagrams and complete ADR index.
- [ ] **P13.2 Demo script.** `just infra-up`, seed accounts, `just loadgen`, Grafana walk-through.
- [ ] **P13.3 Release 0.1.0.** `CHANGELOG.md`, tag, GitHub release, Rust forum review thread,
      mentor reviews.
- [ ] **P13.4 Roadmap.** Margin engine (works in tandem with matching), derivatives, multi-node
      replication, gRPC.

## Testing strategy

| Level       | Tool                              | Where                     | What it proves                          |
|-------------|-----------------------------------|---------------------------|-----------------------------------------|
| unit        | `cargo test`, `rstest`            | each crate                | local contracts, error mapping          |
| property    | `proptest`                        | domain, matching, ledger  | invariants over random inputs           |
| model-based | `proptest` + `testkit::reference` | matching                  | engine ≡ naive oracle                   |
| snapshot    | `insta`                           | protocol, eventstore      | wire and storage formats do not drift   |
| concurrency | `loom`, Miri                      | resilience, runtime       | lock-free code is sound                 |
| integration | `tower::oneshot`, memory adapters | api, runtime              | wiring, end-to-end flows                |
| contract    | shared suites                     | eventstore, bus, adapters | every adapter honours its port          |
| chaos       | `fail`, fakes                     | runtime, client           | recovery and bounded degradation        |
| simulation  | DST harness, `turmoil`            | `tests/` in runtime       | whole system deterministic under faults |
| fuzz        | `cargo fuzz`                      | `fuzz/`                   | no panics, invariants on hostile input  |
| load        | `loadgen`                         | binary                    | throughput and tail latency             |

Mocks: `mockall` mocks live next to their port behind the `mocks` feature and are used for
interaction tests only. Behavioural tests use fakes (scripted, fault-injecting) so that the test
exercises real control flow.

## Benchmark policy

- Benches live in `crates/<crate>/benches/`, run with `just bench <crate>` (host-CPU codegen).
- A hot-path PR pastes the criterion comparison into the PR description and updates
  `docs/PERFORMANCE.md`.
- Baselines are stored under `target/criterion` locally; the nightly job (P12.4) compares against
  `main` and blocks regressions over 10 % on p99.
- Numbers are always reported with the machine (`nproc`, CPU model, governor) and the commit.

## Risk register

| Risk                                                        | Mitigation                                                                  |
|-------------------------------------------------------------|-----------------------------------------------------------------------------|
| Matching semantics disputed late (FOK, STP, fee rounding)   | `docs/DOMAIN.md` worked examples signed off in P1.5                         |
| actix arbiter model fights tokio I/O runtime                | P5.3 decides topology with measurements; fallback is plain threads + `rtrb` |
| Zero-allocation goal blocks progress                        | Goal is measured in P2.13, not a gate for earlier tasks                     |
| Event schema churn breaks replay                            | P1.4 snapshots + upcasters from day one                                     |
| Team unfamiliar with finance domain                         | glossary, examples and reference matcher before the fast engine             |
| Scope exceeds three days                                    | MVP cut line above; stretch items flagged                                   |

## Open questions

None blocking. New questions go here as they surface; anything that constrains future work
becomes an ADR instead.

### Decided

- **Price improvement** (was: does the exchange capture the spread?) — decided 2026-08-26,
  **ADR-0010**: a trade executes at the **maker's price**, improvement goes to the taker.
  Exchange income is maker/taker fees (rounded up) plus the spread its own LP account earns
  when it makes markets (P9.4). `PriceImprovementPolicy` and `Trade::taker_price` are gone.
- **Ledger topology** (was: shard by account or by instrument?) — decided 2026-08-26,
  **ADR-0009**: a **separate single-writer ledger actor, one partition**, keyed by account.
  See P5.8 and P5.11.
- **External identifier format** (was: ULID or integers from the other modules?) — decided
  2026-08-26, **ADR-0011**: the engine treats them as **opaque strings** (`ExternalId`) and maps
  each to a dense internal id once, on first sight; no agreement with other modules needed.
- **Settlement delivery** (was: must MOD-08 confirm a trade synchronously?) — decided
  2026-08-26, **ADR-0011**: **no**. A trade is confirmed once its batch is durable; other
  modules receive it over the bus, at-least-once with gap detection by `(instrument, seq)`.
  MOD-08 authorises *before* the fact (mint, burn, rates) instead of confirming after it.
- **Scale of tradable items** (was: one thread per instrument?) — decided 2026-08-26,
  **ADR-0012**: the target is **hundreds to thousands of order books**, so instruments share a
  **fixed worker pool** (`hash(InstrumentId) % workers`) instead of owning a thread each. Each
  book still has exactly one writer. See P5.3 and P5.4.
- **Where fee money goes** (was: sink or treasury?) — decided 2026-08-26, **ADR-0013**: a
  per-instrument `FeeDestination { burn_bps, treasury }` splits every fee between **burning**
  (leaves the money supply — the sink) and the treasury account. Rates are instrument state set
  by MOD-08 via `SetFeeSchedule`; reservations use a `max_fee_bps` ceiling so a rate change
  never breaks a resting order. See P2.8, P3.1, P3.4, P9.5.
