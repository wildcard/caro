# Issue Analysis Reference

This document provides detailed techniques for root cause analysis of beta tester feedback issues.

## Root Cause Categories

### 1. Empty Implementation (TODO Branches)

**Description**: Code path exists but implementation is missing, often with TODO comments.

**Detection**:
```bash
# Search for TODO comments in critical paths
grep -r "TODO" src/ --include="*.rs"

# Look for empty if/else branches
grep -A5 "if.*{$" src/ | grep -A2 "//.*TODO"
```

**Common locations**:
- Config handling (load but not save)
- User input processing (capture but not persist)
- Error handling (catch but not report)
- Feature toggles (check but not act)

**Real example from session**:
```rust
// src/main.rs:627-636
if user_config.telemetry.first_run {
    if caro::telemetry::consent::prompt_consent() {
        // User accepted telemetry
        // TODO: This would require updating the config file...
    } else {
        // User declined telemetry - update config to disable it
    }
}
```

**Root cause**: Consent result captured but `config_manager.save()` never called.

**Fix pattern**:
1. Implement the missing logic
2. Ensure state persistence (save to disk, update database, etc.)
3. Add verification in regression test

### 2. Missing Config Persistence

**Description**: User preferences or settings are loaded but never saved back to disk.

**Detection**:
```bash
# Find config loads without corresponding saves
grep -n "config.*load" src/ --include="*.rs" > /tmp/loads.txt
grep -n "config.*save" src/ --include="*.rs" > /tmp/saves.txt
# Compare counts and locations
```

**Red flags**:
- Config loaded: ✓
- Config modified: ✓
- Config saved: ✗

**Real example from session**:
```rust
// Config loaded
let mut user_config = config_manager.load().ok().unwrap_or_default();

// Config modified
user_config.telemetry.first_run = false;
user_config.telemetry.enabled = consent;

// Config NEVER saved → user choice lost on next run
```

**Fix pattern**:
```rust
// After modifying config
if let Some(ref cm) = config_manager {
    if let Err(e) = cm.save(&user_config) {
        tracing::warn!("Failed to save config: {}", e);
    }
}
```

**Testing**: Verify config file on disk matches in-memory state.

### 3. Pattern Ordering Issues

**Description**: In static pattern matchers, general patterns match before specific patterns, causing wrong commands to be generated.

**Detection**:
- Test expects specific command: `du -h -d 1`
- Generated command is generic: `ls -lh`
- Pattern matcher returns first match (not best match)

**Specificity hierarchy**:
1. Most specific: 5+ keywords + tight regex
2. Specific: 4 keywords + pattern constraints
3. General: 3 keywords
4. Very general: 2 keywords or regex-only

**Real example from session**:

Query: "show disk space by directory"

```rust
// WRONG ORDER (generic Pattern 4 matched first)
Pattern 4: "disk usage" (requires "usage|size|used" - doesn't match "space")
           → fallback to different pattern
           → wrong command: ls -lh

// CORRECT ORDER (specific Pattern 2a added)
Pattern 2a: "disk space by directory" (4 keywords: disk, space, directory)
            → matches first
            → correct command: du -h -d 1
```

**Fix pattern**:
1. Calculate pattern specificity (count required keywords + regex constraints)
2. Reorder: specific patterns BEFORE general patterns
3. Test: specific queries should match specific patterns

**Specificity calculation**:
```
score = (required_keywords.len() * 10) +
        (optional_keywords.len() * 5) +
        (regex_pattern.is_some() ? 3 : 0) +
        (has_constraints like size/time/type ? 5 : 0)
```

Higher score = more specific → should appear earlier in list.

### 4. Output Format Awareness

**Description**: Interactive prompts run in non-interactive modes, polluting machine-readable output (JSON, YAML).

**Detection**:
- Run with `--output json`
- Output is not valid JSON
- Contains prompt text, progress bars, or ANSI codes

**Red flags**:
- `dialoguer::Confirm` without stderr redirect
- `println!` instead of `eprintln!` for user messages
- Progress bars writing to stdout
- Prompts running before output format check

**Real example from session**:

```bash
# BROKEN
$ caro --output json "list files"
{
  📊 Telemetry & Privacy
  ...
  "command": "ls -la"
}
# ↑ Invalid JSON - jq fails

# FIXED
$ caro --output json "list files" | jq '.'
{
  "command": "ls -la",
  "explanation": "..."
}
# ↑ Valid JSON - jq succeeds
```

**Fix pattern**:
```rust
// Check output format BEFORE prompts
let is_interactive_output = cli.output.as_deref().map_or(true, |format| {
    format != "json" && format != "yaml"
});

if user_config.telemetry.first_run && is_interactive_output {
    // Only prompt in interactive mode
    prompt_consent();
}
```

**Alternative fix**: Redirect dialoguer to stderr
```rust
dialoguer::Confirm::new()
    .with_prompt("...")
    .interact_on(&Term::stderr())?
```

**Testing**:
```bash
# Should be valid JSON
caro --output json "list files" | jq '.'

# Should be valid YAML
caro --output yaml "show disk" | python3 -m yaml
```

### 5. Documentation Mismatch

**Description**: Documentation claims features don't exist or provides incorrect examples.

**Detection**:
- Test instructions say "command X doesn't work"
- But `caro X` actually works fine
- Or: README shows syntax that fails

**Common causes**:
- Docs written before implementation
- Feature added but docs not updated
- Copy-paste errors from different version
- Conditional compilation (feature flags) confusion

**Real example from session**:

`.claude/releases/BETA-TESTING-INSTRUCTIONS.md`:
```markdown
❌ Note: The `caro assess` subcommand does not exist in v1.1.0-beta.1.
❌ Note: The `caro telemetry` subcommands do not exist in v1.1.0-beta.1.
```

