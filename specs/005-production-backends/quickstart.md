# Quickstart: Production-Ready Backend System

**Phase 1 Output** | **Feature**: 005-production-backends | **Date**: 2025-10-14

## Overview

This quickstart guide validates the production-ready backend system integration by running end-to-end workflows that exercise all major components: SQLite command history, interactive configuration, advanced safety validation, streaming generation, and intelligent backend selection.

## Prerequisites

### System Requirements
- Rust 1.75+ with 2021 edition
- SQLite 3.35+ (bundled with rusqlite)
- 8GB+ RAM for local model backends
- 2GB free disk space for model cache

### Development Environment
```bash
# Ensure Rust toolchain is available
. "$HOME/.cargo/env"

# Verify cargo is accessible
which cargo

# Install development dependencies
cargo build --lib
```

### Backend Availability
```bash
# Check which backends are available
cargo run -- --list-backends

# Expected output should include at least one of:
# ✓ MLX (Apple Silicon only)
# ✓ Mock (always available for testing)
# ? Ollama (if running locally)
# ? vLLM (if configured)
```

## Quick Start Scenarios

### Scenario 1: First-Time Setup and Configuration

**Purpose**: Validate interactive configuration system with persistent storage

```bash
# Run interactive configuration
cargo run -- --configure

# Expected workflow:
# 1. Full-screen configuration interface launches
# 2. Backend preferences can be set
# 3. History settings can be configured
# 4. Safety levels can be adjusted
# 5. Configuration persists to TOML file
# 6. Changes take effect immediately

# Verify configuration persistence
cargo run -- --show-config
```

**Success Criteria**:
- Configuration UI renders without errors
- Settings save to `~/.config/cmdai/config.toml`
- `--show-config` displays current settings
- All validation rules enforced during input

### Scenario 2: Command Generation with History Storage

**Purpose**: Validate core generation workflow with automatic history storage

```bash
# Generate a simple command
cargo run -- "list all files in current directory"

# Expected workflow:
# 1. User input processed by selected backend
# 2. Generated command appears with explanation
# 3. Safety validation runs automatically
# 4. Command and metadata stored in history
# 5. Generation time < 2s on Apple Silicon

# Verify history storage
cargo run -- --history

# Expected output:
# Recent command history with timestamps
# Command: ls -la
# Explanation: Lists all files including hidden ones
# Timestamp: 2025-10-14T10:30:45Z
# Status: Safe
```

**Success Criteria**:
- Command generation completes successfully
- Response time meets constitutional requirements
- History entry created automatically
- All metadata fields populated correctly

### Scenario 3: Advanced Safety Validation

**Purpose**: Validate multi-modal safety system with risk assessment

```bash
# Test safe command
cargo run -- "show disk usage for current directory"
# Expected: Safe, no confirmation required

# Test moderate risk command  
cargo run -- "delete all temporary files"
# Expected: Moderate risk, explanation provided

# Test high risk command
cargo run -- "remove all files recursively"
# Expected: High/Critical risk, confirmation required
```

**Success Criteria**:
- Risk levels assigned correctly
- Detailed explanations provided for risky commands
- Confirmation workflows trigger appropriately
- Validation completes in <50ms

### Scenario 4: Semantic History Search

**Purpose**: Validate full-text and semantic search capabilities

```bash
# Generate several commands to build history
cargo run -- "find python files"
cargo run -- "count lines of code"
cargo run -- "show git status"
cargo run -- "list running processes"

# Search by text
cargo run -- --search "python"
# Expected: Returns find python files command

# Search by intent
cargo run -- --search "version control"
# Expected: Returns git status command

# Search with filters
cargo run -- --search "files" --since "today"
# Expected: Returns recent file-related commands
```

**Success Criteria**:
- Text search returns relevant results
- Semantic search understands intent
- Search completes in <50ms for 10K entries
- Results ranked by relevance

### Scenario 5: Streaming Generation

**Purpose**: Validate real-time generation with progress feedback

```bash
# Enable streaming mode
cargo run -- --stream "analyze this large codebase and find all TODO comments"

# Expected workflow:
# 1. Progress bar appears immediately
# 2. Partial commands shown during generation
# 3. Final command appears with explanation
# 4. Generation can be cancelled with Ctrl+C
# 5. First response within 500ms
```

**Success Criteria**:
- Progress feedback displays correctly
- Partial results show refinement
- Cancellation works reliably
- Performance targets met

### Scenario 6: Backend Selection and Fallback

**Purpose**: Validate intelligent backend routing with health monitoring

