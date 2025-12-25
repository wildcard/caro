# Contract: Shell Operator Detection

**Version**: 1.0.0
**Feature**: 002-unquoted-cli-arguments
**Component**: `src/cli/mod.rs` or `src/main.rs` (prompt preprocessing)
**Created**: 2025-12-25

## Purpose

Defines the behavioral contract for detecting POSIX shell operators in trailing arguments and truncating the prompt at the first operator encountered.

## Context

When users type `caro list files > output.txt`, the shell passes `["list", "files"]` to caro and handles `> output.txt` itself. However, if users type this in certain contexts (like shell scripts or quoted strings), the `>` might arrive as a trailing argument. This contract defines how to handle such cases following BSD/GNU best practices.

## Interface

### Input
```rust
pub fn truncate_at_shell_operator(args: Vec<String>) -> Vec<String>
```

### Output
```rust
Vec<String>  // Arguments before first shell operator (or all args if no operator found)
```

## Contract Rules

### Rule 1: Recognize POSIX Shell Operators

**Requirement**: FR-007 (System MUST treat shell operators as terminating the prompt)

**Supported Operators**:
- `>` - Output redirection
- `|` - Pipe to next command
- `<` - Input redirection
- `>>` - Append redirection
- `2>` - Error redirection
- `&` - Background process
- `;` - Command separator

**Reference**: POSIX Shell Command Language specification, BSD/GNU shell conventions

### Rule 2: Truncate at First Operator

**Given**: `args = ["list", "files", ">", "output.txt"]`
**When**: `truncate_at_shell_operator(args)`
**Then**: Returns `["list", "files"]`

**Rationale**: The `>` operator terminates the prompt; everything after is shell syntax.

**Test Case**:
```rust
#[test]
fn test_truncate_at_redirect() {
    let args = vec![
        "list".to_string(),
        "files".to_string(),
        ">".to_string(),
        "output.txt".to_string(),
    ];
    let result = truncate_at_shell_operator(args);
    assert_eq!(result, vec!["list", "files"]);
}
```

### Rule 3: No Operator Means No Truncation

**Given**: `args = ["find", "large", "files"]`
**When**: `truncate_at_shell_operator(args)`
**Then**: Returns `["find", "large", "files"]` (unchanged)

**Test Case**:
```rust
#[test]
fn test_no_operator_no_truncation() {
    let args = vec!["find".to_string(), "large".to_string(), "files".to_string()];
    let result = truncate_at_shell_operator(args);
    assert_eq!(result, vec!["find", "large", "files"]);
}
```

### Rule 4: Operator as First Argument

**Given**: `args = [">", "output.txt"]`
**When**: `truncate_at_shell_operator(args)`
**Then**: Returns `[]` (empty vector)

**Rationale**: Prompt cannot start with shell operator.

**Test Case**:
```rust
#[test]
fn test_operator_first_returns_empty() {
    let args = vec![">".to_string(), "output.txt".to_string()];
    let result = truncate_at_shell_operator(args);
    assert!(result.is_empty());
}
```

### Rule 5: Multiple Operators - Stop at First

**Given**: `args = ["list", "files", ">", "out.txt", "|", "grep", "foo"]`
**When**: `truncate_at_shell_operator(args)`
**Then**: Returns `["list", "files"]` (stops at first `>`)

**Test Case**:
```rust
#[test]
fn test_multiple_operators_stop_at_first() {
    let args = vec![
        "list".to_string(),
        "files".to_string(),
        ">".to_string(),
        "out.txt".to_string(),
        "|".to_string(),
        "grep".to_string(),
    ];
    let result = truncate_at_shell_operator(args);
    assert_eq!(result, vec!["list", "files"]);
}
```

### Rule 6: Operators Not in Standalone Args

**Context**: Shell operators typically arrive as separate arguments. This contract handles the standalone case.

**Given**: `args = ["list", "files>output.txt"]` (operator embedded in word)
**When**: `truncate_at_shell_operator(args)`
**Then**: Returns `["list", "files>output.txt"]` (no truncation)

**Rationale**: The `>` is part of a word, not a standalone operator. This is user's literal text.

**Test Case**:
```rust
#[test]
fn test_embedded_operator_not_detected() {
    let args = vec!["list".to_string(), "files>output.txt".to_string()];
    let result = truncate_at_shell_operator(args);
    assert_eq!(result, vec!["list", "files>output.txt"]);
}
```

## Implementation Reference

