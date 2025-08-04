use crossbeam::queue::ArrayQueue;
use std::sync::Arc;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::engine::order_book_trait::{OrderBookTrait, OrderBookError};
use crate::engine::OrderBookType;
use crate::types::{order::Order, symbol_mapping::SymbolId};

const DEFAULT_QUEUE_SIZE: usize = 4096;

#[derive(Debug)]
struct ArrayQueueMatcher {
    bids: Arc<ArrayQueue<Order>>,
    asks: Arc<ArrayQueue<Order>>,
    best_bid: Option<u64>,
    best_ask: Option<u64>,
}

impl ArrayQueueMatcher {
    fn new() -> Self {
        Self {
            bids: Arc::new(ArrayQueue::new(DEFAULT_QUEUE_SIZE)),
            asks: Arc::new(ArrayQueue::new(DEFAULT_QUEUE_SIZE)),
            best_bid: None,
            best_ask: None,
        }
    }

    #[inline(always)]
    fn add_order(&mut self, order: Order) -> bool {
        match order.order_type {
            crate::types::order::OrderSide::Buy => {
                let price = order.price;
                if self.bids.push(order).is_ok() {
                    self.best_bid = Some(self.best_bid.map_or(price, |current| current.max(price)));
                    true
                } else {
                    false
                }
            }
            crate::types::order::OrderSide::Sell => {
                let price = order.price;
                if self.asks.push(order).is_ok() {
                    self.best_ask = Some(self.best_ask.map_or(price, |current| current.min(price)));
                    true
                } else {
                    false
                }
            }
        }
    }

    #[inline(always)]
    unsafe fn add_order_unchecked(&mut self, order: Order) {
        match order.order_type {
            crate::types::order::OrderSide::Buy => {
                let price = order.price;
                let _ = self.bids.force_push(order);
                self.best_bid = Some(self.best_bid.map_or(price, |current| current.max(price)));
            }
            crate::types::order::OrderSide::Sell => {
                let price = order.price;
                let _ = self.asks.force_push(order);
                self.best_ask = Some(self.best_ask.map_or(price, |current| current.min(price)));
            }
        }
    }

    #[inline(always)]
    fn match_orders(&mut self) {
        let mut matched_count = 0;
        let max_matches = 100;
        
        for _ in 0..max_matches {
            if !self.can_match_optimistic() {
                break;
            }
            
            match (self.bids.pop(), self.asks.pop()) {
                (Some(bid_order), Some(ask_order)) => {
                    if bid_order.price >= ask_order.price {
                        matched_count += 1;
                    } else {
                        let _ = self.bids.push(bid_order);
                        let _ = self.asks.push(ask_order);
                        break;
                    }
                }
                _ => break,
            }
        }
        
        if matched_count > 0 {
            self.recalculate_best_prices();
        }
    }

    #[inline(always)]
    fn can_match_optimistic(&self) -> bool {
        match (self.best_bid, self.best_ask) {
            (Some(bid), Some(ask)) => bid >= ask && !self.bids.is_empty() && !self.asks.is_empty(),
            _ => false,
        }
    }

    #[inline(always)]
    fn recalculate_best_prices(&mut self) {
        self.best_bid = None;
        self.best_ask = None;
    }

    #[inline(always)]
    fn get_best_prices(&self) -> (Option<u64>, Option<u64>) {
        (self.best_bid, self.best_ask)
    }

    #[inline(always)]
    fn can_match(&self) -> bool {
        if self.bids.is_empty() || self.asks.is_empty() {
            return false;
        }
        
        match (self.best_bid, self.best_ask) {
            (Some(bid), Some(ask)) => bid >= ask,
            _ => false,
        }
    }

    #[inline(always)]
    fn queue_capacities(&self) -> (usize, usize) {
        (self.bids.capacity(), self.asks.capacity())
    }

    #[inline(always)]
    fn queue_lengths(&self) -> (usize, usize) {
        (self.bids.len(), self.asks.len())
    }

    #[inline(always)]
    fn is_empty(&self) -> bool {
        self.bids.is_empty() && self.asks.is_empty()
    }
}

#[repr(align(64))]
pub struct ArrayQueueOrderBook {
    symbols: FxHashSet<SymbolId>,
    matchers: FxHashMap<SymbolId, ArrayQueueMatcher>,
}

impl OrderBookTrait for ArrayQueueOrderBook {
    fn new(symbols: FxHashSet<SymbolId>) -> Self {
        let mut matchers = FxHashMap::with_capacity_and_hasher(symbols.len(), Default::default());
        for &symbol in &symbols {
            matchers.insert(symbol, ArrayQueueMatcher::new());
        }
        ArrayQueueOrderBook { symbols, matchers }
    }

    #[inline(always)]
    fn add_order(&mut self, order: Order) -> Result<bool, OrderBookError> {
        if let Some(matcher) = self.matchers.get_mut(&order.symbol) {
            Ok(matcher.add_order(order))
        } else {
            Err(OrderBookError::InvalidSymbol)
        }
    }

