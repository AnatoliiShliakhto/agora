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
#![deny(clippy::arithmetic_side_effects, reason = "finance code uses checked arithmetic only")]

pub mod book;
pub mod engine;