```rust
const SHELL_OPERATORS: &[&str] = &[">", "|", "<", ">>", "2>", "&", ";"];

pub fn truncate_at_shell_operator(args: Vec<String>) -> Vec<String> {
    args.into_iter()
        .take_while(|arg| !SHELL_OPERATORS.contains(&arg.as_str()))
        .collect()
}
```

## Test Cases

### TC1: Output Redirection
```rust
#[test]
fn test_output_redirect() {
    let args = vec!["list".to_string(), "files".to_string(), ">".to_string()];
    let result = truncate_at_shell_operator(args);
    assert_eq!(result, vec!["list", "files"]);
}
```

### TC2: Pipe Operator
```rust
#[test]
fn test_pipe_operator() {
    let args = vec![
        "find".to_string(),
        "files".to_string(),
        "|".to_string(),
        "grep".to_string(),
    ];
    let result = truncate_at_shell_operator(args);
    assert_eq!(result, vec!["find", "files"]);
}
```

### TC3: Append Redirect
```rust
#[test]
fn test_append_redirect() {
    let args = vec![
        "list".to_string(),
        "files".to_string(),
        ">>".to_string(),
        "log.txt".to_string(),
    ];
    let result = truncate_at_shell_operator(args);
    assert_eq!(result, vec!["list", "files"]);
}
```

### TC4: Error Redirect
```rust
#[test]
fn test_error_redirect() {
    let args = vec![
        "compile".to_string(),
        "code".to_string(),
        "2>".to_string(),
        "errors.txt".to_string(),
    ];
    let result = truncate_at_shell_operator(args);
    assert_eq!(result, vec!["compile", "code"]);
}
```

### TC5: Background Operator
```rust
#[test]
fn test_background_operator() {
    let args = vec!["run".to_string(), "server".to_string(), "&".to_string()];
    let result = truncate_at_shell_operator(args);
    assert_eq!(result, vec!["run", "server"]);
}
```

### TC6: Command Separator
```rust
#[test]
fn test_command_separator() {
    let args = vec![
        "list".to_string(),
        "files".to_string(),
        ";".to_string(),
        "ls".to_string(),
    ];
    let result = truncate_at_shell_operator(args);
    assert_eq!(result, vec!["list", "files"]);
}
```

### TC7: All Supported Operators
```rust
#[test]
fn test_all_operators() {
    let operators = vec![">", "|", "<", ">>", "2>", "&", ";"];
    for op in operators {
        let args = vec!["cmd".to_string(), op.to_string(), "arg".to_string()];
        let result = truncate_at_shell_operator(args);
        assert_eq!(result, vec!["cmd"], "Failed for operator: {}", op);
    }
}
```

## Edge Cases

### EC1: Empty Arguments
**Input**: `[]`
**Output**: `[]`
**Rationale**: No arguments to process.

### EC2: Only Operator
**Input**: `[">"]`
**Output**: `[]`
**Rationale**: Operator is first element, so prompt is empty.

### EC3: Operator-like Words
**Input**: `["find", "files", "greater", "than", "1MB"]`
**Output**: `["find", "files", "greater", "than", "1MB"]`
**Rationale**: "than" contains ">", but as a whole word it's not an operator.

### EC4: Unicode or Special Characters
**Input**: `["list", "→", "files"]` (unicode arrow)
**Output**: `["list", "→", "files"]`
**Rationale**: Only ASCII shell operators are recognized.

## Important Note: Shell Behavior

**In normal shell usage**, operators like `>` and `|` are handled by the shell **before** arguments reach caro:

```bash
# User types this:
caro list files > output.txt

# Shell parses as:
#   Command: caro
#   Args: ["list", "files"]
#   Redirect: > output.txt
```

So in typical usage, **this contract will rarely truncate anything** because the shell has already separated operators.

This contract primarily handles edge cases:
1. Shell scripts that quote everything: `caro 'list files > output.txt'`
2. Testing scenarios where operators are passed as literal strings
3. Programmatic invocation with raw arguments

## Performance Requirements

- **Detection Time**: O(n) where n = number of arguments (< 1ms for typical CLI)
- **Memory**: No heap allocations (iterator-based implementation)
- **Startup Impact**: Negligible

## Dependencies

- **std::iter**: For `take_while` iterator adapter

## Related Contracts

- [cli-argument-parsing.contract.md](cli-argument-parsing.contract.md) - Provides trailing_args input
- [prompt-source-resolution.contract.md](prompt-source-resolution.contract.md) - May use truncated args
- [validation.contract.md](validation.contract.md) - Validates final prompt after operator removal

## Status

**Current**: ⏳ Specification Phase
**Next**: Implementation with TDD (RED-GREEN-REFACTOR)
