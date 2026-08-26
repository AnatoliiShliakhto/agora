//! Liveness and readiness.

use axum::http::StatusCode;

/// Liveness: the process is running.
pub async fn healthz() -> StatusCode {
    StatusCode::OK
}

/// Readiness: shards are up and the degradation level permits trading (Phase 7 wires the
/// runtime health in).
pub async fn readyz() -> StatusCode {
    StatusCode::SERVICE_UNAVAILABLE
}
