# agora-api

axum router for the public API.

- `rest` — `/v1/orders`, `/v1/books/{symbol}`, `/v1/accounts/{id}`; idempotency via
  `Idempotency-Key`.
- `ws` — `/v1/stream`: market data (conflated) and private executions (lossless).
- `middleware` — request id, tracing, per-client rate limit, load shedding with `Retry-After`,
  deadline propagation.
- `health` — `/healthz`, `/readyz`, `/metrics`.

The router is a plain function of its dependencies (`AppState`), so integration tests run it
in-process with `tower::ServiceExt::oneshot` and never bind a port.
