//! Version 1 of the public API.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Request to place an order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct PlaceOrderRequest {
    /// Client idempotency key (ULID).
    pub client_order_id: String,
    /// Instrument symbol.
    pub symbol: String,
    /// `BUY` or `SELL`.
    pub side: String,
    /// `LIMIT`, `MARKET`, `STOP_MARKET`, `STOP_LIMIT`.
    pub order_type: String,
    /// `GTC`, `IOC`, `FOK`, `GTD`.
    pub time_in_force: String,
    /// Decimal quantity in base units.
    #[schema(value_type = String)]
    pub quantity: Decimal,
    /// Decimal limit price, if applicable.
    #[schema(value_type = Option<String>)]
    pub price: Option<Decimal>,
    /// Decimal trigger price, if applicable.
    #[schema(value_type = Option<String>)]
    pub trigger_price: Option<Decimal>,
}

/// Error body returned by every failing endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ApiError {
    /// Stable machine-readable code.
    pub code: String,
    /// Human-readable message.
    pub message: String,
    /// Seconds after which a retry may succeed (back-pressure, degradation).
    pub retry_after: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn place_order_json_shape() {
        let req = PlaceOrderRequest {
            client_order_id: "01J0000000000000000000000".into(),
            symbol: "EURUSD".into(),
            side: "BUY".into(),
            order_type: "LIMIT".into(),
            time_in_force: "GTC".into(),
            quantity: Decimal::new(1_000_000, 0),
            price: Some(Decimal::new(11_200, 4)),
            trigger_price: None,
        };
        insta::assert_json_snapshot!(req);
    }
}
