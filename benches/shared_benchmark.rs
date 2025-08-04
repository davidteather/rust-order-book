use criterion::{BatchSize, BenchmarkId, Criterion, Throughput};
use rustc_hash::FxHashSet;
use rust_order_book::{
    engine::{OrderBookTrait, OrderBookType},
    types::{order::{new_order, Order, OrderSide}, symbol_mapping::SymbolId},
};
use rand::prelude::*;
use rand_distr::{Normal, Distribution};

pub fn get_impl_name(order_book_type: OrderBookType) -> &'static str {
    match order_book_type {
        OrderBookType::HashMap => "hashmap",
        OrderBookType::PriorityQueue => "priorityqueue", 
        OrderBookType::ArrayQueue => "arrayqueue",
    }
}

pub struct MarketSimParams<'a> {
    pub count: usize,
    pub initial_price: f64,
    pub mean_price: f64,
    pub drift: f64,
    pub mean_reversion_strength: f64,
    pub volatility: f64,
    pub symbols: &'a [SymbolId],
}

impl<'a> Default for MarketSimParams<'a> {
    fn default() -> Self {
        static DEFAULT_SYMBOLS: &[SymbolId] = &[0, 1, 2];
        Self {
            count: 1000,
            initial_price: 100.0,
            mean_price: 100.0,
            drift: 0.0001,
            mean_reversion_strength: 0.05,
            volatility: 0.02,
            symbols: DEFAULT_SYMBOLS,
        }
    }
}

pub fn generate_realistic_orders(params: MarketSimParams) -> Vec<Order> {
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
        
        let quantity = rng.gen_range(1..=1000);
        let symbol = params.symbols[i % params.symbols.len()];
        
        let order = new_order(i as u64, symbol, quantity, price, side);
        orders.push(order);
    }
    
    orders
}

pub struct BenchmarkData {
    pub single_symbol_orders: Vec<Order>,
    pub multi_symbol_orders: Vec<Order>,
    pub matching_buy_orders: Vec<Order>,
    pub matching_sell_orders: Vec<Order>,
    pub high_frequency_orders: Vec<Order>,
    pub large_orders: Vec<Order>,
    pub symbols: FxHashSet<SymbolId>,
}

impl Default for BenchmarkData {
    fn default() -> Self {
        Self::new()
    }
}

