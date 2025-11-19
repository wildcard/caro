# cmdai Demo Makefile
#
# Convenient commands for recording and managing demos
#
# Usage:
#   make record-full    - Record full demo (2-3 min)
#   make record-short   - Record short demo (30 sec)
#   make play-full      - Play back full demo
#   make gif-full       - Convert full demo to GIF
#   make gif-short      - Convert short demo to GIF (for README)
#   make clean          - Remove all generated files
#   make help           - Show this help message

# Configuration
DEMO_SPEED ?= 50
PAUSE_SHORT ?= 1.5
PAUSE_MEDIUM ?= 2.5
PAUSE_LONG ?= 4.0

# Output files
CAST_FULL := cmdai_demo_full.cast
CAST_SHORT := cmdai_demo_short.cast
GIF_FULL := cmdai_demo.gif
GIF_SHORT := cmdai_demo_short.gif
SVG_FULL := cmdai_demo.svg
SVG_SHORT := cmdai_demo_short.svg

# GIF settings
GIF_SPEED ?= 1.5
GIF_FPS ?= 15
GIF_SPEED_SHORT ?= 2.0

.PHONY: help
help:
	@echo "cmdai Demo Management"
	@echo ""
	@echo "Recording:"
	@echo "  make record-full       Record full demo (2-3 minutes)"
	@echo "  make record-short      Record short demo (30 seconds)"
	@echo "  make record-manual     Start manual recording session"
	@echo ""
	@echo "Playback:"
	@echo "  make play-full         Play full demo"
	@echo "  make play-short        Play short demo"
	@echo "  make play-full-fast    Play full demo at 2x speed"
	@echo ""
	@echo "Conversion:"
	@echo "  make gif-full          Convert full demo to GIF"
	@echo "  make gif-short         Convert short demo to GIF (README-ready)"
	@echo "  make svg-full          Convert full demo to SVG"
	@echo "  make svg-short         Convert short demo to SVG"
	@echo "  make all-formats       Generate all output formats"
	@echo ""
	@echo "Publishing:"
	@echo "  make upload-full       Upload full demo to asciinema.org"
	@echo "  make upload-short      Upload short demo to asciinema.org"
	@echo ""
	@echo "Utilities:"
	@echo "  make test-script       Test demo script for errors"
	@echo "  make preview           Quick preview of full demo"
	@echo "  make check-size        Check GIF file sizes"
	@echo "  make optimize-gifs     Optimize GIF file sizes"
	@echo "  make clean             Remove all generated files"
	@echo ""
	@echo "Environment variables:"
	@echo "  DEMO_SPEED=50          Characters per second (default: 50)"
	@echo "  GIF_SPEED=1.5          Playback speed for GIF (default: 1.5)"
	@echo "  GIF_FPS=15             Frames per second (default: 15)"

# Recording targets
.PHONY: record-full
record-full:
	@echo "🎬 Recording full demo..."
	@echo "Press Ctrl+C to cancel, or wait for automatic completion"
	@chmod +x demo_script.sh
	DEMO_SPEED=$(DEMO_SPEED) asciinema rec $(CAST_FULL) -c ./demo_script.sh --overwrite
	@echo "✅ Recording saved to $(CAST_FULL)"

.PHONY: record-short
record-short:
	@echo "🎬 Recording short demo..."
	@chmod +x demo_short.sh
	DEMO_SPEED=$(DEMO_SPEED) asciinema rec $(CAST_SHORT) -c ./demo_short.sh --overwrite
	@echo "✅ Recording saved to $(CAST_SHORT)"

.PHONY: record-manual
record-manual:
	@echo "🎬 Starting manual recording..."
	@echo "Follow DEMO_STORYBOARD.md"
	@echo "Press Ctrl+D when done"
	@asciinema rec cmdai_demo_manual.cast
	@echo "✅ Recording saved to cmdai_demo_manual.cast"

# Playback targets
.PHONY: play-full
play-full:
	@echo "▶️  Playing full demo..."
	@test -f $(CAST_FULL) || (echo "❌ $(CAST_FULL) not found. Run 'make record-full' first." && exit 1)
	asciinema play $(CAST_FULL)

.PHONY: play-short
play-short:
	@echo "▶️  Playing short demo..."
	@test -f $(CAST_SHORT) || (echo "❌ $(CAST_SHORT) not found. Run 'make record-short' first." && exit 1)
	asciinema play $(CAST_SHORT)

