# agora-telemetry

`init` installs the tracing subscriber (JSON in production, pretty in dev, `RUST_LOG` filter)
and the Prometheus exporter. `names` is the single list of metric names so dashboards and code
cannot drift. `latency` wraps `hdrhistogram` for p50/p99/p99.9 of the hot path without
allocating per sample.

The only crate allowed to do float arithmetic (percentiles), via a crate-level `expect`.
