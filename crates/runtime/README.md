# agora-runtime

Actor topology (ADR-0003) and self-healing (ADR-0007).

```text
                       ┌─────────────┐
   API / bus ──cmd──▶  │  Sequencer  │  assigns CommandId order, dedups, routes by instrument
                       └──────┬──────┘
                              │ SPSC ring per shard
                  ┌───────────┼───────────┐
                  ▼           ▼           ▼
             ┌─────────┐ ┌─────────┐ ┌─────────┐
             │ Shard 1 │ │ Shard 2 │ │ Shard N │  one dedicated arbiter thread each:
             │ engine  │ │         │ │         │  reserve (sync) → match → append → publish
             └────┬────┘ └────┬────┘ └────┬────┘
                  │  reserve  │           │  logs
                  ▼           ▼           ▼
             ┌─────────────────────────────────┐
             │ Ledger actor (one partition)    │  balances, escrow, postings; settles from logs
             └─────────────────────────────────┘
                              ▼
                        Supervisor: restart-from-snapshot with jittered backoff, health, watchdog
```

- `shard` — the single-writer pipeline for one instrument.
- `ledger` *(planned, P5.11)* — the single-writer ledger actor (ADR-0009).
- `sequencer` — global command intake and routing.
- `supervisor` — actix supervision, restart policy, replay on restart.
- `health` — liveness/readiness state machine consumed by the API.
- `pipeline` — command → events → side effects, as a pure function where possible.
