Code style
==========

Rules for Rust sources in this workspace. Most are enforced by the lint set in `Cargo.toml`;
the rest are enforced in review.

> [!NOTE]\
> The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT",
> "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in
> [RFC 2119].

- [1. Crate layout](#1-crate-layout)
- [2. Lints](#2-lints)
- [3. Arithmetic and money](#3-arithmetic-and-money)
- [4. Errors and panics](#4-errors-and-panics)
- [5. Concurrency](#5-concurrency)
- [6. Naming](#6-naming)
- [7. Types and conversions](#7-types-and-conversions)
- [8. Tests and benches](#8-tests-and-benches)
- [9. Code documentation](#9-code-documentation)
    - [9.1. Functions and macros start with a verb](#91-functions-and-macros-start-with-a-verb)
    - [9.2. Other items start with a noun](#92-other-items-start-with-a-noun)
    - [9.3. Linking to items by name](#93-linking-to-items-by-name)
    - [9.4. Examples](#94-examples)
    - [9.5. Panics](#95-panics)
    - [9.6. Errors](#96-errors)
    - [9.7. `README.md` is crate-level documentation](#97-readmemd-is-crate-level-documentation)
- [10. Comments](#10-comments)




## 1. Crate layout

Every library crate **MUST** start with this preamble in `lib.rs`:

```rust
#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing, reason = "tests may panic"))]
```

Finance crates (`domain`, `matching`, `ledger`, `contracts`) **MUST** add:

```rust
#![deny(clippy::arithmetic_side_effects, reason = "finance code uses checked arithmetic only")]
```

A crate that needs `unsafe` replaces `forbid` with
`#![allow(unsafe_code, reason = "...")]` and documents every block with a `// SAFETY:` comment
(`clippy::undocumented_unsafe_blocks = deny`).

`lib.rs` **SHOULD** contain only the preamble, `mod` declarations and re-exports. One module per
file; `mod.rs` files are forbidden (`clippy::self_named_module_files`).

`Cargo.toml` of every crate **MUST** use `[lints] workspace = true`, inherit package metadata
from the workspace, and declare dependencies as `{ workspace = true }`. Workspace `members`
and dependency lists **MUST** stay alphabetical.


## 2. Lints

The workspace denies `clippy::all`, warns `pedantic`, `nursery` and `cargo`, and enables a set of
restriction lints; CI runs with `-D warnings`, so a warning is an error.

- Silencing a lint **MUST** use `#[expect(lint, reason = "...")]`. Bare `#[allow]` is denied
  (`clippy::allow_attributes`). The reason **MUST** say why the lint is wrong here, not what it is.
- Widening an `expect` to a whole module or crate is **NOT RECOMMENDED**; prefer the item.
- Adding a lint to the workspace `allow` list requires a comment with the justification, as the
  existing entries have.


## 3. Arithmetic and money

- Floating point types **MUST NOT** appear outside `telemetry` and `loadgen`
  (`clippy::float_arithmetic = deny`).
- Prices, quantities and amounts **MUST** use `domain::money` types. Raw `i64`/`u64`/`i128`
  **MUST NOT** carry money across a function boundary.
- Arithmetic in finance crates **MUST** be `checked_*` and return `ArithmeticError` on overflow.
  `wrapping_*`/`saturating_*` **MAY** be used only for non-financial counters, with a comment.
- Every division **MUST** name its rounding direction in code (`checked_div_floor`, a
  `RoundingMode` argument) and in the doc comment.
- Loop counters and indices **SHOULD** use iterator adapters instead of manual arithmetic.


## 4. Errors and panics

- Library code **MUST NOT** panic on input: `unwrap`, `expect`, `panic!`, `unreachable!`,
  `unimplemented!` and slice indexing are denied or warned. Use `?`, `get`, pattern matching.
- Each crate defines its error enums with `thiserror`; variants are documented and
  `#[non_exhaustive]` where the set will grow. `anyhow` is for binaries only.
- Domain rejections are `RejectReason`, not `Err(String)`. Infrastructure errors wrap their
  source (`#[source]`/`#[from]`), never stringify it.
- `todo!()` is a warning: it **MAY** exist on a feature branch, **MUST NOT** reach `main`.


## 5. Concurrency

- The command path **MUST NOT** take a lock. `std::sync::Mutex`/`RwLock` are disallowed types;
  `parking_lot` **MAY** be used off the hot path with an `#[expect]` and a reason.
- A lock **MUST NOT** be held across an `.await`.
- Cross-thread hand-off **MUST** use channels or the `rtrb`/`crossbeam` queues; shared
  read-mostly state **MUST** use `arc_swap`.
- Aggregates (`matching`, `ledger`, `contracts`) **MUST NOT** read the wall clock, an RNG, or
  iterate a `HashMap` where order affects output. Time and randomness are inputs.
- `std::thread::sleep` is disallowed; use `tokio::time::sleep` or the testkit clock.
- Hand-written lock-free code **MUST** have a `loom` model and run under Miri in CI.


## 6. Naming

- Use the terms of `docs/DOMAIN.md` verbatim: `Instrument`, `Qty`, `Escrow`, `Posting`,
  `Seq`. Do not invent synonyms (`Volume` for `Qty`, `Symbol` for `Instrument`).
- Ports are traits named for the capability (`EventStore`, `Publisher`, `PriceOracle`);
  adapters are named for the technology (`MemoryStore`, `NatsBus`, `FakeOracle`).
- Events are past tense (`OrderAccepted`, `Reserved`); commands are imperative (`Submit`,
  `Cancel`, `Tick`).
- Type names are not prefixed by their module: `order::OrderId` is fine
  (`module_name_repetitions` is allowed on purpose).


## 7. Types and conversions

- Newtypes over primitives for every identifier and unit; primitives **MUST NOT** be passed
  where a newtype exists.
- Infallible conversions implement `From`; fallible ones implement `TryFrom`. Custom
  `from_*`/`parse_*` methods **SHOULD NOT** exist when a trait fits.
- Transport DTOs (`protocol`) and domain types are different types; convert once at the
  boundary (`protocol::convert`).
- Public enums that will grow are `#[non_exhaustive]`; matches over domain enums **SHOULD** be
  exhaustive (`wildcard_enum_match_arm` warns).
- Derive `Debug`, `Clone`, `PartialEq`, `Eq`, `Hash` and `serde` on value objects; `Copy` when
  the type is small and it is meaningful.


## 8. Tests and benches

- Unit tests live in a `#[cfg(test)] mod tests` at the bottom of the file; integration tests in
  `crates/<crate>/tests/`; benches in `crates/<crate>/benches/` with `harness = false`.
- Test names state the property: `remove_keeps_fifo_order_and_drops_empty_levels`, not
  `test_remove`.
- Invariants **MUST** be property tests (`proptest`); worked examples from `docs/DOMAIN.md`
  **MUST** be pinned as unit tests.
- `assert!` calls **MUST** carry a message (`clippy::missing_assert_message`).
- Tests **MAY** use `unwrap`/`expect`/indexing (allowed by the preamble); they **MUST NOT** use
  floats or unchecked arithmetic in finance crates.
- Fixtures and strategies come from `testkit`, not copies.


## 9. Code documentation

All `pub` items **MUST** be documented (`missing_docs = deny`); `pub(crate)` and private items
**SHOULD** be when non-trivial. Documentation says what the code cannot: semantics,
preconditions, units, rounding direction, ordering guarantees. It does not restate the
signature.

### 9.1. Functions and macros start with a verb

A **third-person singular verb** ("Computes", "Reserves", "Returns") **SHOULD** be the first
word of the first sentence.

```rust
/// Computes the notional value of `qty` at `price` in quote minor units.
pub fn notional(&self, price: Price, qty: Qty) -> Result<Amount, ArithmeticError>
```

Wrong: "Computation of…", "Compute the…".

### 9.2. Other items start with a noun

Types, traits, modules, fields and variants **SHOULD** start with a noun phrase, no article.

```rust
/// Quantity in integer lots of the instrument's lot size.
pub struct Qty(u64);
```

Wrong: "Represents a quantity…", "A quantity…".

### 9.3. Linking to items by name

Code items **MUST** be referenced as intra-doc links with backticks: ``[`Price`]``,
``[`Instrument::notional`]``. Bare names, plain backticks and links without backticks are wrong.
`rustdoc::broken_intra_doc_links = deny`.

### 9.4. Examples

Non-obvious APIs and every macro **SHOULD** have an `# Examples` section with `rust`-tagged
blocks that compile as doctests.

### 9.5. Panics

A function that can panic on some input **MUST** document the condition under `# Panics`. A
function that calls panicking code but cannot panic **MUST NOT** have the section; use
`#[expect(clippy::missing_panics_doc, reason = "...")]`.

### 9.6. Errors

A function returning `Result` **MUST** have an `# Errors` section: either the conditions, or
"See [`ErrorType`]" when the error type documents them. Do not list them twice.

### 9.7. `README.md` is crate-level documentation

The crate's `README.md` **MUST** be its crate-level documentation via
`#![doc = include_str!("../README.md")]`, so it is checked by rustdoc and rendered by
docs. Write it as rustdoc markdown (intra-doc links work).


## 10. Comments

Comments explain what the code cannot: why, a constraint, a non-obvious consequence, a link to
the rule in `docs/DOMAIN.md` or an ADR. They **MUST NOT** narrate the code, justify themselves
or reference other codebases the reader has no access to. Short, direct, no filler.


[RFC 2119]: https://datatracker.ietf.org/doc/html/rfc2119
