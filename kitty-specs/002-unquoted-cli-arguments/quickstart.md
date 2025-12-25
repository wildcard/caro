# Developer Quickstart: Unquoted CLI Arguments

**Feature**: 002-unquoted-cli-arguments
**Status**: Implementation Phase
**Last Updated**: 2025-12-25

This guide helps developers quickly understand and test the unquoted CLI arguments feature.

## What This Feature Does

Allows users to type natural language prompts without quotes while maintaining full backward compatibility.

**Before** (still works):
```bash
caro "list all files"
```

**After** (new capability):
```bash
caro list all files
```

## Quick Test Commands

### Basic Unquoted Prompts
```bash
# Simple prompt
caro list files

# Multi-word prompt
caro find large files in current directory

# With flags before prompt
caro --verbose list all files

# Backward compatibility (still works)
caro "list files"
```

### Multiple Input Methods

```bash
# Explicit prompt flag (highest priority)
caro -p "list files"
caro --prompt "find files"

# Stdin input (second priority)
echo "list files" | caro

# Trailing arguments (default/fallback)
caro list files
```

### Shell Operator Handling

```bash
# Output redirection (> terminates prompt)
caro list files > output.txt
# Prompt is "list files", output redirected to output.txt

# Pipe to another command (| terminates prompt)
caro find files | grep txt
# Prompt is "find files", output piped to grep

# Other operators work similarly
caro list files >> log.txt  # Append redirect
caro list files 2> err.txt  # Error redirect
caro list files ; ls        # Command separator
```

### Edge Cases

```bash
# Empty input shows help
caro

# Whitespace-only shows help
caro

# Flags mixed with prompt
caro --dry-run --shell zsh list all files
# Flags parsed first, "list all files" becomes prompt
```

## Implementation Files

### Primary Changes
- **src/cli/mod.rs** - clap configuration with `trailing_var_arg = true`
- **src/main.rs** - Prompt source resolution (flag > stdin > args)

### Testing
- **tests/e2e_cli_tests.rs** - End-to-end CLI tests for unquoted prompts
- **tests/execution_prompt_behavior.rs** - Tests for all input methods
- **tests/cli_interface_contract.rs** - Contract tests for argument patterns

## Development Workflow

### 1. Run Tests
```bash
# All tests
cargo test

# Specific test file
cargo test --test e2e_cli_tests

# With logging
RUST_LOG=debug cargo test
```

### 2. Build and Test Locally
```bash
# Build release binary
cargo build --release

# Test unquoted prompt
./target/release/caro list files

# Test with flags
./target/release/caro --dry-run list files

# Test stdin
echo "list files" | ./target/release/caro
```

### 3. Cross-Platform Validation
```bash
# Linux
cargo test --target x86_64-unknown-linux-gnu

# macOS Intel
cargo test --target x86_64-apple-darwin

# macOS Apple Silicon
cargo test --target aarch64-apple-darwin

# Windows
cargo test --target x86_64-pc-windows-msvc
```

## Key Design Patterns

### Prompt Source Resolution
```rust
// Priority order: flag > stdin > trailing args
let prompt = if let Some(p) = args.prompt_flag {
    p  // -p/--prompt flag (highest priority)
} else if stdin_available() {
    read_stdin()  // stdin piped input
} else {
    args.trailing_args.join(" ")  // unquoted words (default)
};
```

### Shell Operator Detection
```rust
// Detect POSIX metacharacters that terminate prompt
const SHELL_OPERATORS: &[&str] = &[">", "|", "<", ">>", "2>", "&", ";"];

fn truncate_at_operator(args: Vec<String>) -> Vec<String> {
    args.into_iter()
        .take_while(|arg| !SHELL_OPERATORS.contains(&arg.as_str()))
        .collect()
}
```

### Empty Input Handling
```rust
// Show help for empty/whitespace-only input
let prompt = prompt.trim();
if prompt.is_empty() {
    show_help();
    return Ok(());
}
```

## Testing Scenarios

### Functional Requirements Coverage

| FR | Description | Test Command |
|----|-------------|--------------|
| FR-001 | Unquoted text as prompt | `caro list files` |
| FR-002 | Backward compatibility | `caro "list files"` |
| FR-003 | Flags before prompt | `caro --verbose list files` |
| FR-004 | Help on empty input | `caro` |
| FR-005 | -p/--prompt flag | `caro -p "list files"` |
| FR-006 | Stdin support | `echo "list files" \| caro` |
| FR-007 | Shell operator handling | `caro list files > out.txt` |
| FR-008 | Input prioritization | Multiple input sources |
| FR-009 | Word joining | `caro list all files` → "list all files" |
| FR-010 | Special char preservation | `caro find *.txt files` |
| FR-011 | Cross-platform tests | CI validation |
| FR-012 | PR #68 test fixes | All platforms pass |

### Success Criteria Validation

| SC | Criteria | Validation Method |
|----|----------|-------------------|
| SC-001 | 100% accuracy for 2-5 word prompts | E2E tests with common prompts |
| SC-002 | Backward compatibility | All existing tests pass unchanged |
| SC-003 | Cross-platform tests pass | CI checks on Linux/macOS/Windows |
| SC-004 | Help displays correctly | Manual and automated tests |
| SC-005 | Non-interactive mode works | `-p` flag tests without confirmation |
| SC-006 | Stdin processing works | Pipe tests from various sources |
| SC-007 | 100% shell operator accuracy | Tests for all 7 operators |

## Troubleshooting

### Tests Failing
```bash
# Check which tests are failing
cargo test 2>&1 | grep FAILED

# Run specific failing test with output
cargo test test_name -- --nocapture

# Enable debug logging
RUST_LOG=debug cargo test test_name
```

### Shell Operators Not Working
```bash
# Verify operator detection logic
cargo test shell_operator_detection

# Test manually with quotes to bypass shell
caro 'list files > output.txt'  # Should fail (> not detected in quoted string)
caro list files > output.txt    # Should work (> detected and handled by shell)
```

### Backward Compatibility Issues
```bash
# Run all existing tests
cargo test

# Specific contract tests
cargo test --test cli_interface_contract
```

## Performance Benchmarks

Argument parsing performance target: **< 10ms**

```bash
# Measure startup time
time cargo run --release -- list files

# Profile argument parsing
cargo build --release
hyperfine './target/release/caro list files' './target/release/caro "list files"'
```

## Next Steps After Implementation

1. **Run `/spec-kitty.tasks`** - Generate implementation task breakdown
2. **Run `/spec-kitty.implement`** - Execute TDD workflow
3. **Run `/spec-kitty.review`** - Code review and quality checks
4. **Run `/spec-kitty.accept`** - Final acceptance testing
5. **Run `/spec-kitty.merge`** - Merge to main branch

## References

- **Feature Spec**: [spec.md](spec.md)
- **Implementation Plan**: [plan.md](plan.md)
- **Research Decisions**: [research.md](research.md)
- **Data Model**: [data-model.md](data-model.md)
- **API Contracts**: [contracts/](contracts/)
