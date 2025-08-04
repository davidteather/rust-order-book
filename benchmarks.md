## matching

### Operation: `match_orders`

| Implementation | Mean (µs) | Throughput |
|----------------|-----------|------------|
| arrayqueue | 93.71 | 1.07 Melem/s |
| hashmap | 1.36 | 73.31 Melem/s |
| priorityqueue | **1.18** | **84.98 Melem/s** |

### Operation: `match_orders_200`

| Implementation | Mean (µs) | Throughput |
|----------------|-----------|------------|
| arrayqueue | 100.60 | 1.99 Melem/s |
| hashmap | **2.37** | **84.35 Melem/s** |
| priorityqueue | 2.77 | 72.17 Melem/s |

## routing_errors

### Operation: `invalid_symbol`

| Implementation | Mean (µs) | Throughput |
|----------------|-----------|------------|
| arrayqueue | 10.11 | 98.95 Kelem/s |
| hashmap | 0.14 | 7.40 Melem/s |
| priorityqueue | **0.12** | **8.30 Melem/s** |

## routing_bulk

### Operation: `100`

| Implementation | Mean (µs) | Throughput |
|----------------|-----------|------------|
| arrayqueue/bulk | 50.74 | 1.97 Melem/s |
| hashmap/bulk | 7.73 | 12.94 Melem/s |
| priorityqueue/bulk | **2.72** | **36.80 Melem/s** |

### Operation: `50`

| Implementation | Mean (µs) | Throughput |
|----------------|-----------|------------|
| arrayqueue/bulk | 54.56 | 916.38 Kelem/s |
| hashmap/bulk | 4.26 | 11.73 Melem/s |
| priorityqueue/bulk | **1.51** | **33.17 Melem/s** |

### Operation: `500`

| Implementation | Mean (µs) | Throughput |
|----------------|-----------|------------|
| arrayqueue/bulk | 54.51 | 9.17 Melem/s |
| hashmap/bulk | 46.46 | 10.76 Melem/s |
| priorityqueue/bulk | **9.35** | **53.48 Melem/s** |

## routing_multi_symbol

### Operation: `multi_symbol_3`

| Implementation | Mean (µs) | Throughput |
|----------------|-----------|------------|
| arrayqueue | 27.25 | 110.10 Kelem/s |
| hashmap | 0.67 | 4.45 Melem/s |
| priorityqueue | **0.43** | **6.97 Melem/s** |

## queries

### Operation: `can_match`

| Implementation | Mean (µs) | Throughput |
|----------------|-----------|------------|
| arrayqueue | 93.55 | 10.69 Kelem/s |
| hashmap | 0.33 | 3.00 Melem/s |
| priorityqueue | **0.11** | **9.19 Melem/s** |

### Operation: `get_best_prices`

| Implementation | Mean (µs) | Throughput |
|----------------|-----------|------------|
| arrayqueue | 93.75 | 10.67 Kelem/s |
| hashmap | 4.64 | 215.72 Kelem/s |
| priorityqueue | **0.15** | **6.80 Melem/s** |

### Operation: `is_valid_symbol`

| Implementation | Mean (µs) | Throughput |
|----------------|-----------|------------|
| arrayqueue | 97.04 | 10.30 Kelem/s |
| hashmap | 0.24 | 4.24 Melem/s |
| priorityqueue | **0.07** | **15.32 Melem/s** |

### Operation: `multi_symbol_best_prices_5`

| Implementation | Mean (µs) | Throughput |
|----------------|-----------|------------|
| arrayqueue | 101.06 | 49.48 Kelem/s |
| hashmap | 33.00 | 151.50 Kelem/s |
| priorityqueue | **0.59** | **8.47 Melem/s** |

## add_order

### Operation: `10`

| Implementation | Mean (µs) | Throughput |
|----------------|-----------|------------|
| arrayqueue/batch_fast | 98.04 | 102.00 Kelem/s |
| arrayqueue/batch_unchecked | 97.09 | 103.00 Kelem/s |
| hashmap/batch_fast | 0.90 | 11.16 Melem/s |
| hashmap/batch_unchecked | 0.90 | 11.12 Melem/s |
| priorityqueue/batch_fast | **0.18** | **54.49 Melem/s** |
| priorityqueue/batch_unchecked | 0.18 | 54.19 Melem/s |

### Operation: `100`

| Implementation | Mean (µs) | Throughput |
|----------------|-----------|------------|
| arrayqueue/batch_fast | 97.99 | 1.02 Melem/s |
| arrayqueue/batch_unchecked | 98.90 | 1.01 Melem/s |
| hashmap/batch_fast | 7.84 | 12.76 Melem/s |
| hashmap/batch_unchecked | 7.91 | 12.64 Melem/s |
| priorityqueue/batch_fast | **1.21** | **82.35 Melem/s** |
| priorityqueue/batch_unchecked | 1.24 | 80.52 Melem/s |

### Operation: `50`

| Implementation | Mean (µs) | Throughput |
|----------------|-----------|------------|
| arrayqueue/batch_fast | 97.01 | 515.39 Kelem/s |
| arrayqueue/batch_unchecked | 95.97 | 521.01 Kelem/s |
| hashmap/batch_fast | 3.85 | 12.98 Melem/s |
| hashmap/batch_unchecked | 3.84 | 13.01 Melem/s |
| priorityqueue/batch_fast | **0.70** | **71.32 Melem/s** |
| priorityqueue/batch_unchecked | 0.71 | 70.50 Melem/s |

### Operation: `500`

