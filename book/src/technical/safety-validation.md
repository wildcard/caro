# Safety Validation

Deep dive into cmdai's safety validation system and how it protects against dangerous commands.

> **📚 User Guide:** For user-facing safety features, see [Safety & Security](../user-guide/safety.md).

## Overview

Safety validation is the core security feature of cmdai, preventing dangerous commands from being executed through multiple layers of protection.

### Architecture

```
┌─────────────────────────────────────────────┐
│           Generated Command                 │
│   "find / -name '*.log' -delete"           │
└──────────────────┬──────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────┐
│    Layer 1: Pattern Matching               │
│                                             │
│  ┌─────────────────────────────────┐      │
│  │ System Destruction Patterns     │ ✅    │
│  │ • rm -rf /                       │      │
│  │ • rm -rf ~                       │      │
│  │ • mkfs                           │      │
│  └─────────────────────────────────┘      │
│                                             │
│  ┌─────────────────────────────────┐      │
│  │ Fork Bomb Patterns              │ ✅    │
│  │ • :(){ :|:& };:                 │      │
│  │ • $0 & $0 &                      │      │
│  └─────────────────────────────────┘      │
└──────────────────┬──────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────┐
│    Layer 2: Path Analysis                  │
│                                             │
│  Protected Paths:                          │
│  • /                                       │
│  • /bin, /usr, /etc                        │
│  • /System (macOS)                         │
│                                             │
│  ✅ Not operating on protected paths       │
└──────────────────┬──────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────┐
│    Layer 3: Metacharacter Detection        │
│                                             │
│  Checking for injection patterns:          │
│  • Command chaining (;, &&, ||)            │
│  • Redirection (>, >>, <)                  │
│  • Pipe abuse (| dangerous_cmd)            │
│                                             │
│  ⚠️ Found: -delete flag                    │
└──────────────────┬──────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────┐
│    Layer 4: Scope Analysis                 │
│                                             │
│  • Target: / (root directory)              │
│  • Operation: deletion                     │
│  • Scope: System-wide                      │
│                                             │
│  🔴 Risk Level: CRITICAL                   │
└──────────────────┬──────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────┐
│         Final Risk Assessment               │
│                                             │
│  Risk: 🔴 CRITICAL                         │
│  Reason: System-wide deletion from root    │
│  Action: Block in strict mode              │
│         Warn in moderate/permissive        │
└─────────────────────────────────────────────┘
```

## Pattern Matching System

### Built-in Patterns

cmdai includes comprehensive dangerous command patterns:

**Category: System Destruction**
```rust
pub const SYSTEM_DESTRUCTION: &[DangerousPattern] = &[
    DangerousPattern {
        name: "rm_root",
        pattern: r"rm\s+(-[rf]+\s+)?/(\s|$)",
        severity: RiskLevel::Critical,
        message: "Attempting to delete root filesystem",
    },
    DangerousPattern {
        name: "rm_home",
        pattern: r"rm\s+(-[rf]+\s+)?~(/\s|$)",
        severity: RiskLevel::Critical,
        message: "Attempting to delete home directory",
    },
];
```

**Category: Fork Bombs**
```rust
pub const FORK_BOMBS: &[DangerousPattern] = &[
    DangerousPattern {
        name: "fork_bomb_bash",
        pattern: r":\(\)\s*\{[^}]*:\|:[^}]*\}\s*;\s*:",
        severity: RiskLevel::Critical,
        message: "Fork bomb detected - creates processes infinitely",
    },
];
```

**Category: Disk Operations**
```rust
pub const DISK_OPERATIONS: &[DangerousPattern] = &[
    DangerousPattern {
        name: "mkfs",
        pattern: r"mkfs(\.\w+)?\s",
        severity: RiskLevel::Critical,
        message: "Filesystem creation - will destroy data",
    },
    DangerousPattern {
        name: "dd_zero",
        pattern: r"dd\s+.*if=/dev/zero",
        severity: RiskLevel::Critical,
        message: "Writing zeros to disk - data destruction",
    },
];
```

See [Safety & Security](../user-guide/safety.md) for the complete list.

### Pattern Implementation

