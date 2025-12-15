# Makefile for VHS demo generation

.PHONY: all quickstart features vancouver before-after social clean help

# Generate all demos
all: quickstart features vancouver before-after social
	@echo "✅ All demos generated!"
	@ls -lh demos/*.gif demos/*.mp4 2>/dev/null || true

# Individual demos
quickstart:
	@echo "🎬 Generating quickstart demo..."
	vhs demos/caro-quickstart.tape
	@echo "✅ demos/caro-quickstart.gif"

features:
	@echo "🎬 Generating features demo..."
	vhs demos/caro-features.tape
	@echo "✅ demos/caro-features.gif"

vancouver:
	@echo "🎬 Generating Vancouver.Dev demo..."
	vhs demos/vancouver-dev-demo.tape
	@echo "✅ demos/vancouver-dev-demo.gif"

before-after:
	@echo "🎬 Generating before/after demo..."
	vhs demos/caro-before-after.tape
	@echo "✅ demos/caro-before-after.gif"

social:
	@echo "🎬 Generating social media demo..."
	vhs demos/caro-social.tape
	@echo "✅ demos/caro-social.gif"

# Clean generated files
clean:
	@echo "🧹 Cleaning generated demos..."
	rm -f demos/*.gif demos/*.mp4 demos/*.webm
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
