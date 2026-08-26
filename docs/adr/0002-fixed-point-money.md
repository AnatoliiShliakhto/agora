# 0002. Fixed-point integer money, no floats

Date: 2026-08-26
Status: accepted

## Context

Binary floating point cannot represent decimal prices exactly and accumulates error under
repeated arithmetic; in a ledger that is money created or destroyed. Exchanges represent prices
and quantities as integers in instrument-defined units.

## Decision

- `Price = i64` in ticks, `Qty = u64` in lots, `Amount = i128` in minor units of an asset.
- Each instrument carries a `NotionalScale { num, den }` converting `ticks × lots` to quote
  minor units; division is explicit with a `RoundingMode`.
- `clippy::float_arithmetic`, `float_cmp`, `lossy_float_literal` are `deny` workspace-wide;
  `telemetry` and `loadgen` opt out for percentiles.
- `clippy::arithmetic_side_effects` is `deny` in `domain`, `matching`, `ledger`, `contracts`:
  every operation is `checked_*` and overflow is an `ArithmeticError`.
- Decimals (`rust_decimal`) exist only in `protocol::convert`, and misaligned inputs are
  rejected rather than rounded.

## Consequences

Arithmetic is verbose (`a.checked_add(b)?`) but every overflow is a handled error. `i128`
notional cannot overflow for any realistic instrument. Rounding is a visible design point
(favour the exchange when paying out, round up when charging).

## Alternatives considered

- `rust_decimal` everywhere: 16 bytes, slower, and still needs a rounding policy; kept at the
  boundary only.
- `f64` with epsilon comparisons: rejected, the failure mode is silent.
