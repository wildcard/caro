# Makefile for VHS demo generation
# Run from demos/ directory: make all

.PHONY: all quickstart features vancouver before-after social clean help

# Get absolute path to binary
BINARY := $(shell cd .. && pwd)/target/release/cmdai
VHS_DIR := vhs

# Generate all demos
all: quickstart features vancouver before-after social
	@echo "✅ All demos generated!"
	@ls -lh $(VHS_DIR)/*.gif 2>/dev/null || true

# Individual demos - use sed to inject absolute path (with quotes)
quickstart:
	@echo "🎬 Generating quickstart demo..."
	@sed 's|Require ../target/release/cmdai|Require "$(BINARY)"|g' $(VHS_DIR)/caro-quickstart.tape > /tmp/caro-quickstart.tape.tmp
	@vhs /tmp/caro-quickstart.tape.tmp
	@mv caro-quickstart.gif $(VHS_DIR)/
	@rm /tmp/caro-quickstart.tape.tmp
	@echo "✅ $(VHS_DIR)/caro-quickstart.gif"

features:
	@echo "🎬 Generating features demo..."
	@sed 's|Require ../target/release/cmdai|Require "$(BINARY)"|g' $(VHS_DIR)/caro-features.tape > /tmp/caro-features.tape.tmp
	@vhs /tmp/caro-features.tape.tmp
	@mv caro-features.gif $(VHS_DIR)/
	@rm /tmp/caro-features.tape.tmp
	@echo "✅ $(VHS_DIR)/caro-features.gif"

vancouver:
	@echo "🎬 Generating Vancouver.Dev demo..."
	@sed 's|Require ../target/release/cmdai|Require "$(BINARY)"|g' $(VHS_DIR)/vancouver-dev-demo.tape > /tmp/vancouver-dev-demo.tape.tmp
	@vhs /tmp/vancouver-dev-demo.tape.tmp
	@mv vancouver-dev-demo.gif $(VHS_DIR)/
	@rm /tmp/vancouver-dev-demo.tape.tmp
	@echo "✅ $(VHS_DIR)/vancouver-dev-demo.gif"

before-after:
	@echo "🎬 Generating before/after demo..."
	@sed 's|Require ../target/release/cmdai|Require "$(BINARY)"|g' $(VHS_DIR)/caro-before-after.tape > /tmp/caro-before-after.tape.tmp
	@vhs /tmp/caro-before-after.tape.tmp
	@mv caro-before-after.gif $(VHS_DIR)/
	@rm /tmp/caro-before-after.tape.tmp
	@echo "✅ $(VHS_DIR)/caro-before-after.gif"

social:
	@echo "🎬 Generating social media demo..."
	@sed 's|Require ../target/release/cmdai|Require "$(BINARY)"|g' $(VHS_DIR)/caro-social.tape > /tmp/caro-social.tape.tmp
	@vhs /tmp/caro-social.tape.tmp
	@mv caro-social.gif $(VHS_DIR)/
	@rm /tmp/caro-social.tape.tmp
	@echo "✅ $(VHS_DIR)/caro-social.gif"

# Clean generated files
clean:
	@echo "🧹 Cleaning generated demos..."
	@rm -f $(VHS_DIR)/*.gif $(VHS_DIR)/*.mp4 $(VHS_DIR)/*.webm
	@echo "✅ Cleaned!"

# Help
help:
	@echo "Caro.sh VHS Demo Generator"
	@echo ""
	@echo "Usage:"
	@echo "  make all          Generate all demos"
	@echo "  make quickstart   Generate quickstart demo"
	@echo "  make features     Generate features demo"
	@echo "  make vancouver    Generate Vancouver.Dev demo"
	@echo "  make before-after Generate before/after comparison"
	@echo "  make social       Generate social media version"
	@echo "  make clean        Remove generated files"
	@echo "  make help         Show this help"
	@echo ""
	@echo "Prerequisites:"
	@echo "  - VHS installed (brew install vhs)"
	@echo "  - Binary built (cargo build --release --features embedded-mlx)"
	@echo ""
	@echo "Binary location: $(BINARY)"
	@echo "VHS tapes: $(VHS_DIR)/"
	@echo ""
	@echo "Note: VHS demos are for static GIFs (README, social media)"
	@echo "      For interactive demos, use: ./demo.sh (asciinema)"
