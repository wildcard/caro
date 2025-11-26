#!/bin/bash
# setup-inference-model.sh
# Downloads and verifies Qwen models for cmdai inference testing in CI

set -euo pipefail

# Configuration
CACHE_DIR="${HOME}/.cache/cmdai/models"
MODEL_SIZE="${MODEL_SIZE:-0.5B}"
MODEL_QUANT="${MODEL_QUANT:-Q4_K_S}"

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Model mappings
declare -A MODEL_URLS=(
    ["0.5B-Q4_K_S"]="https://huggingface.co/Qwen/Qwen2.5-Coder-0.5B-Instruct-GGUF/resolve/main/qwen2.5-coder-0.5b-instruct-q4_k_s.gguf"
    ["1.5B-Q4_K_M"]="https://huggingface.co/Qwen/Qwen2.5-Coder-1.5B-Instruct-GGUF/resolve/main/qwen2.5-coder-1.5b-instruct-q4_k_m.gguf"
)

# SHA256 checksums - TODO: Update with actual checksums after first download
declare -A MODEL_SHA256=(
    ["0.5B-Q4_K_S"]=""  # Will be filled after first successful download
    ["1.5B-Q4_K_M"]=""  # Will be filled after first successful download
)

# Select model
MODEL_KEY="${MODEL_SIZE}-${MODEL_QUANT}"
MODEL_URL="${MODEL_URLS[$MODEL_KEY]:-}"
if [ -z "$MODEL_URL" ]; then
    echo -e "${RED}❌ Unknown model configuration: ${MODEL_KEY}${NC}"
    echo "Available models:"
    for key in "${!MODEL_URLS[@]}"; do
        echo "  - $key"
    done
    exit 1
fi

MODEL_FILE="$(basename "$MODEL_URL")"
EXPECTED_SHA256="${MODEL_SHA256[$MODEL_KEY]:-}"

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}  cmdai CI Model Setup${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo "Model: ${MODEL_KEY}"
echo "File: ${MODEL_FILE}"
echo "Cache: ${CACHE_DIR}"
echo ""

mkdir -p "${CACHE_DIR}"

# Check if model already cached
if [ -f "${CACHE_DIR}/${MODEL_FILE}" ]; then
    echo -e "${GREEN}✅ Model found in cache: ${MODEL_FILE}${NC}"

    # Verify checksum if we have one
    if [ -n "$EXPECTED_SHA256" ]; then
        echo "🔍 Verifying checksum..."
        ACTUAL_SHA256=$(sha256sum "${CACHE_DIR}/${MODEL_FILE}" | cut -d' ' -f1)
        if [ "${ACTUAL_SHA256}" == "${EXPECTED_SHA256}" ]; then
            echo -e "${GREEN}✅ Checksum verified${NC}"
            echo "CMDAI_CI_MODEL_PATH=${CACHE_DIR}/${MODEL_FILE}" >> "${GITHUB_ENV:-/dev/null}"
            exit 0
        else
            echo -e "${YELLOW}⚠️  Checksum mismatch, re-downloading...${NC}"
            echo "   Expected: ${EXPECTED_SHA256}"
            echo "   Actual:   ${ACTUAL_SHA256}"
            rm "${CACHE_DIR}/${MODEL_FILE}"
        fi
    else
        echo -e "${YELLOW}⚠️  No checksum available, skipping verification${NC}"
        echo "CMDAI_CI_MODEL_PATH=${CACHE_DIR}/${MODEL_FILE}" >> "${GITHUB_ENV:-/dev/null}"
        exit 0
    fi
fi

# Download model
echo -e "${BLUE}📦 Downloading model: ${MODEL_FILE}${NC}"
echo "   URL: ${MODEL_URL}"
echo ""

START_TIME=$(date +%s)

# Use curl with progress bar and retry logic
if ! curl -L --retry 3 --retry-delay 5 \
     --progress-bar \
     -o "${CACHE_DIR}/${MODEL_FILE}.tmp" \
     "${MODEL_URL}"; then
    echo -e "${RED}❌ Download failed after retries${NC}"
    rm -f "${CACHE_DIR}/${MODEL_FILE}.tmp"
    exit 1
fi

# Verify checksum if we have one
if [ -n "$EXPECTED_SHA256" ]; then
    echo ""
    echo "🔍 Verifying checksum..."
    ACTUAL_SHA256=$(sha256sum "${CACHE_DIR}/${MODEL_FILE}.tmp" | cut -d' ' -f1)

    if [ "${ACTUAL_SHA256}" != "${EXPECTED_SHA256}" ]; then
        echo -e "${RED}❌ Checksum verification failed!${NC}"
        echo "   Expected: ${EXPECTED_SHA256}"
        echo "   Actual:   ${ACTUAL_SHA256}"
        rm "${CACHE_DIR}/${MODEL_FILE}.tmp"
        exit 1
    fi
    echo -e "${GREEN}✅ Checksum verified${NC}"
else
    # Calculate and display checksum for future use
    echo ""
    echo "📝 Calculating checksum (add to script for future verification):"
    ACTUAL_SHA256=$(sha256sum "${CACHE_DIR}/${MODEL_FILE}.tmp" | cut -d' ' -f1)
    echo "   MODEL_SHA256[\"${MODEL_KEY}\"]=\"${ACTUAL_SHA256}\""
fi

# Move to final location
mv "${CACHE_DIR}/${MODEL_FILE}.tmp" "${CACHE_DIR}/${MODEL_FILE}"

END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))

echo ""
echo -e "${GREEN}✅ Model downloaded successfully in ${DURATION}s${NC}"
echo "   Path: ${CACHE_DIR}/${MODEL_FILE}"

# Get file size
FILE_SIZE=$(stat -c%s "${CACHE_DIR}/${MODEL_FILE}" 2>/dev/null || stat -f%z "${CACHE_DIR}/${MODEL_FILE}" 2>/dev/null || echo "unknown")
if [ "$FILE_SIZE" != "unknown" ]; then
    FILE_SIZE_MB=$((FILE_SIZE / 1024 / 1024))
    echo "   Size: ${FILE_SIZE_MB}MB"
fi

# Export for tests
if [ -n "${GITHUB_ENV:-}" ]; then
    echo "CMDAI_CI_MODEL_PATH=${CACHE_DIR}/${MODEL_FILE}" >> "${GITHUB_ENV}"
fi

# Create metadata
cat > "${CACHE_DIR}/metadata.json" <<EOF
{
  "model": "${MODEL_FILE}",
  "model_key": "${MODEL_KEY}",
  "size": "${MODEL_SIZE}",
  "quantization": "${MODEL_QUANT}",
  "sha256": "${ACTUAL_SHA256:-unknown}",
  "downloaded_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "download_duration_seconds": ${DURATION},
  "file_size_bytes": ${FILE_SIZE}
}
EOF

echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}✅ Setup complete!${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
