# FunctionGemma CLI Tool Recommender - Project Status & Roadmap

## Current Status: 70% Complete

The project has core infrastructure in place but needs validation, testing, and integration work.

---

## Gaps Analysis

### 1. Training Data Gaps

| Gap | Status | Priority |
|-----|--------|----------|
| Only 10 hand-crafted examples | ❌ Need 500+ | High |
| Limited OS coverage in examples | ❌ Missing BSD, Windows | Medium |
| No multi-turn conversations | ❌ Not implemented | Low |
| No tool response examples | ❌ Missing | Medium |

**What's Missing:**
- Training data only covers ~5 task categories well
- No examples for: `archive`, `version_control`, `containers`, `permissions`
- Missing Windows/PowerShell examples entirely
- No examples with tool responses (full conversation loop)

### 2. Code Gaps

| Component | Status | Gap Description |
|-----------|--------|-----------------|
| `finetune.py` | ✅ 90% | Needs GPU testing |
| `inference.py` | ⚠️ 70% | Still uses old manual format, needs update |
| `generate_dataset.py` | ⚠️ 60% | Doesn't generate HF-style tool_calls format |
| `tool_functions.py` | ⚠️ 50% | Schemas not aligned with HF format |

### 3. Integration Gaps

| Integration | Status | Description |
|-------------|--------|-------------|
| Caro CLI integration | ❌ Not started | No bridge to main caro project |
| Model export for Rust | ❌ Not started | Need GGUF export + Rust loader |
| Real-time inference | ❌ Not started | No streaming support |

---

## Testing Requirements

### Phase 1: Unit Tests (No GPU Required)

```bash
# Test 1: Data format validation
python -c "
from scripts.finetune import prepare_messages_and_tools, CLI_TOOL_SCHEMA
import json

# Load examples
with open('data/training_examples.json') as f:
    data = json.load(f)

# Test each example
for ex in data['examples']:
    msgs, tools = prepare_messages_and_tools(ex)
    assert msgs is not None, f'Failed: {ex[\"id\"]}'
    assert len(tools) > 0
    print(f'✓ Example {ex[\"id\"]} valid')
"

# Test 2: Dataset generation
python scripts/generate_dataset.py --num_examples 10 --output /tmp/test_data.json
python -c "
import json
with open('/tmp/test_data.json') as f:
    data = json.load(f)
assert len(data['examples']) == 10
print('✓ Dataset generation works')
"

# Test 3: Tool schema validation
python -c "
from schemas.tool_functions import FUNCTION_SCHEMAS
assert 'recommend_tools' in FUNCTION_SCHEMAS
print('✓ Schemas valid')
"
```

### Phase 2: Integration Tests (GPU Required)

```bash
# Test 4: Model loading
python -c "
from unsloth import FastLanguageModel
model, tokenizer = FastLanguageModel.from_pretrained(
    'unsloth/functiongemma-270m-it',
    max_seq_length=4096,
    load_in_16bit=True,
)
print(f'✓ Model loaded: {model.num_parameters():,} params')
"

# Test 5: Chat template formatting
python -c "
from unsloth import FastLanguageModel

model, tokenizer = FastLanguageModel.from_pretrained(
    'unsloth/functiongemma-270m-it',
    max_seq_length=4096,
    load_in_16bit=True,
)

tools = [{
    'type': 'function',
    'function': {
        'name': 'recommend_tools',
        'description': 'Recommend CLI tools',
        'parameters': {'type': 'object', 'properties': {}}
    }
}]

messages = [
    {'role': 'system', 'content': 'You are a CLI assistant.'},
    {'role': 'user', 'content': 'OS: darwin\\nQuery: find files'},
]

result = tokenizer.apply_chat_template(messages, tools=tools, tokenize=False)
assert '<start_function_declaration>' in result
print('✓ Chat template works with tools')
"

# Test 6: Training smoke test (1 step)
python scripts/finetune.py \
    --data_path data/training_examples.json \
    --output_dir /tmp/test_output \
    --max_steps 1 \
    --batch_size 1

# Test 7: Inference test
python scripts/inference.py \
    --model_path /tmp/test_output/final_model \
    --query "find all python files" \
    --os darwin \
    --shell zsh
```

### Phase 3: End-to-End Validation

```bash
# Full training run (500 steps)
python scripts/finetune.py \
    --data_path data/training_data.json \
    --output_dir ./output \
    --max_steps 500 \
    --lora_r 128

# Evaluate on test queries
python -c "
test_queries = [
    ('find all python files', 'darwin', 'zsh'),
    ('search for error in logs', 'ubuntu', 'bash'),
    ('compress a folder', 'linux', 'fish'),
    ('view running processes', 'darwin', 'bash'),
    ('parse JSON response', 'ubuntu', 'zsh'),
]

# Run inference on each
from scripts.inference import FunctionGemmaInference, SystemContext

inference = FunctionGemmaInference('./output/final_model')
inference.load_model()

for query, os_type, shell in test_queries:
    ctx = SystemContext(os_type=os_type, shell=shell)
    result = inference.recommend(query, ctx)

    # Validate output structure
    assert len(result.primary_tools) > 0, f'No primary tools for: {query}'
    assert result.task_category, f'No category for: {query}'

    print(f'✓ {query}: {[t.name for t in result.primary_tools]}')
"
```

---

## Path to Completion

### Step 1: Fix Data Format Alignment (Local Agent Task)

**Task:** Update `generate_dataset.py` to produce HF-style training data.

