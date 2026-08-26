# agora-adapters

Ports (traits) for everything outside the process, plus adapters and test doubles.

| Port                | Purpose                                                   | Adapters                |
|---------------------|-----------------------------------------------------------|-------------------------|
| `LiquidityProvider` | external liquidity when the local book is thin            | `fake` (scripted), HTTP |
| `PriceOracle`       | reference price for the deviation guard and stop triggers | `fake`, bus-fed         |
| `Macroeconomy`      | MOD-08: mint/burn, rates, taxes                           | `fake`, bus-fed         |

Every port has a `mockall` mock (feature `mocks`) and a fault-injecting fake (latency, errors,
partial answers) used by the chaos tests. Calls are wrapped with the `agora-resilience`
breaker + timeout + backoff by the `guarded` decorator.
