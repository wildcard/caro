#!/usr/bin/env python3
"""
Monthly Benchmark Aggregation Script for Caro CI

Aggregates daily benchmark artifacts into monthly summaries with statistics.
Implements FR2.2 historical data storage from Issue #9 specification.

Usage:
    python3 scripts/benchmark-monthly-aggregate.py \\
        --daily-data benchmarks-2026-01-*.json \\
        --output benchmark-history-2026-01.json
"""

import argparse
import glob
import json
import sys
from datetime import datetime
from pathlib import Path
from typing import Dict, List


def parse_daily_artifact(file_path: Path) -> Dict:
    """Parse a daily benchmark artifact JSON file."""
    try:
        with open(file_path, "r") as f:
            return json.load(f)
    except (FileNotFoundError, json.JSONDecodeError) as e:
        print(f"Warning: Failed to parse {file_path}: {e}", file=sys.stderr)
        return None


def aggregate_benchmark_stats(benchmark_id: str, daily_results: List[Dict]) -> Dict:
    """Calculate monthly statistics for a single benchmark across daily runs."""

    # Extract mean values (convert all to ns for consistent comparison)
    mean_values_ns = []
    for result in daily_results:
        value = result["mean"]["value"]
        unit = result["mean"]["unit"]

        # Convert to nanoseconds
        if unit == "ns":
            mean_values_ns.append(value)
        elif unit == "us":
            mean_values_ns.append(value * 1000)
        elif unit == "ms":
            mean_values_ns.append(value * 1_000_000)
        elif unit == "s":
            mean_values_ns.append(value * 1_000_000_000)

    if not mean_values_ns:
        return None

    # Calculate statistics
    import statistics

    mean_ns = statistics.mean(mean_values_ns)
    median_ns = statistics.median(mean_values_ns)
    min_ns = min(mean_values_ns)
    max_ns = max(mean_values_ns)
    std_dev_ns = statistics.stdev(mean_values_ns) if len(mean_values_ns) > 1 else 0

    # Convert back to best unit for display
    def to_best_unit(time_ns):
        if time_ns < 1000:
            return time_ns, "ns"
        elif time_ns < 1_000_000:
            return time_ns / 1000, "us"
        elif time_ns < 1_000_000_000:
            return time_ns / 1_000_000, "ms"
        else:
            return time_ns / 1_000_000_000, "s"

    mean_val, mean_unit = to_best_unit(mean_ns)
    median_val, median_unit = to_best_unit(median_ns)
    min_val, min_unit = to_best_unit(min_ns)
    max_val, max_unit = to_best_unit(max_ns)

    return {
        "benchmark_id": benchmark_id,
        "runs": len(mean_values_ns),
        "mean": {
            "value": round(mean_val, 2),
            "unit": mean_unit,
        },
        "median": {
            "value": round(median_val, 2),
            "unit": median_unit,
        },
        "min": {
            "value": round(min_val, 2),
            "unit": min_unit,
        },
        "max": {
            "value": round(max_val, 2),
            "unit": max_unit,
        },
        "std_dev_ns": round(std_dev_ns, 2),
    }


def aggregate_monthly(daily_files: List[Path]) -> Dict:
    """Aggregate multiple daily artifacts into a monthly summary."""

    # Parse all daily files
    daily_artifacts = []
    for file_path in daily_files:
        artifact = parse_daily_artifact(file_path)
        if artifact:
            daily_artifacts.append(artifact)

    if not daily_artifacts:
        print("Error: No valid daily artifacts found", file=sys.stderr)
        sys.exit(1)

    print(f"Processing {len(daily_artifacts)} daily artifacts")

    # Group benchmarks by ID across all days
    benchmark_data = {}
    for artifact in daily_artifacts:
        for bench in artifact.get("benchmarks", []):
            bench_id = bench["benchmark_id"]
            if bench_id not in benchmark_data:
                benchmark_data[bench_id] = []
            benchmark_data[bench_id].append(bench)

    # Calculate monthly statistics for each benchmark
    monthly_benchmarks = []
    for bench_id, daily_results in benchmark_data.items():
        stats = aggregate_benchmark_stats(bench_id, daily_results)
        if stats:
            monthly_benchmarks.append(stats)
            print(f"✓ {bench_id}: {stats['mean']['value']} {stats['mean']['unit']} (avg over {stats['runs']} runs)")

    # Extract month/year from first artifact
    first_timestamp = daily_artifacts[0]["timestamp"]
    month_str = datetime.fromisoformat(first_timestamp.replace("Z", "+00:00")).strftime("%Y-%m")

    # Build monthly aggregate
    monthly_aggregate = {
        "month": month_str,
        "period_start": min(a["timestamp"] for a in daily_artifacts),
        "period_end": max(a["timestamp"] for a in daily_artifacts),
        "total_runs": len(daily_artifacts),
        "benchmarks": monthly_benchmarks,
        "summary": {
            "unique_benchmarks": len(monthly_benchmarks),
            "total_measurements": sum(b["runs"] for b in monthly_benchmarks),
        },
    }

    return monthly_aggregate


def main():
    parser = argparse.ArgumentParser(description="Aggregate daily benchmarks into monthly summary")
    parser.add_argument(
        "--daily-data",
        type=str,
        required=True,
        help="Glob pattern for daily benchmark files (e.g., benchmarks-*.json)",
    )
    parser.add_argument(
        "--output",
        type=Path,
        required=True,
        help="Output JSON file for monthly aggregate",
    )

    args = parser.parse_args()

    # Find all daily files matching the pattern
    daily_files = [Path(f) for f in glob.glob(args.daily_data)]
    if not daily_files:
        print(f"Error: No files found matching pattern: {args.daily_data}", file=sys.stderr)
        sys.exit(1)

    print(f"Found {len(daily_files)} daily artifact files")

    # Aggregate into monthly summary
    monthly_aggregate = aggregate_monthly(daily_files)

    # Write monthly aggregate to file
    with open(args.output, "w") as f:
        json.dump(monthly_aggregate, f, indent=2)

    print(f"\n📅 Monthly aggregate generated: {args.output}")
    print(f"Period: {monthly_aggregate['period_start']} to {monthly_aggregate['period_end']}")
    print(f"Total runs: {monthly_aggregate['total_runs']}")
    print(f"Unique benchmarks: {monthly_aggregate['summary']['unique_benchmarks']}")

    sys.exit(0)


if __name__ == "__main__":
    main()
