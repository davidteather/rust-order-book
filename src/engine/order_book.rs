use std::fmt;
use rustc_hash::FxHashSet;
use crate::types::symbol_mapping::SymbolId;
use crate::engine::order_book_trait::OrderBookTrait;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Default)]
pub enum OrderBookType {
    #[default]
    HashMap,
    PriorityQueue,
    ArrayQueue,
}

impl fmt::Display for OrderBookType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            OrderBookType::HashMap => "HashMap",
            OrderBookType::PriorityQueue => "PriorityQueue",
            OrderBookType::ArrayQueue => "ArrayQueue",
        };
        write!(f, "{s}")
    }
}

pub fn create_order_book(
    order_book_type: OrderBookType,
    symbols: FxHashSet<SymbolId>,
) -> Box<dyn crate::engine::OrderBookTrait + Send + Sync> {
    match order_book_type {
        OrderBookType::HashMap => {
            Box::new(crate::engine::hashmap_order_book::HashMapOrderBook::new(symbols))
        }
        OrderBookType::PriorityQueue => {
            Box::new(crate::engine::priority_queue_order_book::PriorityQueueOrderBook::new(symbols))
        }
        OrderBookType::ArrayQueue => {
            Box::new(crate::engine::array_queue_order_book::ArrayQueueOrderBook::new(symbols))
        }
    }
}

pub mod factories {
    use super::*;
    use crate::engine::OrderBookTrait;
    
    pub fn create_hashmap_order_book(symbols: FxHashSet<SymbolId>) -> impl OrderBookTrait {
        crate::engine::hashmap_order_book::HashMapOrderBook::new(symbols)
    }
    
    pub fn create_priority_queue_order_book(symbols: FxHashSet<SymbolId>) -> impl OrderBookTrait {
        crate::engine::priority_queue_order_book::PriorityQueueOrderBook::new(symbols)
    }
    
    pub fn create_array_queue_order_book(symbols: FxHashSet<SymbolId>) -> impl OrderBookTrait {
        crate::engine::array_queue_order_book::ArrayQueueOrderBook::new(symbols)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::OrderBookTrait;
    use crate::types::order::{new_order, OrderSide};

    #[test]
    fn test_factory_creates_all_types() {
        let symbols = FxHashSet::from_iter([0]);
        
        let hashmap_book = create_order_book(OrderBookType::HashMap, symbols.clone());
        let priority_book = create_order_book(OrderBookType::PriorityQueue, symbols.clone());
        let array_book = create_order_book(OrderBookType::ArrayQueue, symbols.clone());
        
        assert_eq!(hashmap_book.order_book_type(), OrderBookType::HashMap);
        assert_eq!(priority_book.order_book_type(), OrderBookType::PriorityQueue);
        assert_eq!(array_book.order_book_type(), OrderBookType::ArrayQueue);
    }
    
    #[test]
    fn test_factory_functions_work() {
        let symbols = FxHashSet::from_iter([0]);
        
        let mut hashmap_book = factories::create_hashmap_order_book(symbols.clone());
        let order = new_order(1, 0, 100, 150.0, OrderSide::Buy);
        
        assert!(hashmap_book.add_order(order).is_ok());
    }
}