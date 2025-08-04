use crate::{engine::OrderBookType, types::{order::Order, symbol_mapping::SymbolId}};
use rustc_hash::FxHashSet;

#[derive(Debug, Clone, Copy)]
pub enum OrderBookError {
    InvalidSymbol,
}

pub trait OrderBookTrait: Send + Sync {
    fn new(symbols: FxHashSet<SymbolId>) -> Self where Self: Sized;
    
    fn add_order(&mut self, order: Order) -> Result<bool, OrderBookError>;
    
    fn add_order_fast(&mut self, order: Order) -> bool;
    
    /// # Safety
    /// Caller must guarantee that the symbol is valid.
    unsafe fn add_order_unchecked(&mut self, order: Order);
    
    fn match_orders(&mut self);
    
    fn add_orders_batch_fast(&mut self, orders: &[Order]) -> (u32, u32);
    
    /// # Safety
    /// Caller must guarantee that all symbols are valid.
    unsafe fn add_orders_batch_unchecked(&mut self, orders: &[Order]) -> u32;
    
    fn get_best_prices(&self, symbol: SymbolId) -> Option<(Option<u64>, Option<u64>)>;
    
    fn can_match(&self, symbol: SymbolId) -> bool;
    
    fn is_valid_symbol(&self, symbol: SymbolId) -> bool;
    
    fn get_symbols(&self) -> &FxHashSet<SymbolId>;

    fn order_book_type(&self) -> OrderBookType;
}