```
PROMPT FOR AGENT:

Update finetune-functiongemma/scripts/generate_dataset.py to:

1. Generate training examples in HF-style format with tool_calls:
   {
     "messages": [
       {"role": "system", "content": "..."},
       {"role": "user", "content": "OS: darwin, Shell: zsh\nQuery: find files"},
       {
         "role": "assistant",
         "content": "<think>reasoning here</think>",
         "tool_calls": [{
           "id": "call_1",
           "type": "function",
           "function": {
             "name": "recommend_tools",
             "arguments": {...}
           }
         }]
       }
     ]
   }

2. Ensure all examples have valid <think>...</think> blocks
3. Cover all task categories in QUERY_TEMPLATES
4. Generate diverse OS/shell combinations

Test with:
  python scripts/generate_dataset.py --num_examples 100
  python -c "import json; d=json.load(open('data/training_data.json')); print(len(d['examples']), 'examples')"
```

### Step 2: Update Inference Script (Local Agent Task)

**Task:** Update `inference.py` to use `apply_chat_template`.

```
PROMPT FOR AGENT:

Update finetune-functiongemma/scripts/inference.py to:

1. Use tokenizer.apply_chat_template() instead of manual string building
2. Pass tools=[CLI_TOOL_SCHEMA] to apply_chat_template
3. Parse tool_calls from model response properly
4. Update SystemContext.detect() to work cross-platform

Test with:
  python scripts/inference.py --detect --interactive
```

### Step 3: Generate Large Dataset (Local Agent Task)

**Task:** Generate 1000+ diverse training examples.

```
PROMPT FOR AGENT:

Generate a high-quality training dataset:

1. Run: python scripts/generate_dataset.py --num_examples 1000 --seed 42
2. Verify coverage of all categories
3. Check for duplicate queries
4. Validate JSON structure

Quality checks:
- Each OS type has at least 100 examples
- Each task category has at least 50 examples
- No duplicate queries
- All examples have valid thinking blocks
```

### Step 4: Train and Evaluate (GPU Required)

**Task:** Fine-tune on GPU and evaluate.

```
PROMPT FOR AGENT:

Train the model on Google Colab or GPU machine:

1. Upload finetune-functiongemma/ to Colab
2. Run the notebook: notebooks/finetune_colab.ipynb
3. Download trained model
4. Run evaluation:
   python scripts/inference.py --model_path ./output/final_model --interactive

Evaluate on these queries:
- "find all javascript files modified today"
- "search for TODO comments in code"
- "monitor CPU and memory usage"
- "compress logs folder to tar.gz"
- "parse JSON from curl response"

Save output examples for review.
```

### Step 5: Integration with Caro (Future)

**Task:** Integrate trained model into caro CLI.

```
REQUIREMENTS:

1. Export model to GGUF format:
   model.save_pretrained_gguf("caro-tool-recommender", tokenizer, quantization_method="Q8_0")

2. Create Rust bindings using llama.cpp or candle

3. Add to caro's inference pipeline:
   - Before command generation, call tool recommender
   - Use recommended tools in system prompt
   - Suggest installs for missing tools

4. CLI integration:
   caro --suggest-tools "find all python files"
   # Output: Recommended: find (installed), fd (install: brew install fd)
```

---

## File Checklist

| File | Status | Action Needed |
|------|--------|---------------|
| `scripts/finetune.py` | ✅ Ready | Test on GPU |
| `scripts/inference.py` | ⚠️ Update | Use apply_chat_template |
| `scripts/generate_dataset.py` | ⚠️ Update | HF-style format |
| `data/training_examples.json` | ⚠️ Limited | Need more examples |
| `tools_kb/cli_tools.json` | ✅ Good | Add more tools |
| `schemas/tool_functions.py` | ⚠️ Update | Align with HF format |
| `notebooks/finetune_colab.ipynb` | ✅ Ready | Test on Colab |
| `configs/training_config.yaml` | ✅ Ready | - |

---

## Quick Validation Script

Run this to check current project health:

```bash
#!/bin/bash
# save as: validate_project.sh

cd /home/user/caro/finetune-functiongemma

echo "=== Project Validation ==="

# 1. Check files exist
echo -n "Files: "
required_files=(
    "scripts/finetune.py"
    "scripts/inference.py"
    "scripts/generate_dataset.py"
    "data/training_examples.json"
    "tools_kb/cli_tools.json"
)
for f in "${required_files[@]}"; do
    if [ ! -f "$f" ]; then
        echo "MISSING: $f"
        exit 1
    fi
done
echo "✓ All required files present"

# 2. Check Python syntax
echo -n "Syntax: "
for f in scripts/*.py schemas/*.py; do
    python -m py_compile "$f" 2>/dev/null || { echo "FAIL: $f"; exit 1; }
done
echo "✓ Python syntax OK"

# 3. Check training data
echo -n "Data: "
count=$(python -c "import json; print(len(json.load(open('data/training_examples.json'))['examples']))")
echo "✓ $count training examples"

# 4. Check tools KB
echo -n "Tools KB: "
tools=$(python -c "import json; print(len(json.load(open('tools_kb/cli_tools.json'))['tools']))")
echo "✓ $tools tools in knowledge base"

echo ""
echo "=== Status: READY FOR TESTING ==="
```

---

## Commands Summary

```bash
# Setup
./setup.sh

# Generate data
python scripts/generate_dataset.py --num_examples 1000

# Train (GPU)
python scripts/finetune.py --data_path data/training_data.json --max_steps 500

# Inference
python scripts/inference.py --model_path ./output/final_model --interactive

# Export GGUF
python -c "
from unsloth import FastLanguageModel
model, tokenizer = FastLanguageModel.from_pretrained('./output/final_model')
model.save_pretrained_gguf('./output/gguf', tokenizer, quantization_method='Q8_0')
"
```
