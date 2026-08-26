# 0010. Trades execute at the maker's price; exchange income is fees plus market making

Date: 2026-08-26
Status: accepted (refined by ADR-0013: where the fee goes and who sets the rate)

## Context

The original brief described the exchange pocketing the difference between the taker's limit
and the maker's price ("spread capture"). `docs/PLAN.md` open question 1 asked which policy the
game economy wants.

## Decision

- A trade has **one price: the resting (maker) order's price**. Price improvement goes to the
  taker, as on every real exchange. `PriceImprovementPolicy` and `Trade::taker_price` are
  removed.
- Exchange income comes from two standard, tunable sources:
  1. **fees** — maker and taker rates in basis points of notional, rounded up when charged
     (`RoundingMode::Ceil`), credited to the exchange account; this is the game designers'
     money sink;
  2. **market making** — when the exchange provides liquidity itself (liquidity backstop,
     P9.4), its LP account rests bids and asks and earns the spread through ordinary matching.

## Consequences

Single code path for all order types (spread capture was undefined for market orders); one
price per trade for market data, stop triggers and P&L; two-leg postings plus fee legs. The
economic effect the brief wanted (exchange earns the spread) is preserved wherever the exchange
is the counterparty, and expressed with standard semantics that reviewers recognise.

## Alternatives considered

- `Exchange` policy (taker pays its limit): undefined for market orders, two prices per trade,
  three-leg postings, rewards gaming the limit price; rejected.
- Splitting the improvement between taker and exchange: same complexity for a smaller sink;
  fees do the job with one parameter.
