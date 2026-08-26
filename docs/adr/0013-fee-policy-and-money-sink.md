# 0013. Fees are the economy's money sink: configurable burn/treasury split, rates set by MOD-08

Date: 2026-08-26
Status: accepted

## Context

A closed game economy accumulates currency: quests, drops and NPC purchases mint money faster
than players destroy it. Without a sink, prices inflate until the currency is meaningless. The
exchange is the natural place to drain it, because every trade passes through it.

ADR-0010 already decided *what* the exchange earns (maker/taker fees, plus the spread when it
makes markets). This decides *where that money goes* and *who sets the rates*.

## Decision

- A collected fee is split by a per-instrument **`FeeDestination { burn_bps, treasury }`**:
  `burn_bps` of the fee is **destroyed** — it leaves the money supply, and MOD-08 sees the
  supply shrink — and the remainder is credited to the `treasury` account, from which the game
  can fund rewards, events and NPC liquidity.
  Both ends are reachable: `burn_bps = 10_000` is a pure sink, `burn_bps = 0` is a pure
  treasury. The split is a game-design knob, not a code change.
- Burning is expressed as an ordinary ledger event (`Burned`), so the money supply stays
  derivable from the log alone.
- **Rates are not constants.** `FeeSchedule` is instrument state, changed by a `SetFeeSchedule`
  command that only MOD-08 may issue (ADR-0011: authorisation before the fact). The change is an
  event, so replay reproduces every historical rate exactly.
- The fee charged for a trade is the one in force **at the moment of matching**, not at the
  moment the order was submitted.
- To keep that safe for escrow, each instrument also carries a **`max_fee_bps` ceiling**.
  Reservations are sized with the ceiling (P3.4 already over-reserves), the actual fee is charged
  at the current rate, and the difference is released. A rate change therefore never invalidates
  a resting order, and raising the ceiling is a deliberate, separate act.

## Consequences

The game gets a dial with a clear economic meaning — "how much liquidity per trade leaves the
economy" — and macroeconomics can react to inflation by raising fees, all without a deployment.
Money conservation stays provable: every asset's postings sum to zero once burns are counted as
a leg, which the conservation proptest must include.

The cost is that a fee is no longer a pure function of the trade: reconstructing it needs the
rate in force at that sequence number, so the ledger applies fees from the instrument log rather
than recomputing them later (ADR-0009 already works that way).

Refines ADR-0010.

## Alternatives considered

- **Fixed fees**: no regulation lever; inflation would need a separate tax mechanism doing the
  same job worse.
- **Burning everything**: simple, but throws away a funded treasury the designers can use for
  rewards and market-making.
- **Applying a new rate to resting orders**: would let a rate change break an existing
  reservation, turning a policy decision into a rejected order.
