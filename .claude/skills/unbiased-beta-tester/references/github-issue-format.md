# GitHub Issue Format Reference

This document describes how to format bug reports to match the project's issue templates.

## Caro Bug Report Template

The caro project uses a structured bug report template. Beta testers should generate issues that map to this template.

### Template Fields

| Field | Required | Source |
|-------|----------|--------|
| caro Version | Yes | `caro --version` output |
| Rust Version | Yes | `rustc --version` output (or "N/A - installed via binary") |
| Operating System | Yes | Profile + verification |
| OS Version | Yes | `sw_vers` or `/etc/os-release` |
| Backend | Yes | Config or default ("Mock for testing") |
| Shell Type | Yes | `echo $SHELL` |
| Bug Description | Yes | Tester observation |
| Expected Behavior | Yes | What docs/profile suggest |
| Actual Behavior | Yes | What actually happened |
| Exact Command | Yes | Copy-paste from terminal |
| Steps to Reproduce | Yes | Numbered list |
| Reproducibility | Yes | How often it happens |
| Debug Logs | Optional | `RUST_LOG=debug` output |
| Configuration | Optional | `~/.config/caro/config.toml` |
| Additional Context | Optional | Extra info |

## Issue Templates by Category

### Installation Bug

```markdown
---
name: Bug Report
title: "[Bug]: Installation fails on [OS] via [method]"
labels: ["bug", "installation", "triage"]
---

**caro Version:** Unable to install (attempting vX.Y.Z)

**Rust Version:** [version or "not installed"]

**Operating System:** [from dropdown]

**OS Version:** [specific version]

**Backend:** N/A (installation failed)

**Shell Type:** [shell]

**Bug Description:**
When attempting to install caro using [method], the installation
fails with [error type].

**Expected Behavior:**
Following the [section name] in the README, the installation
should complete successfully and `caro --version` should work.

**Actual Behavior:**
The installation fails with the following error:
[error message]

**Exact Command:**
```
[exact command that was run]
```

**Steps to Reproduce:**
1. On a fresh [OS] [version] system
2. With [prerequisites or lack thereof]
3. Run `[command]`
4. Observe error

**Reproducibility:** Every time (100%)

**Debug Logs:**
```
[full terminal output]
```

**Additional Context:**
- Documentation section followed: [link/section]
- Alternative methods tried: [list]
- Workarounds discovered: [if any]
```

### Runtime Bug

```markdown
---
name: Bug Report
title: "[Bug]: [brief description of the problem]"
labels: ["bug", "triage"]
---

**caro Version:** [output of caro --version]

**Rust Version:** [output of rustc --version]

**Operating System:** [from dropdown]

**OS Version:** [specific version]

**Backend:** [Mock/MLX/Ollama/vLLM]

**Shell Type:** [bash/zsh/fish/etc]

**Bug Description:**
When [describing the action], caro [what went wrong].

**Expected Behavior:**
Based on [documentation/help output], caro should [expected result].

**Actual Behavior:**
Instead, caro [actual result]. The output was:
```
[output]
```

**Exact Command:**
```
caro "[prompt]"
```

**Steps to Reproduce:**
1. [First step]
2. [Second step]
3. [Third step]
4. Observe [result]

**Reproducibility:** [Every time/Usually/Sometimes/Rarely]

**Debug Logs:**
```
$ RUST_LOG=debug caro "[prompt]"
[debug output]
```

**Configuration:**
```toml
# ~/.config/caro/config.toml
[relevant configuration]
```

**Additional Context:**
- Environment: [any relevant env vars]
- Related issues: [if any]
- Workaround found: [if any]
```

### Documentation Bug

```markdown
---
name: Bug Report
title: "[Docs]: [description of documentation issue]"
labels: ["bug", "docs", "triage"]
---

**Location:** [URL or file path]

**Issue Type:**
- [ ] Missing information
- [ ] Incorrect information
- [ ] Unclear/confusing
- [ ] Outdated
- [ ] Broken link

**Description:**
The documentation at [location] says [what it says], but [what's wrong].

**What I Tried:**
Following the documentation, I:
1. [step 1]
2. [step 2]
3. [step 3]

**What Happened:**
[result that differs from docs]

**Suggested Fix:**
[if you have a suggestion]

**User Profile:**
This would particularly affect users who:
- [characteristic 1]
- [characteristic 2]
```

### UX Friction Report

```markdown
---
name: Enhancement Request
title: "[UX]: [description of friction point]"
labels: ["enhancement", "ux", "triage"]
---

**User Profile:**
[Brief description of the tester persona]

**Friction Point:**
When [performing action], users experience [friction].

**Current Experience:**
1. [What happens now, step by step]
2. [Including confusing/difficult parts]

**Impact:**
- Frustration level: [Low/Medium/High]
- Likely outcome: [give up/seek help/work around/succeed anyway]
- Affected users: [which personas would hit this]

**Suggested Improvement:**
[If you have ideas for fixing it]

**Evidence:**
```
[Terminal output or screenshots showing the friction]
```
```

### Positive Feedback

```markdown
---
name: Feedback
title: "[Feedback]: [what worked well]"
labels: ["feedback", "positive"]
---

**User Profile:**
[Brief description of the tester persona]

**What Worked Well:**
[Description of positive experience]

**Specific Examples:**
1. [Example 1]
2. [Example 2]

**Comparison to Alternatives:**
Compared to [alternative tools/approaches], caro excels at:
- [Strength 1]
- [Strength 2]

**Suggestions for Highlighting:**
This could be emphasized in marketing/docs because:
- [Reason]
```

## Formatting Guidelines

### Command Output Formatting

Always use code blocks with appropriate language hints:

```markdown
$ caro "find large files"
```

