//! QoS classes.

/// Delivery class of a topic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Qos {
    /// Lossless, ordered, bounded; producer experiences back-pressure.
    Trading,
    /// Latest-value wins; intermediate updates may be dropped for slow consumers.
    MarketData,
}
