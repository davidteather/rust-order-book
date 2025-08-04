use crate::types::symbol_mapping::SymbolId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Order {
    pub id: u64,
    pub symbol: SymbolId,
    pub quantity: u64,
    pub price: u64,
    pub order_type: OrderSide,
}

pub fn new_order(id: u64, symbol: SymbolId, quantity: u64, price: f64, order_type: OrderSide) -> Order {
    let price = price_to_u64(price);

    Order {
        id,
        symbol,
        quantity,
        price,
        order_type,
    }
}

#[inline(always)]
pub const fn price_to_u64(price: f64) -> u64 {
    (price * 1000.0) as u64
}

#[inline(always)]
pub const fn u64_to_price(price: u64) -> f64 {
    price as f64 / 1000.0
}

