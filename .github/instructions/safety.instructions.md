---
applyTo: "src/safety/**/*.rs"
---

# Safety Module Review Instructions

This module is SECURITY-CRITICAL. Apply the highest level of scrutiny.

## Critical Security Concerns

### Dangerous Pattern Detection
The safety module MUST detect and block:

**Critical Risk (Always Block)**
- `rm -rf /` - Recursive root deletion
- `rm -rf ~` - Home directory deletion
- `mkfs.*` - Filesystem formatting
- `dd if=/dev/zero` - Disk overwrite
- `:(){ :|:& };:` - Fork bombs
- `chmod -R 777 /` - System-wide permission changes
- `> /dev/sda` - Direct disk writes

**High Risk (Block or Confirm)**
- `sudo su`, `sudo -i` - Shell escalation
- `curl | bash`, `wget | sh` - Remote code execution
- `chmod 777` - Overly permissive permissions
- Commands with `--no-preserve-root`

**Moderate Risk (Warn)**
- `rm -rf` without safeguards
- System directory modifications (`/etc`, `/usr`, `/bin`)
- Environment variable manipulation

## Pattern Validation Requirements

### Regex Patterns Must Be
1. Pre-compiled at startup (use `once_cell::Lazy`)
2. Context-aware (distinguish executable vs. quoted strings)
3. Tested with both positive and negative cases
4. Documented with examples of what they match

### Context-Aware Detection
The `is_dangerous_in_context()` function prevents false positives:

```rust
// This IS dangerous (executable context)
assert!(is_dangerous("rm -rf /"));

// This is NOT dangerous (inside quotes - documentation)
assert!(!is_dangerous("echo 'rm -rf /' > docs.md"));

// Edge case: ensure proper quote counting
assert!(!is_dangerous(r#"echo "example: rm -rf /""#));
```

## Test Requirements

### Every Pattern MUST Have
1. Positive test case (pattern correctly matches dangerous command)
2. Negative test case (pattern doesn't match safe commands)
3. Edge case tests (unicode, whitespace, quotes)
4. Property-based tests for input validation

### Required Test Structure
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_pattern_detects_dangerous_command() {
        let validator = SafetyValidator::new(SafetyConfig::strict()).unwrap();
        let result = block_on(validator.validate_command("rm -rf /", ShellType::Bash)).unwrap();

        assert!(!result.allowed);
        assert_eq!(result.risk_level, RiskLevel::Critical);
    }

    #[test]
    fn test_pattern_allows_safe_variant() {
        let validator = SafetyValidator::new(SafetyConfig::strict()).unwrap();
        let result = block_on(validator.validate_command("rm file.txt", ShellType::Bash)).unwrap();

        assert!(result.allowed);
    }

    #[test]
    fn test_pattern_ignores_quoted_content() {
        let validator = SafetyValidator::new(SafetyConfig::strict()).unwrap();
        let result = block_on(validator.validate_command("echo 'rm -rf /'", ShellType::Bash)).unwrap();

        assert!(result.allowed); // In quotes = safe
    }
}
```

## Code Review Checklist

### For Any Changes to This Module
- [ ] No new patterns bypass safety validation
- [ ] All regex patterns compile at startup (no runtime regex compilation in hot paths)
- [ ] Error handling preserves fail-safe behavior (on error, default to blocking)
- [ ] No `unwrap()` or `expect()` on user input
- [ ] New patterns have comprehensive test coverage
- [ ] Documentation updated for any new risk categories

### Red Flags to Block
1. Removing patterns without justification
2. Lowering risk levels without security review
3. Adding allowlist patterns that could bypass critical checks
4. Ignoring validation errors (must fail-safe)
5. Using `unwrap()` on regex compilation with untrusted patterns

## Configuration Security

### SafetyConfig Guidelines
```rust
// NEVER allow empty patterns or zero limits
impl SafetyConfig {
    pub fn new(...) -> Result<Self, ValidationError> {
        if max_command_length == 0 {
            return Err(ValidationError::InvalidConfig { ... });
        }
        // Validate all inputs
    }
}
```

### Custom Pattern Validation
- Custom patterns must compile successfully
- Invalid patterns should be rejected, not silently ignored
- Log warnings for deprecated or suspicious patterns

## Performance Considerations

### Requirements
- Pattern matching must complete in < 10ms for typical commands
- Use pre-compiled regex with `once_cell::Lazy`
- Avoid allocation in hot validation paths
- Cache compiled patterns by shell type

### Benchmarks
New changes should not degrade performance significantly:
```bash
cargo bench --bench performance -- safety
```

## Documentation Requirements

### Every DangerPattern Needs
```rust
DangerPattern {
    pattern: r"rm\s+-rf\s+/(?:\s|$)".to_string(),
    risk_level: RiskLevel::Critical,
    description: "Recursive root deletion - complete filesystem wipe".to_string(),
    shell_specific: None, // or Some(ShellType::Bash)
}
```

### Risk Level Justification
Document why each pattern has its assigned risk level in `patterns.rs` comments.
