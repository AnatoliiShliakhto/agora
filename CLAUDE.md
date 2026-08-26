# CLAUDE.md

Guidance for Claude Code in this repository. `AGENTS.md` is the short version for other agents;
this file is the authority for Claude.

## What this is

Event-sourced order-matching engine with escrow and a smart-contract rules engine, module CL-02
of a game-world sandbox. Rust 2024, one Cargo workspace, 16 crates under `crates/`, MIT.
Read in this order before non-trivial work: `docs/DOMAIN.md` (semantics), `docs/ARCHITECTURE.md`
(structure), `docs/PLAN.md` (backlog and acceptance criteria), `docs/adr/` (decisions),
`CODESTYLE.md` (rules the lints do not catch).

## Commands

```bash
just                 # list recipes
just verify          # full local CI gate: fmt-check lint test test-doc doc deny typos bench-build
just fmt             # cargo +nightly fmt + taplo (rustfmt.toml uses unstable options)
just lint            # clippy --workspace --all-targets --all-features -- -D warnings
just test [args]     # nextest; `just test -p agora-matching book::` narrows
just doc --open      # rustdoc with -D warnings
just bench matching  # criterion, host-CPU codegen
just run / loadgen / infra-up
```

CI (`.github/workflows/ci.yml`) runs exactly the `verify` gates. A PR is not done until
`just verify` is green locally.

## Working rules

- **Pick work from `docs/PLAN.md`.** Tasks have ids, dependencies, deliverables and acceptance.
  Tick the checkbox in the PR that completes the task. Out-of-scope findings become new tasks,
  not detours.
- **Non-negotiables** (see the plan): no floats outside `telemetry`/`loadgen`; checked
  arithmetic in `domain`/`matching`/`ledger`/`contracts`; deterministic aggregates (time and
  ids are inputs); single writer per shard, no locks on the command path; events are the only
  persisted state; every port has a mock and a fault-injecting fake; hot-path changes carry a
  bench and numbers in `docs/PERFORMANCE.md`.
- **Lints are strict on purpose.** Fix the code; if the lint is wrong here, use
  `#[expect(lint, reason = "...")]`, never `#[allow]`. Do not weaken `Cargo.toml` lints in a
  feature PR — that is its own PR with a justification.
- **Design before code** for anything touching matching semantics, ledger invariants or the
  threading model: work through ELIMINATE → SIMPLIFY → REUSE → CREATE before writing code (use
  extended thinking, or a structured-reasoning tool if you have one), check `docs/DOMAIN.md`,
  and write or update an ADR if the answer constrains the future. For a one-line fix, don't.
- **Tests are the spec.** Invariants as `proptest`; worked examples from `docs/DOMAIN.md` as unit
  tests; formats as `insta` snapshots. A bug fix starts with the failing test.
- **Docs**: every `pub` item documented per `CODESTYLE.md` §9 (verb for functions, noun for
  types, intra-doc links). Crate `README.md` is the crate doc. Keep comments terse.
- **Commits**: imperative, capitalised, no trailing period; one task per PR. When an agent
  wrote the change, append a `Co-Authored-By:` trailer naming the model that did
  (e.g. `Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>`).
- Report results honestly: paste the failing output, say what was skipped.

## Skills

`.claude/skills/` — invoke with `/name`:

| Skill             | Use when                                                                 |
|-------------------|--------------------------------------------------------------------------|
| `next-task`       | starting work: picks and executes the next plan task end-to-end          |
| `domain-check`    | touching `matching`/`ledger`/`contracts`: invariant & rounding checklist |
| `hot-path-review` | reviewing code on the shard command path: allocation, locks, layout      |
| `new-crate`       | adding a workspace crate with the standard preamble and manifest         |
| `adr`             | recording a decision                                                     |
| `verify`          | running the gate and triaging failures (known pitfalls inside)           |
| `bench`           | running/comparing criterion benches and recording numbers                |

## Tooling

The repo assumes nothing beyond a Rust toolchain (pinned in `rust-toolchain.toml`), `just`, and
the dev tools `just setup` installs. Every project operation is a `just` recipe, so it works the
same in any editor and in CI.

Code navigation is whatever you have; pick by scope and freshness, not by habit:

- **broad "where is X handled"** — a semantic/symbol index if you have one, otherwise `Grep` and
  the `Explore` agent;
- **impact analysis before a change** — find-references over the whole workspace, then read the
  call sites;
- **code written or edited in this session** — an IDE/LSP bridge or a plain `Read`: background
  indexes lag, and stale results are worse than none;
- **compile errors** — `cargo check -p <crate>` is always available; an IDE diagnostics bridge is
  faster if you have one.

Machine-specific tools (local MCP servers, IDE bridges, personal indexes) belong in
**`CLAUDE.local.md`**, which is gitignored and loads automatically alongside this file. Never
add them here, and never commit a `.claude/settings.json` permission for an MCP server other
contributors may not have — put those in `.claude/settings.local.json` (also gitignored;
its `allow` list merges with the shared one). See `CONTRIBUTING.md` § Local setup.

Knowledge that must outlive a session goes into the repo — `docs/PLAN.md`, an ADR, or
`CHANGELOG.md` — not into a personal note store, so the next contributor sees it too.

## Known pitfalls

- `cargo doc` can fail with a stale `Amount: Default`-style error after editing derives while
  another cargo command runs: `cargo clean -p <crate>` and rerun.
- `rustfmt.toml` needs nightly: `cargo +nightly fmt`. `cargo fmt` on stable silently ignores
  the unstable options and `fmt-check` in CI fails.
- `mockall::automock` generates `std::sync::Mutex`, which is a disallowed type; put mocks in a
  `mocks` module with `#![expect(clippy::disallowed_types, reason = "...")]` (see
  `crates/adapters/src/oracle.rs`).
- `insta` snapshots: first run needs `INSTA_UPDATE=always`; commit the `.snap` file.
- After moving or renaming the checkout, run `cargo clean` once: `file!()` bakes the old
  absolute path into test binaries, so `insta` looks for snapshots under a directory that no
  longer exists and every snapshot test fails with a bogus diff.
- `cargo deny` is restricted to Linux targets in `deny.toml`; embedded-only transitive advisories
  are out of scope.
