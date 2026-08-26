#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![cfg_attr(
    not(test),
    expect(
        clippy::expect_used,
        reason = "test-support code: fixtures panic on invariant violations"
    )
)]
#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        reason = "tests may panic"
    )
)]

pub mod clock;
pub mod fixtures;
pub mod reference;
pub mod strategies;
