---
description: Quality assurance skill for systematic bug investigation, reproduction, and documentation
---

**Path reference rule:** When you mention directories or files, provide either the absolute path or a path relative to the project root (for example, `tests/regression_issue_123.rs`). Never refer to a folder by name alone.

## User Input

```text
$ARGUMENTS
```

You **MUST** consider the user input before proceeding (if not empty).

---

## Quick Reference

| Command | Action |
|---------|--------|
| `/caro.qa #123` | Investigate issue #123 (default) |
| `/caro.qa pr #456` | Investigate pull request #456 |
| `/caro.qa --test-only #123` | Skip investigation, create regression tests only |
| `/caro.qa --doc-only #123` | Skip investigation, document findings only |

---

## What This Command Does

`/caro.qa` is your quality assurance specialist that systematically investigates bugs and creates comprehensive documentation. This skill:

- **Reproduces bugs**: Attempts to reproduce the reported bug with actual commands
- **Analyzes code**: Examines implementation to understand behavior
- **Documents findings**: Creates detailed investigation reports with evidence
- **Creates regression tests**: Codifies working behavior to prevent future regressions
- **Reports back**: Communicates findings on issues/PRs with empathy and clarity

**Core QA Workflow**:
```
CONTEXT → REPRODUCE → ANALYZE → DOCUMENT → TEST → REPORT
```

---

## QA Philosophy

**Empathy first**: Bug reports may be stale, confused, or outdated. Everything is okay. We approach investigation with empathy and communicate clearly back to the reporter.

**Evidence-based**: Every conclusion must be supported by evidence (test commands, outputs, code snippets, screenshots).

**Prevention-focused**: Codify working behavior as regression tests to prevent future issues.

---

## Pre-flight Checks

Run these checks before proceeding:

```bash
# Verify GitHub CLI
which gh || echo "ERROR: gh CLI not installed (https://cli.github.com)"

# Verify authentication
gh auth status 2>&1 | grep -q "Logged in" || echo "ERROR: Run 'gh auth login'"

# Verify cargo is available (for Rust projects)
which cargo || echo "WARNING: cargo not in PATH, run: . \"$HOME/.cargo/env\""
```

**If checks fail**: Stop and ask user to install/configure required tools.

---

## Outline

### 1. Parse Arguments and Determine Mode

Parse `$ARGUMENTS` to determine the operation:

```
ARGUMENTS patterns:
- "#123" OR "123" → ISSUE_MODE (investigate issue)
- "pr #456" OR "pr 456" → PR_MODE (investigate pull request)
- "--test-only #123" → TEST_ONLY_MODE (create regression tests)
- "--doc-only #123" → DOC_ONLY_MODE (document existing findings)
```

Extract issue/PR number from arguments.

### 2. Gather Context (CONTEXT Phase)

**For ISSUE_MODE**:
```bash
# Fetch issue details
gh issue view $ISSUE_NUMBER --json number,title,body,labels,milestone,comments

# Find related PRs
gh pr list --search "fixes #$ISSUE_NUMBER OR resolves #$ISSUE_NUMBER" --json number,title,state

# Find related commits
git log --all --grep="#$ISSUE_NUMBER" --oneline
```

**For PR_MODE**:
```bash
# Fetch PR details
gh pr view $PR_NUMBER --json number,title,body,labels,files,commits

# Get PR diff
gh pr diff $PR_NUMBER

# Check if PR was merged or closed
gh pr view $PR_NUMBER --json state,mergedAt,closedAt
```

**Extract key information**:
- What is the reported bug/issue?
- What behavior was expected?
- What behavior was observed?
- What environment details are provided?
- Are there related PRs or commits?

**If insufficient context**: Request clarification from reporter immediately:
```markdown
I'm investigating this issue, but I need more details to reproduce it:

1. What exact command did you run?
2. What output did you see?
3. What output did you expect?
4. What environment? (OS, shell, version)

This will help me reproduce and fix the issue effectively.
```

### 3. Attempt Reproduction (REPRODUCE Phase)

**Create a test environment checklist**:
- Document current environment (OS, branch, commit hash)
- Identify the component being tested (CLI command, API, backend)
- Create test scenarios based on bug report

**Execute reproduction attempts** (minimum 3-5 different approaches):

1. **Exact reproduction**: Try the exact steps from the bug report
2. **Minimal reproduction**: Simplify to the smallest failing case
3. **Variation testing**: Try variations of the reported scenario
4. **Edge case testing**: Test boundary conditions
5. **Related scenario testing**: Test similar functionality

