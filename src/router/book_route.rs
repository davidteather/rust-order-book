use crate::types::{order::Order, symbol_mapping::SymbolId};
use crate::engine::OrderBookTrait;

pub struct BookRoute {
    pub symbol: SymbolId,
    pub order_book: Box<dyn OrderBookTrait + Send + Sync>,
}

impl BookRoute {
    pub fn new(symbol: SymbolId, order_book: Box<dyn OrderBookTrait + Send + Sync>) -> Self {
        BookRoute {
            symbol,
            order_book,
        }
    }

    pub fn process_order(&mut self, order: Order) {
        let _ = self.order_book.add_order_fast(order);
        self.order_book.match_orders();
    }
}