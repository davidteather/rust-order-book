use criterion::{Criterion, criterion_group, criterion_main, BatchSize, Throughput, BenchmarkId};
use rustc_hash::FxHashSet;
use rust_order_book::{
    engine::OrderBookType,
    router::OrderRouter,
    types::order::{new_order, OrderSide},
};

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

fn bench_routing_single(c: &mut Criterion) {
    let mut group = c.benchmark_group("routing");
    group.throughput(Throughput::Elements(1));
    
    for &order_book_type in ORDER_BOOK_TYPES {
        let impl_name = get_impl_name(order_book_type);
        
        group.bench_with_input(
            BenchmarkId::new(impl_name, "single_order"),
            &order_book_type,
            |b, &impl_type| {
                b.iter_batched(
                    || OrderRouter::new_direct(FxHashSet::from_iter([0]), impl_type),
                    |mut router| {
                        let order = new_order(1, 0, 100, 100.0, OrderSide::Buy);
                        router.route_order(order).is_ok()
                    },
                    BatchSize::SmallInput,
                )
            },
        );
    }
    
    group.finish();
}

fn bench_routing_multi_symbol(c: &mut Criterion) {
    let mut group = c.benchmark_group("routing_multi_symbol");
    group.throughput(Throughput::Elements(3)); // 3 orders processed
    let symbols = FxHashSet::from_iter([0, 1, 2]);
    
    for &order_book_type in ORDER_BOOK_TYPES {
        let impl_name = get_impl_name(order_book_type);
        
        group.bench_with_input(
            BenchmarkId::new(impl_name, "multi_symbol_3"),
            &order_book_type,
            |b, &impl_type| {
                b.iter_batched(
                    || OrderRouter::new_direct(symbols.clone(), impl_type),
                    |mut router| {
                        let orders = vec![
                            new_order(1, 0, 100, 100.0, OrderSide::Buy),
                            new_order(2, 1, 50, 200.0, OrderSide::Sell),
                            new_order(3, 2, 75, 300.0, OrderSide::Buy),
                        ];
                        let mut success_count = 0;
                        for order in orders {
                            if router.route_order(order).is_ok() {
                                success_count += 1;
                            }
                        }
                        success_count
                    },
                    BatchSize::SmallInput,
                )
            },
        );
    }
    
    group.finish();
}

fn bench_routing_bulk(c: &mut Criterion) {
    let mut group = c.benchmark_group("routing_bulk");
    let symbols = FxHashSet::from_iter([0, 1, 2, 3, 4]);
    let batch_sizes = [50, 100, 500];
    
    for &batch_size in &batch_sizes {
        group.throughput(Throughput::Elements(batch_size as u64));
        
        let bulk_orders: Vec<_> = (0..batch_size)
            .map(|i| new_order(i as u64, i % 5, 100, 100.0 + (i as f64 * 0.1), 
                              if i % 2 == 0 { OrderSide::Buy } else { OrderSide::Sell }))
            .collect();
        
        for &order_book_type in ORDER_BOOK_TYPES {
            let impl_name = get_impl_name(order_book_type);
            
            group.bench_with_input(
                BenchmarkId::new(format!("{impl_name}/bulk"), batch_size),
                &(order_book_type, bulk_orders.clone()),
                |b, (impl_type, orders)| {
                    b.iter_batched(
                        || OrderRouter::new_direct(symbols.clone(), *impl_type),
                        |mut router| {
                            let mut success_count = 0;
                            for order in orders {
                                if router.route_order(order.clone()).is_ok() {
                                    success_count += 1;
                                }
                            }
                            success_count
                        },
                        BatchSize::SmallInput,
                    )
                },
            );
        }
    }
    
    group.finish();
}

fn bench_routing_error_handling(c: &mut Criterion) {
    let mut group = c.benchmark_group("routing_errors");
    group.throughput(Throughput::Elements(1));
    
    for &order_book_type in ORDER_BOOK_TYPES {
        let impl_name = get_impl_name(order_book_type);
        
        group.bench_with_input(
            BenchmarkId::new(impl_name, "invalid_symbol"),
            &order_book_type,
            |b, &impl_type| {
                b.iter_batched(
                    || OrderRouter::new_direct(FxHashSet::from_iter([0]), impl_type),
                    |mut router| {
                        let order = new_order(1, 999, 100, 100.0, OrderSide::Buy); // Invalid symbol
                        router.route_order(order).is_err()
                    },
                    BatchSize::SmallInput,
                )
            },
        );
    }
    
    group.finish();
}

criterion_group!(benches, 
    bench_routing_single,
    bench_routing_multi_symbol, 
    bench_routing_bulk,
    bench_routing_error_handling
);
criterion_main!(benches);
