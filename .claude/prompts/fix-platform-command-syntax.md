# Fix Platform-Specific Command Syntax (Issue #411)

## Problem Statement

Command generation uses GNU syntax on macOS (BSD platform), causing generated commands to fail. Beta.2 testing found that `du` commands use `--max-depth=1` (GNU) instead of `-d 1` (BSD) on macOS.

**Issue**: #411
**Priority**: P2 (Medium)
**Beta Testing**: Found in v1.1.0-beta.2 verification

## Current Behavior

**Query**: "show disk space by directory"

**Generated on macOS** (incorrect):
```bash
du -h --max-depth=1  # GNU syntax - fails on macOS
```

**Expected on macOS** (correct):
```bash
du -h -d 1  # BSD syntax - works on macOS
```

## Technical Context

### Platform Detection Infrastructure

Platform detection already exists in the codebase:
- `ShellType::detect()` can identify the OS
- Need to leverage this for command syntax selection

### Affected Files

1. **`src/backends/static_matcher.rs`**
   - Contains static command patterns
   - Currently platform-agnostic patterns
   - Need to add platform-aware variants

2. **`src/prompts/smollm_prompt.rs`**
   - LLM prompt builder
   - Should include platform context in prompts
   - Add examples showing platform differences

3. **`tests/beta_test_suite.rs`** (or similar)
   - Add cross-platform test cases
   - Validate correct syntax per platform

## Implementation Approach

### Phase 1: Platform Detection Helper

Create a platform detection module:

```rust
// src/platform.rs (new file)

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Platform {
    Linux,
    MacOS,
    FreeBSD,
    Windows,
    Other,
}

impl Platform {
    pub fn detect() -> Self {
        #[cfg(target_os = "linux")]
        return Platform::Linux;

        #[cfg(target_os = "macos")]
        return Platform::MacOS;

        #[cfg(target_os = "freebsd")]
        return Platform::FreeBSD;

        #[cfg(target_os = "windows")]
        return Platform::Windows;

        #[allow(unreachable_code)]
        Platform::Other
    }

    pub fn is_bsd(&self) -> bool {
        matches!(self, Platform::MacOS | Platform::FreeBSD)
    }

    pub fn is_gnu(&self) -> bool {
        matches!(self, Platform::Linux)
    }
}
```

### Phase 2: Update Static Matcher

Modify `src/backends/static_matcher.rs` to use platform-aware patterns:

```rust
use crate::platform::Platform;

// Old approach (platform-agnostic)
// ("disk space by directory", "du -h --max-depth=1")

// New approach (platform-aware)
pub fn get_command_for_pattern(pattern: &str, platform: Platform) -> Option<String> {
    match (pattern, platform.is_bsd()) {
        ("disk space by directory", true) => {
            Some("du -h -d 1".to_string())  // BSD syntax
        }
        ("disk space by directory", false) => {
            Some("du -h --max-depth=1".to_string())  // GNU syntax
        }
        // ... other patterns
        _ => None
    }
}
```

**Alternative approach** (more maintainable):

```rust
pub struct CommandPattern {
    query: &'static str,
    gnu_cmd: &'static str,
    bsd_cmd: &'static str,
}

const PATTERNS: &[CommandPattern] = &[
    CommandPattern {
        query: "disk space by directory",
        gnu_cmd: "du -h --max-depth=1",
        bsd_cmd: "du -h -d 1",
    },
    CommandPattern {
        query: "list processes by memory",
        gnu_cmd: "ps aux --sort=-%mem",
        bsd_cmd: "ps aux | sort -k4 -rn",
    },
    // Add more patterns...
];

pub fn get_command(query: &str, platform: Platform) -> Option<String> {
    PATTERNS.iter()
        .find(|p| p.query == query)
        .map(|p| {
            if platform.is_bsd() {
                p.bsd_cmd.to_string()
            } else {
                p.gnu_cmd.to_string()
            }
        })
}
```

### Phase 3: Update LLM Prompts

Modify `src/prompts/smollm_prompt.rs` to include platform context:

```rust
let platform = Platform::detect();
let platform_note = if platform.is_bsd() {
    "IMPORTANT: Use BSD command syntax (e.g., 'du -d 1' not 'du --max-depth=1')"
} else {
    "Use GNU/Linux command syntax"
};

let prompt = format!(
    r#"Generate a shell command for: {query}

Platform: {platform_note}

Examples:
# BSD (macOS/FreeBSD):
- List processes by memory: ps aux | sort -k4 -rn
- Disk usage by directory: du -h -d 1

# GNU (Linux):
- List processes by memory: ps aux --sort=-%mem
- Disk usage by directory: du -h --max-depth=1

Generate ONLY the command, no explanation.
"#,
    query = user_query,
    platform_note = platform_note
);
```

### Phase 4: Add Tests

