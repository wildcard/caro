#!/usr/bin/env python3
"""
Benchmark Comparison Script for Caro CI

Parses Criterion benchmark results and detects performance regressions.
Implements FR2.1 regression detection from Issue #9 specification.

Usage:
    python3 scripts/benchmark-compare.py \\
        --criterion-dir target/criterion \\
        --threshold-time 15 \\
        --threshold-memory 20 \\
        --output regression-report.json
"""

import argparse
import json
import os
import sys
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Optional, Tuple


def parse_duration_unit(unit_str: str) -> str:
    """Convert Criterion's time unit strings to standard units."""
    unit_map = {
        "ns": "ns",
        "us": "us",
        "µs": "us",
        "ms": "ms",
        "s": "s",
    }
    return unit_map.get(unit_str, unit_str)


def parse_criterion_estimate(estimate_file: Path) -> Optional[Dict]:
    """Parse Criterion's estimates.json file."""
    try:
        with open(estimate_file, "r") as f:
            data = json.load(f)
            # Criterion stores mean in estimate.point_estimate
            return {
                "mean_ns": data.get("mean", {}).get("point_estimate", 0),
                "std_dev_ns": data.get("std_dev", {}).get("point_estimate", 0),
            }
    except (FileNotFoundError, json.JSONDecodeError, KeyError) as e:
        print(f"Warning: Failed to parse {estimate_file}: {e}", file=sys.stderr)
        return None


def parse_criterion_change(change_file: Path) -> Optional[Dict]:
    """Parse Criterion's change/estimates.json file for baseline comparison."""
    try:
        with open(change_file, "r") as f:
            data = json.load(f)
            return {
                "change_percent": data.get("mean", {}).get("point_estimate", 0) * 100,
                "p_value": data.get("mean", {}).get("confidence_interval", {}).get("upper_bound", 1.0),
            }
    except (FileNotFoundError, json.JSONDecodeError, KeyError) as e:
        print(f"Warning: Failed to parse {change_file}: {e}", file=sys.stderr)
        return None


def convert_ns_to_best_unit(time_ns: float) -> Tuple[float, str]:
    """Convert nanoseconds to the best human-readable unit."""
    if time_ns < 1000:
        return time_ns, "ns"
    elif time_ns < 1_000_000:
        return time_ns / 1000, "us"
    elif time_ns < 1_000_000_000:
        return time_ns / 1_000_000, "ms"
    else:
        return time_ns / 1_000_000_000, "s"


def classify_severity(delta_percent: float, threshold: float) -> str:
    """Classify regression severity based on percentage change."""
    if delta_percent >= threshold * 3:  # 3x threshold = critical
        return "critical"
    elif delta_percent >= threshold * 2:  # 2x threshold = high
        return "high"
    else:
        return "medium"


def find_benchmarks(criterion_dir: Path) -> List[Tuple[str, Path]]:
    """Find all benchmark result directories in Criterion output."""
    benchmarks = []

    # Criterion creates directories like: target/criterion/{group}/{benchmark}/
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


def analyze_benchmark(
    benchmark_id: str,
    bench_dir: Path,
    threshold_time: float,
) -> Optional[Dict]:
    """Analyze a single benchmark for regressions."""

    # Parse current estimates
    current_file = bench_dir / "new" / "estimates.json"
    current_data = parse_criterion_estimate(current_file)

    # Parse baseline estimates
    baseline_file = bench_dir / "base" / "estimates.json"
    baseline_data = parse_criterion_estimate(baseline_file)

    # Parse change statistics (contains p-value)
    change_file = bench_dir / "change" / "estimates.json"
    change_data = parse_criterion_change(change_file)

    if not current_data or not baseline_data or not change_data:
        return None

    # Calculate delta
    delta_percent = change_data["change_percent"]
    p_value = change_data["p_value"]

    # Convert to human-readable units
    baseline_value, baseline_unit = convert_ns_to_best_unit(baseline_data["mean_ns"])
    current_value, current_unit = convert_ns_to_best_unit(current_data["mean_ns"])

    # Determine if this is a regression or improvement
    is_regression = delta_percent > 0 and abs(delta_percent) >= threshold_time
    is_improvement = delta_percent < 0 and abs(delta_percent) >= 5  # 5% threshold for improvements
    is_significant = p_value < 0.05

    result = {
        "benchmark_id": benchmark_id,
        "baseline_mean": {
            "value": round(baseline_value, 2),
            "unit": baseline_unit,
        },
        "current_mean": {
            "value": round(current_value, 2),
            "unit": current_unit,
        },
        "delta_percent": round(delta_percent, 2),
        "statistical_significance": round(p_value, 3),
    }

    if is_regression and is_significant:
        result["severity"] = classify_severity(abs(delta_percent), threshold_time)
        return ("regression", result)
    elif is_improvement:
        return ("improvement", result)
    else:
        return ("no_change", result)


