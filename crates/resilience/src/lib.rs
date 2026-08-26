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

pub mod backoff;
pub mod breaker;
pub mod bulkhead;
pub mod degrade;
pub mod limiter;
pub mod shed;
