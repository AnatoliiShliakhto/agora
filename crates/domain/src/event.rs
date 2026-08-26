//! Domain events.
//!
//! Events are the only thing persisted (event sourcing, ADR-0004). Every aggregate is a pure
//! function `decide(state, command) -> Result<Vec<Event>, RejectReason>` plus
//! `apply(state, event) -> state`.
//!
//! # Schema evolution
//!
//! Events are encoded with `postcard`, which writes an enum variant as its **index**. That makes
//! the rules mechanical:
//!
//! - **Appending a variant at the end** of an enum is backwards compatible: old payloads still
//!   decode, old readers reject the new variant loudly rather than silently misreading it.
//! - **Inserting, reordering or removing a variant**, or changing a field's type or order,
//!   breaks every stored payload. It requires a new [`SchemaVersion`] and an upcaster in
//!   `agora-eventstore`.
//!
//! The `schema_encoding_is_frozen` snapshot tests in this module pin the byte encoding of every
//! variant, so an accidental break shows up as a failing test instead of a corrupt log.
//!
//! Events carry no timestamp and no sequence number: both live in the event store's envelope,
//! which is the single source of truth for ordering and time (ADR-0004).

use crate::error::RejectReason;
use crate::ids::{AccountId, AssetId, CommandId, ContractId, InstrumentId, OrderId, TradeId};
use crate::money::{Amount, Price, Qty};
use crate::order::{OrderStatus, Side};

/// Family of events that share one schema and evolve on one version line.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    strum::Display,
)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
#[repr(u16)]
pub enum SchemaId {
    /// [`BookEvent`]: order book and matching.
    Book = 1,
    /// [`LedgerEvent`]: balances, escrow and postings.
    Ledger = 2,
    /// [`ContractEvent`]: the smart-contract rules engine.
    Contract = 3,
}

/// Version of one [`SchemaId`]'s encoding.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(transparent)]
pub struct SchemaVersion(u16);

impl SchemaVersion {
    /// Version `v`.
    #[must_use]
    pub const fn new(v: u16) -> Self {
        Self(v)
    }

    /// Raw version number.
    #[must_use]
    pub const fn get(self) -> u16 {
        self.0
    }
}

impl core::fmt::Display for SchemaVersion {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "v{}", self.0)
    }
}

/// Event that the store persists and replays.
///
/// The constants let the store stamp an envelope without knowing the concrete type, and let an
/// upcaster decide whether a stored payload needs migrating.
pub trait DomainEvent: serde::Serialize + serde::de::DeserializeOwned + Sized {
    /// Family this event belongs to.
    const SCHEMA: SchemaId;

    /// Version this build writes.
    const VERSION: SchemaVersion;
}

/// Event emitted by the order book / matching aggregate of one instrument.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    strum::EnumDiscriminants,
)]
#[strum_discriminants(
    name(BookEventKind),
    derive(strum::Display, strum::EnumIter, Hash, Ord, PartialOrd)
)]
#[non_exhaustive]
pub enum BookEvent {
    /// Order passed validation and escrow; it is now the engine's responsibility.
    OrderAccepted {
        /// Assigned order identifier.
        order: OrderId,
        /// Idempotency key of the originating command.
        command: CommandId,
        /// Owner.
        account: AccountId,
    },
    /// Order was rejected.
    OrderRejected {
        /// Idempotency key of the originating command.
        command: CommandId,
        /// Owner.
        account: AccountId,
        /// Why.
        reason: RejectReason,
    },
    /// Two orders traded.
    ///
    /// Execution is always at the maker's price; the taker keeps any price improvement
    /// (ADR-0010).
    Trade {
        /// Trade identifier.
        trade: TradeId,
        /// Aggressor (incoming) order.
        taker: OrderId,
        /// Resting order.
        maker: OrderId,
        /// Aggressor side.
        taker_side: Side,
        /// Execution price: the resting order's price.
        price: Price,
        /// Filled quantity.
        qty: Qty,
        /// Fee charged to the maker, in quote minor units.
        maker_fee: Amount,
        /// Fee charged to the taker, in quote minor units.
        taker_fee: Amount,
    },
    /// Order left the book or the pending set.
    OrderDone {
        /// Order identifier.
        order: OrderId,
        /// Final status: `Filled`, `Cancelled` or `Expired`.
        status: OrderStatus,
        /// Quantity that was never filled.
        unfilled: Qty,
    },
    /// Stop order was triggered and converted into its active form.
    StopTriggered {
        /// Order identifier.
        order: OrderId,
        /// Last trade price that triggered it.
        at: Price,
    },
}

impl DomainEvent for BookEvent {
    const SCHEMA: SchemaId = SchemaId::Book;
    const VERSION: SchemaVersion = SchemaVersion::new(1);
}

