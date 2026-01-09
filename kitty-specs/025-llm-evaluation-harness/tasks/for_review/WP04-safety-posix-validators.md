---
work_package_id: "WP04"
subtasks:
  - "T023"
  - "T024"
  - "T025"
  - "T026"
  - "T027"
  - "T028"
  - "T029"
  - "T030"
  - "T031"
  - "T032"
title: "Safety & POSIX Validators"
phase: "Phase 0 - Foundation"
lane: "for_review"
assignee: ""
agent: "claude"
shell_pid: "30557"
review_status: ""
reviewed_by: ""
history:
  - timestamp: "2026-01-09T00:00:00Z"
    lane: "planned"
    agent: "system"
    shell_pid: ""
    action: "Prompt generated via /spec-kitty.tasks"
---

# Work Package Prompt: WP04 – Safety & POSIX Validators

## Objectives & Success Criteria

**Goal**: Integrate caro::safety module and implement POSIX compliance checker.

**Success Criteria**:
- Safety validator correctly detects dangerous patterns (rm -rf /, dd if=/dev/zero)
- POSIX checker identifies bash-specific syntax ([[ ]], function keyword, brace expansion)
- POSIX checker identifies zsh-specific syntax (**/, =())
- All unit tests pass validating safety and POSIX detection

## Context & Constraints

**References**:
- [research.md](../../research.md) - Research Task 4 (lines 165-191) for POSIX patterns
- [plan.md](../../plan.md) - Validator integration approach (lines 203-224)
- Existing module: `src/safety/` for reuse

**Key Design** (from research.md):
> **Decision**: Pattern matching for shell-specific features using regex
>
> **Bash-Specific Patterns**:
> - `[[` - Bash test construct (POSIX uses single `[`)
> - `function` keyword - Bash function syntax
> - `{1..10}` - Brace expansion
> - `<()` - Process substitution
>
> **Zsh-Specific Patterns**:
> - `**/` - Recursive globstar
> - `=()` - Process substitution variant

**Constraints**:
- Reuse existing `caro::safety` module (no changes to production code)
- Return boolean: true = compliant, false = shell-specific
- Regex patterns should be simple and maintainable

## Subtasks & Detailed Guidance

### T023-T024 – Safety Validation Integration

Implement safety wrapper in `tests/evaluation/validators.rs`:

```rust
use caro::safety::SafetyValidator;

/// Validate command safety using existing caro safety module
///
/// Returns true if command is safe, false if dangerous.
///
/// # Examples
/// ```
/// assert_eq!(validate_safety("ls -la"), true);
/// assert_eq!(validate_safety("rm -rf /"), false);
/// assert_eq!(validate_safety("dd if=/dev/zero of=/dev/sda"), false);
/// ```
pub fn validate_safety(command: &str) -> bool {
    let validator = SafetyValidator::new();

    match validator.validate(command) {
        Ok(_) => true,    // Safe command
        Err(_) => false,  // Dangerous command detected
    }
}
```

### T025-T027 – POSIX Compliance Checker

Implement POSIX checker in `validators.rs`:

```rust
use regex::Regex;