**Document each attempt**:
```markdown
### Attempt 1: Exact Reproduction
**Command**: `cargo run --quiet -- list files`
**Expected**: Error or incorrect parsing
**Actual**: ✅ Generated "ls -la" successfully
**Conclusion**: Cannot reproduce with exact steps
```

**Capture evidence**:
- Terminal output (copy exact text)
- Screenshots if UI-related
- Error messages with stack traces
- Log files if available

### 4. Code Analysis (ANALYZE Phase)

**Examine implementation**:

1. **Find relevant code**:
   ```bash
   # Search for related functions/modules
   rg -i "function_name|module_name" --type rust

   # Check git history for recent changes
   git log --oneline -20 -- path/to/relevant/file.rs
   ```

2. **Read implementation**:
   - Use Read tool on relevant files
   - Understand the current behavior
   - Identify the logic flow
   - Check for recent changes

3. **Check test coverage**:
   ```bash
   # Find existing tests
   cargo test <feature_name> --list

   # Run existing tests
   cargo test <feature_name>
   ```

4. **Review related PRs/commits**:
   - If PR exists, read the diff
   - Understand what was attempted
   - Why was it closed/not merged?
   - What feature specs or discussions exist?

**Document analysis findings**:
```markdown
### Code Analysis
**File**: `src/main.rs:223-224`
**Current Implementation**:
```rust
/// Trailing unquoted arguments forming the prompt
#[arg(trailing_var_arg = true, num_args = 0..)]
trailing_args: Vec<String>,
```

**Behavior**: Feature IS implemented correctly using clap's `trailing_var_arg`
**Related**: PR #68 attempted this but was closed; implemented via spec-kitty feature 002
```

### 5. Determine Status (CONCLUSION Phase)

Based on investigation, classify the issue:

**Status Categories**:
1. **CONFIRMED** - Bug reproduced successfully
2. **CANNOT_REPRODUCE** - Bug cannot be reproduced (may be fixed, stale, or environmental)
3. **FIXED** - Bug was fixed in a later commit/PR
4. **INVALID** - Bug report is incorrect or misunderstood
5. **NEEDS_CLARIFICATION** - Insufficient information to investigate

**For each status, define next steps**:

**If CONFIRMED**:
- Document exact reproduction steps
- Create failing test case
- Propose fix or request assignment to developer
- Add to roadmap if not already prioritized

**If CANNOT_REPRODUCE**:
- Document all reproduction attempts with evidence
- Request clarification from reporter with specific questions
- Create regression tests to ensure it stays working
- Keep issue open pending reporter feedback

**If FIXED**:
- Identify which commit/PR fixed it
- Create regression tests to prevent regression
- Close issue with reference to fixing commit
- Thank reporter for finding the issue

**If INVALID**:
- Explain why it's not a bug (with evidence)
- Provide correct usage documentation
- Close issue with helpful explanation
- Suggest documentation improvements if needed

**If NEEDS_CLARIFICATION**:
- List specific information needed
- Ask focused questions to reporter
- Keep issue open pending response
- Do NOT create tests or documentation yet

### 6. Document Findings (DOCUMENT Phase)

Create a comprehensive investigation report to add as a comment:

```markdown
## QA Investigation Report

**Investigated by**: Claude (AI QA Agent)
**Date**: [Current Date]
**Status**: [CONFIRMED|CANNOT_REPRODUCE|FIXED|INVALID|NEEDS_CLARIFICATION]

### Test Environment
- **OS**: [from uname -s]
- **Branch**: [from git branch --show-current]
- **Commit**: [from git rev-parse --short HEAD]
- **Tool Version**: [if applicable]

### Reproduction Attempts

#### Attempt 1: [Description]
**Command**: `[exact command]`
**Expected**: [expected behavior]
**Actual**: [actual result]
**Evidence**: [output/screenshot]

#### Attempt 2: [Description]
...

### Code Analysis

**Relevant Files**:
- `path/to/file.rs:123-145` - [description of relevant code]

**Current Implementation**: [summary of how it currently works]

**Related Changes**: [any relevant PRs/commits]

### Findings

[Clear explanation of what was found]

### Evidence

[Include all evidence: test outputs, code snippets, screenshots]

### Recommended Actions

[For CANNOT_REPRODUCE]:
- Requesting clarification from reporter (see questions below)
- Created regression tests at `tests/regression_issue_XXX.rs` to ensure behavior stays correct
- Keeping issue open pending reporter response

[For CONFIRMED]:
- Created failing test at `tests/regression_issue_XXX.rs`
- Proposed fix: [brief description or reference to PR]

[For FIXED]:
- Fixed in commit [hash] / PR #[number]
- Created regression tests to prevent re-introduction
- Recommending issue closure

### Questions for Reporter

[If CANNOT_REPRODUCE or NEEDS_CLARIFICATION]:
1. Can you provide the exact command you ran?
2. What environment are you using? (OS, shell, version)
3. Are you able to reproduce this on the latest main branch?
4. [Any other specific questions based on investigation]

---

I'm here to help get this resolved! Please provide any additional context you can.
```