Reality:
```bash
$ caro assess
# ✓ Works! Shows CPU, GPU, RAM

$ caro telemetry status
# ✓ Works! Shows telemetry config
```

**Fix pattern**:
1. Test the actual command
2. If it works, update docs
3. If it doesn't work, file as bug (not doc issue)

**Verification**: Run every command example in docs, verify output.

### 6. Regex Over-Matching / Under-Matching

**Description**: Regular expressions match too broadly or too narrowly.

**Over-matching example**:
```rust
// Matches English queries when should only match Japanese
regex_pattern: Some(Regex::new(r"[ぁ-んァ-ヶー一-龯].*").unwrap())
optional_keywords: vec!["find", "files"]  // ← Problem!

// Query: "find files" (English)
// Matches because: "find" and "files" in optional_keywords
// Should only match Japanese characters
```

**Fix**: Remove optional keywords, rely on regex only
```rust
required_keywords: vec![],
optional_keywords: vec![],  // Empty!
regex_pattern: Some(Regex::new(r"^.*[ぁ-んァ-ヶー一-龯].*$").unwrap())
```

**Under-matching example**:
```rust
// Should match "from last week" but only matches "modified last week"
regex_pattern: Some(Regex::new(
    r"(modified|changed|updated).*(last week)"
).unwrap())

// Query: "find python files from last week"
// Doesn't match because: no "modified|changed|updated"
```

**Fix**: Expand alternatives
```rust
regex_pattern: Some(Regex::new(
    r"(modified|changed|updated|from).*(last week)"
).unwrap())
```

**Testing regex**:
```rust
#[test]
fn test_python_files_from_last_week() {
    let pattern = Regex::new(r"(modified|changed|updated|from).*(last week)").unwrap();
    assert!(pattern.is_match("find python files from last week"));
    assert!(pattern.is_match("find python files modified last week"));
}
```

## Analysis Workflow

### Step 1: Reproduce the Issue

```bash
# Follow exact reproduction steps from tester
cd /tmp/fresh-install
caro "exact query from report"

# Capture output
caro "query" > /tmp/actual-output.txt

# Compare with expected
diff /tmp/actual-output.txt /tmp/expected-output.txt
```

### Step 2: Add Debug Logging

```rust
// Before the suspected line
tracing::debug!("Config before: {:?}", user_config);

// After the suspected line
tracing::debug!("Config after: {:?}", user_config);
```

Run with debug logging:
```bash
RUST_LOG=debug caro "query" 2>&1 | grep -A5 -B5 "Config"
```

### Step 3: Binary Search

If execution path is unclear:
```rust
println!("CHECKPOINT 1");
// ... code ...
println!("CHECKPOINT 2");
// ... code ...
println!("CHECKPOINT 3");
```

Narrow down where behavior diverges.

### Step 4: Check Recent Changes

```bash
# When did this break?
git log --oneline --since="2 weeks ago" -- src/main.rs

# What changed?
git diff HEAD~5 HEAD -- src/main.rs

# Who changed it and why?
git blame src/main.rs -L 620,640
```

### Step 5: Consult Tests

```bash
# Do existing tests cover this?
grep -r "telemetry" tests/ --include="*.rs"

# Run related tests
cargo test telemetry

# Check if any tests fail
cargo test 2>&1 | grep FAILED
```

### Step 6: Document Findings

Use `templates/root-cause-template.md` to record:
- Symptoms (what user saw)
- Expected behavior
- Root cause (file, line, problem)
- Fix (what needs to change)
- Regression test (how to prevent)

## Common Anti-Patterns

### Anti-Pattern 1: Assumption Without Verification

❌ **DON'T**: "The config must be saving because there's a save() method"
✅ **DO**: Verify with breakpoint/logging that save() is actually called

### Anti-Pattern 2: Fixing Symptoms, Not Causes

❌ **DON'T**: Add workaround that hides the issue
✅ **DO**: Fix the root cause so issue can't recur

Example:
```rust
// ❌ Symptom fix (hides problem)
if config.telemetry.first_run {
    config.telemetry.first_run = false;  // Force false
}

// ✅ Root cause fix (prevents problem)
if config.telemetry.first_run {
    let consent = prompt_consent();
    config.telemetry.enabled = consent;
    config.telemetry.first_run = false;
    config_manager.save(&config)?;  // ← The actual fix
}
```

### Anti-Pattern 3: Adding Patterns Instead of Reordering

❌ **DON'T**: Add 10 more patterns to fix 10 failing tests
✅ **DO**: Reorder existing patterns for proper specificity

Reordering has higher ROI and reduces future ordering issues.

### Anti-Pattern 4: Incomplete Fixes

❌ **DON'T**: Fix Issue #402 but ignore related Issue #403
✅ **DO**: Recognize shared root cause, fix once for both

### Anti-Pattern 5: No Regression Test

❌ **DON'T**: Fix without test → bug comes back in future PR
✅ **DO**: Add regression test → bug stays fixed

## Verification Checklist

After root cause analysis:

- [ ] Root cause clearly identified (file + line + problem)
- [ ] Fix is minimal (only changes what's necessary)
- [ ] Fix addresses cause, not symptom
- [ ] Related issues also fixed (shared root cause)
- [ ] Regression test written and passes
- [ ] Full test suite still passes
- [ ] Manual verification using reproduction steps
- [ ] Documentation updated if needed
- [ ] Commit message explains root cause and fix

## Resources

- Existing tests: `tests/`
- Beta testing reports: `.claude/releases/BETA-*-REPORT.md`
- Root cause template: `../templates/root-cause-template.md`
- Fix workflow: `fix-workflow.md`
