//! Strongly typed identifiers.
//!
//! Identifiers the engine owns are plain integers, allocated by the sequencer of the owning
//! shard: dense, sortable and cheap to hash. Identifiers that come from the rest of the sandbox
//! are [`ExternalId`]s — opaque strings the engine never interprets, mapped to a dense integer
//! once, on first sight, by the aggregate that owns the entity (ADR-0011).

macro_rules! id_type {
    ($(#[$meta:meta])* $name:ident($inner:ty)) => {
        $(#[$meta])*
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash,
            serde::Serialize, serde::Deserialize,
        )]
        #[serde(transparent)]
        pub struct $name($inner);

        impl $name {
            /// Wraps a raw identifier.
            #[must_use]
            pub const fn new(raw: $inner) -> Self {
                Self(raw)
            }

            /// Returns the raw identifier.
            #[must_use]
            pub const fn raw(self) -> $inner {
                self.0
            }
        }

        impl From<$inner> for $name {
            fn from(raw: $inner) -> Self {
                Self(raw)
            }
        }

        impl core::fmt::Display for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{}#{}", stringify!($name), self.0)
            }
        }
    };
}

id_type! {
    /// Identifier of a trading account.
    AccountId(u64)
}
id_type! {
    /// Identifier of an asset (currency, share, commodity).
    AssetId(u32)
}
id_type! {
    /// Identifier of a tradable instrument (a base/quote asset pair).
    InstrumentId(u32)
}
id_type! {
    /// Identifier of an order, unique within its instrument shard.
    OrderId(u64)
}
id_type! {
    /// Identifier of a trade (a single fill between two orders).
    TradeId(u64)
}
id_type! {
    /// Identifier of a smart contract instance.
    ContractId(u64)
}
id_type! {
    /// Monotonic sequence number of an event within one shard's log.
    Seq(u64)
}
id_type! {
    /// Client-supplied idempotency key of a command.
    CommandId(u128)
}

/// Maximum length of an [`ExternalId`] in bytes.
///
/// Wide enough for a hyphenated UUID (36) and a namespaced name, small enough to keep the
/// boundary registries compact.
pub const MAX_EXTERNAL_ID_LEN: usize = 64;

/// Identifier minted by another sandbox module (a player, an NPC, a currency, a good).
///
/// Opaque on purpose: the engine compares it byte for byte and never parses it, so integrators
/// are free to use ULIDs, UUIDs, numbers or namespaced strings (ADR-0011). Every entity the
/// engine does not create — accounts and assets — enters through one of these and is mapped to
/// a dense internal identifier by the owning aggregate.
///
/// # Examples
///
/// ```rust
/// use agora_domain::ids::ExternalId;
///
/// let ulid = ExternalId::new("01J0000000000000000000000")?;
/// let namespaced = ExternalId::new("player:4211")?;
/// assert_ne!(ulid, namespaced);
/// assert!(ExternalId::new("player 1").is_err(), "spaces are not allowed");
/// # Ok::<(), agora_domain::ids::ExternalIdError>(())
/// ```
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(try_from = "String", into = "String")]
pub struct ExternalId(String);

/// Error of building an [`ExternalId`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, thiserror::Error)]
#[non_exhaustive]
pub enum ExternalIdError {
    /// Empty string.
    #[error("external id is empty")]
    Empty,
    /// Longer than [`MAX_EXTERNAL_ID_LEN`].
    #[error("external id longer than {MAX_EXTERNAL_ID_LEN} bytes")]
    TooLong,
    /// Character outside `A-Za-z0-9._:-`.
    #[error("external id contains invalid character {0:?}")]
    InvalidChar(char),
}

impl ExternalId {
    /// Validates and wraps `raw`.
    ///
    /// # Errors
    ///
    /// See [`ExternalIdError`].
    pub fn new(raw: impl Into<String>) -> Result<Self, ExternalIdError> {
        let raw = raw.into();
        if raw.is_empty() {
            return Err(ExternalIdError::Empty);
        }
        if raw.len() > MAX_EXTERNAL_ID_LEN {
            return Err(ExternalIdError::TooLong);
        }
        let invalid = raw
            .chars()
            .find(|c| !(c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | ':' | '-')));
        if let Some(bad) = invalid {
            return Err(ExternalIdError::InvalidChar(bad));
        }
        Ok(Self(raw))
    }

    /// Identifier text, exactly as the other module wrote it.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl core::fmt::Display for ExternalId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.0)
    }
}

impl TryFrom<String> for ExternalId {
    type Error = ExternalIdError;

    fn try_from(raw: String) -> Result<Self, Self::Error> {
        Self::new(raw)
    }
}

impl TryFrom<&str> for ExternalId {
    type Error = ExternalIdError;

    fn try_from(raw: &str) -> Result<Self, Self::Error> {
        Self::new(raw)
    }
}

impl From<ExternalId> for String {
    fn from(id: ExternalId) -> Self {
        id.0
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case::ulid("01J0000000000000000000000", Ok(()))]
    #[case::uuid("f81d4fae-7dec-11d0-a765-00a0c91e6bf6", Ok(()))]
    #[case::number("4211", Ok(()))]
    #[case::namespaced("player:4211", Ok(()))]
    #[case::dotted("mod08.currency.gold", Ok(()))]
    #[case::empty("", Err(ExternalIdError::Empty))]
    #[case::space("player 1", Err(ExternalIdError::InvalidChar(' ')))]
    #[case::slash("player/1", Err(ExternalIdError::InvalidChar('/')))]
    #[case::non_ascii("гравець", Err(ExternalIdError::InvalidChar('г')))]
    fn external_id_validation(#[case] raw: &str, #[case] expected: Result<(), ExternalIdError>) {
        assert_eq!(ExternalId::new(raw).map(drop), expected, "{raw:?}");
    }

    #[test]
    fn external_id_length_limit_is_inclusive() {
        let at_limit = "x".repeat(MAX_EXTERNAL_ID_LEN);
        let over_limit = "x".repeat(MAX_EXTERNAL_ID_LEN + 1);
        assert!(ExternalId::new(at_limit).is_ok(), "the limit itself is allowed");
        assert_eq!(ExternalId::new(over_limit), Err(ExternalIdError::TooLong));
    }

    #[test]
    fn external_id_is_opaque_and_case_sensitive() {
        let lower = ExternalId::new("player:a").expect("valid");
        let upper = ExternalId::new("player:A").expect("valid");
        assert_ne!(lower, upper, "identifiers are compared byte for byte");
        assert_eq!(lower.as_str(), "player:a", "text is preserved verbatim");
    }

    #[test]
    fn external_id_serde_rejects_invalid_text() {
        let ok: ExternalId = serde_json::from_str("\"player:1\"").expect("valid");
        assert_eq!(ok.to_string(), "player:1");
        assert!(serde_json::from_str::<ExternalId>("\"player 1\"").is_err(), "must reject");
    }

    #[test]
    fn internal_ids_round_trip_through_raw() {
        assert_eq!(AccountId::from(7_u64).raw(), 7);
        assert_eq!(InstrumentId::new(3).to_string(), "InstrumentId#3");
    }
}
