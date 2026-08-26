//! Escrow: two-phase reservation of funds backing a command.
//!
//! `reserve` moves `amount` from `available` to `reserved`; `commit` consumes the reservation
//! into a posting; `release` returns the unused part. Every operation is keyed by the
//! [`CommandId`] so retries are idempotent (ADR-0004).

use agora_domain::error::RejectReason;
use agora_domain::event::LedgerEvent;
use agora_domain::ids::{AccountId, AssetId, CommandId};
use agora_domain::money::Amount;

/// Escrow port used by the shard pipeline before an order reaches the matching engine.
pub trait Escrow {
    /// Reserves `amount` of `asset` on `account` under `command`.
    ///
    /// # Errors
    ///
    /// [`RejectReason::InsufficientFunds`] if `available < amount`,
    /// [`RejectReason::Duplicate`] if `command` already has a reservation.
    fn reserve(
        &mut self,
        account: AccountId,
        asset: AssetId,
        command: CommandId,
        amount: Amount,
    ) -> Result<LedgerEvent, RejectReason>;

    /// Releases up to `amount` of the reservation back to `available`.
    ///
    /// # Errors
    ///
    /// [`RejectReason::Duplicate`] if the reservation does not exist.
    fn release(
        &mut self,
        account: AccountId,
        command: CommandId,
        amount: Amount,
    ) -> Result<LedgerEvent, RejectReason>;

    /// Consumes `amount` of the reservation; the caller posts the matching double entry.
    ///
    /// # Errors
    ///
    /// [`RejectReason::Duplicate`] if the reservation does not exist,
    /// [`RejectReason::InsufficientFunds`] if it is smaller than `amount`.
    fn commit(
        &mut self,
        account: AccountId,
        command: CommandId,
        amount: Amount,
    ) -> Result<LedgerEvent, RejectReason>;
}
