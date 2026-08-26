# agora-bus

Message bus port (SYS-103) and adapters (ADR-0005).

Two QoS classes with separate channels and back-pressure policies:

- **trading** — commands and executions: lossless, bounded queue, producer blocks/sheds
  when full;
- **market data** — quotes and depth: conflated, slow consumers get the latest state, never a
  backlog.

Adapters: `memory` (broadcast channels, used by tests and the simulator) and `nats` (JetStream,
feature `nats`).