/// Event emitted by the ledger aggregate.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    strum::EnumDiscriminants,
)]
#[strum_discriminants(
    name(LedgerEventKind),
    derive(strum::Display, strum::EnumIter, Hash, Ord, PartialOrd)
)]
#[non_exhaustive]
pub enum LedgerEvent {
    /// Funds moved from `available` to `reserved` for a pending command.
    Reserved {
        /// Account.
        account: AccountId,
        /// Asset.
        asset: AssetId,
        /// Reservation key (the command it backs).
        command: CommandId,
        /// Reserved amount.
        amount: Amount,
    },
    /// Reservation released in full or in part back to `available`.
    Released {
        /// Account.
        account: AccountId,
        /// Asset.
        asset: AssetId,
        /// Reservation key.
        command: CommandId,
        /// Released amount.
        amount: Amount,
    },
    /// Reservation consumed by a settlement.
    Committed {
        /// Account.
        account: AccountId,
        /// Asset.
        asset: AssetId,
        /// Reservation key.
        command: CommandId,
        /// Committed amount.
        amount: Amount,
    },
    /// Balanced double-entry posting: `debit` and `credit` sum to zero for `asset`.
    Posted {
        /// Instrument the settlement belongs to, if any.
        instrument: Option<InstrumentId>,
        /// Asset moved.
        asset: AssetId,
        /// Debited account.
        debit: AccountId,
        /// Credited account.
        credit: AccountId,
        /// Amount.
        amount: Amount,
    },
}

impl DomainEvent for LedgerEvent {
    const SCHEMA: SchemaId = SchemaId::Ledger;
    const VERSION: SchemaVersion = SchemaVersion::new(1);
}

/// Outcome of a contract that reached its end.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, strum::Display,
)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum ContractOutcome {
    /// Every obligation was met.
    Fulfilled,
    /// An obligation was not met and the contract's default path ran.
    Defaulted,
    /// Ended early by its parties or by an operator.
    Cancelled,
}

/// Event emitted by the smart-contract rules engine.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    strum::EnumDiscriminants,
)]
#[strum_discriminants(
    name(ContractEventKind),
    derive(strum::Display, strum::EnumIter, Hash, Ord, PartialOrd)
)]
#[non_exhaustive]
pub enum ContractEvent {
    /// Contract instance was created and registered.
    Created {
        /// Instance.
        contract: ContractId,
        /// Account that owes the obligations.
        obligor: AccountId,
        /// Account that receives them.
        beneficiary: AccountId,
    },
    /// An obligation became due and the engine asked the ledger to collect it.
    ObligationDue {
        /// Instance.
        contract: ContractId,
        /// Instalment number, starting at one.
        instalment: u32,
        /// Asset owed.
        asset: AssetId,
        /// Amount owed.
        amount: Amount,
    },
    /// The obligation was collected in full.
    ObligationSettled {
        /// Instance.
        contract: ContractId,
        /// Instalment number.
        instalment: u32,
        /// Amount collected.
        amount: Amount,
    },
    /// The obligor could not pay; the contract's default path runs next.
    ObligationMissed {
        /// Instance.
        contract: ContractId,
        /// Instalment number.
        instalment: u32,
        /// Amount still owed.
        shortfall: Amount,
    },
    /// Contract reached its end.
    Closed {
        /// Instance.
        contract: ContractId,
        /// How it ended.
        outcome: ContractOutcome,
    },
}

impl DomainEvent for ContractEvent {
    const SCHEMA: SchemaId = SchemaId::Contract;
    const VERSION: SchemaVersion = SchemaVersion::new(1);
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::fmt::Write as _;

    use proptest::prelude::*;

    use super::*;
    use crate::error::ArithmeticError;
    use crate::session::SessionState;

    /// Renders `value` as the `postcard` bytes a log would hold.
    fn encode(value: &impl serde::Serialize) -> String {
        let bytes = postcard::to_allocvec(value).expect("domain events always encode");
        bytes.iter().fold(String::new(), |mut acc, b| {
            let _ = write!(acc, "{b:02x} ");
            acc
        })
    }

