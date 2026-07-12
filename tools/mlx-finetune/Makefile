# MLX LoRA fine-tuning pipeline for caro.
#
# Runs on Apple Silicon (tested on M4 Max). Produces a GGUF Q4_K_M drop-in
# for caro's embedded backend. See README.md for full prerequisites.
#
# Quick start:
#   make venv           # one-time: create Python env with mlx-lm
#   make all            # full pipeline (data -> convert -> lora -> fuse -> gguf)
#
# Or step-by-step:
#   make data           # build train/valid/test JSONL from caro eval suite
#   make convert        # download Qwen2.5-Coder-1.5B-Instruct, quantize to MLX
#   make lora           # LoRA train (~15-30 min on M4 Max)
#   make fuse           # merge adapter into base weights
#   make gguf           # convert fused safetensors -> GGUF Q4_K_M
#   make eval           # run caro's regression harness against fine-tuned model

REPO_ROOT   := $(shell git -C $(CURDIR) rev-parse --show-toplevel)
VENV        := $(CURDIR)/.venv
PY          := $(VENV)/bin/python
PIP         := $(VENV)/bin/pip
MLX_LM      := $(VENV)/bin/mlx_lm
HF_MODEL    := Qwen/Qwen2.5-Coder-1.5B-Instruct
MLX_MODEL   := $(CURDIR)/qwen-mlx
ADAPTERS    := $(CURDIR)/adapters
FUSED       := $(CURDIR)/qwen-caro-fused
DATA        := $(CURDIR)/data
SYS_PROMPT  := $(CURDIR)/system_prompt.txt
GGUF_OUT    := $(CURDIR)/qwen2.5-coder-1.5b-caro-v1-q4_k_m.gguf
LLAMA_CPP   ?= $(HOME)/src/llama.cpp

# Tunables — override at invocation, e.g. `make lora ITERS=1000 LR=5e-5`
ITERS       ?= 600
LR          ?= 1e-4
BATCH       ?= 4
LORA_LAYERS ?= 16
SEED        ?= 42

.PHONY: all venv data convert lora fuse gguf eval clean distclean help

help:
	@echo "Targets: venv data convert lora fuse gguf eval all clean distclean"
	@echo "Tunables: ITERS=$(ITERS) LR=$(LR) BATCH=$(BATCH) LORA_LAYERS=$(LORA_LAYERS)"
	@echo "Paths: LLAMA_CPP=$(LLAMA_CPP)"

all: gguf
	@echo ""
	@echo "Pipeline complete. Next:"
	@echo "  1. Upload $(GGUF_OUT) to a HuggingFace repo"
	@echo "  2. Add entry to $(REPO_ROOT)/src/model_catalog.rs"
	@echo "  3. Run: make eval"

venv: $(VENV)/.stamp
$(VENV)/.stamp:
	python3 -m venv $(VENV)
	$(PIP) install --upgrade pip
	$(PIP) install mlx-lm pyyaml
	@touch $@

# ---------- Step 1: dataset ----------
$(SYS_PROMPT):
	cd $(REPO_ROOT) && cargo run --quiet --example render_system_prompt \
		--no-default-features --features embedded-cpu > $@

$(DATA)/train.jsonl: $(SYS_PROMPT) build_dataset.py | venv
	$(PY) build_dataset.py --system-prompt $(SYS_PROMPT) --out $(DATA) --seed $(SEED)

data: $(DATA)/train.jsonl

# ---------- Step 2: base-model MLX conversion ----------
$(MLX_MODEL)/config.json: | venv
	$(MLX_LM).convert --hf-path $(HF_MODEL) --mlx-path $(MLX_MODEL) -q

convert: $(MLX_MODEL)/config.json

# ---------- Step 3: LoRA training ----------
$(ADAPTERS)/adapter_config.json: data convert
	$(MLX_LM).lora \
		--model $(MLX_MODEL) \
		--train \
		--data $(DATA) \
		--adapter-path $(ADAPTERS) \
		--batch-size $(BATCH) \
		--iters $(ITERS) \
		--lora-layers $(LORA_LAYERS) \
		--learning-rate $(LR) \
		--save-every 100 \
		--seed $(SEED)

lora: $(ADAPTERS)/adapter_config.json

# ---------- Step 4: fuse adapter into base ----------
$(FUSED)/config.json: lora
	$(MLX_LM).fuse \
		--model $(MLX_MODEL) \
		--adapter-path $(ADAPTERS) \
		--save-path $(FUSED) \
		--export-gguf=false

fuse: $(FUSED)/config.json

# ---------- Step 5: safetensors -> GGUF Q4_K_M ----------
$(GGUF_OUT): fuse
	@test -d $(LLAMA_CPP) || { echo "ERROR: llama.cpp not found at LLAMA_CPP=$(LLAMA_CPP)"; echo "  set LLAMA_CPP=/path/to/llama.cpp or clone: git clone https://github.com/ggerganov/llama.cpp"; exit 1; }
	cd $(LLAMA_CPP) && python3 convert_hf_to_gguf.py $(FUSED) --outtype f16 --outfile $(FUSED)/model-f16.gguf
	$(LLAMA_CPP)/llama-quantize $(FUSED)/model-f16.gguf $@ Q4_K_M

gguf: $(GGUF_OUT)

# ---------- Step 6: regression eval ----------
eval:
	@test -f $(GGUF_OUT) || { echo "ERROR: $(GGUF_OUT) not found, run 'make gguf' first"; exit 1; }
	@echo "Place $(GGUF_OUT) in caro's cache dir then run:"
	@echo "  cd $(REPO_ROOT) && cargo test --release --package caro -- evaluation"
	@echo "Compare pass-rate against the stock qwen-1.5b-q4 baseline."

# ---------- cleanup ----------
clean:
	rm -rf $(DATA) $(SYS_PROMPT) $(ADAPTERS)

distclean: clean
	rm -rf $(VENV) $(MLX_MODEL) $(FUSED) $(GGUF_OUT)