    #[inline(always)]
    fn add_order_fast(&mut self, order: Order) -> bool {
        if let Some(matcher) = self.matchers.get_mut(&order.symbol) {
            matcher.add_order(order)
        } else {
            false
        }
    }

    #[inline(always)]
    unsafe fn add_order_unchecked(&mut self, order: Order) {
        unsafe {
            self.matchers.get_mut(&order.symbol)
                .unwrap_unchecked()
                .add_order_unchecked(order);
        }
    }

    #[inline(always)]
    fn match_orders(&mut self) {
        for matcher in self.matchers.values_mut() {
            matcher.match_orders();
        }
    }

    #[inline(always)]
    fn add_orders_batch_fast(&mut self, orders: &[Order]) -> (u32, u32) {
        let mut successful = 0;
        let mut failed = 0;
        
        for order in orders {
            if self.add_order_fast(order.clone()) {
                successful += 1;
            } else {
                failed += 1;
            }
        }
        
        (successful, failed)
    }

    #[inline(always)]
    unsafe fn add_orders_batch_unchecked(&mut self, orders: &[Order]) -> u32 {
        for order in orders {
            unsafe { self.add_order_unchecked(order.clone()); }
        }
        orders.len() as u32
    }

    #[inline(always)]
    fn get_best_prices(&self, symbol: SymbolId) -> Option<(Option<u64>, Option<u64>)> {
        self.matchers.get(&symbol)
            .map(|matcher| matcher.get_best_prices())
    }

    #[inline(always)]
    fn can_match(&self, symbol: SymbolId) -> bool {
        self.matchers.get(&symbol)
            .is_some_and(|matcher| matcher.can_match())
    }

    #[inline(always)]
    fn is_valid_symbol(&self, symbol: SymbolId) -> bool {
        self.symbols.contains(&symbol)
    }

    #[inline(always)]
    fn get_symbols(&self) -> &FxHashSet<SymbolId> {
        &self.symbols
    }

    #[inline(always)]
    fn order_book_type(&self) -> OrderBookType {
        OrderBookType::ArrayQueue
    }
}

impl ArrayQueueOrderBook {
    #[inline(always)]
    pub fn get_queue_stats(&self, symbol: SymbolId) -> Option<(usize, usize, usize, usize)> {
        self.matchers.get(&symbol).map(|matcher| {
            let (bid_cap, ask_cap) = matcher.queue_capacities();
            let (bid_len, ask_len) = matcher.queue_lengths();
            (bid_cap, ask_cap, bid_len, ask_len)
        })
    }
    
    #[inline(always)]
    pub fn is_symbol_empty(&self, symbol: SymbolId) -> bool {
        self.matchers.get(&symbol)
            .is_none_or(|matcher| matcher.is_empty())
    }
    
    #[inline(always)]
    pub fn queue_utilization(&self, symbol: SymbolId) -> Option<(f64, f64)> {
        self.matchers.get(&symbol).map(|matcher| {
            let (bid_cap, ask_cap) = matcher.queue_capacities();
            let (bid_len, ask_len) = matcher.queue_lengths();
            
            let bid_util = bid_len as f64 / bid_cap as f64;
            let ask_util = ask_len as f64 / ask_cap as f64;
            
            (bid_util, ask_util)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::order::{new_order, OrderSide};

    const APPLE_SYMBOL: SymbolId = 0;

    #[test]
    fn test_basic_operations() {
        let mut order_book = ArrayQueueOrderBook::new(FxHashSet::from_iter([APPLE_SYMBOL]));
        
        let order = new_order(1, APPLE_SYMBOL, 100, 150.0, OrderSide::Buy);
        assert!(order_book.add_order_fast(order));
        
        let stats = order_book.get_queue_stats(APPLE_SYMBOL);
        assert!(stats.is_some());
        
        let utilization = order_book.queue_utilization(APPLE_SYMBOL);
        assert!(utilization.is_some());
        let (bid_util, ask_util) = utilization.unwrap();
        assert!(bid_util > 0.0);
        assert_eq!(ask_util, 0.0);
    }

    #[test]
    fn test_batch_operations() {
        let mut order_book = ArrayQueueOrderBook::new(FxHashSet::from_iter([APPLE_SYMBOL]));
        
        let orders = vec![
            new_order(1, APPLE_SYMBOL, 100, 150.0, OrderSide::Buy),
            new_order(2, APPLE_SYMBOL, 50, 149.0, OrderSide::Buy),
            new_order(3, APPLE_SYMBOL, 75, 151.0, OrderSide::Sell),
        ];
        
        let (successful, failed) = order_book.add_orders_batch_fast(&orders);
        assert_eq!(successful, 3);
        assert_eq!(failed, 0);
    }
} 