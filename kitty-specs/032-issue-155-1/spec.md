# Feature Specification: Self-Healing for Permission Errors

**Issue**: #155 (Sub-issue #1)
**Type**: Enhancement
**Priority**: Medium
**Effort**: S (2-3 days)
**Milestone**: v1.1.0

## Overview

When a command execution fails with a permission error, automatically detect it and suggest retrying with `sudo`. This is the first phase of self-healing capability for Caro.

## Problem Statement

Users often encounter permission errors when running commands that require elevated privileges. Currently, Caro shows the error but doesn't help the user fix it. This leads to:
- Manual retyping with `sudo` prefix
- Context switching to understand the error
- Repeated failures for users unfamiliar with Unix permissions

## User Stories

### Story 1: Permission Denied on File Access
```bash
$ caro "create file in /etc/caro/config.toml"
> touch /etc/caro/config.toml
touch: cannot touch '/etc/caro/config.toml': Permission denied

⚠️  Permission error detected. Would you like to retry with sudo? [Y/n]: y
> sudo touch /etc/caro/config.toml
✓ Success
```

### Story 2: Permission Denied on Package Installation
```bash
$ caro "install nginx"
> apt-get install nginx
E: Could not open lock file /var/lib/dpkg/lock - open (13: Permission denied)

⚠️  Permission error detected. Would you like to retry with sudo? [Y/n]: y
> sudo apt-get install nginx
✓ Success
```

### Story 3: User Declines Sudo Suggestion
```bash
$ caro "read protected file"
> cat /etc/shadow
cat: /etc/shadow: Permission denied

⚠️  Permission error detected. Would you like to retry with sudo? [Y/n]: n
Command failed with exit code 1
```

## Requirements

### Functional Requirements

1. **Error Detection**
   - Detect "Permission denied" in stderr
   - Detect "permission denied" (case-insensitive)
   - Detect exit code 1 with permission-related messages
   - Support common permission error patterns across platforms

2. **Suggestion Display**
   - Show clear message: "⚠️  Permission error detected"
   - Show corrected command: `sudo [original command]`
   - Explain what sudo does (brief)

3. **User Confirmation**
   - Interactive prompt: "Would you like to retry with sudo? [Y/n]"
   - Default to Yes (Enter = retry with sudo)
   - Accept y/Y/yes/Yes for confirmation
   - Accept n/N/no/No for decline

4. **Command Execution**
   - If confirmed: Execute `sudo [original command]`
   - If declined: Exit with original error
   - Preserve original command arguments exactly
   - Handle quoted arguments correctly

5. **Integration with Knowledge Index**
   - Record successful sudo corrections to knowledge index
   - Include feedback: "Required elevated privileges"
   - Enable learning for future similar requests

### Non-Functional Requirements

1. **Safety**
   - Never auto-apply sudo without user confirmation
   - Preserve all safety validation rules
   - Don't suggest sudo for already-sudo commands
   - Clear warning about security implications

2. **Cross-Platform**
   - Support Linux (all distros)
   - Support macOS
   - Skip on Windows (no sudo equivalent)

3. **Performance**
   - Error detection must be fast (<10ms)
   - No impact on successful command execution

4. **User Experience**
   - Clear, concise error messages
   - Obvious default action (retry)
   - Easy to decline if desired

## Architecture

### New Module Structure

```
src/healing/
├── mod.rs           # Module exports and HealingEngine struct
├── permission.rs    # Permission error detection and correction
└── prompt.rs        # User confirmation prompts
```

### Core Types

```rust
pub struct HealingEngine {
    platform: Platform,
    shell_type: ShellType,
    #[cfg(feature = "knowledge")]
    knowledge: Option<Arc<KnowledgeIndex>>,
}

pub struct PermissionErrorDetector;

pub struct SudoSuggestion {
    pub original_command: String,
    pub corrected_command: String,
    pub explanation: String,
}
```

### Integration Points

1. **src/main.rs**: Check ExecutionResult after command runs
2. **src/execution/executor.rs**: Provide stderr for analysis
3. **src/knowledge/index.rs**: Record successful corrections

### Error Detection Logic

```rust
impl PermissionErrorDetector {
    pub fn detect(stderr: &str, exit_code: i32) -> bool {
        // Exit code 1 or 126 (permission denied)
        let has_error_code = exit_code == 1 || exit_code == 126;

        // Check for permission keywords in stderr
        let stderr_lower = stderr.to_lowercase();
        let has_permission_error = stderr_lower.contains("permission denied")
            || stderr_lower.contains("operation not permitted")
            || stderr_lower.contains("access denied");

        has_error_code && has_permission_error
    }
}
```

## Acceptance Criteria

