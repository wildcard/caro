# FunctionGemma CLI Tool Recommender

Fine-tuning FunctionGemma to recommend relevant CLI tools based on user queries, operating system, shell type, and preferences.

## Overview

This project fine-tunes Google's [FunctionGemma](https://ai.google.dev/gemma/docs/functiongemma) (270M parameter model) using [Unsloth](https://unsloth.ai/) for efficient training. The fine-tuned model can:

- Recommend CLI tools for specific tasks
- Consider OS and shell compatibility (POSIX, Linux, macOS, Ubuntu, BSD, Windows)
- Distinguish between default-installed and optional tools
- Suggest modern alternatives with installation commands
- Provide version-aware recommendations

## Project Structure

```
finetune-functiongemma/
├── README.md                    # This file
├── setup.sh                     # Quick setup script (uses uv)
├── requirements.txt             # Python dependencies
├── .venv/                       # Virtual environment (created by setup)
├── data/
│   ├── training_examples.json   # Hand-crafted training examples
│   └── training_data.json       # Auto-generated training data
├── scripts/
│   ├── finetune.py             # Main fine-tuning script
│   ├── inference.py            # Inference and recommendation
│   └── generate_dataset.py     # Dataset generation utilities
├── schemas/
│   └── tool_functions.py       # Function schemas for FunctionGemma
├── tools_kb/
│   └── cli_tools.json          # CLI tools knowledge base
├── configs/
│   └── (training configs)
├── notebooks/
│   └── (Jupyter notebooks)
└── output/
    └── (trained models)
```

## Requirements

- **GPU Required**: Unsloth requires an NVIDIA GPU with CUDA support
- **Python**: 3.10, 3.11, 3.12, or 3.13
- **uv** (recommended): 10x faster than pip

## Quick Start

### 1. Setup with uv (Recommended)

```bash
# Quick setup with uv
./setup.sh

# Or manually:
uv venv .venv --python 3.11
source .venv/bin/activate
uv pip install unsloth
```

### Alternative: pip install

```bash
python -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt
```

### 2. Generate Training Data

```bash
cd finetune-functiongemma
python scripts/generate_dataset.py --num_examples 1000 --output data/training_data.json
```

### 3. Fine-tune the Model

```bash
python scripts/finetune.py \
    --data_path data/training_data.json \
    --output_dir output \
    --epochs 3 \
    --batch_size 2
```

### 4. Run Inference

```bash
# Single query
python scripts/inference.py \
    --model_path output/final_model \
    --query "find all python files" \
    --os darwin \
    --shell zsh

# Interactive mode
python scripts/inference.py \
    --model_path output/final_model \
    --interactive

# Auto-detect system
python scripts/inference.py \
    --model_path output/final_model \
    --detect \
    --interactive
```

## Training Data Format

Training examples follow the FunctionGemma chat format with thinking blocks:

```json
{
  "id": "001",
  "context": {
    "os": "darwin",
    "shell": "zsh",
    "user_preferences": {"prefer_modern_tools": true}
  },
  "user_query": "find all python files",
  "conversation": [
    {"role": "developer", "content": "System prompt..."},
    {"role": "user", "content": "OS: darwin, Shell: zsh...\nQuery: find all python files"},
    {"role": "model", "content": "<think>reasoning...</think>\n<start_function_call>call:recommend_tools{...}<end_function_call>"}
  ]
}
```

## Function Schema

The model is trained to call `recommend_tools` with the following structure:

```json
{
  "primary_tools": [
    {
      "name": "find",
      "category": "file_management",
      "confidence": 1.0,
      "version_hint": "BSD find",
      "reason": "Standard POSIX tool installed by default"
    }
  ],
  "alternative_tools": [
    {
      "name": "fd",
      "category": "file_management",
      "install_cmd": "brew install fd",
      "reason": "Modern alternative with better UX",
      "improvements": ["faster", "simpler syntax", "colored output"]
    }
  ],
  "task_category": "file_management"
}
```

