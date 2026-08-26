---
name: hot-path-review
description: Performance and lock-freedom review for code on the shard command path (sequencer → shard → append → publish)
disable-model-invocation: true
---

# Hot-path review

Apply to `runtime::{shard,sequencer,pipeline}`, `matching`, `ledger`, `eventstore` append,
`bus` publish, and `resilience` primitives.

## Locks and sharing

- [ ] No `Mutex`/`RwLock` on the path (`parking_lot` included). Shared read-mostly state is
      `arc_swap`; counters are atomics with the weakest correct `Ordering`, justified in a comment.
- [ ] No lock or long borrow held across `.await`; no blocking call on a tokio worker.
- [ ] Cross-thread hand-off is a bounded queue; a full queue is a back-pressure result, never a
      spin or a block without a deadline.
- [ ] Hand-rolled atomics have a `loom` test.

## Allocation and layout

- [ ] Zero allocations per command after warm-up: orders live in the arena, event batches reuse
      a `Vec` buffer, no `format!`/`String` per command, no `Box<dyn>` per command.
- [ ] Hot structs are `#[repr(align(64))]` when shared across threads (false sharing), and
      fields accessed together are adjacent.
- [ ] No `clone()` of orders/levels on the fill loop; iterate by reference or index.
- [ ] `SmallVec`/fixed arrays for bounded small collections (fills per command).

## Algorithms

- [ ] Book operations are O(log levels) for lookup and O(1) for FIFO push/pop and cancel by id.
- [ ] FOK pre-walk is read-only and bounded by the quantity, not the book size.
- [ ] Branches on the fill loop are predictable (sort the common case first; avoid `match` on
      rare enums inside the loop).
- [ ] No per-command tracing spans; counters + periodic histogram snapshots instead.

## Evidence

- [ ] criterion bench exists and was run with `just bench <crate>`; before/after numbers are in
      the PR and in `docs/PERFORMANCE.md`.
- [ ] For structural changes: `perf stat -e cache-misses,branch-misses` on the bench, numbers
      recorded.
- [ ] Allocation counter test (or `dhat`) shows zero allocs per command where claimed.