- [ ] Detects "Permission denied" in stderr (case-insensitive)
- [ ] Detects "Operation not permitted" in stderr
- [ ] Detects "Access denied" in stderr (Windows Git Bash)
- [ ] Shows clear permission error message
- [ ] Shows corrected command with `sudo` prefix
- [ ] Interactive confirmation prompt with [Y/n] default
- [ ] Executes with sudo when confirmed
- [ ] Exits normally when declined
- [ ] Preserves all command arguments and quoting
- [ ] Records successful corrections to knowledge index (if enabled)
- [ ] Unit tests for error detection logic
- [ ] Integration tests with actual permission-restricted commands
- [ ] Works on Linux (Ubuntu, Arch, Debian)
- [ ] Works on macOS
- [ ] Documentation in ADVANCED_PATTERNS.md

## Test Cases

### Unit Tests

```rust
#[test]
fn test_detect_permission_denied() {
    let stderr = "touch: cannot touch 'file': Permission denied";
    assert!(PermissionErrorDetector::detect(stderr, 1));
}

#[test]
fn test_detect_operation_not_permitted() {
    let stderr = "mkdir: cannot create directory: Operation not permitted";
    assert!(PermissionErrorDetector::detect(stderr, 1));
}

#[test]
fn test_no_false_positive_on_success() {
    let stderr = "";
    assert!(!PermissionErrorDetector::detect(stderr, 0));
}

#[test]
fn test_no_false_positive_on_other_errors() {
    let stderr = "command not found";
    assert!(!PermissionErrorDetector::detect(stderr, 127));
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_permission_error_healing() {
    // Create restricted file
    let temp_dir = TempDir::new().unwrap();
    let protected_file = temp_dir.path().join("protected.txt");

    // Make it read-only for owner
    std::fs::write(&protected_file, "test").unwrap();
    std::fs::set_permissions(&protected_file, Permissions::from_mode(0o400)).unwrap();

    // Try to modify (should fail)
    let result = execute_command(&format!("echo 'new' > {:?}", protected_file)).await;
    assert!(!result.success);

    // Healing engine should detect permission error
    let detector = PermissionErrorDetector;
    assert!(detector.detect(&result.stderr, result.exit_code));

    // Generate sudo suggestion
    let suggestion = detector.suggest_correction(&result.command).unwrap();
    assert!(suggestion.corrected_command.starts_with("sudo "));
}
```

## Implementation Plan

### Phase 1: Core Detection (Day 1)
1. Create `src/healing/` module structure
2. Implement `PermissionErrorDetector` with pattern matching
3. Unit tests for detection logic
4. Test on Linux and macOS

### Phase 2: User Confirmation (Day 1-2)
1. Implement `prompt.rs` with dialoguer crate
2. Handle Y/n confirmation logic
3. Test interactive prompts

### Phase 3: Integration (Day 2)
1. Modify `src/main.rs` to check for permission errors after execution
2. Call healing engine on failure
3. Execute corrected command if confirmed
4. Integration tests with real commands

### Phase 4: Knowledge Integration (Day 2-3)
1. Record successful sudo corrections to knowledge index
2. Include feedback: "Required elevated privileges"
3. Test knowledge recording

### Phase 5: Documentation (Day 3)
1. Update ADVANCED_PATTERNS.md with self-healing example
2. Add troubleshooting guide for permission errors
3. Document configuration options (if any)

## Out of Scope (Future Work)

- Automatic detection of which commands typically need sudo
- Learning which files/paths require permissions
- Suggesting `chmod` instead of `sudo` for file ownership issues
- Integration with `doas` or other sudo alternatives
- Windows UAC elevation (PowerShell)
- Sub-issue #2: Command not found healing
- Sub-issue #3: Safety validation healing
- Sub-issue #4: Learning system (requires ChromaDB #166)

## Dependencies

- `dialoguer` crate for interactive prompts (already in Cargo.toml)
- Knowledge index (optional, behind feature flag)

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| False positives (detect non-permission errors) | Medium | Strict pattern matching, multiple checks |
| User accidentally confirms dangerous sudo command | High | Safety validation still applies, clear warning |
| Integration breaks existing error handling | Medium | Add healing as optional step, preserve original behavior |
| Cross-platform differences in error messages | Low | Test on Linux, macOS, BSD |

## Success Metrics

- Permission errors are detected correctly (0% false negatives on test cases)
- No false positives on non-permission errors
- >80% of users accept sudo suggestion (based on telemetry if available)
- Integration tests pass on all supported platforms

## Follow-up Work

After this sub-issue is complete:
1. **Sub-issue #2**: Command not found healing (suggest package installation)
2. **Sub-issue #3**: Safety validation healing (suggest safer alternatives)
3. **Sub-issue #4**: Learning system with ChromaDB (v1.2.0)
