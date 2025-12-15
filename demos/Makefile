# Makefile for VHS demo generation
# Can be run from project root or demos/ directory

.PHONY: all quickstart features vancouver before-after social clean help

# Generate all demos
all: quickstart features vancouver before-after social
	@echo "✅ All demos generated!"
	@ls -lh *.gif *.mp4 2>/dev/null || true

# Individual demos
quickstart:
	@echo "🎬 Generating quickstart demo..."
	vhs caro-quickstart.tape
	@echo "✅ caro-quickstart.gif"

features:
	@echo "🎬 Generating features demo..."
	vhs caro-features.tape
	@echo "✅ caro-features.gif"

vancouver:
	@echo "🎬 Generating Vancouver.Dev demo..."
	vhs vancouver-dev-demo.tape
	@echo "✅ vancouver-dev-demo.gif"

before-after:
	@echo "🎬 Generating before/after demo..."
	vhs caro-before-after.tape
	@echo "✅ caro-before-after.gif"

social:
	@echo "🎬 Generating social media demo..."
	vhs caro-social.tape
	@echo "✅ caro-social.gif"

# Clean generated files
clean:
	@echo "🧹 Cleaning generated demos..."
	rm -f *.gif *.mp4 *.webm
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
