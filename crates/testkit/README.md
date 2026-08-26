# agora-testkit

Shared test infrastructure. Never a dependency of non-test code.

- `clock` — manually advanced logical clock;
- `strategies` — `proptest` generators for domain values that respect tick/lot invariants;
- `fixtures` — canonical instruments and accounts (`eurusd()`, `alice()`, `bob()`);
- `reference` — naive, obviously-correct matcher used as the oracle in model-based tests
  (Phase 2).
