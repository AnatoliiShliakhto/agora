# Architecture decision records

One file per decision, numbered, never edited after acceptance except to change status.
Superseding decisions link back. Template: `0000-template.md`.

| ADR                                             | Title                                                                                   | Status   |
|-------------------------------------------------|-----------------------------------------------------------------------------------------|----------|
| [0001](0001-record-architecture-decisions.md)   | Record architecture decisions                                                           | accepted |
| [0002](0002-fixed-point-money.md)               | Fixed-point integer money, no floats                                                    | accepted |
| [0003](0003-single-writer-shards-on-actix.md)   | Single-writer shards on actix arbiters                                                  | accepted |
| [0004](0004-event-sourcing-and-escrow-sagas.md) | Event sourcing per shard; escrow sagas across shards                                    | accepted |
| [0005](0005-nats-bus-with-qos-split.md)         | NATS JetStream bus with trading / market-data QoS split                                 | accepted |
| [0006](0006-hexagonal-workspace-layout.md)      | Hexagonal workspace layout                                                              | accepted |
| [0007](0007-self-healing-policy.md)             | Self-healing: unwind, supervise, degrade                                                | accepted |
| [0008](0008-testing-strategy.md)                | Testing strategy                                                                        | accepted |
| [0009](0009-ledger-single-writer-actor.md)      | Ledger is a separate single-writer actor                                                | accepted |
| [0010](0010-taker-price-and-exchange-income.md) | Trades execute at the maker's price; exchange income is fees plus market making         | accepted |
| [0011](0011-sandbox-integration-contract.md)    | Sandbox integration contract: opaque external identifiers, eventual settlement delivery | accepted |
| [0012](0012-instrument-worker-pool.md)          | Instruments are assigned to a fixed worker pool, not to one thread each                | accepted |
| [0013](0013-fee-policy-and-money-sink.md)       | Fees are the economy's money sink: burn/treasury split, rates set by MOD-08            | accepted |