def get_git_commit() -> str:
    """Get current git commit SHA."""
    try:
        import subprocess
        result = subprocess.run(
            ["git", "rev-parse", "HEAD"],
            capture_output=True,
            text=True,
            check=True,
        )
        return result.stdout.strip()
    except Exception:
        return "0" * 40  # Fallback for non-git environments


def get_baseline_commit(baseline_name: str = "baseline") -> str:
    """Get baseline commit SHA from Criterion metadata."""
    # For simplicity, use main branch HEAD
    try:
        import subprocess
        result = subprocess.run(
            ["git", "rev-parse", "main"],
            capture_output=True,
            text=True,
            check=True,
        )
        return result.stdout.strip()
    except Exception:
        return "0" * 40


def main():
    parser = argparse.ArgumentParser(description="Compare Criterion benchmark results")
    parser.add_argument(
        "--criterion-dir",
        type=Path,
        required=True,
        help="Path to Criterion output directory (e.g., target/criterion)",
    )
    parser.add_argument(
        "--threshold-time",
        type=float,
        default=15.0,
        help="Time regression threshold percentage (default: 15)",
    )
    parser.add_argument(
        "--threshold-memory",
        type=float,
        default=20.0,
        help="Memory regression threshold percentage (default: 20)",
    )
    parser.add_argument(
        "--output",
        type=Path,
        required=True,
        help="Output JSON file for regression report",
    )

    args = parser.parse_args()

    # Find all benchmarks
    benchmarks = find_benchmarks(args.criterion_dir)
    print(f"Found {len(benchmarks)} benchmarks to analyze")

    # Analyze each benchmark
    regressions = []
    improvements = []

    for benchmark_id, bench_dir in benchmarks:
        result = analyze_benchmark(benchmark_id, bench_dir, args.threshold_time)

        if result:
            result_type, result_data = result
            if result_type == "regression":
                regressions.append(result_data)
                print(f"⚠️  Regression: {benchmark_id} (+{result_data['delta_percent']}%)")
            elif result_type == "improvement":
                improvements.append(result_data)
                print(f"✅ Improvement: {benchmark_id} ({result_data['delta_percent']}%)")

    # Generate regression report
    report = {
        "pr_number": int(os.environ.get("GITHUB_PR_NUMBER", "0")) or None,
        "baseline_commit": get_baseline_commit(),
        "current_commit": get_git_commit(),
        "regressions": regressions,
        "improvements": improvements,
        "threshold": {
            "time_percent": args.threshold_time,
            "memory_percent": args.threshold_memory,
        },
        "status": "fail" if regressions else "pass",
        "timestamp": datetime.utcnow().isoformat() + "Z",
    }

    # Write report to file
    with open(args.output, "w") as f:
        json.dump(report, f, indent=2)

    print(f"\n📊 Report generated: {args.output}")
    print(f"Status: {report['status'].upper()}")
    print(f"Regressions: {len(regressions)}, Improvements: {len(improvements)}")

    # Exit with error code if regressions detected
    if regressions:
        sys.exit(1)
    else:
        sys.exit(0)


if __name__ == "__main__":
    main()