impl BenchmarkData {
    pub fn new() -> Self {
        let symbols = FxHashSet::from_iter([0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
        
        let single_symbol_orders = generate_realistic_orders(MarketSimParams {
            count: 2000,
            symbols: &[0],
            ..Default::default()
        });
        
        let multi_symbol_orders = generate_realistic_orders(MarketSimParams {
            count: 5000,
            symbols: &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9],
            volatility: 0.03, // Higher volatility for multi-symbol
            ..Default::default()
        });
        
        let matching_buy_orders: Vec<_> = (0..1000)
            .map(|i| new_order(i as u64 * 2, 0, 100, 100.0, OrderSide::Buy))
            .collect();
        
        let matching_sell_orders: Vec<_> = (0..1000)
            .map(|i| new_order(i as u64 * 2 + 1, 0, 100, 100.0, OrderSide::Sell))
            .collect();
        
        let high_frequency_orders = generate_realistic_orders(MarketSimParams {
            count: 10000,
            symbols: &[0, 1, 2],
            volatility: 0.001, // Low volatility, tight spreads
            drift: 0.0,
            ..Default::default()
        });
        
        let large_orders: Vec<_> = (0..100)
            .map(|i| {
                let symbol = [0, 1, 2][i % 3];
                let side = if i % 2 == 0 { OrderSide::Buy } else { OrderSide::Sell };
                let price = 100.0 + (i as f64 * 0.5);
                new_order(i as u64, symbol, (5000 + (i * 100)) as u64, price, side) // Large quantities
            })
            .collect();
        
        Self {
            single_symbol_orders,
            multi_symbol_orders,
            matching_buy_orders,
            matching_sell_orders,
            high_frequency_orders,
            large_orders,
            symbols,
        }
    }
}

#[allow(dead_code)]
pub fn full_benchmark_suite<T>(
    c: &mut Criterion,
    create_order_book: impl Fn(FxHashSet<SymbolId>) -> T + Clone,
) where
    T: OrderBookTrait + 'static,
{
    bench_add_order_generic(c, create_order_book.clone());
    bench_matching_generic(c, create_order_book.clone());
    bench_queries_generic(c, create_order_book.clone());
    bench_multi_symbol_generic(c, create_order_book.clone());
    bench_high_frequency_generic(c, create_order_book);
}

#[allow(dead_code)]
pub fn bench_add_order_generic<T>(
    c: &mut Criterion,
    create_order_book: impl Fn(FxHashSet<SymbolId>) -> T,
) where
    T: OrderBookTrait + 'static,
{
    let order_book_type = create_order_book(FxHashSet::from_iter([0])).order_book_type();
    let impl_name = get_impl_name(order_book_type);
    let mut group = c.benchmark_group("add_order");
    group.throughput(Throughput::Elements(1));
    let data = BenchmarkData::new();

    group.bench_with_input(
        BenchmarkId::new(impl_name, "single_safe"),
        &(),
        |b, _| {
            b.iter_batched(
                || create_order_book(data.symbols.clone()),
                |mut order_book| {
                    let order = data.single_symbol_orders[0].clone();
                    order_book.add_order(order).unwrap()
                },
                BatchSize::SmallInput,
            )
        },
    );

    group.bench_with_input(
        BenchmarkId::new(impl_name, "single_fast"),
        &(),
        |b, _| {
            b.iter_batched(
                || create_order_book(data.symbols.clone()),
                |mut order_book| {
                    let order = data.single_symbol_orders[0].clone();
                    order_book.add_order_fast(order)
                },
                BatchSize::SmallInput,
            )
        },
    );

    group.bench_with_input(
        BenchmarkId::new(impl_name, "single_unchecked"),
        &(),
        |b, _| {
            b.iter_batched(
                || create_order_book(data.symbols.clone()),
                |mut order_book| {
                    let order = data.single_symbol_orders[0].clone();
                    unsafe { order_book.add_order_unchecked(order) }
                },
                BatchSize::SmallInput,
            )
        },
    );

    let batch_sizes = [10, 50, 100, 500];
    for &batch_size in &batch_sizes {
        group.throughput(Throughput::Elements(batch_size as u64));
        group.bench_with_input(
            BenchmarkId::new(format!("{impl_name}/batch_fast"), batch_size),
            &batch_size,
            |b, &size| {
                b.iter_batched(
                    || create_order_book(data.symbols.clone()),
                    |mut order_book| {
                        let orders = &data.single_symbol_orders[0..size];
                        order_book.add_orders_batch_fast(orders)
                    },
                    BatchSize::SmallInput,
                )
            },
        );

        group.bench_with_input(
            BenchmarkId::new(format!("{impl_name}/batch_unchecked"), batch_size),
            &batch_size,
            |b, &size| {
                b.iter_batched(
                    || create_order_book(data.symbols.clone()),
                    |mut order_book| {
                        let orders = &data.single_symbol_orders[0..size];
                        unsafe { order_book.add_orders_batch_unchecked(orders) }
                    },
                    BatchSize::SmallInput,
                )
            },
        );
    }

    group.finish();
}

#[allow(dead_code)]
pub fn bench_multi_symbol_generic<T>(
    c: &mut Criterion,
    create_order_book: impl Fn(FxHashSet<SymbolId>) -> T,
) where
    T: OrderBookTrait + 'static,
{
    let order_book_type = create_order_book(FxHashSet::from_iter([0])).order_book_type();
    let impl_name = get_impl_name(order_book_type);
    let mut group = c.benchmark_group("multi_symbol");
    let data = BenchmarkData::new();

    group.throughput(Throughput::Elements(100));
    group.bench_with_input(
        BenchmarkId::new(impl_name, "cross_symbol_100"),
        &(),
        |b, _| {
            b.iter_batched(
                || create_order_book(data.symbols.clone()),
                |mut order_book| {
                    for order in &data.multi_symbol_orders[0..100] {
                        order_book.add_order_fast(order.clone());
                    }
                },
                BatchSize::SmallInput,
            )
        },
    );

    group.throughput(Throughput::Elements(100)); // 50 pairs * 2 orders
    group.bench_with_input(
        BenchmarkId::new(impl_name, "symbol_switching_100"),
        &(),
        |b, _| {
            b.iter_batched(
                || create_order_book(data.symbols.clone()),
                |mut order_book| {
                    for i in 0..50 {
                        let symbol1_order = &data.multi_symbol_orders[i * 2];
                        let symbol2_order = &data.multi_symbol_orders[i * 2 + 1];
                        order_book.add_order_fast(symbol1_order.clone());
                        order_book.add_order_fast(symbol2_order.clone());
                    }
                },
                BatchSize::SmallInput,
            )
        },
    );

    group.throughput(Throughput::Elements(data.large_orders.len() as u64));
    group.bench_with_input(
        BenchmarkId::new(impl_name, format!("large_orders_{}", data.large_orders.len())),
        &(),
        |b, _| {
            b.iter_batched(
                || create_order_book(data.symbols.clone()),
                |mut order_book| {
                    for order in &data.large_orders {
                        order_book.add_order_fast(order.clone());
                    }
                },
                BatchSize::SmallInput,
            )
        },
    );

    group.finish();
}

#[allow(dead_code)]
pub fn bench_high_frequency_generic<T>(
    c: &mut Criterion,
    create_order_book: impl Fn(FxHashSet<SymbolId>) -> T,
) where
    T: OrderBookTrait + 'static,
{
    let order_book_type = create_order_book(FxHashSet::from_iter([0])).order_book_type();
    let impl_name = get_impl_name(order_book_type);
    let mut group = c.benchmark_group("high_frequency");
    let data = BenchmarkData::new();

    group.throughput(Throughput::Elements(1000));
    group.bench_with_input(
        BenchmarkId::new(impl_name, "rapid_fire_1000"),
        &(),
        |b, _| {
            b.iter_batched(
                || create_order_book(FxHashSet::from_iter([0, 1, 2])),
                |mut order_book| {
                    for order in &data.high_frequency_orders[0..1000] {
                        order_book.add_order_fast(order.clone());
                    }
                },
                BatchSize::SmallInput,
            )
        },
    );

    group.throughput(Throughput::Elements(500));
    group.bench_with_input(
        BenchmarkId::new(impl_name, "burst_matching_500"),
        &(),
        |b, _| {
            b.iter_batched(
                || create_order_book(FxHashSet::from_iter([0])),
                |mut order_book| {
                    // Add burst of orders then trigger matching
                    for order in &data.high_frequency_orders[0..500] {
                        order_book.add_order_fast(order.clone());
                    }
                    order_book.match_orders()
                },
                BatchSize::SmallInput,
            )
        },
    );

    group.finish();
}

#[allow(dead_code)]
pub fn bench_matching_generic<T>(
    c: &mut Criterion,
    create_order_book: impl Fn(FxHashSet<SymbolId>) -> T,
) where
    T: OrderBookTrait + 'static,
{
    let order_book_type = create_order_book(FxHashSet::from_iter([0])).order_book_type();
    let impl_name = get_impl_name(order_book_type);
    let mut group = c.benchmark_group("matching");
    let data = BenchmarkData::new();

    group.throughput(Throughput::Elements(200)); // 100 buy + 100 sell orders
    group.bench_with_input(
        BenchmarkId::new(impl_name, "match_orders_200"),
        &(),
        |b, _| {
            b.iter_batched(
                || {
                    let mut order_book = create_order_book(data.symbols.clone());
                    for order in &data.matching_buy_orders[0..100] {
                        order_book.add_order_fast(order.clone());
                    }
                    for order in &data.matching_sell_orders[0..100] {
                        order_book.add_order_fast(order.clone());
                    }
                    order_book
                },
                |mut order_book| {
                    order_book.match_orders()
                },
                BatchSize::SmallInput,
            )
        },
    );



    group.finish();
}

#[allow(dead_code)]
pub fn bench_queries_generic<T>(
    c: &mut Criterion,
    create_order_book: impl Fn(FxHashSet<SymbolId>) -> T,
) where
    T: OrderBookTrait + 'static,
{
    let order_book_type = create_order_book(FxHashSet::from_iter([0])).order_book_type();
    let impl_name = get_impl_name(order_book_type);
    let mut group = c.benchmark_group("queries");
    group.throughput(Throughput::Elements(1));
    let data = BenchmarkData::new();

    group.bench_with_input(
        BenchmarkId::new(impl_name, "get_best_prices"),
        &(),
        |b, _| {
            b.iter_batched(
                || {
                    let mut order_book = create_order_book(data.symbols.clone());
                    for order in &data.single_symbol_orders[0..100] {
                        order_book.add_order_fast(order.clone());
                    }
                    order_book
                },
                |order_book| {
                    order_book.get_best_prices(0)
                },
                BatchSize::SmallInput,
            )
        },
    );

    group.bench_with_input(
        BenchmarkId::new(impl_name, "can_match"),
        &(),
        |b, _| {
            b.iter_batched(
                || {
                    let mut order_book = create_order_book(data.symbols.clone());
                    order_book.add_order_fast(new_order(1, 0, 100, 100.0, OrderSide::Buy));
                    order_book.add_order_fast(new_order(2, 0, 100, 100.0, OrderSide::Sell));
                    order_book
                },
                |order_book| {
                    order_book.can_match(0)
                },
                BatchSize::SmallInput,
            )
        },
    );

    group.bench_with_input(
        BenchmarkId::new(impl_name, "is_valid_symbol"),
        &(),
        |b, _| {
            b.iter_batched(
                || create_order_book(data.symbols.clone()),
                |order_book| {
                    order_book.is_valid_symbol(0) && order_book.is_valid_symbol(999)
                },
                BatchSize::SmallInput,
            )
        },
    );

    // Multi-symbol query performance
    group.throughput(Throughput::Elements(5)); // 5 symbols queried
    group.bench_with_input(
        BenchmarkId::new(impl_name, "multi_symbol_best_prices_5"),
        &(),
        |b, _| {
            b.iter_batched(
                || {
                    let mut order_book = create_order_book(data.symbols.clone());
                    for order in &data.multi_symbol_orders[0..500] {
                        order_book.add_order_fast(order.clone());
                    }
                    order_book
                },
                |order_book| {
                    // Query best prices across multiple symbols
                    for symbol in [0, 1, 2, 3, 4] {
                        order_book.get_best_prices(symbol);
                    }
                },
                BatchSize::SmallInput,
            )
        },
    );

    group.finish();
}