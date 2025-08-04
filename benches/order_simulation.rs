use rand::prelude::*;
use rand_distr::{Normal, Distribution};
use rust_order_book::types::order::{Order, OrderSide, new_order};

/*
    Generate orders using an Ornstein-Uhlenbeck process.
    This simulates a market where prices revert to a mean with some drift and volatility.
    Useful for generating more "realistic" order flows in simulations.
*/

// MarketSimParams defines the parameters for generating orders using an Ornstein-Uhlenbeck process.
pub struct MarketSimParams<'a> {
    pub count: usize,
    pub initial_price: f64,
    pub mean_price: f64,
    pub drift: f64,
    pub mean_reversion_strength: f64,
    pub volatility: f64,
    pub symbols: &'a Vec<u16>,
}

pub fn generate_ou_orders<'a>(params: MarketSimParams<'a>) -> Vec<Order> {
    let mut rng = thread_rng();
    let normal = Normal::new(0.0, 1.0).unwrap();
    let mut price = params.initial_price;
    let mut orders = Vec::with_capacity(params.count);
    let side_dist = rand::distributions::Bernoulli::new(0.5).unwrap();

    for i in 0..params.count {
        let shock = normal.sample(&mut rng);

        let reversion = params.mean_reversion_strength * (params.mean_price - price);
        let drift_term = params.drift + reversion;
        let noise_term = params.volatility * shock;
        let log_return = drift_term + noise_term;

        price = (price * log_return.exp()).max(0.01);

        let side = if side_dist.sample(&mut rng) {
            OrderSide::Buy
        } else {
            OrderSide::Sell
        };

        let quantity = rng.gen_range(1..=100);
        let symbol = params.symbols[i % params.symbols.len()];

        let order = new_order(
            i as u64,
            symbol,
            quantity,
            price,
            side,
        );

        orders.push(order);
    }

    orders
}
