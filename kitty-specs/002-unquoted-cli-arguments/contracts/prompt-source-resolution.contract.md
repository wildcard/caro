# Contract: Prompt Source Resolution

**Version**: 1.0.0
**Feature**: 002-unquoted-cli-arguments
**Component**: `src/main.rs` (prompt resolution logic)
**Created**: 2025-12-25

## Purpose

Defines the behavioral contract for resolving which input source provides the final prompt when multiple sources are available (flag, stdin, trailing arguments).

## Interface

### Input
```rust
pub struct PromptSources {
    pub flag: Option<String>,          // From -p/--prompt flag
    pub stdin: Option<String>,         // From piped stdin
    pub trailing_args: Vec<String>,    // From command-line trailing args
}
```

### Output
```rust
pub struct ResolvedPrompt {
    pub text: String,           // The final prompt text
    pub source: PromptSource,   // Where it came from
}

pub enum PromptSource {
    Flag,           // From -p/--prompt flag
    Stdin,          // From piped stdin
    TrailingArgs,   // From unquoted command-line words
}
```

## Contract Rules

### Rule 1: Input Source Priority

**Requirement**: FR-008 (System MUST prioritize input sources: flag > stdin > trailing args)

**Priority Order**:
1. `-p`/`--prompt` flag (highest)
2. stdin (piped input)
3. Trailing arguments (default/fallback)

**Rationale**: Follows Unix tool conventions (cat, grep, etc.)

### Rule 2: Flag Takes Precedence

**Given**: User provides both flag and stdin: `echo "stdin prompt" | caro -p "flag prompt"`
**When**: Prompt resolution executes
**Then**:
- `resolved.text` = `"flag prompt"`
- `resolved.source` = `PromptSource::Flag`

**Test Case**:
```rust
#[test]
fn test_flag_overrides_stdin() {
    let sources = PromptSources {
        flag: Some("flag prompt".to_string()),
        stdin: Some("stdin prompt".to_string()),
        trailing_args: vec!["trailing".to_string(), "args".to_string()],
    };
    let resolved = resolve_prompt(sources);
    assert_eq!(resolved.text, "flag prompt");
    assert_eq!(resolved.source, PromptSource::Flag);
}
```

### Rule 3: Stdin Takes Precedence Over Trailing Args

**Given**: User provides both stdin and trailing args: `echo "stdin prompt" | caro extra words`
**When**: Prompt resolution executes
**Then**:
- `resolved.text` = `"stdin prompt"`
- `resolved.source` = `PromptSource::Stdin`

**Test Case**:
```rust
#[test]
fn test_stdin_overrides_trailing_args() {
    let sources = PromptSources {
        flag: None,
        stdin: Some("stdin prompt".to_string()),
        trailing_args: vec!["extra".to_string(), "words".to_string()],
    };
    let resolved = resolve_prompt(sources);
    assert_eq!(resolved.text, "stdin prompt");
    assert_eq!(resolved.source, PromptSource::Stdin);
}
```

### Rule 4: Trailing Args as Default

**Given**: User provides only trailing args: `caro list files`
**When**: Prompt resolution executes
**Then**:
- `resolved.text` = `"list files"`
- `resolved.source` = `PromptSource::TrailingArgs`

**Test Case**:
```rust
#[test]
fn test_trailing_args_default() {
    let sources = PromptSources {
        flag: None,
        stdin: None,
        trailing_args: vec!["list".to_string(), "files".to_string()],
    };
    let resolved = resolve_prompt(sources);
    assert_eq!(resolved.text, "list files");
    assert_eq!(resolved.source, PromptSource::TrailingArgs);
}
```

### Rule 5: Join Trailing Arguments with Spaces

**Requirement**: FR-009 (System MUST join multiple trailing argument words with spaces)

**Given**: Trailing args = `["find", "large", "files"]`
**When**: Joined to form prompt
**Then**: Prompt = `"find large files"` (spaces between words)

**Test Case**:
```rust
#[test]
fn test_join_trailing_args() {
    let args = vec!["find".to_string(), "large".to_string(), "files".to_string()];
    let prompt = args.join(" ");
    assert_eq!(prompt, "find large files");
}
```

### Rule 6: Preserve Whitespace in Flag and Stdin

**Given**: Flag value = `"find  files"` (double space)
**When**: Prompt resolution executes
**Then**: Prompt text preserves double space: `"find  files"`

**Note**: Whitespace normalization (if any) happens in validation stage, not resolution stage.

