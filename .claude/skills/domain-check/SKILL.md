---
name: domain-check
description: Checklist for changes in matching, ledger or contracts — invariants, rounding, determinism, required tests
disable-model-invocation: true
---

# Domain check

Run against a diff in `crates/{domain,matching,ledger,contracts}` before opening the PR.

## Semantics

- [ ] Every rule touched is stated in `docs/DOMAIN.md`; if the rule changed, the doc changed in
      the same PR.
- [ ] Time-in-force: GTC/GTD rests, IOC cancels the remainder, FOK pre-walks without mutation
      and rejects in full.
- [ ] Fills execute at the resting order's price; the taker keeps the improvement (ADR-0010).
- [ ] The order reached the engine as a `ValidOrder`; no stateless check is repeated there.
- [ ] STP mode applied before the fill, per resting order; no trade with equal accounts.
- [ ] Deviation guard bounds market orders and stop activations.
- [ ] Stop activation order is `(trigger price, accepted seq)`; activated orders keep their id.

## Invariants (must have a test)

- [ ] Book never crossed after a command.
- [ ] Quantity conserved: taker filled == Σ maker filled.
- [ ] Price-time priority: an order fills only after every older order at a better-or-equal price.
- [ ] `available + reserved == total`; no negative balances; postings sum to zero per asset.
- [ ] Reservation per `CommandId` commits or releases at most once; retries are no-ops.
- [ ] Replay of the event batch reproduces the state (`apply` ≡ `decide` side effects).

## Arithmetic

- [ ] Only `checked_*`; every division names its rounding direction; rounding favours the
      exchange when paying out and rounds up when charging.
- [ ] Fees cannot exceed notional, are charged with `Ceil` and paid out with `Floor`;
      rounding never creates money (Σ legs = 0 per asset).
- [ ] No `as` casts that can truncate money; use `i128::from`/`try_from`.

## Determinism

- [ ] No `Instant`, `SystemTime`, RNG, `HashMap` iteration or thread ids in the aggregate.
- [ ] Ids come from the shard sequence; time comes from `Tick`.
- [ ] Two runs over the same command stream produce identical events (add to the e2e test if new
      command types were introduced).

## Tests to add

- unit: worked example from `docs/DOMAIN.md`;
- property: the invariants above over random streams (`testkit::strategies`);
- model-based: fast engine vs `testkit::reference` if matching changed;
- snapshot: if an event changed, refresh the `schema_encoding_is_frozen` snapshots — and check
  the change against the compatibility table in `crates/eventstore/README.md`: only appending a
  variant at the end is free, everything else needs a new `SchemaVersion` and an upcaster.