Create `tests/platform_command_syntax.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::Platform;

    #[test]
    fn test_du_command_on_bsd() {
        let platform = Platform::MacOS;
        let cmd = get_command_for_pattern("disk space by directory", platform);

        assert!(cmd.is_some());
        let cmd = cmd.unwrap();

        // Should use BSD syntax
        assert!(cmd.contains("-d 1"));
        assert!(!cmd.contains("--max-depth"));
    }

    #[test]
    fn test_du_command_on_linux() {
        let platform = Platform::Linux;
        let cmd = get_command_for_pattern("disk space by directory", platform);

        assert!(cmd.is_some());
        let cmd = cmd.unwrap();

        // Should use GNU syntax
        assert!(cmd.contains("--max-depth=1"));
        assert!(!cmd.contains("-d 1"));
    }

    #[test]
    fn test_ps_command_on_bsd() {
        let platform = Platform::MacOS;
        let cmd = get_command_for_pattern("list processes by memory", platform);

        assert!(cmd.is_some());
        let cmd = cmd.unwrap();

        // Should use pipe and sort, not --sort flag
        assert!(cmd.contains("sort"));
        assert!(!cmd.contains("--sort"));
    }

    #[test]
    fn test_ps_command_on_linux() {
        let platform = Platform::Linux;
        let cmd = get_command_for_pattern("list processes by memory", platform);

        assert!(cmd.is_some());
        let cmd = cmd.unwrap();

        // Should use --sort flag
        assert!(cmd.contains("--sort"));
    }
}
```

### Phase 5: Update Beta Test Suite

Modify `tests/beta_test_suite.rs` to include platform expectations:

```rust
#[tokio::test]
async fn test_disk_space_by_directory() {
    let result = generate_command("show disk space by directory").await;

    #[cfg(target_os = "macos")]
    {
        // On macOS, expect BSD syntax
        assert!(result.contains("du"));
        assert!(result.contains("-d 1") || result.contains("-d1"));
        assert!(!result.contains("--max-depth"),
                "Should not use GNU syntax on macOS");
    }

    #[cfg(target_os = "linux")]
    {
        // On Linux, expect GNU syntax
        assert!(result.contains("du"));
        assert!(result.contains("--max-depth=1") || result.contains("--max-depth 1"));
    }
}
```

## Common Platform Differences to Address

### 1. `du` (Disk Usage)
- **GNU**: `du --max-depth=1 -h`
- **BSD**: `du -d 1 -h`

### 2. `ps` (Process List)
- **GNU**: `ps aux --sort=-%mem` or `ps aux --sort=-rss`
- **BSD**: `ps aux | sort -k4 -rn` or `ps aux | sort -k6 -rn`

### 3. `find` (File Search)
- **GNU**: `find . -printf "%T@ %p\n"`
- **BSD**: `find . -print0 | xargs -0 stat -f "%m %N"`

### 4. `date` (Date Formatting)
- **GNU**: `date -d "yesterday"`
- **BSD**: `date -v -1d`

### 5. `sed` (Stream Editor)
- **GNU**: `sed -i 's/old/new/' file`
- **BSD**: `sed -i '' 's/old/new/' file` (requires empty string for in-place)

## Testing Strategy

### Manual Testing

Test on multiple platforms:

```bash
# On macOS
cargo run -- "show disk space by directory"
# Expected: du -h -d 1

# On Linux
cargo run -- "show disk space by directory"
# Expected: du -h --max-depth=1

# Verify command actually works
cargo run -- "show disk space by directory" | bash
# Should succeed without errors
```

### Automated Testing

Run tests on GitHub Actions with multiple OS targets:

```yaml
# .github/workflows/test.yml
strategy:
  matrix:
    os: [ubuntu-latest, macos-latest]

steps:
  - name: Run platform tests
    run: cargo test platform_command_syntax
```

### Beta Testing

Re-run beta test suite on both platforms:

```bash
# File Management test case #3 should now pass 100%
cargo test test_file_management_commands
```

## Success Criteria

- [ ] Platform detection module implemented
- [ ] Static matcher uses platform-aware patterns
- [ ] LLM prompts include platform context
- [ ] All 5 File Management test cases pass on macOS (100%)
- [ ] All 5 File Management test cases pass on Linux (100%)
- [ ] Cross-platform tests added
- [ ] GitHub Actions tests pass on both macOS and Linux
- [ ] Beta test pass rate: 100% (vs 80% in beta.2)

## Verification

After implementing the fix:

1. **Run automated tests**:
   ```bash
   cargo test platform_command_syntax
   cargo test beta_test_suite
   ```

2. **Manual verification on macOS**:
   ```bash
   cargo run -- "show disk space by directory"
   # Verify output: du -h -d 1

   # Test it actually works
   cargo run -- "show disk space by directory" | sh
   # Should show directory sizes without errors
   ```

3. **Manual verification on Linux** (if available):
   ```bash
   cargo run -- "show disk space by directory"
   # Verify output: du -h --max-depth=1
   ```

4. **Re-run beta test suite**:
   ```bash
   cargo test test_issue_161
   # All 7 test cases should pass
   ```

## Known Edge Cases

1. **Cross-compilation**: If binary is built on Linux but runs on macOS (or vice versa), runtime detection must work correctly

2. **WSL (Windows Subsystem for Linux)**: Should behave as Linux (GNU syntax)

3. **Cygwin/MSYS**: May require special handling

4. **FreeBSD**: Should use BSD syntax like macOS

## References

- **Issue**: #411
- **Beta Report**: `.claude/releases/BETA-2-POWER-USER-REPORT.md`
- **Beta.1 Report**: `.claude/releases/BETA-1-POWER-USER-REPORT.md` (Issue #406)
- **Beta Testing Instructions**: `.claude/releases/BETA-TESTING-INSTRUCTIONS.md`

## Estimated Effort

- **Implementation**: 2-3 hours
- **Testing**: 1-2 hours
- **Documentation**: 30 minutes
- **Total**: 4-6 hours

## Notes

- This is the ONLY remaining issue blocking v1.1.0 GA release
- Beta.2 already improved pass rate from 40% to 80%
- This fix should bring it to 100%
- Consider this for v1.1.0-beta.3 or v1.1.0 GA
