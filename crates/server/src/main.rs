#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

use std::net::SocketAddr;

use clap::Parser;

#[global_allocator]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

/// Command-line options.
#[derive(Debug, Parser)]
#[command(name = "agora-server", version, about)]
struct Opts {
    /// Address to bind the HTTP API to.
    #[arg(long, env = "AGORA_BIND", default_value = "127.0.0.1:8080")]
    bind: SocketAddr,

    /// Emit JSON logs.
    #[arg(long, env = "AGORA_LOG_JSON")]
    log_json: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opts = Opts::parse();
    let format =
        if opts.log_json { agora_telemetry::Format::Json } else { agora_telemetry::Format::Pretty };
    agora_telemetry::init(format)?;

    let listener = tokio::net::TcpListener::bind(opts.bind).await?;
    tracing::info!(addr = %opts.bind, "listening");
    axum::serve(listener, agora_api::router()).with_graceful_shutdown(shutdown_signal()).await?;
    Ok(())
}

async fn shutdown_signal() {
    if let Err(err) = tokio::signal::ctrl_c().await {
        tracing::error!(%err, "failed to listen for ctrl-c");
    }
    tracing::info!("shutdown requested");
}
