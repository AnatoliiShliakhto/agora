# agora-ledger

Ledger aggregate: accounts, per-asset balances split into `available` and `reserved`, escrow
(reserve → commit/release) and double-entry postings. Pure and deterministic like the matching
engine; the runtime sequences it in the same shard pipeline so a trade and its settlement are
one atomic event batch (ADR-0004).

Invariants enforced by tests:

- `available + reserved == total` for every balance;
- the sum of all postings for one asset is zero (money is neither created nor destroyed, except
  through explicit `mint`/`burn` commands owned by the macroeconomy module);
- a reservation can be committed or released at most once per `CommandId` (idempotency).
