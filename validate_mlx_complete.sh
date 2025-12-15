#!/bin/bash
# Complete MLX backend validation script

set -e

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  MLX Backend Implementation - Complete Validation"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo

# Phase 1: Binary Verification
echo "📦 Phase 1: Binary Verification"
echo "─────────────────────────────────────────────────────────────"

BINARY="./target/release/cmdai"

if [ ! -f "$BINARY" ]; then
    echo "❌ Binary not found: $BINARY"
    echo "   Run: cargo build --release --features embedded-mlx"
    exit 1
fi

echo "✅ Binary exists: $BINARY"

# Check binary size
SIZE=$(ls -lh "$BINARY" | awk '{print $5}')
SIZE_BYTES=$(stat -f%z "$BINARY")
SIZE_MB=$((SIZE_BYTES / 1024 / 1024))

echo "✅ Binary size: $SIZE ($SIZE_MB MB)"

if [ "$SIZE_MB" -gt 50 ]; then
    echo "⚠️  Warning: Binary exceeds 50MB target"
else
    echo "✅ Binary size under 50MB target"
fi

# Check architecture
ARCH=$(file "$BINARY" | grep -o "arm64")
if [ "$ARCH" == "arm64" ]; then
    echo "✅ Architecture: Apple Silicon (arm64)"
else
    echo "❌ Wrong architecture: Expected arm64"
    exit 1
fi

# Check Metal linkage
echo
echo "🔗 Checking Metal framework linkage..."
METAL_LIBS=$(otool -L "$BINARY" | grep -i metal | wc -l)

if [ "$METAL_LIBS" -gt 0 ]; then
    echo "✅ Metal frameworks linked: $METAL_LIBS frameworks found"
    otool -L "$BINARY" | grep -i metal | while read -r line; do
        echo "   - $line"
    done
else
    echo "❌ No Metal frameworks found"
    exit 1
fi

echo
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo

# Phase 2: Model Verification
echo "📁 Phase 2: Model File Verification"
echo "─────────────────────────────────────────────────────────────"

MODEL_PATH="$HOME/Library/Caches/cmdai/models/qwen2.5-coder-1.5b-instruct-q4_k_m.gguf"

if [ ! -f "$MODEL_PATH" ]; then
    echo "❌ Model file not found: $MODEL_PATH"
    exit 1
fi

echo "✅ Model file exists"

MODEL_SIZE=$(ls -lh "$MODEL_PATH" | awk '{print $5}')
echo "✅ Model size: $MODEL_SIZE"

echo
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo

# Phase 3: Runtime Tests
echo "🚀 Phase 3: Runtime Inference Tests"
echo "─────────────────────────────────────────────────────────────"

TEST_PROMPTS=(
    "list all files in the current directory"
    "find all python files"
    "show disk usage"
    "count lines in all text files"
    "search for TODO in source files"
)

SUCCESS_COUNT=0
TOTAL_TESTS=${#TEST_PROMPTS[@]}

for prompt in "${TEST_PROMPTS[@]}"; do
    echo
    echo "Testing: '$prompt'"
    echo -n "  Running... "
    
    # Run inference with timeout
    if timeout 30s "$BINARY" "$prompt" > /tmp/cmdai_test_output.txt 2>&1; then
        # Check if command was generated
        if grep -q "Command:" /tmp/cmdai_test_output.txt; then
            COMMAND=$(grep -A1 "Command:" /tmp/cmdai_test_output.txt | tail -1 | tr -d ' ')
            if [ -n "$COMMAND" ]; then
                echo "✅ Generated: $COMMAND"
                ((SUCCESS_COUNT++))
            else
                echo "❌ Empty command"
                cat /tmp/cmdai_test_output.txt
            fi
        else
            echo "❌ No command in output"
            cat /tmp/cmdai_test_output.txt
        fi
    else
        echo "❌ Timeout or error"
        cat /tmp/cmdai_test_output.txt
    fi
done

echo
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo

# Phase 4: Performance Benchmarks
echo "⚡ Phase 4: Performance Benchmarks"
echo "─────────────────────────────────────────────────────────────"

echo "Running performance test..."
START_TIME=$(date +%s%N)
"$BINARY" "list files" > /dev/null 2>&1
END_TIME=$(date +%s%N)

ELAPSED_MS=$(( (END_TIME - START_TIME) / 1000000 ))
echo "✅ End-to-end latency: ${ELAPSED_MS}ms"

if [ "$ELAPSED_MS" -lt 5000 ]; then
    echo "✅ Performance target met (<5s)"
else
    echo "⚠️  Performance slower than expected (target: <5s)"
fi

echo
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo

# Phase 5: Summary
echo "📊 Phase 5: Validation Summary"
echo "─────────────────────────────────────────────────────────────"

echo
echo "Binary Information:"
echo "  Size: $SIZE ($SIZE_MB MB)"
echo "  Architecture: Apple Silicon (arm64)"
echo "  Metal Frameworks: $METAL_LIBS linked"
echo

echo "Model Information:"
echo "  Path: $MODEL_PATH"
echo "  Size: $MODEL_SIZE"
echo

echo "Inference Tests:"
echo "  Passed: $SUCCESS_COUNT / $TOTAL_TESTS"
echo "  Success Rate: $(( SUCCESS_COUNT * 100 / TOTAL_TESTS ))%"
echo

echo "Performance:"
echo "  Latency: ${ELAPSED_MS}ms"
echo

if [ "$SUCCESS_COUNT" -eq "$TOTAL_TESTS" ]; then
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "✅ ALL VALIDATIONS PASSED - MLX BACKEND FULLY OPERATIONAL"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    exit 0
else
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "⚠️  SOME VALIDATIONS FAILED - CHECK OUTPUT ABOVE"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    exit 1
fi
