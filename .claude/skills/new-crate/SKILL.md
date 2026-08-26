---
name: new-crate
description: Add a workspace crate with the standard manifest, lib.rs preamble, README-as-doc and alphabetical registration
disable-model-invocation: true
---

# New crate

Argument: crate short name (e.g. `margin`). Package name is `agora-<name>`, directory
`crates/<name>`.

1. Copy the manifest shape from `crates/ledger/Cargo.toml`: `name`, `description`,
   `readme = "README.md"`, every other `[package]` field `.workspace = true`, dependencies as
   `{ workspace = true }`, `[lints] workspace = true`. Binaries add `[[bin]]`; benches add
   `[[bench]] harness = false`.
2. Write `README.md` as rustdoc markdown: one paragraph of purpose, a bullet per module, the
   hot-path or invariant rules that apply.
3. `src/lib.rs` preamble from `CODESTYLE.md` §1 (`#![doc = include_str!("../README.md")]`,
   `#![forbid(unsafe_code)]`, the test `cfg_attr`, and `arithmetic_side_effects = deny` for
   finance crates). Then `mod` lines only, alphabetical.
4. Register: `members` in the root `Cargo.toml` (alphabetical) and a
   `agora-<name> = { path = "crates/<name>" }` line in `[workspace.dependencies]`.
5. Add the row to the workspace map in `docs/PLAN.md` and the layout block in `README.md`.
6. `just verify`. New crates must pass with zero `#[expect]`s at birth.
