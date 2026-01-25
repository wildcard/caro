# Caro Operational Guide

> Concise operational reference for Ralph loops and AI agents.
> ~60 lines of essential commands, patterns, and learnings.

## Build & Run

```bash
# Development
cargo build                    # Debug build
cargo run -- "list files"      # Run with query

# Release
cargo build --release          # Optimized build
make release                   # Full release build

# Quick iteration
cargo check                    # Type check only (fast)
```

## Test Commands

```bash
# Primary validation
make check                     # lint + fmt + test + audit (USE THIS)

# Specific tests
make test                      # All tests (quiet)
make test-verbose              # Detailed output
make test-contract             # Contract tests only
make test-integration          # Integration tests

# Safety validation
cargo test --test safety_validator_contract
cargo test --test backend_trait_contract

# Beta regression
cargo test --test beta_regression
```

## Code Quality

```bash
make lint                      # Clippy (warnings = errors)
make fmt                       # Rustfmt check
make audit                     # Security audit
cargo tarpaulin                # Coverage report
```

## Key Patterns

### Module Structure
- `src/lib.rs` - All modules exported here
- `src/*/mod.rs` - Module entry points
- One concern per module, one file per struct

### TDD Cycle
1. Write failing test in `tests/` or `src/**/tests.rs`
2. `make test-verbose` - verify RED
3. Implement minimum code
4. `make test-verbose` - verify GREEN
5. Refactor, keep green
6. `make check` - full validation

### Safety Patterns
- Patterns in `src/safety/patterns/*.rs`
- 52+ dangerous command patterns
- Regex-based with severity levels
- Always run contract tests after changes

### Backend Trait
- `src/backends/mod.rs` - Backend trait definition
- Implementations: embedded (MLX/CPU), remote (Ollama/vLLM)
- Mock backend for testing

## Ralph Loop Commands

```bash
# Planning mode - analyze and generate tasks
./loop.sh plan

# Building mode - implement tasks iteratively
./loop.sh build

# With iteration limit
RALPH_MAX_ITERATIONS=10 ./loop.sh build

# View log
tail -f ralph.log
```

## Learnings

### Pattern Ordering Matters
More specific patterns must come before general ones. When adding patterns, check ordering against existing similar patterns.

### Test Before Commit
`make check` catches most issues. Never skip it.

### One Task Per Iteration
Ralph works best with focused, atomic tasks. Split large tasks.

### Plan Regeneration is Cheap
When stuck, run `./loop.sh plan` to regenerate the implementation plan.
