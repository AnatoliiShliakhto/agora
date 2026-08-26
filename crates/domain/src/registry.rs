//! Instrument symbols and registry.

use std::collections::BTreeMap;
use std::fmt;

use crate::ids::InstrumentId;
use crate::instrument::Instrument;

/// Maximum length of a [`Symbol`] in bytes.
pub const MAX_SYMBOL_LEN: usize = 16;

/// Human-readable instrument identifier: `1..=16` characters of `A-Z`, `0-9`, `_`.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(try_from = "String", into = "String")]
pub struct Symbol(String);

/// Error of building a [`Symbol`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, thiserror::Error)]
#[non_exhaustive]
pub enum SymbolError {
    /// Empty string.
    #[error("symbol is empty")]
    Empty,
    /// Longer than [`MAX_SYMBOL_LEN`].
    #[error("symbol longer than {MAX_SYMBOL_LEN} bytes")]
    TooLong,
    /// Character outside `A-Z`, `0-9`, `_`.
    #[error("symbol contains invalid character {0:?}")]
    InvalidChar(char),
}

impl Symbol {
    /// Validates and wraps `raw`.
    ///
    /// # Errors
    ///
    /// See [`SymbolError`].
    pub fn new(raw: impl Into<String>) -> Result<Self, SymbolError> {
        let raw = raw.into();
        if raw.is_empty() {
            return Err(SymbolError::Empty);
        }
        if raw.len() > MAX_SYMBOL_LEN {
            return Err(SymbolError::TooLong);
        }
        if let Some(bad) =
            raw.chars().find(|c| !(c.is_ascii_uppercase() || c.is_ascii_digit() || *c == '_'))
        {
            return Err(SymbolError::InvalidChar(bad));
        }
        Ok(Self(raw))
    }

    /// Symbol text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl TryFrom<String> for Symbol {
    type Error = SymbolError;

    fn try_from(raw: String) -> Result<Self, Self::Error> {
        Self::new(raw)
    }
}

impl TryFrom<&str> for Symbol {
    type Error = SymbolError;

    fn try_from(raw: &str) -> Result<Self, Self::Error> {
        Self::new(raw)
    }
}

impl From<Symbol> for String {
    fn from(symbol: Symbol) -> Self {
        symbol.0
    }
}

/// Error of registering an instrument.
#[derive(Debug, Clone, PartialEq, Eq, Hash, thiserror::Error)]
#[non_exhaustive]
pub enum RegistryError {
    /// An instrument with this id exists.
    #[error("duplicate instrument id {0}")]
    DuplicateId(InstrumentId),
    /// An instrument with this symbol exists.
    #[error("duplicate symbol {0}")]
    DuplicateSymbol(Symbol),
}

/// Static registry of instruments: `id ↔ symbol`, iteration in id order.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct InstrumentRegistry {
    by_id: BTreeMap<InstrumentId, Instrument>,
    by_symbol: BTreeMap<Symbol, InstrumentId>,
}

impl InstrumentRegistry {
    /// Empty registry.
    #[must_use]
    pub const fn new() -> Self {
        Self { by_id: BTreeMap::new(), by_symbol: BTreeMap::new() }
    }

    /// Adds `instrument`.
    ///
    /// # Errors
    ///
    /// See [`RegistryError`]; the registry is unchanged on error.
    pub fn insert(&mut self, instrument: Instrument) -> Result<(), RegistryError> {
        if self.by_id.contains_key(&instrument.id) {
            return Err(RegistryError::DuplicateId(instrument.id));
        }
        if self.by_symbol.contains_key(&instrument.symbol) {
            return Err(RegistryError::DuplicateSymbol(instrument.symbol));
        }
        self.by_symbol.insert(instrument.symbol.clone(), instrument.id);
        self.by_id.insert(instrument.id, instrument);
        Ok(())
    }

    /// Instrument by id.
    #[must_use]
    pub fn get(&self, id: InstrumentId) -> Option<&Instrument> {
        self.by_id.get(&id)
    }