For error output:
```
error: Model not found
  --> ~/.cache/caro/models/
  |
  = help: Run `caro --download-model` to fetch the model
```

### Version Information Block

Always include at the start of bug reports:

```markdown
**Environment:**
- caro: X.Y.Z
- Rust: 1.75.0 (if applicable)
- OS: macOS 14.5 (arm64)
- Shell: zsh 5.9
- Backend: Embedded (MLX)
```

### Reproduction Steps

Always numbered, always copy-paste ready:

```markdown
**Steps to Reproduce:**
1. Open terminal (zsh on macOS 14.5)
2. Ensure caro is installed: `caro --version`
3. Run: `caro "find files modified today"`
4. Observe the error message
```

### Error Classification

When describing errors, classify them:

| Type | Description | Example |
|------|-------------|---------|
| Panic | Program crashes | `thread 'main' panicked at...` |
| Error | Handled error, bad result | `error: Could not connect...` |
| Wrong output | No error, but incorrect result | Generated `ls` but expected `find` |
| Silent failure | No output, no error | Command runs but nothing happens |
| Hang | Program doesn't complete | Waiting indefinitely for response |

## Evidence Gathering

### Minimum Evidence for Any Bug

1. Exact command run (copy-paste)
2. Full output (stdout and stderr)
3. caro version
4. OS and version
5. Shell type

### Enhanced Evidence (When Possible)

```bash
# Collect comprehensive debug info
echo "=== System Info ===" > debug-output.txt
uname -a >> debug-output.txt
echo $SHELL >> debug-output.txt
caro --version >> debug-output.txt

echo "=== Debug Run ===" >> debug-output.txt
RUST_LOG=debug caro "your prompt" 2>&1 >> debug-output.txt

echo "=== Config ===" >> debug-output.txt
cat ~/.config/caro/config.toml 2>/dev/null >> debug-output.txt
```

### Sensitive Data Handling

Before including logs, remove:
- API keys
- Passwords
- Personal file paths (replace with `~/...`)
- Private server URLs

```markdown
**Configuration:**
```toml
[backend.vllm]
base_url = "http://internal-server:8000"  # Redacted
api_key = "***REDACTED***"
```
```

## Issue Title Conventions

### Format

```
[Category]: Brief description (OS if relevant)
```

### Examples

```
[Bug]: Installation fails on Ubuntu 22.04 via cargo install
[Bug]: Panic when running with empty prompt
[Docs]: Quickstart missing Rust installation step
[UX]: Error message doesn't suggest fix for missing model
[Enhancement]: Add --dry-run flag for previewing commands
[Feedback]: Excellent safety warnings for dangerous commands
```

## Labels Mapping

Based on the issue type, suggest appropriate labels:

| Issue Type | Suggested Labels |
|------------|-----------------|
| Installation failure | `bug`, `installation`, `[os]` |
| Runtime crash | `bug`, `critical` |
| Wrong command generated | `bug`, `command-generation` |
| Safety issue | `bug`, `safety`, `critical` |
| Doc problem | `docs`, `[area]` |
| UX friction | `enhancement`, `ux` |
| Missing feature | `enhancement`, `feature-request` |
| Positive feedback | `feedback` |

## Template: Complete Bug Report

Here's a complete example combining all elements:

```markdown
---
name: Bug Report
title: "[Bug]: cargo install fails on Linux with MLX dependency error"
labels: ["bug", "installation", "linux", "triage"]
---

## Environment
- **caro Version:** Unable to install (attempting vX.Y.Z)
- **Rust Version:** rustc 1.75.0 (82e1608df 2023-12-21)
- **Operating System:** Linux (Ubuntu/Debian)
- **OS Version:** Ubuntu 22.04 LTS
- **Backend:** N/A (installation failed)
- **Shell Type:** bash

## Bug Description
When attempting to install caro via `cargo install caro` on Linux,
the build fails with a dependency error related to MLX.

## Expected Behavior
Following the README "Option 3: Using Cargo" section, running
`cargo install caro` should successfully compile and install the
caro binary.

## Actual Behavior
The build fails with:
```
error[E0433]: failed to resolve: use of undeclared crate or module `mlx_sys`
```

## Exact Command
```bash
cargo install caro
```

## Steps to Reproduce
1. Set up Ubuntu 22.04 system with Rust 1.75.0
2. Verify Rust installation: `rustc --version`
3. Run: `cargo install caro`
4. Observe compilation failure after ~30 seconds

## Reproducibility
Every time (100%)

## Debug Logs
```
$ cargo install caro
    Updating crates.io index
  Downloaded caro vX.Y.Z
  ...
   Compiling mlx-sys v0.1.0
error[E0433]: failed to resolve: use of undeclared crate or module `mlx_sys`
   --> /home/user/.cargo/registry/src/.../mlx.rs:5:5
    |
5   | use mlx_sys::*;
    |     ^^^^^^^ use of undeclared crate or module `mlx_sys`
```

## Additional Context
- The README mentions MLX is for Apple Silicon but doesn't specify
  how to install on non-Apple platforms
- There's no `--no-default-features` option documented
- Searching GitHub issues didn't find a solution

## Suggested Fix
Consider one of:
1. Make MLX a non-default feature on non-macOS platforms
2. Document Linux installation clearly
3. Add error handling that suggests the correct feature flags

## Tester Profile
This was encountered as a Linux developer (Ubuntu) following the
cargo installation path. Impact: Complete blocker for Linux cargo users.

## Pre-submission Checklist
- [x] Searched existing issues to avoid duplicates
- [x] Verified this occurs on latest version
- [x] Provided exact command that triggers the bug
- [x] Included build output
- [x] This is not a security vulnerability
```
