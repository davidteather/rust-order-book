use std::collections::BinaryHeap;
use std::cmp::Ordering;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::engine::order_book_trait::{OrderBookTrait, OrderBookError};
use crate::engine::OrderBookType;
use crate::types::{order::Order, symbol_mapping::SymbolId};

#[derive(Debug, Clone)]
struct BidOrder(Order);

impl PartialEq for BidOrder {
    fn eq(&self, other: &Self) -> bool {
        self.0.price == other.0.price && self.0.id == other.0.id
    }
}

impl Eq for BidOrder {}

impl PartialOrd for BidOrder {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for BidOrder {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.0.price.cmp(&other.0.price) {
            Ordering::Equal => other.0.id.cmp(&self.0.id),
            other => other
        }
    }
}

#[derive(Debug, Clone)]
struct AskOrder(Order);

impl PartialEq for AskOrder {
    fn eq(&self, other: &Self) -> bool {
        self.0.price == other.0.price && self.0.id == other.0.id
    }
}

impl Eq for AskOrder {}

impl PartialOrd for AskOrder {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AskOrder {
    fn cmp(&self, other: &Self) -> Ordering {
        match other.0.price.cmp(&self.0.price) {
            Ordering::Equal => other.0.id.cmp(&self.0.id),
            other => other
        }
    }
}

#[derive(Debug)]
struct PriorityQueueMatcher {
    bids: BinaryHeap<BidOrder>,
    asks: BinaryHeap<AskOrder>,
    best_bid: Option<u64>,
    best_ask: Option<u64>,
}

impl PriorityQueueMatcher {
    fn new() -> Self {
        Self {
            bids: BinaryHeap::new(),
            asks: BinaryHeap::new(),
            best_bid: None,
            best_ask: None,
        }
    }

    #[inline(always)]
    fn add_order(&mut self, order: Order) {
        match order.order_type {
            crate::types::order::OrderSide::Buy => {
                let price = order.price;
                self.bids.push(BidOrder(order));
                self.best_bid = Some(self.best_bid.map_or(price, |current| current.max(price)));
            }
            crate::types::order::OrderSide::Sell => {
                let price = order.price;
                self.asks.push(AskOrder(order));
                self.best_ask = Some(self.best_ask.map_or(price, |current| current.min(price)));
            }
        }
    }

    #[inline(always)]
    fn can_match(&self) -> bool {
        match (self.best_bid, self.best_ask) {
            (Some(bid), Some(ask)) => bid >= ask,
            _ => false,
        }
    }

    #[inline(always)]
    fn match_orders(&mut self) {
        while self.can_match() {
            let bid = self.bids.pop();
            let ask = self.asks.pop();

            match (bid, ask) {
                (Some(_), Some(_)) => {
                    self.best_bid = self.bids.peek().map(|order| order.0.price);
                    self.best_ask = self.asks.peek().map(|order| order.0.price);
                }
                _ => break,
            }
        }
    }

    #[inline(always)]
    fn get_best_prices(&self) -> (Option<u64>, Option<u64>) {
        let best_bid = self.bids.peek().map(|order| order.0.price);
        let best_ask = self.asks.peek().map(|order| order.0.price);
        (best_bid, best_ask)
    }
}

#[repr(align(64))]
pub struct PriorityQueueOrderBook {
    symbols: FxHashSet<SymbolId>,
    matchers: FxHashMap<SymbolId, PriorityQueueMatcher>,
}

impl OrderBookTrait for PriorityQueueOrderBook {
    fn new(symbols: FxHashSet<SymbolId>) -> Self {
        let mut matchers = FxHashMap::with_capacity_and_hasher(symbols.len(), Default::default());
        for &symbol in &symbols {
            matchers.insert(symbol, PriorityQueueMatcher::new());
        }
        Self { symbols, matchers }
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
            let matcher = self.matchers.get_mut(&order.symbol).unwrap_unchecked();
            matcher.add_order(order);
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
        let mut added = 0u32;
        let mut rejected = 0u32;

        for order in orders {
            if self.add_order_fast(order.clone()) {
                added += 1;
            } else {
                rejected += 1;
            }
        }

        (added, rejected)
    }

    #[inline(always)]
    unsafe fn add_orders_batch_unchecked(&mut self, orders: &[Order]) -> u32 {
        let mut added = 0u32;

        for order in orders {
            unsafe { self.add_order_unchecked(order.clone()); }
            added += 1;
        }

        added
    }

    #[inline(always)]
    fn get_best_prices(&self, symbol: SymbolId) -> Option<(Option<u64>, Option<u64>)> {
        self.matchers.get(&symbol).map(|matcher| matcher.get_best_prices())
    }

    #[inline(always)]
    fn can_match(&self, symbol: SymbolId) -> bool {
        self.matchers.get(&symbol)
            .map(|matcher| matcher.can_match())
            .unwrap_or(false)
    }

    #[inline(always)]
    fn is_valid_symbol(&self, symbol: SymbolId) -> bool {
        self.matchers.contains_key(&symbol)
    }

    #[inline(always)]
    fn get_symbols(&self) -> &FxHashSet<SymbolId> {
        &self.symbols
    }

    #[inline(always)]
    fn order_book_type(&self) -> OrderBookType {
        OrderBookType::PriorityQueue
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::order::{new_order, OrderSide};
    use crate::engine::order_book_trait::OrderBookTrait;

    const APPLE_SYMBOL: SymbolId = 0;

    #[test]
    fn test_priority_queue_order_book() {
        let mut order_book = PriorityQueueOrderBook::new(FxHashSet::from_iter([APPLE_SYMBOL]));
        
        let buy_order = new_order(1, APPLE_SYMBOL, 100, 150.0, OrderSide::Buy);
        assert!(order_book.add_order(buy_order).is_ok());
        
        let sell_order = new_order(2, APPLE_SYMBOL, 100, 151.0, OrderSide::Sell);
        assert!(order_book.add_order(sell_order).is_ok());
        
        assert!(!order_book.can_match(APPLE_SYMBOL));
        
        let matching_sell = new_order(3, APPLE_SYMBOL, 50, 150.0, OrderSide::Sell);
        assert!(order_book.add_order(matching_sell).is_ok());
        
        assert!(order_book.can_match(APPLE_SYMBOL));

        assert_eq!(order_book.order_book_type(), OrderBookType::PriorityQueue);
    }

    #[test]
    fn test_price_time_priority() {
        let mut order_book = PriorityQueueOrderBook::new(FxHashSet::from_iter([APPLE_SYMBOL]));
        
        let buy1 = new_order(1, APPLE_SYMBOL, 100, 150.0, OrderSide::Buy);
        let buy2 = new_order(2, APPLE_SYMBOL, 100, 150.0, OrderSide::Buy);
        
        order_book.add_order(buy2.clone()).unwrap();
        order_book.add_order(buy1.clone()).unwrap();
        
        let best_prices = order_book.get_best_prices(APPLE_SYMBOL).unwrap();
        assert_eq!(best_prices.0, Some(crate::types::order::price_to_u64(150.0)));
    }
} 