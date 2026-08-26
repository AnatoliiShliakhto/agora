# agora

[![CI](https://github.com/AnatoliiShliakhto/agora/actions/workflows/ci.yml/badge.svg)](https://github.com/AnatoliiShliakhto/agora/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust 1.98](https://img.shields.io/badge/rust-1.98-orange.svg)](rust-toolchain.toml)

Distributed, event-sourced order-matching engine with escrow and a smart-contract rules engine —
the Economy & Market Engine (CL-02) of a game-world sandbox.

It is the core of a **self-regulating in-game exchange**: players trade any item on a real
order book, fees drain surplus currency out of the economy (a configurable share of every fee is
burned), and macroeconomics answers inflation by moving those fee rates. In-memory price-time
matching, single-writer books with no locks on the command path, ACID-level consistency of funds
without database locks.

> **Status:** Phase 1 of [`docs/PLAN.md`](docs/PLAN.md) — the domain core — is complete: 117
> tests, `just verify` green. The matching engine and everything after it are being built phase
> by phase; the split below says exactly what exists today.
>
> New here? Read [`CAPSTONE.md`](CAPSTONE.md) (Ukrainian) — what the project is, how to work on
> it and what you get out of it.

## What it does

The whole system is specified in [`docs/DOMAIN.md`](docs/DOMAIN.md) and scheduled task by task
in [`docs/PLAN.md`](docs/PLAN.md).

**Working today** — the domain core, with the invariants enforced by types and tests:

- fixed-point money (integer ticks, lots and minor units); floats are a compile error, every
  operation is checked, every division names its rounding direction;
- instruments, symbols, session states and the command matrix each state accepts;
- order model and validation: an order reaches the engine only as a `ValidOrder`, after a fixed,
  documented sequence of checks;
- a frozen event schema with per-family versions, round-trip and byte-level snapshot tests;
- an order book with price-time levels, and the strict lint/test/doc gate that guards all of it.

**Being built**, in plan order:

- matching (Phase 2): `Limit`/`Market`/`StopMarket`/`StopLimit` with `GTC`/`IOC`/`FOK`/`GTD`,
  partial fills, self-trade prevention, deviation guard, fees;
- ledger (Phase 3): escrow before the book, balanced double-entry settlement, conservation
  proofs;
- event store (Phase 4): WAL with group commit, snapshots, hash-chained replay;
- runtime (Phase 5): one actor per instrument, a single-writer ledger actor, supervision and
  restart-from-snapshot;
- bus QoS (6), versioned REST/WebSocket API (7), resilience and client SDK (8), external
  adapters (9), smart contracts (10), observability (11), deterministic simulation and load
  testing (12).

## Quick start

```bash
just setup      # dev tools: nextest, deny, hack, typos, taplo, llvm-cov, fuzz, nightly rustfmt
just verify     # full local CI gate
just run        # exchange server on 127.0.0.1:8080
just infra-up   # NATS JetStream + Prometheus + Grafana via podman/docker compose
```

`just` with no arguments lists every recipe. The toolchain is pinned in `rust-toolchain.toml`;
apart from it and `just`, the repo assumes no particular editor or tooling — see
[`CONTRIBUTING.md`](CONTRIBUTING.md) § Local setup for where machine-specific configuration
goes.

## Layout

```text
crates/
  domain/      pure model: fixed-point money, ids, instruments, orders, events
  matching/    order book + matching engine
  ledger/      balances, escrow, postings
  contracts/   rules engine, scheduler, loan, dividend
  eventstore/  envelopes, hash chain, memory/WAL stores, snapshots
  bus/         QoS-split message bus, memory/NATS adapters
  resilience/  limiter, breaker, backoff, bulkhead, shedding, degradation ladder
  runtime/     shard actors, sequencer, supervisor, health
  adapters/    liquidity / oracle / macroeconomy ports, fakes, mocks
  protocol/    versioned DTOs, decimal ⇄ ticks
  api/         axum REST + WebSocket
  client/      SDK
  telemetry/   tracing, metrics, HDR latency
  testkit/     clock, fixtures, strategies, reference matcher
  server/      composition root (binary)
  loadgen/     open-loop load generator (binary)
fuzz/          cargo-fuzz targets
docs/          PLAN, ARCHITECTURE, DOMAIN, adr/
deploy/        compose file, Prometheus config
```

## Documentation

- [`CAPSTONE.md`](CAPSTONE.md) — what this project is and how to contribute (Ukrainian).
- [`docs/PLAN.md`](docs/PLAN.md) — the backlog, phase by phase, with acceptance criteria.
- [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) — structure, pipeline, threading, consistency.
- [`docs/DOMAIN.md`](docs/DOMAIN.md) — glossary, matching rules, escrow protocol, worked examples.
- [`docs/adr/`](docs/adr/README.md) — decisions and why.
- [`CODESTYLE.md`](CODESTYLE.md), [`CONTRIBUTING.md`](CONTRIBUTING.md).

## Engineering rules in one paragraph

No floats; checked arithmetic in every finance crate; deterministic aggregates; single writer per
shard and no locks on the command path; events are the only persisted state; every port has a
mock and a fault-injecting fake; every phase ships tests and benches; `just verify` must be
green. The lint set in `Cargo.toml` enforces most of this at compile time.

## License

MIT — see [`LICENSE`](LICENSE).
