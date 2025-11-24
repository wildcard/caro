#!/usr/bin/env python3
"""
Simple MLX inference example using a small pre-trained model.
This demonstrates basic text generation with MLX on Apple Silicon.
"""

import mlx.core as mx
import mlx.nn as nn
from mlx_lm import load, generate

def main():
    print("🚀 MLX Simple Inference Test")
    print("=" * 50)
    
    # Check MLX is using Metal
    print(f"MLX default device: {mx.default_device()}")
    print(f"Metal available: {mx.metal.is_available()}")
    print()
    
    # Load a small model (TinyLlama is ~1.1GB)
    model_name = "TinyLlama/TinyLlama-1.1B-Chat-v1.0"
    print(f"Loading model: {model_name}")
    print("(This may take a minute on first run...)")
    
    try:
        model, tokenizer = load(model_name)
        print("✅ Model loaded successfully!")
        print()
        
        # Test prompt
        prompt = "Convert this to a shell command: list all files in current directory"
        print(f"Prompt: {prompt}")
        print()
        print("Generating response...")
        print("-" * 50)
        
        # Generate response
        response = generate(
            model,
            tokenizer,
            prompt=prompt,
            max_tokens=100,
            verbose=True
        )
        
        print("-" * 50)
        print(f"\n✅ Inference complete!")
        print(f"Response: {response}")
        
    except Exception as e:
        print(f"❌ Error: {e}")
        return 1
    
    return 0

if __name__ == "__main__":
    exit(main())
