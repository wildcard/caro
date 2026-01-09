# Keeble Quickstart Guide

> Get started with dogfooding Caro in your daily development workflow.

## What is Keeble?

Keeble is our internal dogfooding program where we use Caro to build Caro. By being our own first users, we:

- Discover issues before they reach end users
- Build features that solve real developer needs
- Validate safety mechanisms with production-like usage
- Create documentation from authentic examples

## Getting Started

### Step 1: Install Caro

If you haven't already:

```bash
# From crates.io
cargo install caro

# Or build from source (dogfooding!)
cargo build --release
alias caro="./target/release/caro"
```

### Step 2: Verify Installation

```bash
# Check version
caro --version

# Run diagnostics
caro doctor
```

### Step 3: Start Using Caro for Development

Replace your common commands with Caro:

```bash
# Instead of: cargo build --release
caro "build in release mode"

# Instead of: cargo test
caro "run the tests"

# Instead of: cargo fmt && cargo clippy
caro "format and lint the code"
```

## Daily Workflow

### Morning Routine

```bash
# Check what's new
caro "show recent commits"
caro "check git status"

# Build and verify
caro "build the project"
caro "run the test suite"
```

### During Development

```bash
# Code exploration
caro "find files containing SafetyValidator"
caro "show the struct definition for Config"
caro "count lines of rust code"

# Testing changes
caro "run tests matching safety"
caro "run tests with output"
```

### Before Committing

```bash
# Quality checks
caro "format all rust files"
caro "run clippy"
caro "check for security vulnerabilities"

# Git operations
caro "add all changed files"
caro "commit with message: Fix safety pattern for rm -rf"
caro "push to origin"
```

## Reporting Friction

When Caro generates a suboptimal command:

### 1. Don't Just Fix It Manually

Let the issue be captured. The friction is valuable data.

### 2. Add to Friction Log

Edit `.claude/keeble/friction-log.yaml`:

```yaml
entries:
  - id: F002
    date: 2025-01-10
    developer: your-initials
    category: test
    intent: "run only failing tests"
    expected: "cargo test -- --failed"
    actual: "cargo test --test failed"
    severity: minor
    platform: linux
    shell: bash
    backend: static
    status: pending
    notes: "Misunderstood 'failing' as test file name"
```

### 3. Continue Working

Use the correct command for now. The weekly review will prioritize fixes.

## Celebrating Success

When Caro generates an excellent command:

### 1. Note It

Edit `.claude/keeble/success-log.yaml`:

```yaml
entries:
  - id: S002
    date: 2025-01-10
    developer: your-initials
    category: search
    intent: "find all unsafe blocks"
    generated: "grep -rn 'unsafe {' src/"
    quality: excellent
    platform: linux
    shell: bash
    backend: static
    harvested: false
    notes: "Correctly included opening brace to avoid false positives"
```

### 2. Consider Expansion

Could this pattern help with similar queries? Flag it for harvest.

## Tier 1 Commands (Start Here)

Focus on these essential commands first:

| Intent | Expected Output |
|--------|-----------------|
| "build the project" | `cargo build` |
| "build in release mode" | `cargo build --release` |
| "run tests" | `cargo test` |
| "run test X" | `cargo test X` |
| "format code" | `cargo fmt` |
| "run clippy" | `cargo clippy` |
| "show git status" | `git status` |
| "show diff" | `git diff` |
| "show recent commits" | `git log --oneline -10` |

## Weekly Review Ritual

Every Friday, spend 15 minutes:

1. **Review friction log** - Any patterns? Quick fixes?
2. **Harvest successes** - Patterns ready for static matcher?
3. **Update metrics** - How's our success rate?
4. **Share findings** - Post summary in team channel

## Tips for Effective Dogfooding

### Be Specific

```bash
# Less useful
caro "test stuff"

# More useful
caro "run tests matching 'safety' with verbose output"
```

### Try Natural Variations

```bash
# Same intent, different phrasing
caro "run the tests"
caro "execute test suite"
caro "cargo test"
caro "test everything"
```

### Test Edge Cases

```bash
# Complex commands
caro "find large rust files modified today"
caro "show commits that changed safety module"
caro "run clippy and fix warnings automatically"
```

### Don't Avoid Failures

Wrong commands are learning opportunities. Log them!

## Resources

- [Full Keeble Strategy](./KEEBLE_DOGFOODING.md) - Complete dogfooding plan
- [Friction Log](./.claude/keeble/friction-log.yaml) - Report issues
- [Success Log](./.claude/keeble/success-log.yaml) - Celebrate wins
- [Beta Testing Framework](./.claude/beta-testing/) - Structured testing

## FAQ

**Q: What if Caro is slower than typing the command directly?**

A: Log it! Speed is a key metric. But also consider: learning a new pattern once saves time forever.

**Q: What if I need a command Caro doesn't handle?**

A: Perfect friction log entry. These gaps drive our feature roadmap.

**Q: Should I use Caro for everything?**

A: Focus on Tier 1 commands first. Expand as they become reliable. For critical production operations, verify commands before execution.

**Q: How do I know if my friction report was addressed?**

A: Check the weekly review summary in the friction log. Fixed items will have a `fix_commit` reference.

---

*Happy dogfooding! Every command you use through Caro makes it better for everyone.*