    /// Builds the `name → encoding` listing that the snapshot pins.
    fn listing<E: serde::Serialize>(samples: &[(String, E)]) -> String {
        samples
            .iter()
            .map(|(name, e)| format!("{name}: {}", encode(e).trim_end()))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Fails when a variant was added without a sample, so the snapshot cannot go stale.
    fn assert_covers_every_variant<K: Ord + core::fmt::Display + strum::IntoEnumIterator>(
        covered: &BTreeSet<String>,
    ) {
        let all: BTreeSet<String> = K::iter().map(|k| k.to_string()).collect();
        let missing: Vec<_> = all.difference(covered).collect();
        assert!(missing.is_empty(), "no sample for {missing:?}; add one and refresh the snapshot");
    }

    fn book_samples() -> Vec<(String, BookEvent)> {
        let samples = vec![
            BookEvent::OrderAccepted {
                order: OrderId::new(7),
                command: CommandId::new(11),
                account: AccountId::new(3),
            },
            BookEvent::OrderRejected {
                command: CommandId::new(11),
                account: AccountId::new(3),
                reason: RejectReason::NotTrading { state: SessionState::Halted },
            },
            BookEvent::Trade {
                trade: TradeId::new(1),
                taker: OrderId::new(7),
                maker: OrderId::new(2),
                taker_side: Side::Buy,
                price: Price::from_ticks(11_200),
                qty: Qty::from_lots(500),
                maker_fee: Amount::from_minor(112),
                taker_fee: Amount::from_minor(560),
            },
            BookEvent::OrderDone {
                order: OrderId::new(7),
                status: OrderStatus::Filled,
                unfilled: Qty::ZERO,
            },
            BookEvent::StopTriggered { order: OrderId::new(9), at: Price::from_ticks(11_195) },
        ];
        samples.into_iter().map(|e| (BookEventKind::from(&e).to_string(), e)).collect()
    }

    fn ledger_samples() -> Vec<(String, LedgerEvent)> {
        let (account, asset, command) = (AccountId::new(3), AssetId::new(2), CommandId::new(11));
        let samples = vec![
            LedgerEvent::Reserved { account, asset, command, amount: Amount::from_minor(1_000) },
            LedgerEvent::Released { account, asset, command, amount: Amount::from_minor(400) },
            LedgerEvent::Committed { account, asset, command, amount: Amount::from_minor(600) },
            LedgerEvent::Posted {
                instrument: Some(InstrumentId::new(1)),
                asset,
                debit: account,
                credit: AccountId::new(4),
                amount: Amount::from_minor(600),
            },
        ];
        samples.into_iter().map(|e| (LedgerEventKind::from(&e).to_string(), e)).collect()
    }

    fn contract_samples() -> Vec<(String, ContractEvent)> {
        let contract = ContractId::new(5);
        let samples = vec![
            ContractEvent::Created {
                contract,
                obligor: AccountId::new(3),
                beneficiary: AccountId::new(4),
            },
            ContractEvent::ObligationDue {
                contract,
                instalment: 1,
                asset: AssetId::new(2),
                amount: Amount::from_minor(10_000),
            },
            ContractEvent::ObligationSettled {
                contract,
                instalment: 1,
                amount: Amount::from_minor(10_000),
            },
            ContractEvent::ObligationMissed {
                contract,
                instalment: 2,
                shortfall: Amount::from_minor(2_500),
            },
            ContractEvent::Closed { contract, outcome: ContractOutcome::Defaulted },
        ];
        samples.into_iter().map(|e| (ContractEventKind::from(&e).to_string(), e)).collect()
    }

    #[test]
    fn book_schema_encoding_is_frozen() {
        let samples = book_samples();
        assert_covers_every_variant::<BookEventKind>(
            &samples.iter().map(|(n, _)| n.clone()).collect(),
        );
        insta::assert_snapshot!(listing(&samples));
    }

    #[test]
    fn ledger_schema_encoding_is_frozen() {
        let samples = ledger_samples();
        assert_covers_every_variant::<LedgerEventKind>(
            &samples.iter().map(|(n, _)| n.clone()).collect(),
        );
        insta::assert_snapshot!(listing(&samples));
    }

    #[test]
    fn contract_schema_encoding_is_frozen() {
        let samples = contract_samples();
        assert_covers_every_variant::<ContractEventKind>(
            &samples.iter().map(|(n, _)| n.clone()).collect(),
        );
        insta::assert_snapshot!(listing(&samples));
    }

    #[test]
    fn schema_ids_and_versions_are_distinct() {
        assert_ne!(BookEvent::SCHEMA, LedgerEvent::SCHEMA);
        assert_ne!(LedgerEvent::SCHEMA, ContractEvent::SCHEMA);
        assert_eq!(BookEvent::VERSION, SchemaVersion::new(1), "bump only with an upcaster");
        assert_eq!(SchemaVersion::new(2).to_string(), "v2");
        assert_eq!(SchemaId::Book.to_string(), "BOOK");
    }

    fn amount() -> impl Strategy<Value = Amount> {
        any::<i128>().prop_map(Amount::from_minor)
    }

    fn reject_reason() -> impl Strategy<Value = RejectReason> {
        prop_oneof![
            Just(RejectReason::InvalidQty),
            Just(RejectReason::InsufficientFunds),
            Just(RejectReason::SelfTrade),
            Just(RejectReason::Arithmetic(ArithmeticError::Overflow)),
            prop_oneof![
                Just(SessionState::PreOpen),
                Just(SessionState::Halted),
                Just(SessionState::Closed),
            ]
            .prop_map(|state| RejectReason::NotTrading { state }),
        ]
    }

    fn book_event() -> impl Strategy<Value = BookEvent> {
        prop_oneof![
            (any::<u64>(), any::<u128>(), any::<u64>()).prop_map(|(o, c, a)| {
                BookEvent::OrderAccepted {
                    order: OrderId::new(o),
                    command: CommandId::new(c),
                    account: AccountId::new(a),
                }
            }),
            (any::<u128>(), any::<u64>(), reject_reason()).prop_map(|(c, a, reason)| {
                BookEvent::OrderRejected {
                    command: CommandId::new(c),
                    account: AccountId::new(a),
                    reason,
                }
            }),
            (any::<u64>(), any::<i64>(), any::<u64>(), amount(), amount()).prop_map(
                |(id, price, qty, maker_fee, taker_fee)| BookEvent::Trade {
                    trade: TradeId::new(id),
                    taker: OrderId::new(id),
                    maker: OrderId::new(id.wrapping_add(1)),
                    taker_side: Side::Sell,
                    price: Price::from_ticks(price),
                    qty: Qty::from_lots(qty),
                    maker_fee,
                    taker_fee,
                }
            ),
            (any::<u64>(), any::<u64>()).prop_map(|(o, q)| BookEvent::OrderDone {
                order: OrderId::new(o),
                status: OrderStatus::Cancelled,
                unfilled: Qty::from_lots(q),
            }),
            (any::<u64>(), any::<i64>()).prop_map(|(o, p)| BookEvent::StopTriggered {
                order: OrderId::new(o),
                at: Price::from_ticks(p),
            }),
        ]
    }

    fn ledger_event() -> impl Strategy<Value = LedgerEvent> {
        (any::<u64>(), any::<u32>(), any::<u128>(), amount()).prop_flat_map(
            |(acc, asset, cmd, amount)| {
                let (account, asset, command) =
                    (AccountId::new(acc), AssetId::new(asset), CommandId::new(cmd));
                prop_oneof![
                    Just(LedgerEvent::Reserved { account, asset, command, amount }),
                    Just(LedgerEvent::Released { account, asset, command, amount }),
                    Just(LedgerEvent::Committed { account, asset, command, amount }),
                    Just(LedgerEvent::Posted {
                        instrument: None,
                        asset,
                        debit: account,
                        credit: AccountId::new(acc.wrapping_add(1)),
                        amount,
                    }),
                ]
            },
        )
    }

    fn contract_event() -> impl Strategy<Value = ContractEvent> {
        (any::<u64>(), any::<u32>(), amount()).prop_flat_map(|(id, instalment, amount)| {
            let contract = ContractId::new(id);
            prop_oneof![
                Just(ContractEvent::Created {
                    contract,
                    obligor: AccountId::new(id),
                    beneficiary: AccountId::new(id.wrapping_add(1)),
                }),
                Just(ContractEvent::ObligationDue {
                    contract,
                    instalment,
                    asset: AssetId::new(1),
                    amount,
                }),
                Just(ContractEvent::ObligationSettled { contract, instalment, amount }),
                Just(ContractEvent::ObligationMissed { contract, instalment, shortfall: amount }),
                Just(ContractEvent::Closed { contract, outcome: ContractOutcome::Fulfilled }),
            ]
        })
    }

    proptest! {
        #[test]
        fn book_events_round_trip(event in book_event()) {
            let bytes = postcard::to_allocvec(&event).expect("encode");
            prop_assert_eq!(postcard::from_bytes::<BookEvent>(&bytes).expect("decode"), event);
        }

        #[test]
        fn ledger_events_round_trip(event in ledger_event()) {
            let bytes = postcard::to_allocvec(&event).expect("encode");
            prop_assert_eq!(postcard::from_bytes::<LedgerEvent>(&bytes).expect("decode"), event);
        }

        #[test]
        fn contract_events_round_trip(event in contract_event()) {
            let bytes = postcard::to_allocvec(&event).expect("encode");
            prop_assert_eq!(postcard::from_bytes::<ContractEvent>(&bytes).expect("decode"), event);
        }

        #[test]
        fn truncated_payloads_never_decode_into_a_different_event(event in book_event()) {
            let bytes = postcard::to_allocvec(&event).expect("encode");
            for cut in 0..bytes.len() {
                let decoded = postcard::from_bytes::<BookEvent>(&bytes[..cut]);
                prop_assert!(decoded.is_err(), "truncation must fail, not decode differently");
            }
        }
    }
}