## Supported Platforms

### Operating Systems
- POSIX (generic)
- Linux (generic)
- macOS/Darwin
- Ubuntu/Debian
- Fedora/RHEL
- Arch Linux
- Alpine Linux
- BSD variants (FreeBSD, OpenBSD)
- Windows

### Shells
- sh (Bourne shell)
- bash
- zsh
- fish
- dash
- ksh
- tcsh
- PowerShell (pwsh)
- CMD (Windows)

## Tool Knowledge Base

The `tools_kb/cli_tools.json` file contains:

- 50+ common CLI tools
- Per-OS availability and confidence scores
- Installation commands for each platform
- Modern alternatives and their improvements
- Use case descriptions and common flags
- Shell builtins per shell type
- Package managers per OS

### Tool Categories
- `file_management` - ls, find, fd, tree
- `search` - grep, rg, ag, ack, fzf
- `text_processing` - sed, awk, cut, sort, uniq
- `file_viewing` - cat, less, head, tail, bat
- `network` - curl, wget, ssh, rsync
- `process` - ps, top, htop, kill
- `disk` - df, du, ncdu
- `archive` - tar, zip, gzip, 7z
- `json_processing` - jq, yq
- `version_control` - git, gh
- `containers` - docker, podman, kubectl
- `package_manager` - apt, brew, npm, pip

## Integration with Caro

This fine-tuned model is designed to integrate with [caro](https://github.com/your-repo/caro), providing intelligent CLI tool recommendations:

```rust
// In caro's inference pipeline
let tools = functiongemma.recommend_tools(
    user_query,
    os_type,
    shell_type,
    user_preferences,
);

// tools.primary contains definitely-available tools
// tools.alternatives contains optional upgrades
```

## Training Recommendations

### Google's Recommended Settings
- `top_k = 64`
- `top_p = 0.95`
- `temperature = 1.0`
- Max context: 32,768 tokens

### LoRA Configuration
- Rank: 16
- Alpha: 16
- Target modules: q_proj, k_proj, v_proj, o_proj, gate_proj, up_proj, down_proj

### Data Recommendations
- Minimum 500 diverse examples
- Cover all task categories
- Include all OS/shell combinations
- Balance default vs. alternative tool recommendations
- Include both simple and complex queries

## Model Export

The fine-tuning script supports multiple export formats:

```bash
# LoRA adapters only (smallest)
python scripts/finetune.py --save_method lora

# Merged 16-bit model
python scripts/finetune.py --save_method merged_16bit

# GGUF for llama.cpp (mobile/edge deployment)
python scripts/finetune.py --save_method gguf
```

## Contributing

### Adding New Tools

Edit `tools_kb/cli_tools.json`:

```json
"your_tool": {
  "name": "your_tool",
  "category": "category_name",
  "description": "What it does",
  "posix_standard": false,
  "availability": {
    "darwin": {"installed_by_default": false, "install_cmd": "brew install your_tool", "confidence": 0.3},
    "ubuntu": {"installed_by_default": false, "install_cmd": "apt install your_tool", "confidence": 0.2}
  },
  "improvements_over": ["existing_tool"],
  "improvements": ["faster", "better output"],
  "use_cases": ["use case 1", "use case 2"]
}
```

### Adding Query Templates

Edit `scripts/generate_dataset.py`:

```python
QUERY_TEMPLATES = {
    "new_category": [
        "template with {variable}",
        "another template",
    ],
    # ...
}
```

## Resources

- [FunctionGemma Documentation](https://ai.google.dev/gemma/docs/functiongemma)
- [Unsloth Fine-tuning Guide](https://unsloth.ai/docs/models/functiongemma)
- [FunctionGemma on Hugging Face](https://huggingface.co/google/functiongemma-270m-it)
- [Unsloth Colab Notebooks](https://github.com/unslothai/unsloth)

## License

This project is part of caro and follows the same license terms.
