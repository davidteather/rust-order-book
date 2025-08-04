# Rust Order Book

A simple limit order book implementation written in Rust as a learning project. Features multiple backend implementations (HashMap, PriorityQueue, ArrayQueue) to explore different data structures and performance characteristics.

I'm not Rust expert (my first rust project), so probably a lot of things to learn and improve.

## What I Built

This started as an interview question about designing an order book, which I expanded into a Rust learning project to learn rust.

Features:
- Trait-based design and polymorphism
- Performance benchmarking with different data structures  
- Memory layout optimization with alignment and padding
- Unsafe Rust for performance-critical paths
- Function inlining

## Quick Start

```bash
cargo run          # Run example
cargo test         # Run tests
cargo bench        # Run benchmarks
```

See [benchmarks.md](benchmarks.md) for detailed performance comparisons between implementations.

Some highlights from benches on Macbook Pro M2 

## Performance Highlights

Based on our comprehensive benchmark suite, here are the key performance metrics:

### Order Submission Performance
- **PriorityQueue**: Up to **112.3 million orders/second** (batch processing)
- **HashMap**: Up to **13.0 million orders/second** (batch processing)  
- **ArrayQueue**: Up to **5.1 million orders/second** (batch processing)

### Order Matching Performance
- **PriorityQueue**: **84.98 million matches/second** 
- **HashMap**: **73.31 million matches/second**
- **ArrayQueue**: **1.07 million matches/second**

Note: These may not be the most efficient implementations (array queue is pretty bad) but I'm happy enough with them for now. Although I might come back to try to optimize them a little bit more.

## Architecture

- **HashMap**: BTreeMap-based implementation optimized for general use
- **PriorityQueue**: BinaryHeap-based with price-time priority
- **ArrayQueue**: Lock-free queues optimized for "high-frequency" submissions

Each implementation satisfies the same `OrderBookTrait` interface, making them interchangeable.
 
## Some Potential Improvements

Some ideas for improvements, not sure I'm going to ever get around to them but if you're interested in messing around here's some other ideas:

* For constants try to fine tune them "on the fly" and compare against other implementations
    * As more orders stream in sample the latency and compare against other values for the constants like queue size, you could also do this against the entire implementaitons
* Actually split this cross-threads where each thread is responsible for a set of symbols
* Do a via network submit orders w/ custom protocol