**Test Case**:
```rust
#[test]
fn test_preserve_whitespace_in_flag() {
    let sources = PromptSources {
        flag: Some("find  files".to_string()),  // Double space
        stdin: None,
        trailing_args: vec![],
    };
    let resolved = resolve_prompt(sources);
    assert_eq!(resolved.text, "find  files");  // Double space preserved
}
```

## stdin Detection and Reading

### stdin Availability Check
```rust
fn is_stdin_available() -> bool {
    use std::io::IsTerminal;
    !std::io::stdin().is_terminal()
}
```

### stdin Reading
```rust
fn read_stdin() -> Result<String, std::io::Error> {
    use std::io::Read;
    let mut buffer = String::new();
    std::io::stdin().read_to_string(&mut buffer)?;
    Ok(buffer)
}
```

### Rule 7: stdin Must Be Non-Interactive

**Requirement**: FR-006 (System MUST support reading prompts from stdin when available)

**Given**: stdin is a terminal (interactive mode)
**When**: Checking stdin availability
**Then**: `is_stdin_available()` = `false`

**Given**: stdin is piped from another command
**When**: Checking stdin availability
**Then**: `is_stdin_available()` = `true`

## Test Cases

### TC1: All Three Sources Provided
```rust
#[test]
fn test_all_sources_flag_wins() {
    let sources = PromptSources {
        flag: Some("flag".to_string()),
        stdin: Some("stdin".to_string()),
        trailing_args: vec!["trailing".to_string()],
    };
    let resolved = resolve_prompt(sources);
    assert_eq!(resolved.text, "flag");
    assert_eq!(resolved.source, PromptSource::Flag);
}
```

### TC2: Only Stdin and Trailing Args
```rust
#[test]
fn test_stdin_and_trailing_stdin_wins() {
    let sources = PromptSources {
        flag: None,
        stdin: Some("stdin".to_string()),
        trailing_args: vec!["trailing".to_string()],
    };
    let resolved = resolve_prompt(sources);
    assert_eq!(resolved.text, "stdin");
    assert_eq!(resolved.source, PromptSource::Stdin);
}
```

### TC3: Only Trailing Args
```rust
#[test]
fn test_only_trailing_args() {
    let sources = PromptSources {
        flag: None,
        stdin: None,
        trailing_args: vec!["list".to_string(), "files".to_string()],
    };
    let resolved = resolve_prompt(sources);
    assert_eq!(resolved.text, "list files");
    assert_eq!(resolved.source, PromptSource::TrailingArgs);
}
```

### TC4: No Sources (Empty Prompt)
```rust
#[test]
fn test_no_sources_empty_prompt() {
    let sources = PromptSources {
        flag: None,
        stdin: None,
        trailing_args: vec![],
    };
    let resolved = resolve_prompt(sources);
    assert_eq!(resolved.text, "");
    assert_eq!(resolved.source, PromptSource::TrailingArgs);  // Default source
}
```

## Interactive vs Non-Interactive Mode

### Rule 8: Non-Interactive Mode with Flag

**Requirement**: FR-005, SC-005 (Non-interactive mode with -p flag outputs without confirmation)

**Given**: User runs `caro -p "list files"`
**When**: Prompt source is `PromptSource::Flag`
**Then**: System should skip interactive confirmation

**Implementation Note**: The interactive/non-interactive decision happens in execution layer, but source information from this contract informs that decision.

### Rule 9: Interactive Mode with Trailing Args

**Given**: User runs `caro list files`
**When**: Prompt source is `PromptSource::TrailingArgs`
**Then**: System should show interactive confirmation (existing behavior)

## Error Conditions

### EC1: stdin Read Error
**Input**: Piped stdin becomes unavailable mid-read
**Expected**: Return `Err` with descriptive IO error
**Behavior**: Propagate error to caller for handling

### EC2: Empty stdin
**Input**: `echo "" | caro`
**Expected**: Resolved prompt with empty string, source = Stdin
**Behavior**: Downstream validation will handle empty prompt

## Performance Requirements

- **Resolution Time**: < 1ms (simple conditional logic)
- **stdin Read Time**: Bounded by IO, but should complete within startup budget (< 100ms)
- **Memory**: Minimal - one String allocation for prompt text

## Dependencies

- **std::io**: For stdin detection and reading
- **CliArgs**: From argument parsing contract

## Related Contracts

- [cli-argument-parsing.contract.md](cli-argument-parsing.contract.md) - Provides flag and trailing_args
- [validation.contract.md](validation.contract.md) - Validates resolved prompt
- [shell-operator-detection.contract.md](shell-operator-detection.contract.md) - May preprocess trailing_args

## Status

**Current**: ⏳ Specification Phase
**Next**: Implementation with TDD (RED-GREEN-REFACTOR)