```rust
pub struct DangerousPattern {
    /// Pattern identifier
    pub name: &'static str,

    /// Regex pattern to match
    pub pattern: Regex,

    /// Severity level
    pub severity: RiskLevel,

    /// User-facing message
    pub message: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RiskLevel {
    Safe = 0,
    Moderate = 1,
    High = 2,
    Critical = 3,
}
```

## Risk Assessment Algorithm

Commands are evaluated through multiple checks:

### 1. Exact Pattern Matching

```rust
pub fn check_patterns(command: &str) -> Vec<PatternMatch> {
    let mut matches = Vec::new();

    for pattern in &DANGEROUS_PATTERNS {
        if pattern.pattern.is_match(command) {
            matches.push(PatternMatch {
                pattern_name: pattern.name,
                severity: pattern.severity,
                message: pattern.message,
            });
        }
    }

    matches
}
```

### 2. Path Analysis

```rust
pub fn analyze_paths(command: &str) -> PathAnalysis {
    let protected_paths = vec!["/", "/bin", "/usr", "/etc", "/var", "/boot"];

    for path in protected_paths {
        if command.contains(path) {
            return PathAnalysis {
                affects_protected_path: true,
                path: path.to_string(),
                risk_increase: RiskLevel::High,
            };
        }
    }

    PathAnalysis::safe()
}
```

### 3. Metacharacter Detection

```rust
pub fn detect_metacharacters(command: &str) -> MetacharacterAnalysis {
    let dangerous_chars = ['|', ';', '&', '>', '<', '$', '`'];

    let found: Vec<char> = dangerous_chars
        .iter()
        .filter(|&&c| command.contains(c))
        .copied()
        .collect();

    MetacharacterAnalysis {
        found_chars: found,
        risk_level: if found.len() > 2 {
            RiskLevel::High
        } else if !found.is_empty() {
            RiskLevel::Moderate
        } else {
            RiskLevel::Safe
        },
    }
}
```

### 4. Scope Analysis

```rust
pub fn analyze_scope(command: &str) -> ScopeAnalysis {
    let is_recursive = command.contains("-r") || command.contains("-R");
    let is_global = command.starts_with('/') || command.contains("/*");
    let has_wildcards = command.contains('*');

    ScopeAnalysis {
        is_recursive,
        is_global,
        has_wildcards,
        risk_level: match (is_recursive, is_global, has_wildcards) {
            (true, true, true) => RiskLevel::Critical,
            (true, true, false) => RiskLevel::High,
            (_, true, _) => RiskLevel::High,
            (true, _, true) => RiskLevel::High,
            _ => RiskLevel::Moderate,
        },
    }
}
```

### Combined Risk Assessment

```rust
pub fn assess_risk(command: &str) -> RiskAssessment {
    let pattern_matches = check_patterns(command);
    let path_analysis = analyze_paths(command);
    let meta_analysis = detect_metacharacters(command);
    let scope_analysis = analyze_scope(command);

    // Take the highest risk level from all analyses
    let risk = pattern_matches
        .iter()
        .map(|m| m.severity)
        .chain(vec![
            path_analysis.risk_increase,
            meta_analysis.risk_level,
            scope_analysis.risk_level,
        ])
        .max()
        .unwrap_or(RiskLevel::Safe);

    RiskAssessment {
        risk_level: risk,
        pattern_matches,
        path_analysis,
        meta_analysis,
        scope_analysis,
    }
}
```

## Custom Patterns

Users can add custom patterns via configuration:

```toml
[safety.custom_patterns]
patterns = [
    { name = "kubectl_delete_all", pattern = "kubectl delete.*--all", severity = "Critical" },
    { name = "docker_prune", pattern = "docker.*prune.*-a", severity = "High" },
    { name = "npm_force", pattern = "npm.*--force", severity = "Moderate" },
]
```

Implementation:

```rust
pub fn load_custom_patterns(config: &Config) -> Vec<DangerousPattern> {
    config.safety.custom_patterns
        .iter()
        .map(|p| DangerousPattern {
            name: p.name,
            pattern: Regex::new(&p.pattern).unwrap(),
            severity: p.severity,
            message: p.message.unwrap_or("Custom pattern match"),
        })
        .collect()
}
```

## Safety Modes

### Strict Mode (Default)

```rust
pub fn handle_strict(risk: &RiskAssessment) -> SafetyDecision {
    match risk.risk_level {
        RiskLevel::Safe => SafetyDecision::AllowWithConfirmation,
        RiskLevel::Moderate => SafetyDecision::AllowWithConfirmation,
        RiskLevel::High => SafetyDecision::AllowWithWarning,
        RiskLevel::Critical => SafetyDecision::Block,
    }
}
```

### Moderate Mode

```rust
pub fn handle_moderate(risk: &RiskAssessment) -> SafetyDecision {
    match risk.risk_level {
        RiskLevel::Safe => SafetyDecision::AllowWithConfirmation,
        RiskLevel::Moderate => SafetyDecision::AllowWithConfirmation,
        RiskLevel::High => SafetyDecision::AllowWithWarning,
        RiskLevel::Critical => SafetyDecision::AllowWithStrongWarning,
    }
}
```

### Permissive Mode

```rust
pub fn handle_permissive(risk: &RiskAssessment) -> SafetyDecision {
    match risk.risk_level {
        RiskLevel::Safe => SafetyDecision::AllowWithConfirmation,
        RiskLevel::Moderate => SafetyDecision::AllowWithConfirmation,
        RiskLevel::High => SafetyDecision::AllowWithConfirmation,
        RiskLevel::Critical => SafetyDecision::AllowWithWarning,
    }
}
```

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_destruction_detection() {
        assert_eq!(
            assess_risk("rm -rf /").risk_level,
            RiskLevel::Critical
        );

        assert_eq!(
            assess_risk("rm -rf ~/Documents").risk_level,
            RiskLevel::High
        );
    }

    #[test]
    fn test_safe_commands() {
        assert_eq!(
            assess_risk("ls -la").risk_level,
            RiskLevel::Safe
        );

        assert_eq!(
            assess_risk("cat file.txt").risk_level,
            RiskLevel::Safe
        );
    }
}
```