.PHONY: play-full-fast
play-full-fast:
	@echo "▶️  Playing full demo at 2x speed..."
	@test -f $(CAST_FULL) || (echo "❌ $(CAST_FULL) not found. Run 'make record-full' first." && exit 1)
	asciinema play -s 2 $(CAST_FULL)

.PHONY: preview
preview:
	@echo "▶️  Quick preview (3x speed)..."
	@test -f $(CAST_FULL) || (echo "❌ $(CAST_FULL) not found. Run 'make record-full' first." && exit 1)
	asciinema play -s 3 $(CAST_FULL)

# Conversion targets
.PHONY: gif-full
gif-full:
	@echo "🎨 Converting full demo to GIF..."
	@test -f $(CAST_FULL) || (echo "❌ $(CAST_FULL) not found. Run 'make record-full' first." && exit 1)
	@command -v agg >/dev/null 2>&1 || (echo "❌ agg not installed. Run: cargo install agg" && exit 1)
	agg $(CAST_FULL) $(GIF_FULL) --speed $(GIF_SPEED) --fps $(GIF_FPS)
	@echo "✅ GIF saved to $(GIF_FULL)"
	@$(MAKE) check-size

.PHONY: gif-short
gif-short:
	@echo "🎨 Converting short demo to GIF (README-ready)..."
	@test -f $(CAST_SHORT) || (echo "❌ $(CAST_SHORT) not found. Run 'make record-short' first." && exit 1)
	@command -v agg >/dev/null 2>&1 || (echo "❌ agg not installed. Run: cargo install agg" && exit 1)
	agg $(CAST_SHORT) $(GIF_SHORT) --speed $(GIF_SPEED_SHORT) --fps $(GIF_FPS)
	@echo "✅ GIF saved to $(GIF_SHORT)"
	@$(MAKE) check-size

.PHONY: svg-full
svg-full:
	@echo "🎨 Converting full demo to SVG..."
	@test -f $(CAST_FULL) || (echo "❌ $(CAST_FULL) not found. Run 'make record-full' first." && exit 1)
	@command -v svg-term >/dev/null 2>&1 || (echo "❌ svg-term not installed. Run: npm install -g svg-term-cli" && exit 1)
	svg-term --in $(CAST_FULL) --out $(SVG_FULL) --window
	@echo "✅ SVG saved to $(SVG_FULL)"

.PHONY: svg-short
svg-short:
	@echo "🎨 Converting short demo to SVG..."
	@test -f $(CAST_SHORT) || (echo "❌ $(CAST_SHORT) not found. Run 'make record-short' first." && exit 1)
	@command -v svg-term >/dev/null 2>&1 || (echo "❌ svg-term not installed. Run: npm install -g svg-term-cli" && exit 1)
	svg-term --in $(CAST_SHORT) --out $(SVG_SHORT) --window
	@echo "✅ SVG saved to $(SVG_SHORT)"

.PHONY: all-formats
all-formats: gif-full gif-short svg-full svg-short
	@echo "✅ All formats generated"

# Publishing targets
.PHONY: upload-full
upload-full:
	@echo "☁️  Uploading full demo to asciinema.org..."
	@test -f $(CAST_FULL) || (echo "❌ $(CAST_FULL) not found. Run 'make record-full' first." && exit 1)
	asciinema upload $(CAST_FULL)

.PHONY: upload-short
upload-short:
	@echo "☁️  Uploading short demo to asciinema.org..."
	@test -f $(CAST_SHORT) || (echo "❌ $(CAST_SHORT) not found. Run 'make record-short' first." && exit 1)
	asciinema upload $(CAST_SHORT)

# Utility targets
.PHONY: test-script
test-script:
	@echo "🧪 Testing demo scripts for syntax errors..."
	@bash -n demo_script.sh && echo "✅ demo_script.sh: OK" || echo "❌ demo_script.sh: ERRORS"
	@bash -n demo_short.sh && echo "✅ demo_short.sh: OK" || echo "❌ demo_short.sh: ERRORS"

.PHONY: dry-run
dry-run:
	@echo "🏃 Dry run (fast execution)..."
	@chmod +x demo_script.sh
	DEMO_SPEED=500 PAUSE_SHORT=0.1 PAUSE_MEDIUM=0.1 PAUSE_LONG=0.1 ./demo_script.sh

