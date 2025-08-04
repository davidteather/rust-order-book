use rustc_hash::{FxHashSet, FxHashMap};

use crate::engine::{OrderBookType, create_order_book, OrderBookTrait};
use crate::types::order::Order;
use crate::types::symbol_mapping::SymbolId;

pub struct OrderRouter {
    direct_order_books: FxHashMap<SymbolId, Box<dyn OrderBookTrait + Send + Sync>>,
    order_book_type: OrderBookType,
}

impl OrderRouter {
    pub fn new_direct(symbols: FxHashSet<SymbolId>, order_book_type: OrderBookType) -> Self {
        let mut direct_order_books = FxHashMap::default();
        for &symbol in &symbols {
            let symbol_set = FxHashSet::from_iter([symbol]);
            direct_order_books.insert(symbol, create_order_book(order_book_type, symbol_set));
        }
        
        Self {
            direct_order_books,
            order_book_type,
        }
    }
    
    #[inline(always)]
    pub fn route_order(&mut self, order: Order) -> Result<(), &'static str> {
        if let Some(order_book) = self.direct_order_books.get_mut(&order.symbol) {
            order_book.add_order_fast(order);
            Ok(())
        } else {
            Err("Invalid symbol")
        }
    }

    #[inline(always)]
    pub fn match_all_orders(&mut self) {
        for order_book in self.direct_order_books.values_mut() {
            order_book.match_orders();
        }
    }

    #[inline(always)]
    pub fn supports_symbol(&self, symbol: SymbolId) -> bool {
        self.direct_order_books.contains_key(&symbol)
    }

    #[inline(always)]
    pub fn get_implementation_name(&self) -> &'static str {
        match self.order_book_type {
            OrderBookType::HashMap => "HashMap",
            OrderBookType::PriorityQueue => "PriorityQueue", 
            OrderBookType::ArrayQueue => "ArrayQueue",
        }
    }

    #[inline(always)]
    pub fn get_symbols(&self) -> Vec<SymbolId> {
        self.direct_order_books.keys().copied().collect()
    }
}
