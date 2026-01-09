#!/usr/bin/env python3
"""
Benchmark Aggregation Script for Caro CI

Aggregates Criterion benchmark results into a single JSON artifact.
Implements FR2.2 historical data collection from Issue #9 specification.

Usage:
    python3 scripts/benchmark-aggregate.py \\
        --criterion-dir target/criterion \\
        --output benchmarks-2026-01-08.json \\
        --commit abc123def456 \\
        --branch main
"""

import argparse
import json
import sys
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Optional


def parse_criterion_estimate(estimate_file: Path) -> Optional[Dict]:
    """Parse Criterion's estimates.json file."""
    try:
        with open(estimate_file, "r") as f:
            data = json.load(f)
            return {
                "mean_ns": data.get("mean", {}).get("point_estimate", 0),
                "std_dev_ns": data.get("std_dev", {}).get("point_estimate", 0),
                "median_ns": data.get("median", {}).get("point_estimate", 0),
                "mad_ns": data.get("median_abs_dev", {}).get("point_estimate", 0),
            }
    except (FileNotFoundError, json.JSONDecodeError, KeyError) as e:
        print(f"Warning: Failed to parse {estimate_file}: {e}", file=sys.stderr)
        return None


def convert_ns_to_best_unit(time_ns: float) -> tuple[float, str]:
    """Convert nanoseconds to the best human-readable unit."""
    if time_ns < 1000:
        return time_ns, "ns"
    elif time_ns < 1_000_000:
        return time_ns / 1000, "us"
    elif time_ns < 1_000_000_000:
        return time_ns / 1_000_000, "ms"
    else:
        return time_ns / 1_000_000_000, "s"


def find_benchmarks(criterion_dir: Path) -> List[tuple[str, Path]]:
    """Find all benchmark result directories in Criterion output."""
    benchmarks = []

    if not criterion_dir.exists():
        print(f"Error: Criterion directory not found: {criterion_dir}", file=sys.stderr)
        return benchmarks

    # Iterate through group directories
    for group_dir in criterion_dir.iterdir():
        if not group_dir.is_dir() or group_dir.name.startswith("."):
            continue

        # Iterate through benchmark directories within group
        for bench_dir in group_dir.iterdir():
            if not bench_dir.is_dir() or bench_dir.name.startswith("."):
                continue

            # Build benchmark ID from group/benchmark
            benchmark_id = f"{group_dir.name}/{bench_dir.name}"
            benchmarks.append((benchmark_id, bench_dir))

    return benchmarks


def aggregate_benchmark(benchmark_id: str, bench_dir: Path) -> Optional[Dict]:
    """Extract benchmark data for historical storage."""

    # Parse current/latest estimates
    estimate_file = bench_dir / "new" / "estimates.json"
    if not estimate_file.exists():
        # Fall back to base estimates if no "new" directory
        estimate_file = bench_dir / "base" / "estimates.json"

    estimate_data = parse_criterion_estimate(estimate_file)
    if not estimate_data:
        return None

    # Convert to human-readable units
    mean_value, mean_unit = convert_ns_to_best_unit(estimate_data["mean_ns"])
    median_value, median_unit = convert_ns_to_best_unit(estimate_data["median_ns"])

    return {
        "benchmark_id": benchmark_id,
        "mean": {
            "value": round(mean_value, 2),
            "unit": mean_unit,
        },
        "median": {
            "value": round(median_value, 2),
            "unit": median_unit,
        },
        "std_dev_ns": round(estimate_data["std_dev_ns"], 2),
        "mad_ns": round(estimate_data["mad_ns"], 2),
    }


def get_system_info() -> Dict:
    """Collect system information for the benchmark run."""
    import platform
    import os

    return {
        "os": platform.system().lower(),
        "os_version": platform.release(),
        "cpu": platform.processor() or "unknown",
        "cpu_cores": os.cpu_count() or 1,
        "python_version": platform.python_version(),
    }


def main():
    parser = argparse.ArgumentParser(description="Aggregate Criterion benchmark results")
    parser.add_argument(
        "--criterion-dir",
        type=Path,
        required=True,
        help="Path to Criterion output directory",
    )
    parser.add_argument(
        "--output",
        type=Path,
        required=True,
        help="Output JSON file for aggregated results",
    )
    parser.add_argument(
        "--commit",
        type=str,
        required=True,
        help="Git commit SHA for this benchmark run",
    )
    parser.add_argument(
        "--branch",
        type=str,
        required=True,
        help="Git branch name for this benchmark run",
    )

    args = parser.parse_args()

    # Find all benchmarks
    benchmarks = find_benchmarks(args.criterion_dir)
    print(f"Found {len(benchmarks)} benchmarks to aggregate")

    # Aggregate all benchmarks
    results = []
    for benchmark_id, bench_dir in benchmarks:
        result = aggregate_benchmark(benchmark_id, bench_dir)
        if result:
            results.append(result)
            print(f"✓ {benchmark_id}: {result['mean']['value']} {result['mean']['unit']}")

    # Build aggregated artifact
    artifact = {
        "timestamp": datetime.utcnow().isoformat() + "Z",
        "commit": args.commit,
        "branch": args.branch,
        "environment": get_system_info(),
        "benchmarks": results,
        "summary": {
            "total_benchmarks": len(results),
            "successful": len(results),
            "failed": len(benchmarks) - len(results),
        },
    }

    # Write artifact to file
    with open(args.output, "w") as f:
        json.dump(artifact, f, indent=2)

    print(f"\n📦 Artifact generated: {args.output}")
    print(f"Total benchmarks: {len(results)}")

    sys.exit(0)


if __name__ == "__main__":
    main()
