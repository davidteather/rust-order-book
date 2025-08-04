use rustc_hash::FxHashSet;
use std::collections::{BTreeMap, VecDeque};

use crate::engine::order_book_trait::{OrderBookTrait, OrderBookError};
use crate::engine::OrderBookType;
use crate::types::{order::{self, Order}, symbol_mapping::SymbolId};

#[repr(align(64))]
#[derive(Debug)]
struct PriceLevel {
    orders: VecDeque<order::Order>,
    count: u32,
    total_quantity: u64,
    _padding: [u8; 28],
}

impl PriceLevel {
    fn new() -> Self {
        Self {
            orders: VecDeque::with_capacity(128),
            count: 0,
            total_quantity: 0,
            _padding: [0; 28],
        }
    }

    #[inline(always)]
    fn is_empty(&self) -> bool {
        self.count == 0
    }

    #[inline(always)]
    fn push_back(&mut self, order: order::Order) {
        self.total_quantity += order.quantity;
        self.count += 1;
        self.orders.push_back(order);
    }

    #[inline(always)]
    fn pop_front(&mut self) -> Option<order::Order> {
        if let Some(order) = self.orders.pop_front() {
            self.total_quantity -= order.quantity;
            self.count -= 1;
            Some(order)
        } else {
            None
        }
    }
}

#[repr(align(64))]
#[derive(Debug)]
struct HashMapMatcher {
    bid_levels: BTreeMap<u64, PriceLevel>,
    ask_levels: BTreeMap<u64, PriceLevel>,
    _padding: [u8; 48],
}

impl HashMapMatcher {
    pub fn new() -> Self {
        Self {
            bid_levels: BTreeMap::new(),
            ask_levels: BTreeMap::new(),
            _padding: [0; 48],
        }
    }

    #[inline(always)]
    pub fn add_order(&mut self, order: order::Order) {
        let price = order.price;
        
        match order.order_type {
            order::OrderSide::Buy => {
                self.bid_levels.entry(price)
                    .or_insert_with(PriceLevel::new)
                    .push_back(order);
            }
            order::OrderSide::Sell => {
                self.ask_levels.entry(price)
                    .or_insert_with(PriceLevel::new)
                    .push_back(order);
            }
        }
    }

    #[inline(always)]
    unsafe fn add_order_unchecked(&mut self, order: order::Order) {
        let price = order.price;
        
        match order.order_type {
            order::OrderSide::Buy => {
                self.bid_levels.entry(price)
                    .or_insert_with(PriceLevel::new)
                    .push_back(order);
            }
            order::OrderSide::Sell => {
                self.ask_levels.entry(price)
                    .or_insert_with(PriceLevel::new)
                    .push_back(order);
            }
        }
    }

    pub fn match_orders(&mut self) {
        loop {
            let can_match = match (self.get_best_bid(), self.get_best_ask()) {
                (Some(bid_price), Some(ask_price)) => bid_price >= ask_price,
                _ => false,
            };

            if !can_match {
                break;
            }

            let bid_price = self.get_best_bid().unwrap();
            let ask_price = self.get_best_ask().unwrap();

            let bid_order = self.bid_levels.get_mut(&bid_price)
                .and_then(|level| level.pop_front());
            let ask_order = self.ask_levels.get_mut(&ask_price)
                .and_then(|level| level.pop_front());

            match (bid_order, ask_order) {
                (Some(_bid_order), Some(_ask_order)) => {
                    if self.bid_levels.get(&bid_price).is_none_or(|level| level.is_empty()) {
                        self.bid_levels.remove(&bid_price);
                    }
                    if self.ask_levels.get(&ask_price).is_none_or(|level| level.is_empty()) {
                        self.ask_levels.remove(&ask_price);
                    }
                }
                _ => break,
            }
        }
    }

    #[inline(always)]
    fn get_best_bid(&self) -> Option<u64> {
        self.bid_levels.iter()
            .rev()
            .find(|(_, level)| !level.is_empty())
            .map(|(&price, _)| price)
    }

    #[inline(always)]
    fn get_best_ask(&self) -> Option<u64> {
        self.ask_levels.iter()
            .find(|(_, level)| !level.is_empty())
            .map(|(&price, _)| price)
    }

    #[inline(always)]
    pub fn get_best_prices(&self) -> (Option<u64>, Option<u64>) {
        (self.get_best_bid(), self.get_best_ask())
    }

    pub fn can_match(&self) -> bool {
        match (self.get_best_bid(), self.get_best_ask()) {
            (Some(bid), Some(ask)) => bid >= ask,
            _ => false,
        }
    }
}

#[repr(align(64))]
pub struct HashMapOrderBook {
    symbols: FxHashSet<SymbolId>,
    matchers: rustc_hash::FxHashMap<SymbolId, HashMapMatcher>,
}

impl OrderBookTrait for HashMapOrderBook {
    fn new(symbols: FxHashSet<SymbolId>) -> Self {
        let mut matchers = rustc_hash::FxHashMap::with_capacity_and_hasher(symbols.len(), Default::default());
        for &symbol in &symbols {
            matchers.insert(symbol, HashMapMatcher::new());
        }
        HashMapOrderBook { 
            symbols, 
            matchers,
        }
    }

    #[inline(always)]
    fn add_order(&mut self, order: Order) -> Result<bool, OrderBookError> {
        if let Some(matcher) = self.matchers.get_mut(&order.symbol) {
            matcher.add_order(order);
            Ok(true)
        } else {
            Err(OrderBookError::InvalidSymbol)
        }
    }

    #[inline(always)]
    fn add_order_fast(&mut self, order: Order) -> bool {
        if let Some(matcher) = self.matchers.get_mut(&order.symbol) {
            matcher.add_order(order);
            true
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
        OrderBookType::HashMap
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::order::{new_order, OrderSide};

    const APPLE_SYMBOL: SymbolId = 0;

    #[test]
    fn test_hashmap_order_book_basic() {
        let mut order_book = HashMapOrderBook::new(FxHashSet::from_iter([APPLE_SYMBOL]));
        let order = new_order(1, APPLE_SYMBOL, 100, 150.0, OrderSide::Buy);
        
        assert!(order_book.add_order(order).is_ok());
        assert_eq!(order_book.get_symbols(), &FxHashSet::from_iter([APPLE_SYMBOL]));
    }

    #[test]
    fn test_hashmap_matcher_best_prices() {
        let mut matcher = HashMapMatcher::new();
        
        let buy_order = new_order(1, 0, 100, 99.50, OrderSide::Buy);
        let sell_order = new_order(2, 0, 100, 100.50, OrderSide::Sell);
        
        matcher.add_order(buy_order);
        matcher.add_order(sell_order);
        
        assert_eq!(matcher.get_best_bid(), Some(99500)); // 99.50 * 1000
        assert_eq!(matcher.get_best_ask(), Some(100500)); // 100.50 * 1000
        assert!(!matcher.can_match());
    }
} 