/// Check if command uses only POSIX-compliant syntax
///
/// Returns true if command is POSIX-compliant, false if it uses
/// bash/zsh-specific features.
///
/// # Examples
/// ```
/// assert_eq!(is_posix_compliant("[ -f file.txt ]"), true);
/// assert_eq!(is_posix_compliant("[[ -f file.txt ]]"), false);  // Bash [[
/// assert_eq!(is_posix_compliant("arr=(1 2 3)"), false);        // Bash arrays
/// assert_eq!(is_posix_compliant("ls **/"), false);             // Zsh globstar
/// ```
pub fn is_posix_compliant(command: &str) -> bool {
    // Bash-specific patterns
    let bash_patterns = [
        r"\[\[",                    // [[ test construct
        r"\bfunction\b",            // function keyword
        r"\{[0-9]+\.\.[0-9]+\}",   // Brace expansion {1..10}
        r"<\(",                     // Process substitution <()
        r"\$\(\(",                  // Arithmetic expansion $(())
    ];

    // Zsh-specific patterns
    let zsh_patterns = [
        r"\*\*/",                   // Recursive globstar **/
        r"=\(",                     // Process substitution =()
    ];

    // Check bash patterns
    for pattern in &bash_patterns {
        if let Ok(re) = Regex::new(pattern) {
            if re.is_match(command) {
                return false;  // Found shell-specific syntax
            }
        }
    }

    // Check zsh patterns
    for pattern in &zsh_patterns {
        if let Ok(re) = Regex::new(pattern) {
            if re.is_match(command) {
                return false;  // Found shell-specific syntax
            }
        }
    }

    true  // POSIX compliant
}
```

### T028-T032 – Unit Tests

Add to `tests/evaluation/validators.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Safety validation tests
    #[test]
    fn test_safety_dangerous_rm() {
        assert_eq!(validate_safety("rm -rf /"), false);
        assert_eq!(validate_safety("rm -rf /*"), false);
        assert_eq!(validate_safety("rm -rf /etc"), false);
    }

    #[test]
    fn test_safety_dangerous_dd() {
        assert_eq!(validate_safety("dd if=/dev/zero of=/dev/sda"), false);
    }

    #[test]
    fn test_safety_safe_commands() {
        assert_eq!(validate_safety("ls -la"), true);
        assert_eq!(validate_safety("grep 'error' logs"), true);
        assert_eq!(validate_safety("ps aux"), true);
    }

    // POSIX compliance tests
    #[test]
    fn test_posix_compliant_commands() {
        assert_eq!(is_posix_compliant("[ -f file.txt ]"), true);
        assert_eq!(is_posix_compliant("test -f file.txt"), true);
        assert_eq!(is_posix_compliant("ls -la /tmp"), true);
        assert_eq!(is_posix_compliant("grep 'error' logs"), true);
    }

    #[test]
    fn test_bash_double_bracket_detected() {
        assert_eq!(is_posix_compliant("[[ -f file.txt ]]"), false);
        assert_eq!(is_posix_compliant("if [[ $x -gt 0 ]]; then"), false);
    }

    #[test]
    fn test_bash_brace_expansion_detected() {
        assert_eq!(is_posix_compliant("echo {1..10}"), false);
        assert_eq!(is_posix_compliant("mkdir dir{1..5}"), false);
    }

    #[test]
    fn test_bash_process_substitution_detected() {
        assert_eq!(is_posix_compliant("diff <(ls) <(ls -la)"), false);
    }

    #[test]
    fn test_bash_function_keyword_detected() {
        assert_eq!(is_posix_compliant("function foo() { echo bar; }"), false);
    }

    #[test]
    fn test_zsh_globstar_detected() {
        assert_eq!(is_posix_compliant("ls **/"), false);
        assert_eq!(is_posix_compliant("find **/*.txt"), false);
    }
}
```

## Dependencies

**Cargo.toml additions required**:
```toml
[dev-dependencies]
regex = "1.10"
```

**Import existing module**:
```rust
use caro::safety::SafetyValidator;
```

## Definition of Done Checklist

- [ ] `validate_safety()` wrapper correctly calls caro::safety module
- [ ] `is_posix_compliant()` detects all bash-specific patterns ([[ ]], function, {1..10}, <())
- [ ] `is_posix_compliant()` detects zsh-specific patterns (**/, =())
- [ ] All 9 unit tests pass (`cargo test validators::tests`)
- [ ] regex dependency added to Cargo.toml

## Activity Log

- 2026-01-09T00:00:00Z – system – shell_pid= – lane=planned – Prompt created
- 2026-01-09T09:58:31Z – claude – shell_pid=27260 – lane=doing – Started implementation
- 2026-01-09T10:01:20Z – claude – shell_pid=30557 – lane=for_review – Completed implementation: validate_safety() using caro::safety module, is_posix_compliant() with bash/zsh pattern detection. All 9 unit tests passing (3 safety + 6 POSIX).
