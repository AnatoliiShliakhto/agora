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

pub mod error;
pub mod event;
pub mod fees;
pub mod ids;
pub mod instrument;
pub mod money;
pub mod order;
pub mod registry;
pub mod session;
pub mod time;
pub mod validation;
