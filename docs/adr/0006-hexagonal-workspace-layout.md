# 0006. Hexagonal workspace layout

Date: 2026-08-26
Status: accepted

## Context

The system must stay testable without infrastructure and must allow adapters to be swapped
(memory → WAL → Postgres, memory → NATS) without touching business logic.

## Decision

Ports-and-adapters over a DDD core, one crate per bounded responsibility:

- domain crates (`domain`, `matching`, `ledger`, `contracts`) have no async, no I/O and depend
  only on `serde`;
- port crates (`eventstore`, `bus`, `adapters`) define traits next to their in-memory adapters
  and put real adapters behind features;
- `runtime` is the application layer; `api`/`client` are interfaces; `server` is the only
  composition root.

Dependency direction is strictly downward; `testkit` is dev-only; lints are inherited from the
workspace; each crate's `README.md` is its crate-level doc.

## Consequences

Many small crates (fast incremental builds, clear ownership) at the price of some ceremony when
adding one (`.claude/skills/new-crate`).
