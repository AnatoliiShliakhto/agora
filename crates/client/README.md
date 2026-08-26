# agora-client

Reference client for the v1 API, used by the load generator, the smart-contract runner and
external modules of the sandbox.

Every call goes through a policy chain configured per method:
deadline → circuit breaker → rate limiter → retry with decorrelated jitter (idempotent calls
only) → transport. On sustained failure the client steps down a `DegradationStrategy`
(e.g. stop placing, keep cancelling; switch to cached market data) instead of hammering the
server.
