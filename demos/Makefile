# Makefile for VHS demo generation
# Can be run from project root or demos/ directory

.PHONY: all quickstart features vancouver before-after social clean help

# Get absolute path to binary
BINARY := $(shell cd .. && pwd)/target/release/cmdai

# Generate all demos
all: quickstart features vancouver before-after social
	@echo "✅ All demos generated!"
	@ls -lh *.gif *.mp4 2>/dev/null || true

# Individual demos - use sed to inject absolute path
quickstart:
	@echo "🎬 Generating quickstart demo..."
	@sed "s|Require ../target/release/cmdai|Require $(BINARY)|g" caro-quickstart.tape | vhs -
	@echo "✅ caro-quickstart.gif"

features:
	@echo "🎬 Generating features demo..."
	@sed "s|Require ../target/release/cmdai|Require $(BINARY)|g" caro-features.tape | vhs -
	@echo "✅ caro-features.gif"

vancouver:
	@echo "🎬 Generating Vancouver.Dev demo..."
	@sed "s|Require ../target/release/cmdai|Require $(BINARY)|g" vancouver-dev-demo.tape | vhs -
	@echo "✅ vancouver-dev-demo.gif"

before-after:
	@echo "🎬 Generating before/after demo..."
	@sed "s|Require ../target/release/cmdai|Require $(BINARY)|g" caro-before-after.tape | vhs -
	@echo "✅ caro-before-after.gif"

social:
	@echo "🎬 Generating social media demo..."
	@sed "s|Require ../target/release/cmdai|Require $(BINARY)|g" caro-social.tape | vhs -
	@echo "✅ caro-social.gif"

# Clean generated files
clean:
	@echo "🧹 Cleaning generated demos..."
	@rm -f *.gif *.mp4 *.webm
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