| Implementation | Mean (µs) | Throughput |
|----------------|-----------|------------|
| arrayqueue/batch_fast | 104.95 | 4.76 Melem/s |
| arrayqueue/batch_unchecked | 101.94 | 4.91 Melem/s |
| hashmap/batch_fast | 43.47 | 11.50 Melem/s |
| hashmap/batch_unchecked | 42.68 | 11.72 Melem/s |
| priorityqueue/batch_fast | **4.45** | **112.30 Melem/s** |
| priorityqueue/batch_unchecked | 4.59 | 108.93 Melem/s |

### Operation: `single_fast`

| Implementation | Mean (µs) | Throughput |
|----------------|-----------|------------|
| arrayqueue | 110.30 | 9.07 Kelem/s |
| hashmap | 0.27 | 3.70 Melem/s |
| priorityqueue | **0.11** | **9.02 Melem/s** |

### Operation: `single_safe`

| Implementation | Mean (µs) | Throughput |
|----------------|-----------|------------|
| arrayqueue | 106.77 | 9.37 Kelem/s |
| hashmap | 0.27 | 3.73 Melem/s |
| priorityqueue | **0.11** | **9.01 Melem/s** |

### Operation: `single_unchecked`

| Implementation | Mean (µs) | Throughput |
|----------------|-----------|------------|
| arrayqueue | 103.73 | 9.64 Kelem/s |
| hashmap | 0.26 | 3.90 Melem/s |
| priorityqueue | **0.11** | **8.96 Melem/s** |

## multi_symbol

### Operation: `cross_symbol_100`

| Implementation | Mean (µs) | Throughput |
|----------------|-----------|------------|
| arrayqueue | 101.33 | 986.83 Kelem/s |
| hashmap | 7.80 | 12.82 Melem/s |
| priorityqueue | **1.37** | **73.19 Melem/s** |

### Operation: `large_orders_100`

| Implementation | Mean (µs) | Throughput |
|----------------|-----------|------------|
| arrayqueue | 100.27 | 997.27 Kelem/s |
| hashmap | 8.12 | 12.31 Melem/s |
| priorityqueue | **1.58** | **63.12 Melem/s** |

### Operation: `symbol_switching_100`

| Implementation | Mean (µs) | Throughput |
|----------------|-----------|------------|
| arrayqueue | 98.17 | 1.02 Melem/s |
| hashmap | 7.78 | 12.85 Melem/s |
| priorityqueue | **1.37** | **73.17 Melem/s** |

## high_frequency

### Operation: `burst_matching_500`

| Implementation | Mean (µs) | Throughput |
|----------------|-----------|------------|
| arrayqueue | 10.94 | 45.72 Melem/s |
| hashmap | 15.85 | 31.54 Melem/s |
| priorityqueue | **3.33** | **150.17 Melem/s** |

### Operation: `rapid_fire_1000`

| Implementation | Mean (µs) | Throughput |
|----------------|-----------|------------|
| arrayqueue | 37.56 | 26.63 Melem/s |
| hashmap | 81.13 | 12.33 Melem/s |
| priorityqueue | **9.21** | **108.52 Melem/s** |

## routing

### Operation: `single_order`

| Implementation | Mean (µs) | Throughput |
|----------------|-----------|------------|
| arrayqueue | 10.16 | 98.45 Kelem/s |
| hashmap | 0.20 | 5.05 Melem/s |
| priorityqueue | **0.14** | **7.16 Melem/s** |

## add_order_batch

### Operation: `10`

| Implementation | Mean (µs) | Throughput |
|----------------|-----------|------------|
| arrayqueue/batch_fast | 107.47 | 93.05 Kelem/s |
| arrayqueue/batch_unchecked | 107.89 | 92.69 Kelem/s |
| hashmap/batch_fast | 0.84 | 11.95 Melem/s |
| hashmap/batch_unchecked | 0.85 | 11.80 Melem/s |
| priorityqueue/batch_fast | 0.21 | 47.57 Melem/s |
| priorityqueue/batch_unchecked | **0.21** | **47.86 Melem/s** |

### Operation: `100`

| Implementation | Mean (µs) | Throughput |
|----------------|-----------|------------|
| arrayqueue/batch_fast | 97.42 | 1.03 Melem/s |
| arrayqueue/batch_unchecked | 94.36 | 1.06 Melem/s |
| hashmap/batch_fast | 7.68 | 13.02 Melem/s |
| hashmap/batch_unchecked | 7.72 | 12.96 Melem/s |
| priorityqueue/batch_fast | **1.20** | **83.09 Melem/s** |
| priorityqueue/batch_unchecked | 1.24 | 80.93 Melem/s |

### Operation: `50`

| Implementation | Mean (µs) | Throughput |
|----------------|-----------|------------|
| arrayqueue/batch_fast | 95.39 | 524.14 Kelem/s |
| arrayqueue/batch_unchecked | 94.18 | 530.87 Kelem/s |
| hashmap/batch_fast | 3.74 | 13.38 Melem/s |
| hashmap/batch_unchecked | 3.75 | 13.35 Melem/s |
| priorityqueue/batch_fast | **0.71** | **70.32 Melem/s** |
| priorityqueue/batch_unchecked | 0.73 | 68.84 Melem/s |

### Operation: `500`

| Implementation | Mean (µs) | Throughput |
|----------------|-----------|------------|
| arrayqueue/batch_fast | 97.68 | 5.12 Melem/s |
| arrayqueue/batch_unchecked | 99.97 | 5.00 Melem/s |
| hashmap/batch_fast | 43.51 | 11.49 Melem/s |
| hashmap/batch_unchecked | 43.75 | 11.43 Melem/s |
| priorityqueue/batch_fast | **4.39** | **113.97 Melem/s** |
| priorityqueue/batch_unchecked | 4.55 | 109.85 Melem/s |