### 7. Create Regression Tests (TEST Phase)

**Always create regression tests** (except for NEEDS_CLARIFICATION status).

Test file naming convention: `tests/regression_issue_XXX.rs` where XXX is the issue number.

**Test file structure**:

```rust
// Regression tests for Issue #XXX: [Title]
//
// **Issue**: #XXX - [Title]
// **Reporter**: @[username]
// **Date Reported**: [Date]
// **QA Tested**: [Current Date]
// **Status**: [CONFIRMED|CANNOT_REPRODUCE|FIXED|INVALID]
//
// [Brief summary of investigation and test purpose]

use caro::cli::{CliApp, IntoCliArgs};

/// Mock CLI arguments for testing
#[derive(Default)]
struct TestArgs {
    prompt: Option<String>,
    shell: Option<String>,
    safety: Option<String>,
    output: Option<String>,
    confirm: bool,
    verbose: bool,
    config_file: Option<String>,
}

impl IntoCliArgs for TestArgs {
    // [Implementation of trait methods]
}

// Test 1: Basic reproduction test
#[tokio::test]
async fn test_issue_XXX_basic_reproduction() {
    // Attempt to reproduce the reported bug
    // [Test implementation]
}

// Test 2-N: Additional test cases covering variations, edge cases, related scenarios
```

**Test coverage requirements**:
1. **Basic reproduction**: Test the exact scenario from bug report
2. **Variations**: Test variations of the scenario
3. **Edge cases**: Test boundary conditions
4. **Related scenarios**: Test similar functionality
5. **Performance** (if relevant): Ensure operation completes in reasonable time

**Test assertions**:
- For CONFIRMED bugs: Tests should FAIL until bug is fixed
- For CANNOT_REPRODUCE: Tests should PASS to document working behavior
- For FIXED bugs: Tests should PASS to prevent regression

**Compile and run tests**:
```bash
# Check tests compile
cargo test --no-run

# Run the new tests
cargo test test_issue_XXX

# Run all related tests
cargo test issue_XXX
```

**Fix any compilation issues** immediately.

### 8. Report Back (REPORT Phase)

**Add the investigation report as a comment**:

```bash
# Create a temporary file with the report
cat > /tmp/qa_report.md <<'EOF'
[Investigation report from step 6]
EOF

# Add comment to issue
gh issue comment $ISSUE_NUMBER --body-file /tmp/qa_report.md

# Clean up
rm /tmp/qa_report.md
```

**For CANNOT_REPRODUCE**, also add a clarification request:
```bash
# Add label requesting more info
gh issue edit $ISSUE_NUMBER --add-label "needs-info"
```

**For CONFIRMED**, update priority if needed:
```bash
# Add appropriate labels
gh issue edit $ISSUE_NUMBER --add-label "confirmed" --add-label "bug"
```

**For FIXED**, close the issue:
```bash
# Close with reference to fix
gh issue close $ISSUE_NUMBER --comment "Fixed in commit [hash]. Added regression tests to prevent re-introduction. Thanks for reporting!"
```

### 9. Update Project Tracking

**If issue is part of a milestone**:
```bash
# Check milestone progress
gh issue view $ISSUE_NUMBER --json milestone

# Update ROADMAP.md if needed (for major bugs)
```

**Create a summary for the user**:
```markdown
## QA Investigation Complete

**Issue**: #XXX - [Title]
**Status**: [Status]
**Time spent**: [Estimate]

### What I Did
1. ✅ Gathered context from issue and related PRs
2. ✅ Attempted reproduction with 5 different approaches
3. ✅ Analyzed code implementation in [files]
4. ✅ Created comprehensive investigation report on GitHub
5. ✅ Created regression tests at `tests/regression_issue_XXX.rs`

### Key Findings
[1-2 sentence summary]

### Next Steps
[What happens next - waiting for reporter, bug confirmed and needs fix, etc.]

### Files Created/Modified
- `tests/regression_issue_XXX.rs` - [X test cases]
- GitHub issue #XXX - Added investigation report

All evidence and detailed findings are documented on the GitHub issue.
```

