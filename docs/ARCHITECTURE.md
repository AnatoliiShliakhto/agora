# Architecture

The Economy & Market Engine is the transactional core of a game-world sandbox: an isolated,
event-driven exchange that matches orders, escrows funds and executes contract rules. It is one
module (cluster CL-02) of a larger simulator and talks to the rest of it over a message bus.

- [Context](#context)
- [Structure](#structure)
- [Command pipeline](#command-pipeline)
- [Threading model](#threading-model)
- [Consistency model](#consistency-model)
- [Failure handling](#failure-handling)
- [Data flow and QoS](#data-flow-and-qos)
- [Decisions](#decisions)

## Context

```text
┌────────────────────────── sandbox ──────────────────────────┐
│                                                             │
│  players / bots ──HTTP,WS──▶ ┌───────────────────────────┐  │
│                              │  CL-02 Market Engine      │  │
│  MOD-08 Macroeconomy ──bus──▶│  matching · escrow ·      │──bus──▶ other modules
│  liquidity providers ──port─▶│  ledger · contracts       │  │
│  price oracle ────────port─▶ │                           │  │
│                              └───────────┬───────────────┘  │
│                                          │ event log (WAL)  │
└──────────────────────────────────────────┼──────────────────┘
                                           ▼
                                   Prometheus / Grafana
```

## Structure

Hexagonal: pure domain crates in the middle, ports as traits, adapters at the edge, one
composition root. See the crate table in `docs/PLAN.md`.

```text
             ┌───────────┐   ┌──────────┐
   in  ───▶  │  api      │   │  client  │  (SDK; used by loadgen and other modules)
             └─────┬─────┘   └──────────┘
                   ▼
             ┌───────────┐  sequencer, shards, supervisor, health
             │  runtime  │
             └─────┬─────┘
       ┌───────────┼──────────────┬───────────────┐
       ▼           ▼              ▼               ▼
  ┌──────────┐ ┌────────┐  ┌───────────┐   ┌────────────┐
  │ matching │ │ ledger │  │ contracts │   │ resilience │
  └────┬─────┘ └───┬────┘  └─────┬─────┘   └────────────┘
       └───────────┴─────────────┘
                   ▼
             ┌───────────┐
             │  domain   │  money, ids, instrument, order, events
             └───────────┘

  ports ⇄ adapters:  eventstore (memory | WAL | Postgres*)   bus (memory | NATS)
                     adapters (liquidity, oracle, macroeconomy: fake | http | bus)
```

## Command pipeline

One shard owns one instrument and runs this for every command, on one thread, with no locks:

```text
 command ─▶ dedup (CommandId journal)
         ─▶ validate (domain rules, session state)
         ─▶ ledger.reserve (sync call to the ledger actor)   ── reject → OrderRejected, done
         ─▶ engine.submit             ── fills, STP, TIF, stops
         ─▶ eventstore.append(batch)  ── ATOMIC: the ACID boundary of the instrument shard
         ─▶ bus.publish(trading, md)  ── after durability
         ─▶ reply to caller

 ledger actor (own thread, own log; ADR-0009):
   consumes instrument logs in order, idempotent by (instrument, seq)
         ─▶ per trade, ONE batch: commit(buyer) + commit(seller) + credits + fees
         ─▶ per OrderDone: release(remainder)
```

Every shard step before `append` is pure: on error the working copy is discarded (aggregates
are applied from events, so nothing is mutated until the batch is durable). The batch is the
transaction. Only `reserve` can fail; credits, releases and fee postings never do, so nothing
after matching needs compensation.

## Threading model

- **I/O runtime**: one tokio multi-thread runtime for HTTP, WebSocket, bus and store I/O.
- **Shards**: one actix `Arbiter` (dedicated OS thread) per instrument, optionally pinned to a
  core. The shard actor is the only writer of its engine and log.
- **Ledger actor**: one more arbiter owning all balances, reservations and the ledger log
  (single partition, keyed by account; ADR-0009).
- **Hand-off**: the sequencer pushes commands into a bounded `rtrb` SPSC ring per shard;
  responses come back through a `tokio::sync::oneshot`. A full ring is a back-pressure signal,
  never a block.
- **Read-mostly configuration** (instrument registry, fee schedules, degradation level) is
  `arc_swap::ArcSwap`; readers never lock.

Lock-free here means "no lock on the command path", not "every structure is a CAS loop". The
single-writer principle removes the need for most synchronization; the few shared counters are
atomics. ADR-0003 has the reasoning and the alternatives considered.

## Consistency model

- **Per shard: serializable.** Commands are applied one at a time in sequence order; the event
  log is the total order.
- **Between instrument shards and the ledger: escrow.** `reserve` is synchronous and is the
  only step that can fail; commit/credit/release are applied from the log, idempotently, one
  ledger batch per trade, so per-asset conservation holds at every instant (ADR-0004, ADR-0009).
- **Durability**: append is fsync'd per batch or group-committed under a latency budget
  (configurable). Publishing happens after durability; consumers detect gaps by `Seq` and
  resync from the log.
- **Recovery**: snapshot + tail replay; hash chain verifies integrity; the command journal makes
  retries idempotent across restarts.

## Failure handling

| Failure                        | Response                                                                                                                           |
|--------------------------------|------------------------------------------------------------------------------------------------------------------------------------|
| shard panics mid-command       | supervisor restarts it from snapshot; the unappended batch is lost, the caller gets an error and retries with the same `CommandId` |
| shard stalls                   | watchdog misses heartbeats → restart                                                                                               |
| repeated restarts              | escalate to `Halted` for that instrument, others keep trading                                                                      |
| queue depth / latency over SLO | degradation ladder: `Throttled → CancelOnly → MarketDataOnly → Halted`                                                             |
| external provider down         | breaker opens, liquidity backstop disabled, level `Throttled`                                                                      |
| bus slow consumer              | trading: back-pressure + resync by `Seq`; market data: conflate                                                                    |
| store I/O error                | shard `Halted`; no event is published without durability                                                                           |

## Data flow and QoS

Two classes of traffic never share a queue:

- **Trading** (commands, executions, ledger events): lossless, ordered, bounded, back-pressured.
- **Market data** (L2 deltas, trades feed): conflated per subscriber; slow consumers see the
  latest state, never a backlog.

Admission control happens at the API edge (per-key limiter, load shedding with `Retry-After`)
and at the sequencer (ring capacity). Degradation level gates which endpoints are accepted.

## Decisions

See `docs/adr/`. Key ones: fixed-point money (0002), single-writer shards on actix (0003),
event sourcing and escrow sagas (0004), NATS with QoS split (0005), hexagonal crate layout
(0006), self-healing policy (0007), testing strategy (0008), separate ledger actor (0009),
maker's price and fee income (0010).
