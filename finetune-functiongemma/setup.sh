#!/bin/bash
# Setup script for FunctionGemma CLI Tool Recommender
# Uses uv for fast package installation

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}FunctionGemma CLI Tool Recommender - Setup${NC}"
echo "============================================"

# Check for uv
if ! command -v uv &> /dev/null; then
    echo -e "${YELLOW}uv not found. Installing uv...${NC}"
    curl -LsSf https://astral.sh/uv/install.sh | sh
    export PATH="$HOME/.local/bin:$PATH"
fi

echo -e "${GREEN}Using uv version:${NC} $(uv --version)"

# Check Python version
PYTHON_VERSION=$(python3 --version 2>&1 | cut -d' ' -f2 | cut -d'.' -f1,2)
echo -e "${GREEN}Python version:${NC} $PYTHON_VERSION"

# Create virtual environment
if [ ! -d ".venv" ]; then
    echo -e "${GREEN}Creating virtual environment...${NC}"
    uv venv .venv --python 3.11 || uv venv .venv
else
    echo -e "${YELLOW}Virtual environment already exists${NC}"
fi

# Activate venv
source .venv/bin/activate

# Check for GPU
if command -v nvidia-smi &> /dev/null; then
    echo -e "${GREEN}NVIDIA GPU detected:${NC}"
    nvidia-smi --query-gpu=name,driver_version,memory.total --format=csv,noheader
    HAS_GPU=true
else
    echo -e "${YELLOW}No NVIDIA GPU detected. Training will not work without a GPU.${NC}"
    echo -e "${YELLOW}You can still generate datasets and prepare training data.${NC}"
    HAS_GPU=false
fi

# Install unsloth (includes all dependencies)
echo -e "${GREEN}Installing unsloth and dependencies...${NC}"
uv pip install unsloth

# Verify installation
echo ""
echo -e "${GREEN}Installation complete!${NC}"
echo ""
echo "Installed packages:"
uv pip list | grep -E "(unsloth|torch|transformers|trl|datasets|peft)" | head -10

echo ""
echo "============================================"
echo -e "${GREEN}Setup complete!${NC}"
echo ""
echo "To activate the environment:"
echo "  source .venv/bin/activate"
echo ""

if [ "$HAS_GPU" = true ]; then
    echo "To start training:"
    echo "  python scripts/finetune.py --data_path data/training_examples.json"
else
    echo "To generate training data (no GPU required):"
    echo "  python scripts/generate_dataset.py --num_examples 1000"
fi