### Integration Tests

```rust
#[test]
fn test_safety_validator_integration() {
    let validator = SafetyValidator::new(SafetyConfig::strict());

    let result = validator.validate("rm -rf /");
    assert_eq!(result.decision, SafetyDecision::Block);
    assert!(result.message.contains("root filesystem"));
}
```

## Future Enhancements

### Machine Learning-Based Detection

```rust
// Planned: Use ML model to detect novel attack patterns
pub struct MLPatternDetector {
    model: Arc<Model>,
}

impl MLPatternDetector {
    pub async fn detect(&self, command: &str) -> f32 {
        // Return probability of command being dangerous
        // 0.0 = safe, 1.0 = dangerous
        self.model.predict(command).await
    }
}
```

### Context-Aware Risk Assessment

```rust
// Planned: Consider user context
pub struct ContextAwareValidator {
    user_history: CommandHistory,
    current_directory: PathBuf,
}

impl ContextAwareValidator {
    pub fn assess_with_context(&self, command: &str) -> RiskAssessment {
        // Lower risk if similar safe commands run before
        // Higher risk if in sensitive directory
        todo!()
    }
}
```

### User-Specific Safety Profiles

```rust
// Planned: Per-user safety customization
pub struct SafetyProfile {
    user_id: String,
    custom_patterns: Vec<DangerousPattern>,
    trusted_commands: HashSet<String>,
    risk_tolerance: RiskLevel,
}
```

### Execution Sandboxing

```rust
// Planned: Run commands in isolated environment
pub struct Sandbox {
    container: Container,
    resource_limits: ResourceLimits,
}

impl Sandbox {
    pub async fn execute(&self, command: &str) -> Result<Output> {
        // Execute in isolated container
        // Limited filesystem access
        // Resource constraints
        todo!()
    }
}
```

## See Also

**User Documentation:**
- [Safety & Security](../user-guide/safety.md) - User-facing safety features
- [Configuration](../user-guide/configuration.md) - Configuring safety settings

**Developer Documentation:**
- [Architecture](../dev-guide/architecture.md) - Safety system in overall design
- [Backend Development](../dev-guide/backends.md) - How backends integrate with safety
- [Testing Strategy](../dev-guide/testing.md) - Testing safety validation

**Related Technical Pages:**
- [Rust Learnings](./rust-learnings.md) - Rust patterns used in validation
- [Performance Optimization](./performance.md) - Optimizing pattern matching

**Community:**
- [Contributing](../dev-guide/contributing.md) - Add new safety patterns
- [Security Policy](../reference/security.md) - Vulnerability reporting
