# 0007. Self-healing: unwind, supervise, degrade

Date: 2026-08-26
Status: accepted

## Context

The engine must keep trading through partial failures and recover without an operator.

## Decision

- `panic = "unwind"` in release. A panic in a shard is caught by the actix supervisor, which
  restarts the shard from snapshot with a jittered exponential backoff and escalates to
  `Halted` after `max_restarts`. Other shards are unaffected.
- A watchdog restarts stalled shards (missed heartbeats).
- A degradation controller moves the exchange (or one shard) along
  `Normal → Throttled → CancelOnly → MarketDataOnly → Halted` based on queue depth, error rate
  and p99 latency versus SLO; the API enforces the level.
- Every outbound call goes through timeout + circuit breaker + bounded retry with jitter;
  clients do the same and carry their own `DegradationStrategy`.
- Nothing is published before it is durable, so a crash can lose only the batch in flight, and
  the caller's retry with the same `CommandId` is idempotent.

## Consequences

`panic = "abort"` is unavailable (it would kill the process); tests that inject panics
(`fail` points) are part of the suite so the recovery path is exercised, not assumed.
