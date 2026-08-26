# 0012. Instruments are assigned to a fixed worker pool, not to one thread each

Date: 2026-08-26
Status: accepted

## Context

ADR-0003 gives every instrument its own actor on its own arbiter thread. That is correct while
instruments are counted in dozens. The game economy, however, lets players trade **any item**,
so the expected order of magnitude is hundreds to thousands of order books — against a machine
with tens of cores. One thread per book would mean thousands of threads, most of them idle,
context-switching against each other and destroying the cache locality the design exists for.

## Decision

- The runtime owns a **fixed pool of worker threads**, sized from the available cores (minus the
  I/O runtime and the ledger actor), not from the instrument count.
- Every instrument is assigned to exactly one worker, deterministically, when it is registered:
  `worker = hash(InstrumentId) % workers`. The assignment is recorded as an event, so replay and
  restart reproduce it exactly.
- A worker owns many order books but processes **one command at a time**, in sequence order.
  Each book therefore still has exactly one writer and no lock, which is the property ADR-0003
  was protecting — the pool changes how writers are scheduled, not how many there are per book.
- Each book keeps its **own event log and its own sequence**; a worker is a scheduling unit, not
  a consistency unit. Two books on one worker never share state.
- Fairness inside a worker is round-robin over its non-empty command queues, so one hot
  instrument cannot starve the rest.
- A panic inside one book's command is caught at the worker boundary: only that book is halted
  and restarted from its snapshot, the worker keeps serving its other books.

## Consequences

Thread count is bounded by hardware, and a book costs memory, not a thread — so the number of
tradable items becomes a data question rather than an architecture question. Latency for a book
now depends on its worker's neighbours, which makes fairness and per-book queue depth things we
must measure (a hot instrument next to a quiet one is fine; two hot ones on one worker are not).
Rebalancing a live assignment is deliberately out of scope for now: it needs a quiescence
protocol, and the deterministic hash makes the cold-start distribution good enough.

Refines ADR-0003; the single-writer rule and the ban on locks in the command path are unchanged.

## Alternatives considered

- **One thread per instrument** (ADR-0003 as written): does not survive a thousand books.
- **Work stealing over a shared queue**: breaks single-writer and determinism — two threads could
  touch one book, and the event order would depend on scheduling.
- **Sharding by hash of instrument into fewer, coarser books**: would put unrelated items in one
  book; the book *is* the market, so it cannot be merged.
