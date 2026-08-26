# agora-contracts

Rules engine (MOD-25). A contract is a deterministic state machine driven by two inputs: domain
events from the bus and ticks from the scheduler. It reacts by emitting commands (reserve, post,
place order) that go back through the normal shard pipeline — contracts never touch the ledger
directly.

- `engine` — contract registry and dispatch.
- `scheduler` — logical-time scheduler (time is an event, not a syscall; ADR-0004).
- `loan` — `LoanAgreement` (COMP-201): schedule, accrual, collection, default handling.
- `dividend` — pro-rata cash distribution to holders.
