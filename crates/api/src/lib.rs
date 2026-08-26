#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        reason = "tests may panic"
    )
)]

pub mod health;
pub mod middleware;
pub mod rest;
pub mod ws;

use axum::Router;
use axum::routing::get;

/// Builds the application router.
pub fn router() -> Router {
    Router::new().route("/healthz", get(health::healthz)).route("/readyz", get(health::readyz))
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt as _;

    use super::*;

    #[tokio::test]
    async fn healthz_is_ok() {
        let res = router()
            .oneshot(Request::builder().uri("/healthz").body(Body::empty()).expect("request"))
            .await
            .expect("response");
        assert_eq!(res.status(), StatusCode::OK);
    }
}
