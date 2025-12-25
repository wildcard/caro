# Contract: Prompt Validation

**Version**: 1.0.0
**Feature**: 002-unquoted-cli-arguments
**Component**: `src/main.rs` (validation logic)
**Created**: 2025-12-25

## Purpose

Defines the behavioral contract for validating the final prompt text and determining whether to show help, display an error, or proceed with command generation.

## Interface

### Input
```rust
pub fn validate_prompt(prompt: &str) -> ValidationResult

pub struct ValidationResult {
    pub is_valid: bool,
    pub action: ValidationAction,
    pub message: Option<String>,
}

pub enum ValidationAction {
    ShowHelp,           // Display help message
    ProceedWithPrompt,  // Continue to inference
    Error(String),      // Display error and exit
}
```

### Output
```rust
ValidationResult
```

## Contract Rules

### Rule 1: Empty String Shows Help

**Requirement**: FR-004 (System MUST show help message when invoked with no arguments or whitespace-only input)

**Given**: Prompt = `""`
**When**: `validate_prompt("")`
**Then**:
- `is_valid` = `false`
- `action` = `ValidationAction::ShowHelp`
- `message` = `None`

**Rationale**: Empty input is not an error - user wants usage information.

**Test Case**:
```rust
#[test]
fn test_empty_prompt_shows_help() {
    let result = validate_prompt("");
    assert!(!result.is_valid);
    assert!(matches!(result.action, ValidationAction::ShowHelp));
}
```

### Rule 2: Whitespace-Only Shows Help

**Requirement**: FR-004 (System MUST show help message for whitespace-only input)

**Given**: Prompt = `"   "` (spaces only)
**When**: `validate_prompt("   ")`
**Then**:
- `is_valid` = `false`
- `action` = `ValidationAction::ShowHelp`
- `message` = `None`

**Test Case**:
```rust
#[test]
fn test_whitespace_only_shows_help() {
    let result = validate_prompt("   ");
    assert!(!result.is_valid);
    assert!(matches!(result.action, ValidationAction::ShowHelp));
}
```

### Rule 3: Whitespace Normalization

**Context**: Multiple spaces should be normalized to single spaces for LLM processing.

**Given**: Prompt = `"find  large   files"` (multiple spaces)
**When**: Normalized during validation
**Then**: Effective prompt = `"find large files"` (single spaces)

**Note**: This is informational - the actual normalization may happen before or during validation.

**Test Case**:
```rust
#[test]
fn test_whitespace_normalization() {
    let prompt = "find  large   files";
    let normalized = prompt.split_whitespace().collect::<Vec<_>>().join(" ");
    assert_eq!(normalized, "find large files");
}
```

### Rule 4: Valid Non-Empty Prompt

**Given**: Prompt = `"list files"`
**When**: `validate_prompt("list files")`
**Then**:
- `is_valid` = `true`
- `action` = `ValidationAction::ProceedWithPrompt`
- `message` = `None`

**Test Case**:
```rust
#[test]
fn test_valid_prompt() {
    let result = validate_prompt("list files");
    assert!(result.is_valid);
    assert!(matches!(result.action, ValidationAction::ProceedWithPrompt));
}
```

### Rule 5: Special Characters Preserved

**Requirement**: FR-010 (System MUST preserve special characters within the prompt text)

**Given**: Prompt = `"find *.txt files"`
**When**: `validate_prompt("find *.txt files")`
**Then**:
- `is_valid` = `true`
- Prompt text is unchanged (special characters preserved)
- `action` = `ValidationAction::ProceedWithPrompt`

**Test Case**:
```rust
#[test]
fn test_special_characters_preserved() {
    let prompt = "find *.txt files";
    let result = validate_prompt(prompt);
    assert!(result.is_valid);
    // Special characters like * should be preserved in the prompt
}
```

### Rule 6: Very Long Prompts

**Context**: Extremely long prompts should be allowed but may be truncated or handled by LLM.

**Given**: Prompt with 1000+ characters
**When**: `validate_prompt(long_prompt)`
**Then**:
- `is_valid` = `true` (no hard length limit)
- `action` = `ValidationAction::ProceedWithPrompt`

**Note**: LLM backend may have token limits, but validation layer should not reject long prompts.

**Test Case**:
```rust
#[test]
fn test_very_long_prompt() {
    let long_prompt = "word ".repeat(500);  // 2500+ characters
    let result = validate_prompt(&long_prompt);
    assert!(result.is_valid);
}
```

### Rule 7: Newlines and Tabs

**Given**: Prompt = `"line1\nline2\tword"`
**When**: `validate_prompt("line1\nline2\tword")`
**Then**:
- `is_valid` = `true`
- Newlines and tabs preserved (or normalized to spaces, depending on implementation)

**Test Case**:
```rust
#[test]
fn test_newlines_and_tabs() {
    let result = validate_prompt("line1\nline2\tword");
    assert!(result.is_valid);
}
```

## Implementation Reference

```rust
pub fn validate_prompt(prompt: &str) -> ValidationResult {
    let trimmed = prompt.trim();

    if trimmed.is_empty() {
        ValidationResult {
            is_valid: false,
            action: ValidationAction::ShowHelp,
            message: None,
        }
    } else {
        ValidationResult {
            is_valid: true,
            action: ValidationAction::ProceedWithPrompt,
            message: None,
        }
    }
}
```

