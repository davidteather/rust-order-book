use criterion::{criterion_group, criterion_main, BatchSize, Criterion, Throughput, BenchmarkId};

use rust_order_book::engine::{OrderBookType, create_order_book};
use rust_order_book::types::order::{new_order, OrderSide};

mod shared_benchmark;
use shared_benchmark::*;

const ORDER_BOOK_TYPES: &[OrderBookType] = &[
    OrderBookType::HashMap,
    OrderBookType::PriorityQueue,
    OrderBookType::ArrayQueue,
];

fn get_impl_name(order_book_type: OrderBookType) -> &'static str {
    match order_book_type {
        OrderBookType::HashMap => "hashmap",
        OrderBookType::PriorityQueue => "priorityqueue", 
        OrderBookType::ArrayQueue => "arrayqueue",
    }
}

fn bench_add_order_single(c: &mut Criterion) {
    let mut group = c.benchmark_group("add_order");
    group.throughput(Throughput::Elements(1));
    let data = BenchmarkData::new();
    
    for &order_book_type in ORDER_BOOK_TYPES {
        let impl_name = get_impl_name(order_book_type);
        
        group.bench_with_input(
            BenchmarkId::new(impl_name, "single_safe"),
            &order_book_type,
            |b, &impl_type| {
                b.iter_batched(
                    || create_order_book(impl_type, data.symbols.clone()),
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
            &order_book_type,
            |b, &impl_type| {
                b.iter_batched(
                    || create_order_book(impl_type, data.symbols.clone()),
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
            &order_book_type,
            |b, &impl_type| {
                b.iter_batched(
                    || create_order_book(impl_type, data.symbols.clone()),
                    |mut order_book| {
                        let order = data.single_symbol_orders[0].clone();
                        unsafe { order_book.add_order_unchecked(order) }
                    },
                    BatchSize::SmallInput,
                )
            },
        );
    }
    
    group.finish();
}

fn bench_add_order_batch(c: &mut Criterion) {
    let mut group = c.benchmark_group("add_order_batch");
    let data = BenchmarkData::new();
    let batch_sizes = [10, 50, 100, 500];
    
    for &batch_size in &batch_sizes {
        group.throughput(Throughput::Elements(batch_size as u64));
        
        for &order_book_type in ORDER_BOOK_TYPES {
            let impl_name = get_impl_name(order_book_type);
            
            group.bench_with_input(
                BenchmarkId::new(format!("{impl_name}/batch_fast"), batch_size),
                &(order_book_type, batch_size),
                |b, &(impl_type, size)| {
                    b.iter_batched(
                        || create_order_book(impl_type, data.symbols.clone()),
                        |mut order_book| {
                            let batch = &data.single_symbol_orders[0..size.min(data.single_symbol_orders.len())];
                            order_book.add_orders_batch_fast(batch)
                        },
                        BatchSize::SmallInput,
                    )
                },
            );
            
            group.bench_with_input(
                BenchmarkId::new(format!("{impl_name}/batch_unchecked"), batch_size),
                &(order_book_type, batch_size),
                |b, &(impl_type, size)| {
                    b.iter_batched(
                        || create_order_book(impl_type, data.symbols.clone()),
                        |mut order_book| {
                            let batch = &data.single_symbol_orders[0..size.min(data.single_symbol_orders.len())];
                            unsafe { order_book.add_orders_batch_unchecked(batch) }
                        },
                        BatchSize::SmallInput,
                    )
                },
            );
        }
    }
    
    group.finish();
}

fn bench_matching(c: &mut Criterion) {
    let mut group = c.benchmark_group("matching");
    group.throughput(Throughput::Elements(100)); // 50 buy orders + 50 sell orders
    let data = BenchmarkData::new();
    
    for &order_book_type in ORDER_BOOK_TYPES {
        let impl_name = get_impl_name(order_book_type);
        
        group.bench_with_input(
            BenchmarkId::new(impl_name, "match_orders"),
            &order_book_type,
            |b, &impl_type| {
                b.iter_batched(
                    || {
                        let mut order_book = create_order_book(impl_type, data.symbols.clone());
                        for order in &data.matching_buy_orders[0..50] {
                            order_book.add_order_fast(order.clone());
                        }
                        for order in &data.matching_sell_orders[0..50] {
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
    }
    
    group.finish();
}

fn bench_queries(c: &mut Criterion) {
    let mut group = c.benchmark_group("queries");
    group.throughput(Throughput::Elements(1));
    let data = BenchmarkData::new();
    
    for &order_book_type in ORDER_BOOK_TYPES {
        let impl_name = get_impl_name(order_book_type);
        
        group.bench_with_input(
            BenchmarkId::new(impl_name, "get_best_prices"),
            &order_book_type,
            |b, &impl_type| {
                b.iter_batched(
                    || {
                        let mut order_book = create_order_book(impl_type, data.symbols.clone());
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
            &order_book_type,
            |b, &impl_type| {
                b.iter_batched(
                    || {
                        let mut order_book = create_order_book(impl_type, data.symbols.clone());
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
    }
    
    group.finish();
}



criterion_group!(benches,
    bench_add_order_single,
    bench_add_order_batch,
    bench_matching,
    bench_queries
);
criterion_main!(benches); 