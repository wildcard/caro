#!/usr/bin/env python3
"""
Batch inference test for performance benchmarking.
Tests throughput and consistency across multiple runs.
"""

import json
import mlx.core as mx
from mlx_lm import load, generate
import time
from typing import List, Dict


QUICK_PROMPTS = [
    "list files",
    "show directory",
    "find python files",
    "check disk space",
    "count lines in file.txt",
    "search for 'error' in logs",
    "copy file.txt to backup.txt",
    "show git status",
    "display current date",
    "print environment variables"
]


def batch_generate(model, tokenizer, prompts: List[str], max_tokens: int = 50) -> List[Dict]:
    """Generate commands for multiple prompts"""
    results = []
    
    for i, prompt in enumerate(prompts):
        system_prompt = f"""Convert to POSIX shell command. Output only JSON:
{{"cmd": "command here"}}

Request: {prompt}
JSON:"""
        
        start = time.time()
        response = generate(
            model,
            tokenizer,
            prompt=system_prompt,
            max_tokens=max_tokens,
            verbose=False
        )
        elapsed = time.time() - start
        
        # Try to parse JSON
        try:
            if "{" in response:
                json_start = response.find("{")
                json_end = response.rfind("}") + 1
                parsed = json.loads(response[json_start:json_end])
                command = parsed.get("cmd", response)
            else:
                command = response.strip()
        except:
            command = response.strip()
        
        results.append({
            "prompt": prompt,
            "command": command,
            "time": elapsed,
            "raw": response
        })
        
        print(f"  [{i+1}/{len(prompts)}] {elapsed:.2f}s - {prompt[:40]}")
    
    return results


def main():
    print("🔥 MLX Batch Inference Benchmark")
    print("=" * 60)
    
    # Load model
    model_name = "TinyLlama/TinyLlama-1.1B-Chat-v1.0"
    print(f"Loading {model_name}...\n")
    
    model, tokenizer = load(model_name)
    print("✅ Model loaded\n")
    
    # Warm-up run
    print("Warming up...")
    generate(model, tokenizer, prompt="test", max_tokens=10, verbose=False)
    print("✅ Warm-up complete\n")
    
    # Batch inference
    print(f"Running batch inference ({len(QUICK_PROMPTS)} prompts)...")
    print("-" * 60)
    
    overall_start = time.time()
    results = batch_generate(model, tokenizer, QUICK_PROMPTS)
    overall_time = time.time() - overall_start
    
    # Statistics
    print("\n" + "=" * 60)
    print("RESULTS")
    print("=" * 60)
    
    times = [r["time"] for r in results]
    avg_time = sum(times) / len(times)
    min_time = min(times)
    max_time = max(times)
    
    print(f"\nTotal Time: {overall_time:.2f}s")
    print(f"Average per prompt: {avg_time:.2f}s")
    print(f"Min: {min_time:.2f}s | Max: {max_time:.2f}s")
    print(f"Throughput: {len(QUICK_PROMPTS)/overall_time:.2f} prompts/sec")
    
    print("\nGenerated Commands:")
    print("-" * 60)
    for r in results:
        print(f"  {r['prompt']:<35} -> {r['command'][:40]}")
    
    # Save results
    with open("batch_results.json", "w") as f:
        json.dump({
            "stats": {
                "total_time": overall_time,
                "avg_time": avg_time,
                "min_time": min_time,
                "max_time": max_time,
                "throughput": len(QUICK_PROMPTS)/overall_time
            },
            "results": results
        }, f, indent=2)
    
    print("\n📄 Saved to batch_results.json")
    
    return 0


if __name__ == "__main__":
    exit(main())
