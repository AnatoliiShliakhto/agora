# 0005. NATS JetStream bus with trading / market-data QoS split

Date: 2026-08-26
Status: accepted

## Context

SYS-103 needs a broker between this module and the sandbox. Fault tolerance and throughput
pull in opposite directions; a slow market-data consumer must never delay an execution report.

## Decision

- Two QoS classes with separate subjects, queues and policies:
  trading (lossless, ordered, bounded, back-pressured) and market data (conflated, latest wins).
- Bus port in `bus::port`; in-memory adapter for tests and the simulator; NATS JetStream
  adapter behind feature `nats` (persistent streams, `Nats-Msg-Id` dedup, pull consumers).
- Every message carries the shard `Seq`; consumers detect gaps and resync from the log.
- Publish happens only after the event batch is durable.

## Consequences

NATS is light to run locally (one container) and has first-class Rust support. Kafka-class
throughput is not needed inside a game world; if it becomes needed the port isolates the change.

## Alternatives considered

- Kafka/Redpanda: heavier operationally, `rdkafka` needs a C toolchain.
- Redis Streams: fine for prototypes, weaker durability story.
