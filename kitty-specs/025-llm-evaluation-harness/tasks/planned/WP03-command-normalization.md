---
work_package_id: "WP03"
subtasks:
  - "T015"
  - "T016"
  - "T017"
  - "T018"
  - "T019"
  - "T020"
  - "T021"
  - "T022"
title: "Command Normalization & Comparison"
phase: "Phase 0 - Foundation"
lane: "planned"
assignee: ""
agent: ""
shell_pid: ""
review_status: ""
reviewed_by: ""
history:
  - timestamp: "2026-01-09T00:00:00Z"
    lane: "planned"
    agent: "system"
    shell_pid: ""
    action: "Prompt generated via /spec-kitty.tasks"
---

# Work Package Prompt: WP03 – Command Normalization & Comparison

## Objectives & Success Criteria

**Goal**: Implement semantic command equivalence checking with normalization.

**Success Criteria**:
- Normalizer correctly handles whitespace consolidation
- Flag sorting logic recognizes equivalent flag variations (`ls -la` == `ls -l -a`)
- Comparison logic identifies semantically equivalent commands
- Unit tests validate all normalization rules from research.md

## Context & Constraints

**References**:
- [research.md](../../research.md) - Research Task 2 (lines 62-92) for normalization strategy
- [plan.md](../../plan.md) - Normalization approach (lines 179-201)
- [data-model.md](../../data-model.md) - TestCase expected_command field

**Key Design** (from research.md):
> **Decision**: Semantic equivalence checking with normalization
>
> **Normalization Rules**:
> 1. **Whitespace**: Collapse multiple spaces/tabs to single space
> 2. **Flag Consolidation**: `ls -l -a` → `ls -la` (sorted alphabetically)
> 3. **Exact Match**: After normalization, use string comparison

**Constraints**:
- Focus on correctness, not performance (test dataset < 100 items)
- Pure functions (no side effects)
- Return normalized strings for debugging/logging

## Subtasks & Detailed Guidance

### T015-T017 – Implement normalize_command() Core Logic

Implement in `tests/evaluation/validators.rs`:

```rust
/// Normalize a shell command for semantic comparison
///
/// Applies these transformations:
/// 1. Collapse whitespace (multiple spaces/tabs → single space)
/// 2. Sort consolidated flags (e.g., -la → -al)
/// 3. Trim leading/trailing whitespace
pub fn normalize_command(cmd: &str) -> String {
    let mut normalized = cmd.to_string();

    // Step 1: Collapse whitespace
    normalized = collapse_whitespace(&normalized);

    // Step 2: Sort flags (e.g., "ls -l -a" → "ls -al")
    normalized = sort_flags(&normalized);

    // Step 3: Trim
    normalized.trim().to_string()
}

fn collapse_whitespace(cmd: &str) -> String {
    cmd.split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
}

fn sort_flags(cmd: &str) -> String {
    // Parse command into tokens
    let tokens: Vec<&str> = cmd.split_whitespace().collect();
    let mut result = Vec::new();

    for token in tokens {
        if token.starts_with('-') && !token.starts_with("--") && token.len() > 2 {
            // Short flag consolidation: -la → sort to -al
            let mut chars: Vec<char> = token.chars().skip(1).collect();
            chars.sort_unstable();
            result.push(format!("-{}", chars.iter().collect::<String>()));
        } else {
            result.push(token.to_string());
        }
    }

    result.join(" ")
}
```

### T018-T019 – Implement commands_match() Comparison

Add comparison logic in `validators.rs`:

```rust
/// Compare two commands for semantic equivalence
///
/// Returns true if commands are semantically equivalent after normalization.
///
/// # Examples
/// ```
/// assert!(commands_match("ls -la", "ls -l -a"));
/// assert!(commands_match("grep  'error'  logs", "grep 'error' logs"));
/// assert!(!commands_match("ls -la", "ls -lh"));
/// ```
pub fn commands_match(expected: &str, actual: &str) -> bool {
    let normalized_expected = normalize_command(expected);
    let normalized_actual = normalize_command(actual);

    normalized_expected == normalized_actual
}
```

### T020-T022 – Unit Tests

Add to `tests/evaluation/validators.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_whitespace() {
        assert_eq!(
            normalize_command("ls  -la   /tmp"),
            "ls -al /tmp"
        );
        assert_eq!(
            normalize_command("  grep\t'error'\t\tlogs  "),
            "grep 'error' logs"
        );
    }

    #[test]
    fn test_normalize_flags() {
        assert_eq!(normalize_command("ls -la"), "ls -al");
        assert_eq!(normalize_command("ls -l -a"), "ls -al");
        assert_eq!(normalize_command("tar -czf"), "tar -cfz");
    }

    #[test]
    fn test_long_flags_unchanged() {
        // Long flags should not be sorted
        assert_eq!(
            normalize_command("ls --all --long"),
            "ls --all --long"
        );
    }

    #[test]
    fn test_commands_match_equivalent() {
        assert!(commands_match("ls -la", "ls -l -a"));
        assert!(commands_match("ls -la", "ls  -al"));
        assert!(commands_match("grep 'error' logs", "grep  'error'  logs"));
    }

    #[test]
    fn test_commands_match_different() {
        assert!(!commands_match("ls -la", "ls -lh"));
        assert!(!commands_match("grep 'error' logs", "grep 'warning' logs"));
        assert!(!commands_match("ls", "ls -l"));
    }

    #[test]
    fn test_edge_cases() {
        // Empty command
        assert_eq!(normalize_command(""), "");

        // Single flag
        assert_eq!(normalize_command("-l"), "-l");

        // Command with arguments containing spaces (quoted)
        assert_eq!(
            normalize_command("echo 'hello  world'"),
            "echo 'hello world'"
        );
    }
}
```

## Definition of Done Checklist

- [ ] `normalize_command()` implements whitespace collapsing
- [ ] `sort_flags()` correctly sorts short flags alphabetically
- [ ] `commands_match()` returns true for semantically equivalent commands
- [ ] All 6 unit tests pass (`cargo test validators::tests`)
- [ ] Edge cases handled (empty strings, single flags, quoted arguments)

## Activity Log

- 2026-01-09T00:00:00Z – system – shell_pid= – lane=planned – Prompt created