.PHONY: check-size
check-size:
	@echo ""
	@echo "📊 File sizes:"
	@ls -lh *.gif 2>/dev/null | awk '{printf "  %s: %s\n", $$9, $$5}' || echo "  No GIF files found"
	@echo ""
	@echo "GitHub limit: 10MB for README GIFs"
	@echo "Recommended: <5MB for fast loading"

.PHONY: optimize-gifs
optimize-gifs:
	@echo "⚡ Optimizing GIFs..."
	@command -v gifsicle >/dev/null 2>&1 || (echo "❌ gifsicle not installed. Run: brew install gifsicle" && exit 1)
	@if [ -f $(GIF_FULL) ]; then \
		echo "Optimizing $(GIF_FULL)..."; \
		gifsicle -O3 --colors 256 $(GIF_FULL) -o $(GIF_FULL).tmp && mv $(GIF_FULL).tmp $(GIF_FULL); \
		echo "✅ $(GIF_FULL) optimized"; \
	fi
	@if [ -f $(GIF_SHORT) ]; then \
		echo "Optimizing $(GIF_SHORT)..."; \
		gifsicle -O3 --colors 256 $(GIF_SHORT) -o $(GIF_SHORT).tmp && mv $(GIF_SHORT).tmp $(GIF_SHORT); \
		echo "✅ $(GIF_SHORT) optimized"; \
	fi
	@$(MAKE) check-size

.PHONY: check-deps
check-deps:
	@echo "🔍 Checking dependencies..."
	@command -v asciinema >/dev/null 2>&1 && echo "✅ asciinema installed" || echo "❌ asciinema not found (required)"
	@command -v agg >/dev/null 2>&1 && echo "✅ agg installed" || echo "⚠️  agg not found (optional: cargo install agg)"
	@command -v svg-term >/dev/null 2>&1 && echo "✅ svg-term installed" || echo "⚠️  svg-term not found (optional: npm install -g svg-term-cli)"
	@command -v gifsicle >/dev/null 2>&1 && echo "✅ gifsicle installed" || echo "⚠️  gifsicle not found (optional: brew install gifsicle)"

.PHONY: clean
clean:
	@echo "🧹 Cleaning generated files..."
	rm -f *.cast *.gif *.svg *_manual.cast *.tmp
	@echo "✅ Clean complete"

# Complete workflow targets
.PHONY: demo-full-workflow
demo-full-workflow: record-full play-full gif-full
	@echo "✅ Full workflow complete!"
	@echo "GIF ready at: $(GIF_FULL)"

.PHONY: demo-short-workflow
demo-short-workflow: record-short play-short gif-short
	@echo "✅ Short workflow complete!"
	@echo "README-ready GIF at: $(GIF_SHORT)"

.PHONY: readme-demo
readme-demo: demo-short-workflow
	@echo ""
	@echo "📝 Add to README.md:"
	@echo ""
	@echo "![cmdai demo](demo/$(GIF_SHORT))"
	@echo ""

# Development helpers
.PHONY: watch-script
watch-script:
	@echo "👀 Watching demo scripts for changes..."
	@command -v fswatch >/dev/null 2>&1 || (echo "❌ fswatch not installed. Run: brew install fswatch" && exit 1)
	fswatch -o *.sh | xargs -n1 -I{} make test-script

.PHONY: stats
stats:
	@echo "📊 Demo Statistics:"
	@echo ""
	@if [ -f $(CAST_FULL) ]; then \
		echo "Full demo:"; \
		duration=$$(asciinema cat $(CAST_FULL) | jq -r '.[] | select(.type == "o") | .time' | tail -1); \
		echo "  Duration: $${duration}s"; \
		echo "  Size: $$(ls -lh $(CAST_FULL) | awk '{print $$5}')"; \
	fi
	@if [ -f $(CAST_SHORT) ]; then \
		echo "Short demo:"; \
		duration=$$(asciinema cat $(CAST_SHORT) | jq -r '.[] | select(.type == "o") | .time' | tail -1); \
		echo "  Duration: $${duration}s"; \
		echo "  Size: $$(ls -lh $(CAST_SHORT) | awk '{print $$5}')"; \
	fi

.DEFAULT_GOAL := help
