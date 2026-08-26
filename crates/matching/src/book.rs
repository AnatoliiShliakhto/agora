//! Price-level order book.

use std::collections::{BTreeMap, VecDeque};

use agora_domain::error::ArithmeticError;
use agora_domain::ids::OrderId;
use agora_domain::money::{Price, Qty};
use agora_domain::order::Side;

/// Resting order as stored at a price level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Resting {
    /// Order identifier.
    pub id: OrderId,
    /// Remaining quantity.
    pub qty: Qty,
}

/// Orders at one price, in arrival order.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Level {
    orders: VecDeque<Resting>,
    total: Qty,
}

impl Level {
    /// Total resting quantity at this level.
    #[must_use]
    pub const fn total(&self) -> Qty {
        self.total
    }

    /// Number of resting orders.
    #[must_use]
    pub fn len(&self) -> usize {
        self.orders.len()
    }

    /// Returns `true` if no orders rest at this level.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.orders.is_empty()
    }

    /// Resting orders in priority order.
    pub fn iter(&self) -> impl Iterator<Item = &Resting> {
        self.orders.iter()
    }
}

/// One side of the book, best price first.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BookSide {
    side: Side,
    levels: BTreeMap<Price, Level>,
}

impl BookSide {
    /// Empty side.
    #[must_use]
    pub const fn new(side: Side) -> Self {
        Self { side, levels: BTreeMap::new() }
    }

    /// Best price: highest bid or lowest ask.
    #[must_use]
    pub fn best(&self) -> Option<Price> {
        match self.side {
            Side::Buy => self.levels.keys().next_back().copied(),
            Side::Sell => self.levels.keys().next().copied(),
        }
    }

    /// Level at `price`, if any.
    #[must_use]
    pub fn level(&self, price: Price) -> Option<&Level> {
        self.levels.get(&price)
    }

    /// Appends a resting order at `price`.
    ///
    /// # Errors
    ///
    /// [`ArithmeticError::Overflow`] if the level total overflows.
    pub fn push(&mut self, price: Price, order: Resting) -> Result<(), ArithmeticError> {
        let level = self.levels.entry(price).or_default();
        level.total = level.total.checked_add(order.qty)?;
        level.orders.push_back(order);
        Ok(())
    }

    /// Removes the order `id` at `price`; returns it if found.
    pub fn remove(&mut self, price: Price, id: OrderId) -> Option<Resting> {
        let level = self.levels.get_mut(&price)?;
        let pos = level.orders.iter().position(|o| o.id == id)?;
        let removed = level.orders.remove(pos)?;
        level.total = level.total.checked_sub(removed.qty).unwrap_or(Qty::ZERO);
        if level.orders.is_empty() {
            self.levels.remove(&price);
        }
        Some(removed)
    }

    /// Levels from best to worst.
    #[must_use]
    pub fn levels(&self) -> Box<dyn Iterator<Item = (&Price, &Level)> + '_> {
        match self.side {
            Side::Buy => Box::new(self.levels.iter().rev()),
            Side::Sell => Box::new(self.levels.iter()),
        }
    }
}

/// Two-sided order book of one instrument.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderBook {
    bids: BookSide,
    asks: BookSide,
}

impl Default for OrderBook {
    fn default() -> Self {
        Self::new()
    }
}

impl OrderBook {
    /// Empty book.
    #[must_use]
    pub const fn new() -> Self {
        Self { bids: BookSide::new(Side::Buy), asks: BookSide::new(Side::Sell) }
    }

    /// Side to rest on for `side`.
    #[must_use]
    pub const fn side(&self, side: Side) -> &BookSide {
        match side {
            Side::Buy => &self.bids,
            Side::Sell => &self.asks,
        }
    }

    /// Mutable side to rest on for `side`.
    pub const fn side_mut(&mut self, side: Side) -> &mut BookSide {
        match side {
            Side::Buy => &mut self.bids,
            Side::Sell => &mut self.asks,
        }
    }

    /// Best bid price.
    #[must_use]
    pub fn best_bid(&self) -> Option<Price> {
        self.bids.best()
    }

    /// Best ask price.
    #[must_use]
    pub fn best_ask(&self) -> Option<Price> {
        self.asks.best()
    }

    /// Spread in ticks, if both sides are present.
    #[must_use]
    pub fn spread(&self) -> Option<Price> {
        self.best_ask()?.checked_sub(self.best_bid()?).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn resting(id: u64, lots: u64) -> Resting {
        Resting { id: OrderId::new(id), qty: Qty::from_lots(lots) }
    }

    #[test]
    fn best_bid_is_highest_and_best_ask_is_lowest() -> Result<(), ArithmeticError> {
        let mut book = OrderBook::new();
        book.side_mut(Side::Buy).push(Price::from_ticks(10), resting(1, 5))?;
        book.side_mut(Side::Buy).push(Price::from_ticks(12), resting(2, 5))?;
        book.side_mut(Side::Sell).push(Price::from_ticks(15), resting(3, 5))?;
        book.side_mut(Side::Sell).push(Price::from_ticks(13), resting(4, 5))?;
        assert_eq!(book.best_bid(), Some(Price::from_ticks(12)));
        assert_eq!(book.best_ask(), Some(Price::from_ticks(13)));
        assert_eq!(book.spread(), Some(Price::from_ticks(1)));
        Ok(())
    }

    #[test]
    fn remove_keeps_fifo_order_and_drops_empty_levels() -> Result<(), ArithmeticError> {
        let mut side = BookSide::new(Side::Sell);
        let p = Price::from_ticks(7);
        side.push(p, resting(1, 1))?;
        side.push(p, resting(2, 2))?;
        side.push(p, resting(3, 3))?;
        assert_eq!(side.remove(p, OrderId::new(2)), Some(resting(2, 2)));
        let ids: Vec<_> =
            side.level(p).map(|l| l.iter().map(|o| o.id.raw()).collect()).unwrap_or_default();
        assert_eq!(ids, vec![1, 3]);
        assert_eq!(side.level(p).map(Level::total), Some(Qty::from_lots(4)));
        side.remove(p, OrderId::new(1));
        side.remove(p, OrderId::new(3));
        assert!(side.level(p).is_none(), "empty level must be dropped");
        Ok(())
    }
}
