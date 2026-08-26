# 0008. Testing strategy

Date: 2026-08-26
Status: accepted

## Context

Correctness of matching and money is the product. Price-time priority must be proven, not
claimed.

## Decision

Layered proofs, cheapest first:

1. property tests (`proptest`) for invariants on domain types and aggregates;
2. model-based tests: the fast engine against a naive reference matcher in `testkit`;
3. snapshot tests (`insta`) for every wire and storage format;
4. `loom`/Miri for lock-free code;
5. integration tests through the real router (`tower::oneshot`) and memory adapters;
6. contract test suites shared by every adapter of a port;
7. chaos tests with `fail` points and fault-injecting fakes;
8. deterministic simulation (seeded, `turmoil`) for whole-system runs;
9. fuzzing of the command stream;
10. open-loop load tests with HDR histograms.

Mocks (`mockall`) are for interaction tests only; behaviour is tested with fakes. Benchmarks are
criterion with committed baselines and a regression gate.

## Consequences

Test code will be a large share of the codebase; that is intended. Every task in `docs/PLAN.md`
names which layers it must add to.
