# Security Architecture Patterns for Caro

## Overview

This document defines security architecture patterns and design principles for caro development. All new features should align with these patterns.

## Core Security Architecture

### Defense in Depth

Caro implements multiple security layers:

```
┌─────────────────────────────────────────────────────────────┐
│                    User Input Layer                          │
│  ├── Input validation                                        │
│  ├── Prompt sanitization                                     │
│  └── Context boundaries                                      │
├─────────────────────────────────────────────────────────────┤
│                    LLM Layer                                  │
│  ├── System prompt hardening                                 │
│  ├── Output validation                                       │
│  └── Response parsing                                        │
├─────────────────────────────────────────────────────────────┤
│                    Safety Layer                               │
│  ├── Pattern matching (52+ patterns)                         │
│  ├── Risk assessment                                         │
│  ├── POSIX compliance checking                               │
│  └── Critical path protection                                │
├─────────────────────────────────────────────────────────────┤
│                    Confirmation Layer                         │
│  ├── Risk visualization                                      │
│  ├── User consent                                            │
│  └── Execution gating                                        │
├─────────────────────────────────────────────────────────────┤
│                    Execution Layer                            │
│  ├── Shell sandboxing (where possible)                       │
│  ├── Output capture                                          │
│  └── Error handling                                          │
└─────────────────────────────────────────────────────────────┘
```

### Trust Boundaries

```
    ┌─────────────────┐
    │   User Input    │  UNTRUSTED - validate everything
    └────────┬────────┘
             │
    ┌────────▼────────┐
    │   LLM Output    │  UNTRUSTED - validate commands
    └────────┬────────┘
             │
    ┌────────▼────────┐
    │   File System   │  SEMI-TRUSTED - config only
    └────────┬────────┘
             │
    ┌────────▼────────┐
    │   Shell Env     │  UNTRUSTED - validate before use
    └─────────────────┘
```

## Security Design Patterns

### 1. Fail-Safe Defaults

**Principle**: When in doubt, fail securely.

```rust
// Good: Fail closed
fn assess_risk(command: &str) -> RiskLevel {
    match analyze(command) {
        Ok(level) => level,
        Err(_) => RiskLevel::High, // Assume worst case
    }
}

// Bad: Fail open
fn assess_risk(command: &str) -> RiskLevel {
    analyze(command).unwrap_or(RiskLevel::Low) // Dangerous!
}
```

**Caro Applications**:
- Unknown commands → High risk (not Low)
- Parse failures → Block execution (not proceed)
- Missing config → Use secure defaults (not permissive)

### 2. Least Privilege

**Principle**: Request only necessary permissions.

**Caro Applications**:
- Don't require sudo to run
- Don't modify system files
- Don't access unnecessary directories
- Telemetry uses minimal data collection

### 3. Input Validation

**Principle**: Validate all input at trust boundaries.

```rust
// Validate user natural language input
fn validate_prompt(input: &str) -> Result<ValidatedPrompt> {
    // Length check
    if input.len() > MAX_PROMPT_LENGTH {
        return Err(ValidationError::TooLong);
    }

    // Control character check
    if input.contains(CONTROL_CHARS) {
        return Err(ValidationError::InvalidChars);
    }

    // Injection pattern check
    if contains_injection_patterns(input) {
        return Err(ValidationError::SuspiciousPattern);
    }

    Ok(ValidatedPrompt(input.to_string()))
}
```

### 4. Output Encoding

**Principle**: Encode output appropriately for context.

**Caro Applications**:
- Shell metacharacter handling
- JSON output escaping
- Terminal control sequence filtering

### 5. Secure by Default Configuration

```toml
# Default config should be secure
[safety]
enabled = true           # Safety ON by default
confirmation = true      # Always confirm dangerous commands
block_critical = true    # Block most dangerous patterns

[telemetry]
enabled = false          # GA: OFF by default (opt-in)
level = "minimal"        # Minimum data collection
redact_all = true        # Redact sensitive patterns

[execution]
timeout = 30             # Prevent runaway commands
max_retries = 2          # Limit retry loops
```

### 6. Separation of Concerns

**Principle**: Security logic should be isolated and auditable.

```
src/
├── safety/               # All safety logic here
│   ├── patterns.rs       # Pattern definitions
│   ├── validator.rs      # Validation logic
│   └── risk.rs           # Risk assessment
├── telemetry/            # All telemetry here
│   ├── events.rs         # Event definitions
│   ├── redaction.rs      # Privacy controls
│   └── collector.rs      # Collection logic
```

Benefits:
- Easy to audit
- Easy to test
- Clear responsibility

## Threat-Specific Patterns

### Command Injection Prevention

```rust
// Pattern: Never interpolate untrusted data into commands
fn execute_safely(command: &ValidatedCommand) -> Result<Output> {
    // Use process builder, not string concatenation
    Command::new("sh")
        .arg("-c")
        .arg(command.as_str()) // Pre-validated
        .output()
}
```

### Credential Protection

```rust
// Pattern: Detect and redact credentials
fn redact_sensitive(text: &str) -> String {
    CREDENTIAL_PATTERNS.iter()
        .fold(text.to_string(), |s, pattern| {
            pattern.replace_all(&s, "[REDACTED]").to_string()
        })
}
```

### Path Traversal Prevention

```rust
// Pattern: Validate paths are within allowed directories
fn validate_path(path: &Path) -> Result<SafePath> {
    let canonical = path.canonicalize()?;

    if PROTECTED_PATHS.iter().any(|p| canonical.starts_with(p)) {
        return Err(PathError::ProtectedPath);
    }

    Ok(SafePath(canonical))
}
```

## Security Testing Patterns

### 1. Pattern Coverage Testing

```rust
#[test]
fn test_all_dangerous_patterns_detected() {
    for (pattern, example) in DANGEROUS_PATTERNS {
        assert!(
            is_dangerous(example),
            "Pattern '{}' not detected in: {}",
            pattern, example
        );
    }
}
```

### 2. Negative Testing

```rust
#[test]
fn test_safe_commands_not_blocked() {
    let safe_commands = ["ls -la", "cat README.md", "grep pattern file"];

    for cmd in safe_commands {
        assert!(
            !is_blocked(cmd),
            "Safe command incorrectly blocked: {}",
            cmd
        );
    }
}
```

### 3. Fuzzing

```rust
#[test]
fn fuzz_command_parser() {
    // Property: Parser should never panic
    arbtest::proptest!(|s: String| {
        let _ = parse_command(&s); // Should not panic
    });
}
```

## Secure Development Checklist

Before merging security-relevant code:

- [ ] Input validation at all trust boundaries
- [ ] Fail-safe defaults implemented
- [ ] Sensitive data redacted in logs/telemetry
- [ ] Error messages don't leak internals
- [ ] Tests cover security cases
- [ ] STRIDE analysis completed
- [ ] Risk assessment documented
- [ ] Security review approved

## Architecture Decision Records

When making security architecture decisions, document:

1. **Context**: What security problem are we solving?
2. **Decision**: What approach did we choose?
3. **Consequences**: What are the security trade-offs?
4. **Alternatives**: What other options were considered?

Example:
```markdown
## ADR-001: Telemetry Opt-In vs Opt-Out

### Context
We need to collect usage data to improve caro, but respect privacy.

### Decision
- Beta: Opt-out (on by default) with clear consent prompt
- GA: Opt-in (off by default)

### Consequences
- Beta gets more data for improvement
- GA respects privacy-conscious users
- Trade-off: Less data in GA

### Alternatives Considered
- Always opt-in: Less data during beta
- Always opt-out: Privacy concerns in GA
```
