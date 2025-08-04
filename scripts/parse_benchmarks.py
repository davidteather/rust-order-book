import json
import argparse
from pathlib import Path
from collections import defaultdict

SI_UNITS = ["", "K", "M", "G", "T"]

def format_throughput(value, unit="elem"):
    if value == 0:
        return "0 " + unit + "/s"
    i = 0
    while value >= 1000 and i < len(SI_UNITS) - 1:
        value /= 1000
        i += 1
    return f"{value:.2f} {SI_UNITS[i]}{unit}/s"

def parse_benchmarks(base_path):
    results = defaultdict(dict)
    found_any = False

    for estimates_file in base_path.glob("**/estimates.json"):
        benchmark_file = estimates_file.parent / "benchmark.json"
        if not benchmark_file.exists():
            continue

        try:
            with open(estimates_file, "r") as f:
                data = json.load(f)
            with open(benchmark_file, "r") as f:
                meta = json.load(f)

            mean_ns = data.get("mean", {}).get("point_estimate", 0)
            throughput_data = meta.get("throughput", {})
            units_per_iter = throughput_data.get("Elements", 1)
            throughput_per_sec = (units_per_iter * 1_000_000_000) / mean_ns if mean_ns > 0 else 0
            throughput_human = format_throughput(throughput_per_sec)

            group_name = meta.get("group_id") or "unknown_group"
            impl_name = meta.get("function_id") or "unknown_impl"
            operation_name = meta.get("value_str") or "default"

            bench_id = f"{impl_name}/{operation_name}"

            results[group_name][bench_id] = {
                "implementation": impl_name,
                "operation": operation_name,
                "mean_ns": mean_ns,
                "throughput_per_sec": throughput_per_sec,
                "mean_us": mean_ns / 1000.0,
                "throughput_human": throughput_human
            }
            found_any = True

        except Exception as e:
            print(f"Error processing {estimates_file.parent}: {e}")

    if not found_any:
        print("⚠️ No benchmarks found in:", base_path)

    return results

def write_markdown(results, output_file):
    with open(output_file, "w") as f:
        for group, benches in results.items():
            f.write(f"## {group}\n\n")
            impls = sorted(set(b["implementation"] for b in benches.values()))
            ops = sorted(set(b["operation"] for b in benches.values()))

            for op in ops:
                f.write(f"### Operation: `{op}`\n\n")
                f.write("| Implementation | Mean (µs) | Throughput |\n")
                f.write("|----------------|-----------|------------|\n")

                filtered = [b for b in benches.values() if b["operation"] == op]
                best = min(filtered, key=lambda x: x["mean_us"] if x["mean_us"] > 0 else float("inf"), default=None)

                for impl in impls:
                    b = next((b for b in filtered if b["implementation"] == impl), None)
                    if b:
                        is_best = best and b["mean_us"] == best["mean_us"]
                        mean_us = f"**{b['mean_us']:.2f}**" if is_best else f"{b['mean_us']:.2f}"
                        throughput = f"**{b['throughput_human']}**" if is_best else b['throughput_human']
                        f.write(f"| {impl} | {mean_us} | {throughput} |\n")
                f.write("\n")

def main():
    parser = argparse.ArgumentParser(description="Parse Criterion benchmarks and output a markdown table.")
    parser.add_argument("--base", type=Path, default=Path("./target/criterion"), help="Base path to Criterion benchmark results")
    parser.add_argument("--output", type=Path, default=Path("benchmarks.md"), help="Output markdown file")
    args = parser.parse_args()

    results = parse_benchmarks(args.base)
    write_markdown(results, args.output)

if __name__ == "__main__":
    main()
