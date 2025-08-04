use rust_order_book::{
    engine::OrderBookType,
    types::{order::{new_order, OrderSide}, symbol_mapping::SymbolId},
    router::OrderRouter,
};
use rustc_hash::FxHashSet;

const APPLE_SYMBOL: SymbolId = 0;
const GOOGLE_SYMBOL: SymbolId = 1;

fn main() {
    let symbols = FxHashSet::from_iter([APPLE_SYMBOL, GOOGLE_SYMBOL]);
    let mut router = OrderRouter::new_direct(symbols.clone(), OrderBookType::HashMap);
    
    let orders = vec![
        new_order(1, APPLE_SYMBOL, 100, 150.0, OrderSide::Buy),
        new_order(2, APPLE_SYMBOL, 50, 151.0, OrderSide::Sell),
        new_order(3, GOOGLE_SYMBOL, 200, 2500.0, OrderSide::Buy),
        new_order(4, GOOGLE_SYMBOL, 100, 2501.0, OrderSide::Sell),
    ];
    
    for order in orders {
        let _ = router.route_order(order);
    }
    
    router.match_all_orders();
}