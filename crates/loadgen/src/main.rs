#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

use core::time::Duration;

use clap::Parser;

/// Command-line options.
#[derive(Debug, Parser)]
#[command(name = "agora-loadgen", version, about)]
struct Opts {
    /// Server base URL.
    #[arg(long, env = "AGORA_URL", default_value = "http://127.0.0.1:8080")]
    url: String,

    /// Target arrival rate, orders per second.
    #[arg(long, default_value_t = 1_000)]
    rate: u32,

    /// Test duration, e.g. `30s`.
    #[arg(long, default_value = "10s", value_parser = humantime::parse_duration)]
    duration: Duration,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opts = Opts::parse();
    agora_telemetry::init(agora_telemetry::Format::Pretty)?;
    let client = agora_client::Client::new(&opts.url);
    client.healthz().await?;
    tracing::info!(url = %opts.url, rate = opts.rate, duration = ?opts.duration, "server reachable; generator is Phase 12");
    Ok(())
}
