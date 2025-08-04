use criterion::{Criterion, criterion_group, criterion_main};
use rust_order_book::engine::factories;

mod shared_benchmark;
use shared_benchmark::*;

fn structured_order_book_benchmarks(c: &mut Criterion) {
    full_benchmark_suite(c, factories::create_hashmap_order_book);
    full_benchmark_suite(c, factories::create_priority_queue_order_book);
    full_benchmark_suite(c, factories::create_array_queue_order_book);
}

fn structured_multi_symbol_comparison(c: &mut Criterion) {
    bench_multi_symbol_generic(c, factories::create_hashmap_order_book);
    bench_multi_symbol_generic(c, factories::create_priority_queue_order_book);
    bench_multi_symbol_generic(c, factories::create_array_queue_order_book);
}

fn structured_high_frequency_trading(c: &mut Criterion) {
    bench_high_frequency_generic(c, factories::create_hashmap_order_book);
    bench_high_frequency_generic(c, factories::create_priority_queue_order_book);
    bench_high_frequency_generic(c, factories::create_array_queue_order_book);
}

fn configure_criterion() -> Criterion {
    Criterion::default()
        .measurement_time(std::time::Duration::from_secs(10))
        .sample_size(100)
}

criterion_group! {
    name = structured_order_book_benches;
    config = configure_criterion();
    targets = structured_order_book_benchmarks, structured_multi_symbol_comparison, structured_high_frequency_trading
}
criterion_main!(structured_order_book_benches);