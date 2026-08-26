# agora-domain

Pure domain model of the exchange. No I/O, no async, no allocation on the hot path beyond what the
types themselves own. Every other crate depends on this one; this one depends on nothing but `serde`.

Contents:

- `money` — fixed-point [`Price`](money::Price) (ticks), [`Qty`](money::Qty) (lots) and
  [`Amount`](money::Amount) (minor units). Floats are forbidden; every operation is checked and
  every division names its [`RoundingMode`](money::RoundingMode).
- `fees` — [`FeeBps`](fees::FeeBps) rates capped at 100 % and the maker/taker
  [`FeeSchedule`](fees::FeeSchedule); fees round up (ADR-0010).
- `ids` — strongly typed identifiers.
- `instrument` — tradable instrument spec and notional calculation with explicit rounding.
- `order` — order model: side, type, time-in-force, self-trade prevention, lifecycle status.
- `registry` — validated [`Symbol`](registry::Symbol) and the
  [`InstrumentRegistry`](registry::InstrumentRegistry) (`id ↔ symbol`, iteration in id order).
- `session` — [`SessionState`](session::SessionState): which commands each state accepts and
  the allowed transitions.
- `time` — [`LogicalTime`](time::LogicalTime): the only clock the domain sees.
- `validation` — [`ValidOrder`](validation::ValidOrder), the parsed-not-validated order the
  matching engine accepts, and the fixed order of the checks that produce it.
- `event` — domain events (`BookEvent`, `LedgerEvent`, `ContractEvent`), their `SchemaId` and
  `SchemaVersion`, and the rules for evolving them; snapshot tests pin the byte encoding.
- `error` — arithmetic and rejection errors.

The crate compiles with `clippy::arithmetic_side_effects = deny`: use `checked_*` and return
[`ArithmeticError`](error::ArithmeticError) instead of wrapping or panicking.
