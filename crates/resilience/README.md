# agora-resilience

Runtime-agnostic resilience primitives shared by the server, the adapters and the client SDK.

- `limiter` — token bucket / GCRA admission control per key.
- `breaker` — circuit breaker with half-open probing.
- `backoff` — exponential backoff with full and decorrelated jitter.
- `bulkhead` — bounded concurrency per dependency.
- `shed` — load shedding by queue depth and deadline.
- `degrade` — degradation ladder: `Normal → Throttled → CancelOnly → MarketDataOnly → Halted`.

Lock-free where it matters (atomics, no mutex on the request path); the `loom` tests under
`tests/` model the interleavings.
