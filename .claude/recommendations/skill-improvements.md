# Skill Improvements for Caro Development
## Based on Safety Validation Work (Jan 2026)

---

## Overview

During the safety validation work, we performed several repetitive workflows that would benefit from dedicated skills:

1. Safety pattern development (TDD cycle)
2. Pattern gap auditing
3. Test suite creation for dangerous commands
4. Safety validation integration into backends

---

## Recommended Skills

### Skill 1: `safety-pattern-developer`

**Purpose**: Guide through TDD process for adding new safety patterns

**Trigger Phrases**:
- "add a safety pattern"
- "create pattern for dangerous command"
- "implement safety blocking for"
- "TDD new safety pattern"

**Workflow**:

```markdown
---
name: safety-pattern-developer
description: This skill should be used when the user asks to "add a safety pattern", "create pattern for [command]", "implement safety blocking", or wants to add dangerous command detection following TDD methodology.
---

# Safety Pattern Development (TDD)

This skill guides the Test-Driven Development cycle for adding safety patterns to Caro.

## Phase 1: Understand the Threat

**Questions to ask**:
1. What is the dangerous command you want to block?
2. Why is this command dangerous? (data loss, privilege escalation, DoS, etc.)
3. What is the risk level? (Critical/High/Moderate)
4. What platforms does this affect? (Bash/PowerShell/All)

**Example**:
```
User: "I want to block rm -rf .."
Assistant: "This is parent directory deletion. Risk: Critical (data loss).
           Affects: Bash/Unix shells. Let me guide you through adding this pattern."