## Test Cases

### TC1: Empty String
```rust
#[test]
fn test_validation_empty() {
    let result = validate_prompt("");
    assert!(!result.is_valid);
    assert!(matches!(result.action, ValidationAction::ShowHelp));
}
```

### TC2: Single Space
```rust
#[test]
fn test_validation_single_space() {
    let result = validate_prompt(" ");
    assert!(!result.is_valid);
    assert!(matches!(result.action, ValidationAction::ShowHelp));
}
```

### TC3: Multiple Spaces
```rust
#[test]
fn test_validation_multiple_spaces() {
    let result = validate_prompt("     ");
    assert!(!result.is_valid);
    assert!(matches!(result.action, ValidationAction::ShowHelp));
}
```

### TC4: Tabs Only
```rust
#[test]
fn test_validation_tabs_only() {
    let result = validate_prompt("\t\t\t");
    assert!(!result.is_valid);
    assert!(matches!(result.action, ValidationAction::ShowHelp));
}
```

### TC5: Newlines Only
```rust
#[test]
fn test_validation_newlines_only() {
    let result = validate_prompt("\n\n\n");
    assert!(!result.is_valid);
    assert!(matches!(result.action, ValidationAction::ShowHelp));
}
```

### TC6: Mixed Whitespace
```rust
#[test]
fn test_validation_mixed_whitespace() {
    let result = validate_prompt(" \t\n \t ");
    assert!(!result.is_valid);
    assert!(matches!(result.action, ValidationAction::ShowHelp));
}
```

### TC7: Valid Single Word
```rust
#[test]
fn test_validation_single_word() {
    let result = validate_prompt("list");
    assert!(result.is_valid);
    assert!(matches!(result.action, ValidationAction::ProceedWithPrompt));
}
```

### TC8: Valid Multiple Words
```rust
#[test]
fn test_validation_multiple_words() {
    let result = validate_prompt("find large files");
    assert!(result.is_valid);
    assert!(matches!(result.action, ValidationAction::ProceedWithPrompt));
}
```

### TC9: Leading/Trailing Whitespace
```rust
#[test]
fn test_validation_with_leading_trailing_whitespace() {
    let result = validate_prompt("  list files  ");
    assert!(result.is_valid);
    assert!(matches!(result.action, ValidationAction::ProceedWithPrompt));
}
```

### TC10: Special Characters
```rust
#[test]
fn test_validation_special_characters() {
    let prompts = vec![
        "find *.txt",
        "list ~/files",
        "search file?.doc",
        "find files | sort",  // Pipe character in prompt (rare but valid)
    ];

    for prompt in prompts {
        let result = validate_prompt(prompt);
        assert!(result.is_valid, "Failed for prompt: {}", prompt);
    }
}
```

## Edge Cases

### EC1: Unicode Characters
**Input**: `"查找文件"` (Chinese characters)
**Expected**: Valid prompt, proceed with inference
**Rationale**: LLM should handle unicode

### EC2: Emoji in Prompt
**Input**: `"list files 📁"`
**Expected**: Valid prompt, proceed with inference
**Rationale**: No restriction on character types

### EC3: Control Characters
**Input**: Prompt with control characters (e.g., `\x00`, `\x1B`)
**Expected**: Valid (though LLM may not handle well)
**Rationale**: Validation is permissive; downstream issues handled by LLM/execution

### EC4: Null Byte
**Input**: `"list\x00files"`
**Expected**: Valid (though behavior is undefined)
**Rationale**: Rust handles null bytes safely; LLM may truncate

## Non-Requirements

This contract explicitly **does NOT**:

1. **Validate prompt semantics** - We don't check if prompt makes sense
2. **Check for dangerous commands** - That's handled by safety validation layer
3. **Enforce prompt structure** - Any natural language is valid
4. **Limit prompt length** - No hard character/word limit
5. **Normalize encoding** - UTF-8 is assumed, no conversion

## Performance Requirements

- **Validation Time**: < 0.1ms (simple string operations)
- **Memory**: No allocations (uses `&str` reference)
- **Startup Impact**: Negligible

## Dependencies

- **std::str**: For string trimming and whitespace checks

## Integration Points

### With Help System
When `ValidationAction::ShowHelp` is returned:
```rust
if matches!(result.action, ValidationAction::ShowHelp) {
    print_help();  // Existing caro help function
    return Ok(());
}
```

### With Inference Backend
When `ValidationAction::ProceedWithPrompt` is returned:
```rust
if matches!(result.action, ValidationAction::ProceedWithPrompt) {
    let command = generate_command(prompt).await?;
    // ... continue with execution
}
```

## Related Contracts

- [cli-argument-parsing.contract.md](cli-argument-parsing.contract.md) - Provides raw arguments
- [prompt-source-resolution.contract.md](prompt-source-resolution.contract.md) - Resolves prompt source
- [shell-operator-detection.contract.md](shell-operator-detection.contract.md) - Preprocesses arguments

## Status

**Current**: ⏳ Specification Phase
**Next**: Implementation with TDD (RED-GREEN-REFACTOR)

## Success Criteria Reference

**SC-004**: Help message displays correctly when caro is invoked with no arguments or whitespace-only
- ✅ Empty string triggers help
- ✅ Whitespace-only triggers help
- ✅ No error message displayed
- ✅ Non-punitive user experience
