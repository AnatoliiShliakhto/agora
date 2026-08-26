# Domain: ubiquitous language and matching rules

The single reference for what the words mean and how orders execute. Code uses these names
verbatim. When a rule here changes, the tests in `matching` and `ledger` change with it.

- [Glossary](#glossary)
- [Units and arithmetic](#units-and-arithmetic)
- [Matching rules](#matching-rules)
- [Escrow protocol](#escrow-protocol)
- [Worked examples](#worked-examples)
- [Sandbox identifiers](#sandbox-identifiers)

## Glossary

| Term                             | Meaning                                                                                                                                                  |
|----------------------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------|
| **Asset**                        | Something owned: a currency, a share, a commodity. Balances are per asset.                                                                               |
| **Instrument**                   | A tradable pair `base/quote` with a tick size and a lot size. `EURUSD` trades EUR (base) priced in USD (quote).                                          |
| **Tick**                         | Smallest price increment. Prices are integer multiples of it.                                                                                            |
| **Lot**                          | Smallest quantity increment. Quantities are integer multiples of it.                                                                                     |
| **Order**                        | Intent to buy or sell a quantity of base at some price condition. `BuyOrder`/`SellOrder` in the sandbox are the two sides of one `Order` type.           |
| **Side**                         | `Buy` (bid) or `Sell` (ask).                                                                                                                             |
| **Limit order**                  | Executes at the limit price or better; remainder may rest.                                                                                               |
| **Market order**                 | Executes against whatever liquidity exists, bounded by the deviation guard; never rests.                                                                 |
| **Stop order**                   | Dormant until the last trade price crosses the trigger; then becomes a market (`StopMarket`) or limit (`StopLimit`) order.                               |
| **Time-in-force (TIF)**          | `GTC` rest until cancelled; `IOC` fill what is possible now, cancel the rest; `FOK` fill all now or nothing; `GTD` rest until a logical time.            |
| **Resting order**                | An order on the book waiting for a counterparty; the **maker**.                                                                                          |
| **Aggressor**                    | The incoming order that trades against resting orders; the **taker**.                                                                                    |
| **Fill**                         | One execution of part or all of an order. A **trade** is a fill seen from both sides.                                                                    |
| **Partial fill**                 | An order filled for less than its quantity.                                                                                                              |
| **Price-time priority**          | Better price first; at the same price, earlier acceptance first (FIFO).                                                                                  |
| **Book**                         | All resting orders of one instrument: bids sorted descending, asks ascending.                                                                            |
| **Best bid / best ask / spread** | Highest bid, lowest ask, their difference. A book where best bid ≥ best ask is **crossed** and must never exist after a command.                         |
| **L2 / L3**                      | Aggregated quantity per price level / individual orders per level.                                                                                       |
| **Self-trade prevention (STP)**  | Rule preventing one account from trading with itself. Modes: `CancelNewest`, `CancelOldest`, `CancelBoth`, `DecrementAndCancel`.                         |
| **Deviation guard**              | Maximum distance in ticks from the reference price at which a market order (or stop activation) may execute. Protects against slippage and manipulation. |
| **Reference price**              | Last trade price; if none, mid price; if none, the oracle.                                                                                               |
| **Fee**                          | Maker and taker fees in basis points of notional, rounded up, credited to the exchange account (ADR-0010).                                               |
| **Market making**                | The exchange's own LP account resting bids and asks (liquidity backstop); the spread it earns is ordinary matching income, not a special rule.           |
| **Money sink**                   | Mechanism that removes currency from the economy. Here: the burned share of every fee.                                                                   |
| **Treasury**                     | Exchange-owned account that keeps the non-burned share of fees; funds rewards, events and market making.                                                 |
| **Notional**                     | `price × quantity` in quote minor units, after the instrument scale.                                                                                     |
| **Account**                      | Owner of balances and orders. The **exchange account** receives fees.                                                                                    |
| **Balance**                      | Per asset: `available` (free) and `reserved` (locked by escrow); `total = available + reserved`.                                                         |
| **Escrow**                       | Two-phase lock on funds backing a command: `reserve` → `commit` \| `release`.                                                                            |
| **Posting**                      | Balanced set of debit/credit legs; per asset they sum to zero.                                                                                           |
| **Settlement**                   | Turning a trade into postings and consuming reservations.                                                                                                |
| **Event**                        | Immutable fact emitted by an aggregate; the only thing persisted.                                                                                        |
| **Shard**                        | One instrument's engine, ledger slice and log, owned by one thread.                                                                                      |
| **Seq**                          | Position of an event in a shard log.                                                                                                                     |
| **CommandId**                    | Client idempotency key; the same id never executes twice.                                                                                                |
| **Logical time**                 | Monotonic time fed to the shard as an input; the only clock the domain sees.                                                                             |
| **Session state**                | `PreOpen`, `Open`, `Halted`, `Closed`; decides which commands are accepted (matrix below).                                                               |
| **Symbol**                       | Human-readable instrument name, `1..=16` characters of `A-Z`, `0-9`, `_` (`EURUSD`, `BTC_USD`).                                                          |
| **External id**                  | Identifier minted by another sandbox module (account, asset). Opaque: 1..=64 characters of `A-Za-z0-9._:-`, never parsed by the engine.                  |
| **Price band**                   | Inclusive `min..=max` range of prices an instrument accepts; a spot pair starts at one tick.                                                             |
| **Valid order**                  | Order that passed validation. The engine accepts nothing else, so an unchecked order cannot reach the book.                                              |
| **Degradation level**            | `Normal`, `Throttled`, `CancelOnly`, `MarketDataOnly`, `Halted`.                                                                                         |
| **LoanAgreement**                | Contract: lender, borrower, principal, rate, schedule; collects instalments through escrow, liquidates collateral on default.                            |
| **Dividend**                     | Contract: pro-rata payout of a cash asset to holders of a share asset at a record time.                                                                  |

## Units and arithmetic

- `Price = i64` ticks, `Qty = u64` lots, `Amount = i128` minor units (ADR-0002).
- `notional = ticks × lots × scale.num / scale.den`, rounded **toward negative infinity** by
  default. Rounding direction is always explicit in code (`RoundingMode`) and always favours
  the exchange when money is owed to a participant.
- Rounding modes: `Floor` (paying out to a participant), `Ceil` (charging a participant),
  `HalfEven` (reporting, pro-rata splits). `RoundingMode` is a parameter, never a default.
- Fees: `fee = notional × bps / 10_000`, `Ceil`. A rate is at most `10_000` bps (100 %).
- No floats anywhere in the domain. Decimal strings are parsed once at the API boundary into
  ticks/lots; misaligned values are rejected (`InvalidPrice`, `InvalidQty`), never rounded.

## Matching rules

1. **Validation** happens before anything else; a rejected order emits `OrderRejected` and
   touches nothing.
2. **Escrow** reserves the full worst-case cost of the order before it reaches the book. If the
   reservation fails the order is rejected with `InsufficientFunds`.
3. **Stop orders** are parked until triggered; triggering happens after each command that
   produced trades, in `(trigger price, accepted seq)` order, and each activated order runs the
   full pipeline as if newly submitted (but keeps its original `OrderId`).
4. **FOK** first computes fillable quantity at acceptable prices without mutating the book; if
   it is less than the order quantity the order is rejected with `FokUnfillable`, and nothing is
   reserved on the book.
5. **Fill loop**: walk the opposite side best price first; within a level FIFO. For each resting
   order:
   1. if both accounts are equal, apply the STP mode;
   2. fill `min(remaining, resting)` at the **resting order's price**; price improvement goes
      to the taker (ADR-0010);
   3. emit `Trade { price, qty, maker_fee, taker_fee }`;
   4. stop when the incoming order is filled, the next level is worse than the limit, or (market
      orders) the next level exceeds the deviation guard.
6. **Remainder**: `GTC`/`GTD` rests at its limit (`OrderAccepted` then `Open`/`PartiallyFilled`);
   `IOC` and market orders cancel the remainder (`OrderDone(Cancelled)`) and release escrow.
7. **Book is never crossed** after a command. **Quantity is conserved**: taker filled quantity
   equals the sum of maker filled quantities.
8. **Amend**: reducing quantity keeps priority; any other change re-queues the order.
9. **Expiry**: on `Tick(t)`, every `GTD` order with `expires_at ≤ t` emits `OrderDone(Expired)`
   and releases escrow.
10. **Determinism**: the same command sequence yields the same event sequence, byte for byte.

## Session states

| State     | limit | market | stop | cancel | amend | matching |
|-----------|:-----:|:------:|:----:|:------:|:-----:|:--------:|
| `PreOpen` |  ✓   |   ✗   |  ✓  |   ✓   |  ✓   |    ✗    |
| `Open`    |  ✓   |   ✓   |  ✓  |   ✓   |  ✓   |    ✓    |
| `Halted`  |  ✗   |   ✗   |  ✗  |   ✓   |  ✗   |    ✗    |
| `Closed`  |  ✗   |   ✗   |  ✗  |   ✓   |  ✗   |    ✗    |

Rejected commands get `NotTrading { state }`. Cancels are always accepted: a participant must
always be able to pull an order. Transitions: `PreOpen → Open`, `Open ⇄ Halted`,
`Open | Halted → Closed`, `Closed → PreOpen`. `Halted` is abnormal (circuit breaker, operator);
`Closed` is scheduled; resting GTC orders survive both.

## Order validation

Validation is stateless: it needs only the order, its instrument, the session state and the
current logical time. Checks run in a fixed order, so the same command always yields the same
rejection:

| # | Check                                            | Rejection                 |
|---|--------------------------------------------------|---------------------------|
| 1 | session accepts this kind of command             | `NotTrading { state }`    |
| 2 | order targets this instrument                    | `UnknownInstrument`       |
| 3 | quantity is non-zero                             | `InvalidQty`              |
| 4 | every price lies in the instrument's band        | `InvalidPrice`            |
| 5 | stop-limit's limit is reachable from its trigger | `InconsistentStopLimit`   |
| 6 | time-in-force suits the order type               | `IncompatibleTimeInForce` |
| 7 | GTD expiry is strictly in the future             | `ExpiryInThePast`         |
| 8 | worst-case notional does not overflow            | `Arithmetic`              |

Rules behind the less obvious ones:

- **Reachable stop-limit**: after a buy stop triggers, the market has moved *up* through the
  trigger, so a limit below the trigger could never fill; the mirrored rule holds for sells.
  `limit == trigger` is allowed.
- **Time-in-force**: a market order never rests, so `GTC` and `GTD` are refused on it; `IOC`
  and `FOK` are the only valid choices. Every time-in-force is valid for limit and stop orders.
- **Expiry**: `expires_at == now` is already in the past — the order would expire on the same
  tick that accepted it.
- **Notional**: computed at the worst-case price with `Ceil`, so an order whose escrow
  reservation could not be represented is refused before it reaches the book. Market and
  stop-market orders carry no price; the deviation guard bounds them at execution time.

Everything that needs more than the order itself — funds, self-trade prevention, fillability,
deviation from the reference price, duplicate command ids — is checked later in the shard
pipeline.

## Fees and the money sink

Fees are what the exchange earns (ADR-0010) **and** the economy's main drain (ADR-0013). Every
trade charges both sides at the schedule in force **at matching time**:

```text
maker_fee = ceil(notional × maker_bps / 10_000)
taker_fee = ceil(notional × taker_bps / 10_000)

for each fee:
    burned   = ceil(fee × burn_bps / 10_000)   → destroyed, leaves the money supply
    treasury = fee − burned                    → credited to the treasury account
```

- `burn_bps = 10_000` makes the fee a pure sink; `burn_bps = 0` makes it pure treasury income.
  Everything in between is a game-design decision, not a code change.
- Burning is a ledger event (`Burned`), so the money supply is derivable from the log alone and
  per-asset conservation holds once burns count as a leg.
- Rates are **instrument state**, not constants: MOD-08 changes them with `SetFeeSchedule` when
  it needs to answer inflation. The change is an event, so replay reproduces every historical
  rate.
- Each instrument also has a `max_fee_bps` **ceiling**. Escrow reserves at the ceiling and
  releases the difference after the fill, so raising a rate can never invalidate an order that
  is already resting.

## Escrow protocol

```text
reserve(account, asset, command, amount)   available -= amount; reserved += amount
commit(account, command, amount)           reserved  -= amount; (posting moves it elsewhere)
release(account, command, amount)          reserved  -= amount; available += amount
```

All three are idempotent per `command`: a repeated `commit` or `release` for the same amount is
a no-op, a different amount is `Duplicate`. Reservation sizes:

| Order       | Reserves                                                                |
|-------------|-------------------------------------------------------------------------|
| limit buy   | `notional(limit, qty) + taker fee at limit` in quote                    |
| limit sell  | `qty` in base                                                           |
| market buy  | `notional(ref × (1 + deviation), qty) + fee`, over-reserve then release |
| market sell | `qty` in base                                                           |
| stop        | as the order it becomes, at trigger time                                |

## Worked examples

### Partial fill across levels (from the project brief)

Book (asks): `500_000 @ 1.1200`, `2_000_000 @ 1.1198`, `100_000 @ 1.1197`.
Incoming: buy `10_000_000 @ 1.1200`, GTC.

Fill loop walks asks ascending: `100_000 @ 1.1197`, `2_000_000 @ 1.1198`, `500_000 @ 1.1200`.
Result: `2_600_000` filled in three trades at the makers' prices; `7_400_000` rests at `1.1200`
as the new best bid; the three makers are fully filled.

The buyer pays the makers' prices (`1.1197`, `1.1198`, `1.1200`), not its own limit: total
notional `100_000 × 1.1197 + 2_000_000 × 1.1198 + 500_000 × 1.1200 = 2_911_570 USD` instead of
`2_912_000 USD` at the limit. With a taker fee of `10 bps` the buyer is charged
`ceil(2_911_570 × 0.001) = 2_911.57 USD` → `291_157` cents; each maker pays its maker fee on its
own fill.

### Rounding

`7 bps` on `12_345` cents is `8.6415` cents: `Ceil → 9` (what is charged), `Floor → 8` (what
would be paid out), `HalfEven → 9`. `7 bps` on `-12_345` cents (a refund) is `-8.6415`:
`Ceil → -8`, `Floor → -9`, `HalfEven → -9`. Ties go to even: `Amount(5) / 2 = 2.5 → 2`,
`Amount(7) / 2 = 3.5 → 4`.

### FOK that cannot fill

Same book, incoming sell `3_000_000 @ 1.1197` FOK. Fillable at ≥ 1.1197 on the bid side is
whatever bids exist at or above that price; if less than `3_000_000`, reject `FokUnfillable`
with no side effects.

### Self-trade

Alice rests sell `100 @ 10`. Alice sends buy `100 @ 10` with `CancelOldest`: the resting order
is cancelled (`OrderDone(Cancelled)`), the incoming one continues and rests. With
`CancelNewest` the incoming order is rejected with `SelfTrade`.

## Sandbox identifiers

This engine is cluster **CL-02 (Economy & Market Engine)** of the sandbox. The brief's
identifiers map to this codebase as follows.

| Sandbox id                    | Here                                  | Notes                                                                   |
|-------------------------------|---------------------------------------|-------------------------------------------------------------------------|
| CL-02 Economy & Market Engine | this workspace                        | the whole module                                                        |
| COMP-107 `BuyOrder`           | `Order` with `Side::Buy`              | one order type, two sides — a buy and a sell differ only by `Side`      |
| COMP-108 `SellOrder`          | `Order` with `Side::Sell`             |                                                                         |
| SYS-101 ACID consistency      | per-shard atomic event batch + escrow | the append of one command's event batch is the transaction (ADR-0004)   |
| SYS-102 Escrow                | `ledger::escrow`, the ledger actor    | `reserve` → `commit` \| `release`, idempotent by `CommandId` (ADR-0009) |
| SYS-103 Messaging             | `bus`                                 | trading and market-data QoS classes (ADR-0005)                          |
| MOD-08 Macroeconomy           | `adapters::macroeconomy` port         | authorises minting, burning, rates; consumes settlement from the bus    |
| MOD-16 Order books            | `matching`                            | one book and one engine per instrument                                  |
| MOD-25 Smart contracts        | `contracts`                           | rules engine driven by events and scheduler ticks                       |
| COMP-201 `LoanAgreement`      | `contracts::loan`                     | instalments, collection through escrow, default path                    |

### Identifiers across the boundary

Decided in **ADR-0011**, so integration needs no agreement on naming:

- Entities the engine **does not create** — accounts and assets — arrive as an **`ExternalId`**:
  an opaque string of `1..=64` characters from `A-Za-z0-9._:-`, compared byte for byte. ULIDs,
  UUIDs, plain numbers and namespaced names (`player:4211`, `mod08.currency.gold`) are all
  valid; the engine never parses one.
- Each `ExternalId` is mapped **once, on first sight**, to a dense internal identifier
  (`AccountId(u64)`, `AssetId(u32)`), and the mapping is recorded as an event, so it survives
  replay and restart unchanged.
- Entities the engine **does create** — `OrderId`, `TradeId`, `InstrumentId`, `ContractId`,
  `Seq` — are dense integers allocated by the owning shard's sequencer and exposed outward as
  decimal numbers.
- `CommandId` is the client's idempotency key: 128 bits, supplied verbatim (the API accepts a
  ULID in `Idempotency-Key`).

### Delivery guarantees to other modules

Also ADR-0011:

- A trade is confirmed to its client as soon as the event batch is **durable** — never waiting
  on another module. Execution reports, ledger postings and market data follow over the bus.
- Every message carries `(instrument, seq)`. Delivery is at-least-once with **detectable** loss:
  a consumer that sees a gap resyncs from the log, so every consumer-side effect must be
  idempotent by `(instrument, seq)`.
- When MOD-08 needs to *authorise* rather than observe — minting, burning, tax and interest
  rates — it sends a **command into** the engine, which travels the ordinary synchronous path.
  Authorisation happens before the fact; there is no synchronous confirmation after it.