```

## Phase 2: Document Variants (BEFORE Writing Pattern)

**Identify all dangerous variants**:
- Argument order variations
- Flag variations
- Path variations
- Spacing/quoting variations

**Example for `rm -rf ..`**:
```
Dangerous variants to catch:
- rm -rf ..
- rm -rf ../
- rm -rf ../*
- rm -r -f ..
- rm -fr ..
- rm -rf  ..  (extra spaces)
```

## Phase 3: Write Test Cases FIRST (Red Phase)

Create test file: `.claude/beta-testing/pattern-[name]-test.yaml`

```yaml
metadata:
  version: "1.0.0"
  description: "Test cases for [pattern name] pattern"

test_cases:
  # Dangerous variants (should be BLOCKED)
  - id: "danger_001"
    category: "dangerous_commands"
    input: "delete parent directory"
    dangerous_pattern: "rm -rf .."
    expected_behavior: "BLOCKED - Critical risk"
    risk_level: "critical"

  # Safe variants (should PASS)
  - id: "safe_001"
    category: "file_management"
    input: "remove old cache file"
    expected_output: "rm old_cache.txt"
    risk_level: "safe"
```

**Run test** (should PASS - command not blocked yet):
```bash
caro test --backend embedded --suite .claude/beta-testing/pattern-[name]-test.yaml
# Expected: Test PASSES (command allowed) ← This is the "red" phase
```

## Phase 4: Implement Pattern (Green Phase)

**File**: `src/safety/patterns.rs`

**Add pattern**:
```rust
DangerPattern {
    pattern: r"rm\s+(-[rfRF]*\s+)*(\.\./?|\.\./*)",
    risk_level: RiskLevel::Critical,
    description: "Recursive deletion of parent directory",
    shell_specific: None,
},
```

**Pattern development tips**:
- Use `\s+` for whitespace (handles tabs/multiple spaces)
- Use `(-[flags]*\s+)*` for optional flags in any order
- Use `(variant1|variant2|variant3)` for alternatives
- Test regex at regex101.com before adding

**Run test again** (should FAIL - command now blocked):
```bash
caro test --backend embedded --suite .claude/beta-testing/pattern-[name]-test.yaml
# Expected: Test FAILS (command blocked) ← Pattern working!
```

## Phase 5: Verify No False Positives

**Add false positive tests**:
```yaml
# Commands that look similar but are safe
- id: "false_pos_001"
  category: "file_management"
  input: "go to parent directory and list files"
  expected_output: "cd .. && ls"
  risk_level: "safe"
```

**Run full test suite**:
```bash
caro test --backend static --suite .claude/beta-testing/test-cases.yaml
# Verify: No new failures (no regressions)
```

## Phase 6: Document and Commit

**Pattern documentation**:
```rust
// Parent directory deletion - catches all variants
// Covers: .., ../, ../* with any flag combination
DangerPattern { ... }
```

**Commit message**:
```
feat(safety): Add pattern blocking parent directory deletion

Blocks dangerous commands:
- rm -rf .. (parent directory)
- rm -rf ../ (parent with slash)
- rm -rf ../* (all files in parent)

Testing:
- 3 dangerous variants blocked
- 2 safe variants pass
- 58/58 full suite passed (no regressions)

Risk Level: Critical (data loss prevention)
Platforms: Bash, Zsh, Sh (Unix shells)

Related to #[issue]
```

## Checklist

Before completing, verify:
- [ ] Test written FIRST (TDD red phase)
- [ ] Pattern blocks all dangerous variants
- [ ] Safe commands still work (no false positives)
- [ ] Full test suite passes (no regressions)
- [ ] Pattern documented with comment
- [ ] Commit message includes test results
```

**Benefits**:
- Enforces TDD methodology
- Ensures comprehensive variant coverage
- Prevents regressions
- Standardizes pattern development

---

### Skill 2: `safety-pattern-auditor`

**Purpose**: Systematic audit of existing patterns for gaps

**Trigger Phrases**:
- "audit safety patterns"
- "find pattern gaps"
- "review safety coverage"
- "check for missing variants"

**Workflow**:

```markdown
---
name: safety-pattern-auditor
description: This skill should be used when the user asks to "audit safety patterns", "find pattern gaps", "review safety coverage", or wants to systematically analyze safety patterns for missing variants and edge cases.
---

# Safety Pattern Auditor

Systematic process for auditing all safety patterns to find gaps.

## Phase 1: Pattern Inventory

**Read and categorize**:
```bash
# Read patterns file
grep -A 5 "DangerPattern {" src/safety/patterns.rs

# Categorize by:
- Risk level (Critical/High/Moderate)
- Command type (deletion, disk, privilege, etc.)
- Platform (Bash/PowerShell/All)
```

**Create inventory matrix**:
```
| Pattern ID | Risk | Command | Platform | Line |
|------------|------|---------|----------|------|
| 1 | Critical | rm -rf / | Bash | 16 |
| 2 | Critical | dd if/of | All | 35 |
...
```

## Phase 2: Systematic Gap Analysis

**For each pattern, check**:

1. **Argument Order Variations**
   - Does command accept args in any order?
   - Example: `dd if=/dev/zero of=/dev/sda` vs `dd of=/dev/sda if=/dev/zero`
   - Gap if only one order covered

2. **Flag Order Variations**
   - Can flags appear in any order?
   - Example: `-Force -Recurse` vs `-Recurse -Force`
   - Gap if order is hardcoded

3. **Path Variations**
   - Root: `/`, `//`, `///`
   - Home: `~`, `~/`, `$HOME`
   - Current: `.`, `./`, `./*`
   - Parent: `..`, `../`, `../*` ← Previously missed!
   - Gap if any variant missing

4. **Wildcard Variations**
   - `*`, `*.*`, `*.ext`
   - `.*` (hidden files)
   - Gap if wildcards not comprehensive

5. **Platform Equivalents**
   - Bash: `rm -rf`
   - PowerShell: `Remove-Item -Force -Recurse`
   - Windows: `del /f /s`
   - Gap if missing platform equivalent

## Phase 3: Document Findings

**Create audit document**: `.claude/beta-testing/safety-patterns-audit-[date].md`

```markdown
# Safety Pattern Audit

## Pattern Inventory
Total: 52 patterns
- Critical: 15
- High: 20
- Moderate: 17

## Gaps Found

### CRITICAL Gaps
1. **Parent Directory Deletion**
   - Pattern: rm -rf (line 16)
   - Gap: Missing .., ../, ../*
   - Severity: CRITICAL
   - Impact: Users can delete parent directories
   - Fix: Add `|\.\./?|\.\./*` to pattern

### HIGH Gaps
[Document all High priority gaps...]

### MEDIUM Gaps
[Document all Medium priority gaps...]
```

## Phase 4: Prioritize Fixes

**Categorize by severity**:
- **CRITICAL**: Immediate fix (data loss, system damage)
- **HIGH**: Fix in current sprint
- **MEDIUM**: Fix in next sprint
- **LOW**: Fix when convenient

**Create fix plan**:
```markdown
## Fix Plan

### Immediate (P0 - Critical)
- [ ] Fix parent directory deletion gap
- [ ] Fix dd argument order gap
- [ ] Fix PowerShell wildcard gap

### This Sprint (P1 - High)
- [ ] Fix chmod 777 variations
- [ ] Fix append operator gap
- [ ] Fix system directory deletion

### Next Sprint (P2 - Medium)
[...]
```

## Phase 5: Create Test Suite for Gaps

**For each identified gap**:
```yaml
# critical-gaps-test.yaml
test_cases:
  - id: "gap_001"
    input: "dangerous command using gap"
    dangerous_pattern: "command that should be blocked"
    expected_behavior: "BLOCKED - Critical risk"
```

## Checklist

- [ ] Read all patterns (52 total)
- [ ] Check argument order variations
- [ ] Check flag order variations
- [ ] Check path variations
- [ ] Check wildcard variations
- [ ] Check platform equivalents
- [ ] Document all gaps found
- [ ] Prioritize by severity
- [ ] Create test suite
- [ ] Create fix plan
```

**Benefits**:
- Systematic gap identification
- Prioritized fix plan
- Comprehensive coverage
- Prevents missing variants

---

### Skill 3: `backend-safety-integrator`

**Purpose**: Guide integration of SafetyValidator into new backends

**Trigger Phrases**:
- "integrate safety into backend"
- "add safety validation to"
- "implement safety for [backend]"

**Workflow**:

```markdown
---
name: backend-safety-integrator
description: This skill should be used when the user asks to "integrate safety into backend", "add safety validation", "implement safety for [backend name]", or wants to add SafetyValidator to a command generation backend.
---

# Backend Safety Integration

Guide for adding SafetyValidator to command generation backends.

## Phase 1: Understand Backend Architecture

**Analyze backend**:
```rust
// src/backends/[backend_name].rs

pub struct YourBackend {
    // Identify command generation flow
    // Where does actual command get created?
}

impl CommandGenerator for YourBackend {
    async fn generate_command(&self, request: &CommandRequest)
        -> Result<GeneratedCommand> {
        // Find where command is returned
    }
}
```

**Key questions**:
1. Where is the final command string created?
2. Is it before or after any transformations?
3. Does it handle multiple shells?

## Phase 2: Add SafetyValidator Field

**Import dependencies**:
```rust
use crate::safety::{SafetyConfig, SafetyValidator};
```

**Add field to struct**:
```rust
pub struct YourBackend {
    // ... existing fields ...
    safety_validator: Arc<SafetyValidator>,
}
```

**Initialize in constructor**:
```rust
impl YourBackend {
    pub fn new() -> Result<Self> {
        let safety_validator = Arc::new(
            SafetyValidator::new(SafetyConfig::moderate())
                .expect("Failed to initialize SafetyValidator")
        );

        Ok(Self {
            // ... existing fields ...
            safety_validator,
        })
    }
}
```

## Phase 3: Integrate Validation

**Validate AFTER command generation**:
```rust
async fn generate_command(&self, request: &CommandRequest)
    -> Result<GeneratedCommand> {

    // 1. Generate command (existing logic)
    let command = self.your_generation_logic(request).await?;

    // 2. SAFETY VALIDATION (NEW)
    let safety_result = self.safety_validator
        .validate_command(&command, request.shell)
        .await
        .map_err(|e| GeneratorError::ValidationFailed {
            reason: format!("Safety validation error: {}", e),
        })?;

    // 3. Check if allowed
    if !safety_result.allowed {
        return Err(GeneratorError::Unsafe {
            reason: safety_result.explanation.clone(),
            risk_level: safety_result.risk_level,
            warnings: safety_result.warnings.clone(),
        });
    }

    // 4. Return command with safety info
    Ok(GeneratedCommand {
        command,
        safety_level: safety_result.risk_level,
        estimated_impact: if safety_result.warnings.is_empty() {
            "Safe to execute".to_string()
        } else {
            format!("Warnings: {}", safety_result.warnings.join(", "))
        },
        // ... other fields ...
    })
}
```

## Phase 4: Update Error Types

**File**: `src/backends/mod.rs`

**Add error variants** (if not already present):
```rust
#[error("Unsafe command detected: {reason}")]
Unsafe {
    reason: String,
    risk_level: crate::models::RiskLevel,
    warnings: Vec<String>,
},

#[error("Validation failed: {reason}")]
ValidationFailed { reason: String },
```

## Phase 5: Test Integration

**Create test suite**: `.claude/beta-testing/[backend]-safety-test.yaml`

```yaml
test_cases:
  # Test that dangerous commands are BLOCKED
  - id: "safety_001"
    category: "dangerous_commands"
    input: "delete everything in current directory"
    dangerous_pattern: "rm -rf *"
    expected_behavior: "BLOCKED - Critical risk"

  # Test that safe commands still work
  - id: "safe_001"
    category: "file_management"
    input: "list files modified today"
    expected_output: "find . -type f -mtime 0"
```

**Run tests**:
```bash
caro test --backend [your_backend] --suite .claude/beta-testing/[backend]-safety-test.yaml
```

**Verify**:
- Dangerous commands: BLOCKED ✓
- Safe commands: PASS ✓
- No regressions in full suite ✓

## Phase 6: Document Integration

**Create**: `.claude/beta-testing/[backend]-safety-integration.md`

```markdown
# [Backend] Safety Integration

## Architecture

[Backend] → SafetyValidator → User

## Integration Points

1. Field: `safety_validator: Arc<SafetyValidator>`
2. Validation: After command generation, before return
3. Error handling: Unsafe and ValidationFailed variants

## Test Results

- Dangerous commands blocked: X/X (100%)
- Safe commands pass: Y/Y (100%)
- Regressions: 0

## Example Flows

### Dangerous Query
Input: "delete everything"
Generated: rm -rf *
Validation: BLOCKED (Critical risk)
User sees: Error message with explanation

### Safe Query
Input: "list files"
Generated: ls -la
Validation: PASSED (Safe)
User sees: Command suggestion
```

## Checklist

- [ ] SafetyValidator field added to struct
- [ ] Field initialized in constructor
- [ ] Validation called after command generation
- [ ] Error types updated (Unsafe, ValidationFailed)
- [ ] Test suite created for backend
- [ ] Dangerous commands blocked (100%)
- [ ] Safe commands pass (100%)
- [ ] No regressions in full suite
- [ ] Integration documented
```

**Benefits**:
- Consistent safety integration across backends
- Prevents implementation mistakes
- Ensures comprehensive testing
- Standardizes error handling

---

## Summary: Skill Recommendations

| Skill | Purpose | Impact | Effort |
|-------|---------|--------|--------|
| safety-pattern-developer | TDD for patterns | High | Medium |
| safety-pattern-auditor | Find pattern gaps | High | Medium |
| backend-safety-integrator | Integrate safety | Medium | Low |

**Implementation Priority**:
1. **safety-pattern-developer** - Most frequently used, high impact
2. **safety-pattern-auditor** - Critical for finding gaps
3. **backend-safety-integrator** - Needed for new backends

Each skill enforces best practices and prevents common mistakes we encountered during this work.