    /// Instrument by symbol.
    #[must_use]
    pub fn by_symbol(&self, symbol: &Symbol) -> Option<&Instrument> {
        self.by_symbol.get(symbol).and_then(|id| self.by_id.get(id))
    }

    /// Number of instruments.
    #[must_use]
    pub fn len(&self) -> usize {
        self.by_id.len()
    }

    /// Returns `true` if no instrument is registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.by_id.is_empty()
    }

    /// Instruments in id order.
    pub fn iter(&self) -> impl Iterator<Item = &Instrument> {
        self.by_id.values()
    }
}

#[cfg(test)]
mod tests {
    use core::num::NonZeroU32;

    use rstest::rstest;

    use super::*;
    use crate::ids::AssetId;
    use crate::instrument::{NotionalScale, PriceBounds};
    use crate::money::Price;

    fn instrument(id: u32, symbol: &str) -> Instrument {
        Instrument {
            id: InstrumentId::new(id),
            symbol: Symbol::new(symbol).expect("valid symbol"),
            base: AssetId::new(1),
            quote: AssetId::new(2),
            scale: NotionalScale::new(NonZeroU32::MIN, NonZeroU32::MIN),
            price_bounds: PriceBounds::new(Price::from_ticks(1), Price::MAX).expect("min ≤ max"),
            max_deviation: None,
        }
    }

    #[rstest]
    #[case("EURUSD", Ok(()))]
    #[case("BTC_USD", Ok(()))]
    #[case("A", Ok(()))]
    #[case("ABCDEFGHIJKLMNOP", Ok(()))]
    #[case("", Err(SymbolError::Empty))]
    #[case("ABCDEFGHIJKLMNOPQ", Err(SymbolError::TooLong))]
    #[case("eurusd", Err(SymbolError::InvalidChar('e')))]
    #[case("EUR/USD", Err(SymbolError::InvalidChar('/')))]
    #[case("EUR USD", Err(SymbolError::InvalidChar(' ')))]
    #[case("ÉUR", Err(SymbolError::InvalidChar('É')))]
    fn symbol_validation(#[case] raw: &str, #[case] expected: Result<(), SymbolError>) {
        assert_eq!(Symbol::new(raw).map(drop), expected, "{raw:?}");
    }

    #[test]
    fn symbol_serde_rejects_invalid_text() {
        let ok: Symbol = serde_json::from_str("\"EURUSD\"").expect("valid");
        assert_eq!(ok.as_str(), "EURUSD");
        assert!(serde_json::from_str::<Symbol>("\"eur usd\"").is_err(), "must reject");
    }

    #[test]
    fn registry_resolves_both_ways_in_id_order() {
        let mut reg = InstrumentRegistry::new();
        reg.insert(instrument(2, "GBPUSD")).expect("insert");
        reg.insert(instrument(1, "EURUSD")).expect("insert");
        let eurusd = Symbol::new("EURUSD").expect("valid");
        assert_eq!(reg.by_symbol(&eurusd).map(|i| i.id), Some(InstrumentId::new(1)));
        assert_eq!(reg.get(InstrumentId::new(2)).map(|i| i.symbol.as_str()), Some("GBPUSD"));
        let ids: Vec<_> = reg.iter().map(|i| i.id.raw()).collect();
        assert_eq!(ids, vec![1, 2], "iteration must be deterministic by id");
        assert_eq!(reg.len(), 2);
    }

    #[test]
    fn registry_rejects_duplicates_without_side_effects() {
        let mut reg = InstrumentRegistry::new();
        reg.insert(instrument(1, "EURUSD")).expect("insert");
        assert_eq!(
            reg.insert(instrument(1, "GBPUSD")),
            Err(RegistryError::DuplicateId(InstrumentId::new(1)))
        );
        assert_eq!(
            reg.insert(instrument(2, "EURUSD")),
            Err(RegistryError::DuplicateSymbol(Symbol::new("EURUSD").expect("valid")))
        );
        assert_eq!(reg.len(), 1, "failed inserts must not leak entries");
        assert!(
            reg.by_symbol(&Symbol::new("GBPUSD").expect("valid")).is_none(),
            "no partial insert"
        );
    }
}
