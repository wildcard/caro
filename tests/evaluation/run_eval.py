#!/usr/bin/env python3
"""
Simple evaluation harness for Caro CLI
"""
import json
import re
import subprocess
import sys
from pathlib import Path

def normalize_command(cmd):
    """Normalize a command for semantic comparison"""
    if not cmd or cmd.startswith("ERROR"):
        return cmd

    # Normalize quotes (single vs double)
    cmd = cmd.replace("'", "\"")

    # Normalize paths (remove ./ prefix)
    cmd = re.sub(r'\./(?!\.)', '', cmd)

    # Normalize whitespace
    cmd = ' '.join(cmd.split())

    return cmd

def commands_equivalent(expected, actual):
    """Check if two commands are semantically equivalent"""
    if expected == actual:
        return True

    # Don't try semantic comparison if actual is an error
    if actual.startswith("ERROR"):
        return False

    # Normalize both commands
    norm_expected = normalize_command(expected)
    norm_actual = normalize_command(actual)

    if norm_expected == norm_actual:
        return True

    # TODO: Add more equivalence rules here:
    # - Flag ordering (ls -la == ls -al)
    # - Equivalent commands (ls == ls .)
    # - Path equivalents (documents/ == documents)

    return False

def run_caro(prompt, backend="embedded", caro_bin="../../target/release/caro"):
    """Run caro and return the generated command"""
    try:
        cmd = [caro_bin, "--backend", backend, "--output", "json", prompt]
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=30)

        if result.returncode != 0:
            return f"ERROR: {result.stderr}"

        output = json.loads(result.stdout)
        return output.get("generated_command", "")
    except subprocess.TimeoutExpired:
        return "ERROR: Timeout"
    except json.JSONDecodeError:
        return f"ERROR: Invalid JSON: {result.stdout}"
    except Exception as e:
        return f"ERROR: {e}"

def main():
    dataset_path = sys.argv[1] if len(sys.argv) > 1 else "datasets/correctness/file_operations.json"
    backend = sys.argv[2] if len(sys.argv) > 2 else "embedded"
    caro_bin = sys.argv[3] if len(sys.argv) > 3 else "../../target/release/caro"

    print("=" * 70)
    print("Caro Evaluation Harness")
    print("=" * 70)
    print(f"Dataset: {dataset_path}")
    print(f"Backend: {backend}")
    print(f"Binary:  {caro_bin}")
    print("=" * 70)
    print()

    # Load dataset
    with open(dataset_path) as f:
        dataset = json.load(f)

    test_cases = dataset.get("test_cases", [])
    total = len(test_cases)
    passed = 0
    failed = 0
    results = []

    print(f"Running {total} tests...\n")

    for test in test_cases:
        test_id = test["id"]
        prompt = test["prompt"]
        expected = test["expected_command"]

        # Run caro
        actual = run_caro(prompt, backend, caro_bin)

        # Compare using semantic equivalence
        success = commands_equivalent(expected, actual)
        if success:
            passed += 1
            print(f"✓ {test_id}: PASS")
        else:
            failed += 1
            print(f"✗ {test_id}: FAIL")
            print(f"  Prompt:   {prompt}")
            print(f"  Expected: {expected}")
            print(f"  Actual:   {actual}")
            print()

        results.append({
            "test_id": test_id,
            "prompt": prompt,
            "expected": expected,
            "actual": actual,
            "passed": success
        })

    # Summary
    print()
    print("=" * 70)
    print("Results Summary")
    print("=" * 70)
    print(f"Total:     {total}")
    print(f"Passed:    {passed}")
    print(f"Failed:    {failed}")
    if total > 0:
        pass_rate = (passed / total) * 100
        print(f"Pass Rate: {pass_rate:.1f}%")
    print("=" * 70)

    # Save results
    results_file = "/tmp/caro_eval_results.json"
    with open(results_file, "w") as f:
        json.dump({
            "dataset": dataset_path,
            "backend": backend,
            "total": total,
            "passed": passed,
            "failed": failed,
            "pass_rate": (passed / total * 100) if total > 0 else 0,
            "results": results
        }, f, indent=2)

    print(f"\nResults saved to: {results_file}")

    # Exit with failure if any tests failed
    sys.exit(0 if failed == 0 else 1)

if __name__ == "__main__":
    main()
