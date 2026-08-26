# Changelog

All notable changes to this project are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versions follow SemVer.

## [Unreleased]

### Added

- ADR-0012 (instruments share a fixed worker pool — the economy targets hundreds to thousands of
  order books) and ADR-0013 (fees are the money sink: configurable burn/treasury split, rates set
  by the macroeconomy module); `docs/DOMAIN.md` § Fees and the money sink.
- `CAPSTONE.md` — Ukrainian onboarding document for contributors: what the project is, the
  engineering challenges it covers, how to start, how to take tasks and what to expect.

### Changed

- `README.md` now separates what works today from what the plan adds, instead of describing the
  finished system in the present tense.
- Removed the project-rename script and its `just` recipe; the rename is done.
- The workspace is IDE- and tool-agnostic: shared instructions name no machine-specific tool.
  Local MCP servers, IDE bridges and their permissions belong in `CLAUDE.local.md` and
  `.claude/settings.local.json` (both gitignored); `CONTRIBUTING.md` § Local setup explains the
  split, and editor state (`.idea/`, `.vscode/`, …) is no longer tracked.

### Added

- ADR-0011 (sandbox integration contract) and `domain::ids::ExternalId`; `docs/DOMAIN.md` maps
  every brief identifier to its implementation (P1.5) — Phase 1 complete.
- Frozen event schema: `SchemaId`, `SchemaVersion`, `DomainEvent`, `ContractEvent`, envelope
  metadata (`schema`, `LogicalTime`, `is_from_the_future`), round-trip proptests and byte-level
  snapshots (P1.4).
- `ValidOrder` with a fixed validation order, `PriceBounds` on instruments and `domain::time`
  (`LogicalTime`); `Engine::submit` accepts only validated orders (P1.3).
- `Symbol`, `InstrumentRegistry`, `SessionState` with the command matrix and transitions,
  `RejectReason::NotTrading` (P1.2).
- `RoundingMode` with `Amount::checked_div`, `FeeBps`/`FeeSchedule` (P1.1); `NotionalScale`
  is non-zero by construction; `Trade` carries maker and taker fees.
- ADR-0009 (separate single-writer ledger actor) and ADR-0010 (maker's price, income from fees
  and market making); `PriceImprovementPolicy` removed.
- Workspace skeleton: 16 crates, strict lint set, CI, `just verify` gate.
- Domain types: fixed-point `Price`/`Qty`/`Amount`, identifiers, `Instrument`, order model,
  `BookEvent`/`LedgerEvent`.
- Order book price levels, engine scaffold, fee schedule, first benchmark.
- In-memory event store and bus, resilience primitives scaffold, actix shard actor scaffold.
- axum health endpoints, client SDK scaffold, telemetry init, test kit.
- Documentation: plan, architecture, domain glossary and rules, ADR-0001…0008, code style,
  agent guidance and skills.
