# Contract: CLI Argument Parsing

**Version**: 1.0.0
**Feature**: 002-unquoted-cli-arguments
**Component**: `src/cli/mod.rs`
**Created**: 2025-12-25

## Purpose

Defines the behavioral contract for parsing command-line arguments using clap with support for both quoted and unquoted prompts.

## Interface

### Input
```rust
// Raw command-line arguments from shell
std::env::args()
```

### Output
```rust
pub struct CliArgs {
    pub prompt_flag: Option<String>,     // Value from -p/--prompt flag
    pub trailing_args: Vec<String>,      // Unquoted words after all flags
    pub verbose: bool,                   // --verbose flag
    pub dry_run: bool,                   // --dry-run flag
    pub shell: Option<String>,           // --shell flag value
    // ... other existing flags
}
```

## Contract Rules

### Rule 1: Accept Trailing Variable Arguments

**Requirement**: FR-001 (System MUST accept unquoted text after all flags as the prompt)

**Given**: User invokes `caro list files`
**When**: clap parses arguments
**Then**:
- `trailing_args` = `["list", "files"]`
- `prompt_flag` = `None`

**Implementation**:
```rust
#[derive(Parser)]
pub struct CliArgs {
    #[arg(trailing_var_arg = true, num_args = 0..)]
    pub trailing_args: Vec<String>,
}
```

### Rule 2: Preserve Quoted Prompt Compatibility

**Requirement**: FR-002 (System MUST maintain backward compatibility with quoted prompts)

**Given**: User invokes `caro "list files"`
**When**: clap parses arguments
**Then**:
- `trailing_args` = `["list files"]` (single element with spaces)
- `prompt_flag` = `None`

**Note**: Shell removes quotes before passing to program, so "list files" arrives as single argument.

### Rule 3: Parse Flags Before Prompt

**Requirement**: FR-003 (System MUST parse command-line arguments with flags before the prompt)

**Given**: User invokes `caro --verbose --shell zsh list files`
**When**: clap parses arguments
**Then**:
- `verbose` = `true`
- `shell` = `Some("zsh")`
- `trailing_args` = `["list", "files"]`

**Validation**: All flags must be recognized before `trailing_args` begins.

### Rule 4: Support Explicit Prompt Flag

**Requirement**: FR-005 (System MUST support -p or --prompt flag for explicit prompt specification)

**Given**: User invokes `caro -p "list files"`
**When**: clap parses arguments
**Then**:
- `prompt_flag` = `Some("list files")`
- `trailing_args` = `[]`

**Given**: User invokes `caro -p "list files" extra args`
**When**: clap parses arguments
**Then**:
- `prompt_flag` = `Some("list files")`
- `trailing_args` = `["extra", "args"]`

**Note**: Prompt source resolution (separate component) will prioritize `prompt_flag` over `trailing_args`.

### Rule 5: Empty Trailing Arguments

**Requirement**: FR-004 (System MUST show help message when invoked with no arguments)

**Given**: User invokes `caro`
**When**: clap parses arguments
**Then**:
- `prompt_flag` = `None`
- `trailing_args` = `[]`

**Validation**: Downstream validation component will detect empty prompt and show help.

### Rule 6: Special Characters Preservation

**Requirement**: FR-010 (System MUST preserve special characters within the prompt text)

**Given**: User invokes `caro find *.txt files`
**When**: clap parses arguments
**Then**:
- `trailing_args` = `["find", "*.txt", "files"]`

**Note**: Shell may expand `*.txt` to matching files before passing to program. User must quote if they want literal `*`: `caro find '*.txt' files` → `["find", "*.txt", "files"]`

## Test Cases

### TC1: Basic Unquoted Prompt
```rust
#[test]
fn test_unquoted_prompt() {
    let args = CliArgs::parse_from(vec!["caro", "list", "files"]);
    assert_eq!(args.trailing_args, vec!["list", "files"]);
    assert!(args.prompt_flag.is_none());
}
```

### TC2: Quoted Prompt (Backward Compatibility)
```rust
#[test]
fn test_quoted_prompt() {
    // Shell removes quotes, so this arrives as single arg
    let args = CliArgs::parse_from(vec!["caro", "list files"]);
    assert_eq!(args.trailing_args, vec!["list files"]);
    assert!(args.prompt_flag.is_none());
}
```

### TC3: Flags Before Prompt
```rust
#[test]
fn test_flags_before_prompt() {
    let args = CliArgs::parse_from(vec!["caro", "--verbose", "list", "files"]);
    assert!(args.verbose);
    assert_eq!(args.trailing_args, vec!["list", "files"]);
}
```

### TC4: Explicit Prompt Flag
```rust
#[test]
fn test_prompt_flag() {
    let args = CliArgs::parse_from(vec!["caro", "-p", "list files"]);
    assert_eq!(args.prompt_flag, Some("list files".to_string()));
    assert!(args.trailing_args.is_empty());
}
```

### TC5: Empty Arguments
```rust
#[test]
fn test_empty_arguments() {
    let args = CliArgs::parse_from(vec!["caro"]);
    assert!(args.prompt_flag.is_none());
    assert!(args.trailing_args.is_empty());
}
```

### TC6: Mixed Flags and Prompt
```rust
#[test]
fn test_mixed_flags_and_prompt() {
    let args = CliArgs::parse_from(vec![
        "caro", "--dry-run", "--shell", "zsh", "list", "all", "files"
    ]);
    assert!(args.dry_run);
    assert_eq!(args.shell, Some("zsh".to_string()));
    assert_eq!(args.trailing_args, vec!["list", "all", "files"]);
}
```

## Error Conditions

### EC1: Invalid Flag
**Input**: `caro --invalid-flag list files`
**Expected**: clap error message with usage help
**Behavior**: Program exits with non-zero status code

### EC2: Missing Flag Value
**Input**: `caro --shell list files` (--shell expects value)
**Expected**: clap error message indicating missing value
**Behavior**: Program exits with non-zero status code

## Performance Requirements

- **Parsing Time**: < 1ms for typical command lines (< 50 arguments)
- **Memory**: No heap allocations for flag parsing (stack-only)
- **Startup Impact**: Argument parsing should be negligible part of < 100ms startup budget

## Dependencies

- **clap**: Version 4.5+ with `trailing_var_arg` and `num_args` support
- **std::env**: For accessing raw arguments

## Backward Compatibility

- ✅ All existing flag combinations continue to work
- ✅ Quoted prompts work exactly as before
- ✅ No breaking changes to CLI interface
- ✅ Existing scripts using quoted prompts require no modifications

## Related Contracts

- [prompt-source-resolution.contract.md](prompt-source-resolution.contract.md) - Prioritizes between prompt_flag, stdin, and trailing_args
- [validation.contract.md](validation.contract.md) - Validates the final prompt
- [shell-operator-detection.contract.md](shell-operator-detection.contract.md) - Processes trailing_args for shell operators

## Status

**Current**: ⏳ Specification Phase
**Next**: Implementation with TDD (RED-GREEN-REFACTOR)
