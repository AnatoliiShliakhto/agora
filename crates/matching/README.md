# agora-matching

Order book and matching engine for one instrument. Pure and deterministic: no clock, no I/O, no
threads. The same command sequence always yields the same event sequence, which is what makes
event-sourced replay and the deterministic simulator possible.

- `book` — price levels with FIFO queues; L2/L3 views.

The engine accepts only `agora_domain::validation::ValidOrder`, so stateless rejections
(quantity, price band, time-in-force, expiry) are already impossible when a command arrives.
- `engine` — the command interpreter: validation, stop triggers, time-in-force, self-trade
  prevention, deviation guard, fills at the maker's price with fees from the instrument's
  `FeeSchedule` (ADR-0010).

Hot-path rules: no allocation per command after warm-up (orders live in a `slotmap` arena),
checked arithmetic only, `arithmetic_side_effects = deny`.