---

## Examples

### Example 1: Cannot Reproduce Bug

```
User: /caro.qa 161
Claude:
  Investigating issue #161: Fix list command argument parsing

  [Gathers context from issue and PR #68]

  Reproduction attempts:
  ✅ Attempt 1: `cargo run --quiet -- list files` → Generated "ls -la"
  ✅ Attempt 2: `cargo run -- list all files` → Generated "ls -la"
  ✅ Attempt 3: Ran existing tests → All 3 pass

  [Code analysis shows feature is implemented correctly]

  Status: CANNOT_REPRODUCE - Feature working as expected

  Created:
  - Investigation report on issue #161
  - Regression tests at tests/regression_issue_161.rs (7 test cases)

  Waiting for reporter clarification.
```

### Example 2: Confirmed Bug

```
User: /caro.qa 145
Claude:
  Investigating issue #145: Memory leak in MLX initialization

  [Gathers context]

  Reproduction attempts:
  ❌ Attempt 1: Run 10 consecutive inferences → Memory grows from 100MB to 500MB
  ❌ Attempt 2: Check with valgrind → Definite leak detected

  Status: CONFIRMED - Memory leak reproduced

  Created:
  - Investigation report on issue #145 with reproduction steps
  - Failing regression test at tests/regression_issue_145.rs
  - Proposed fix in comment (FFI cleanup needed)

  Bug confirmed and documented. Ready for developer assignment.
```

### Example 3: Bug Already Fixed

```
User: /caro.qa 98
Claude:
  Investigating issue #98: JSON parsing fails on empty response

  [Gathers context]

  Reproduction attempts:
  ✅ All attempts pass - JSON parsing handles empty responses correctly

  [Code analysis]
  Found fix in commit abc123f: "Add empty response handling"

  Status: FIXED - Bug was fixed in commit abc123f

  Created:
  - Investigation report on issue #98
  - Regression tests to prevent re-introduction
  - Closed issue with reference to fix

  Bug was already fixed. Created tests to prevent regression.
```

---

## Error Handling

### Issue/PR Not Found
```
ERROR: Issue #[number] not found in wildcard/caro.

Verify the issue exists:
  https://github.com/wildcard/caro/issues/[number]
```

### Cannot Access Repository
```
ERROR: GitHub CLI not authenticated or insufficient permissions.

Run: gh auth login
```

### Test Creation Fails
```
ERROR: Failed to create regression tests.

Compilation error:
[error details]

Please fix the compilation issues and run /caro.qa --test-only #[number] to retry.
```

### Missing Reproduction Information
```
WARNING: Insufficient information to reproduce bug.

The issue description doesn't include:
- Exact command that fails
- Expected vs actual behavior
- Environment details

Adding clarification request to issue. Investigation paused pending response.
```

---

## Integration with Other Skills

### With /caro.roadmap
```bash
# After QA investigation, update roadmap if bug is critical
/caro.roadmap blocked  # Check if this is a blocker
/caro.roadmap start #XXX  # Start fixing confirmed bugs
```

### With /spec-kitty.* or /caro.feature
```bash
# For complex bugs requiring feature work
/caro.feature "Fix issue #XXX: [description]"
```

### With /caro.release.*
```bash
# Before releases, ensure no critical bugs
/caro.qa 150  # Investigate release blocker
```

---

## Notes for Maintainers

- **Evidence is key**: Every conclusion must be supported by concrete evidence
- **Empathy in communication**: Bug reports may be outdated or confused - approach with understanding
- **Regression tests are mandatory**: Even for CANNOT_REPRODUCE, create tests to ensure it stays working
- **Document everything**: Future developers will thank you for thorough documentation
- **Performance**: QA investigations should complete in < 10 minutes for most issues

**QA Quality Metrics**:
- Investigation thoroughness (minimum 3 reproduction attempts)
- Evidence completeness (commands, outputs, code references)
- Test coverage (minimum 3 test cases)
- Communication clarity (empathetic, specific, actionable)

---

## Summary

The `/caro.qa` skill provides systematic bug investigation following the methodology:

1. **Gather context** from issues, PRs, and code history
2. **Attempt reproduction** with multiple approaches and document evidence
3. **Analyze code** to understand current behavior
4. **Determine status** (confirmed, cannot reproduce, fixed, invalid, needs clarification)
5. **Document findings** in comprehensive investigation report
6. **Create regression tests** to codify behavior and prevent regressions
7. **Report back** on GitHub with empathy and clarity

This ensures every bug investigation is thorough, evidence-based, and creates lasting value through regression tests and documentation.