```bash
# Test automatic backend selection
cargo run -- --backend auto "complex analysis task"

# Test specific backend
cargo run -- --backend mlx "simple file operation"

# Test fallback behavior (simulate backend failure)
cargo run -- --backend unavailable_backend "test command"
# Expected: Automatic fallback to available backend
```

**Success Criteria**:
- Backend selection completes in <50ms
- Fallback chain executes correctly
- Performance metrics collected
- Health status updated appropriately

## Integration Testing

### End-to-End Workflow Test

**Purpose**: Validate complete system integration

```bash
# Complete workflow test
./tests/integration/end_to_end_test.sh

# This script should:
# 1. Reset configuration to defaults
# 2. Generate 10 diverse commands
# 3. Search history with various queries
# 4. Test safety validation with risky commands
# 5. Verify streaming generation
# 6. Test backend selection
# 7. Export and import configuration
# 8. Validate performance requirements
```

### Performance Validation

**Purpose**: Ensure constitutional performance requirements are met

```bash
# Run performance benchmarks
cargo test --test performance_benchmarks -- --nocapture

# Expected results:
# ✓ Startup time: <100ms
# ✓ First inference: <2s (Apple Silicon)
# ✓ Safety validation: <50ms
# ✓ History write: <10ms
# ✓ History search: <50ms
# ✓ Backend selection: <50ms
```

### Contract Test Validation

**Purpose**: Verify all API contracts are satisfied

```bash
# Run all contract tests
cargo test --test history_manager_contract
cargo test --test interactive_config_contract
cargo test --test safety_validator_contract
cargo test --test streaming_generator_contract
cargo test --test backend_selector_contract

# All tests should pass without errors
```

## Expected Test Results

### Successful Completion Indicators

1. **Configuration Management**:
   - Interactive UI renders correctly
   - Settings persist across restarts
   - Validation rules enforced
   - Real-time updates work

2. **Command Generation**:
   - Natural language processed correctly
   - Commands generated with explanations
   - Response times meet requirements
   - Multiple backends supported

3. **History System**:
   - Commands stored automatically
   - Metadata captured completely
   - Search functions work effectively
   - Performance targets met

4. **Safety Validation**:
   - Risk assessment accurate
   - Dangerous patterns detected
   - Confirmation workflows trigger
   - Validation speed adequate

5. **Streaming Support**:
   - Progress feedback functional
   - Partial results displayed
   - Cancellation mechanism works
   - Performance requirements met

6. **Backend Intelligence**:
   - Selection algorithm effective
   - Health monitoring active
   - Fallback chains execute
   - Load balancing functional

### Performance Benchmarks

| Component | Requirement | Expected |
|-----------|------------|----------|
| Startup time | <100ms | 45-80ms |
| First inference | <2s | 800ms-1.5s |
| Safety validation | <50ms | 15-35ms |
| History write | <10ms | 2-8ms |
| History search | <50ms | 10-30ms |
| Backend selection | <50ms | 5-25ms |

## Troubleshooting

### Common Issues

1. **Backend Not Available**:
   ```bash
   # Check backend status
   cargo run -- --list-backends
   # Solution: Install/configure required backend or use mock
   ```

2. **Performance Issues**:
   ```bash
   # Enable debug logging
   RUST_LOG=debug cargo run -- "test command"
   # Check for bottlenecks in output
   ```

3. **Database Errors**:
   ```bash
   # Reset history database
   rm ~/.config/cmdai/history.db
   # Restart application to recreate
   ```

4. **Configuration Problems**:
   ```bash
   # Reset to defaults
   rm ~/.config/cmdai/config.toml
   cargo run -- --configure
   ```

### Development Debugging

```bash
# Run with full logging
RUST_LOG=trace cargo run -- --verbose "debug command"

# Test specific components
cargo test history::models::tests --nocapture
cargo test safety::validator::tests --nocapture
cargo test streaming::generator::tests --nocapture

# Performance profiling
cargo build --release
time target/release/cmdai "performance test"
```

## Next Steps

After successful quickstart completion:

1. **Run Full Test Suite**: `cargo test`
2. **Execute Property Tests**: `cargo test --test property_tests`
3. **Performance Benchmarks**: `cargo test --test benchmarks`
4. **Integration Scenarios**: `./tests/integration/run_all.sh`

## Validation Checklist

- [ ] Interactive configuration system functional
- [ ] Command generation with multiple backends working
- [ ] History storage and search operational
- [ ] Safety validation detecting risky commands
- [ ] Streaming generation providing real-time feedback
- [ ] Backend selection intelligently routing requests
- [ ] All performance requirements met
- [ ] Contract tests passing
- [ ] Integration tests successful
- [ ] Error handling graceful throughout system