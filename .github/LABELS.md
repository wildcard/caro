# GitHub Labels for Caro

This document defines all GitHub labels used in the Caro repository. Use these labels to categorize issues and PRs.

## Creating Labels

Run this script to create all labels:

```bash
#!/bin/bash
# create-labels.sh

# Contribution Lanes
gh label create "lane/security" --description "Security, guardrails, red-team testing" --color "d73a4a"
gh label create "lane/runtime" --description "Tokio, streaming, backend orchestration" --color "0075ca"
gh label create "lane/inference" --description "Performance, quantization, benchmarking" --color "7057ff"
gh label create "lane/ux" --description "Ratatui, TUI design, user experience" --color "a2eeef"
gh label create "lane/ecosystem" --description "MCP, IDE integration, plugins" --color "008672"
gh label create "lane/distribution" --description "Packaging, signing, offline bundles" --color "d876e3"

# Difficulty / Experience Level
gh label create "good-first-issue" --description "Good for newcomers" --color "7057ff"
gh label create "first-time-contributor" --description "Specially designed for first-timers" --color "0e8a16"
gh label create "advanced" --description "Requires significant experience" --color "b60205"
gh label create "expert" --description "Expert-level contribution" --color "d93f0b"

# Priority
gh label create "critical" --description "Critical priority, blocking release" --color "b60205"
gh label create "high-priority" --description "High priority" --color "d93f0b"
gh label create "medium-priority" --description "Medium priority" --color "fbca04"
gh label create "low-priority" --description "Low priority" --color "0e8a16"

# Type of Work
gh label create "bug" --description "Something isn't working" --color "d73a4a"
gh label create "enhancement" --description "New feature or request" --color "a2eeef"
gh label create "documentation" --description "Improvements or additions to documentation" --color "0075ca"
gh label create "refactor" --description "Code refactoring without behavior change" --color "d4c5f9"
gh label create "performance" --description "Performance optimization" --color "7057ff"
gh label create "testing" --description "Test coverage expansion" --color "1d76db"
gh label create "tooling" --description "Development tools and infrastructure" --color "bfdadc"

# Technical Areas
gh label create "safety" --description "Command safety and validation" --color "d73a4a"
gh label create "backend" --description "LLM backend implementation" --color "0052cc"
gh label create "agent" --description "Agent logic and decision-making" --color "5319e7"
gh label create "ui" --description "User interface and terminal output" --color "d876e3"
gh label create "cli" --description "Command-line interface" --color "0e8a16"
gh label create "config" --description "Configuration management" --color "fbca04"
gh label create "cache" --description "Model caching system" --color "006b75"
gh label create "execution" --description "Command execution engine" --color "b60205"

# Technology-Specific
gh label create "tokio" --description "Async runtime (Tokio)" --color "0052cc"
gh label create "ratatui" --description "Terminal UI framework" --color "d876e3"
gh label create "mcp" --description "Model Context Protocol" --color "5319e7"
gh label create "mlx" --description "Apple Silicon MLX backend" --color "0075ca"
gh label create "candle" --description "Candle ML framework" --color "7057ff"
gh label create "packaging" --description "Package management (Nix, Homebrew, etc.)" --color "006b75"

# Platform
gh label create "macos" --description "macOS-specific" --color "000000"
gh label create "linux" --description "Linux-specific" --color "fbca04"
gh label create "windows" --description "Windows-specific" --color "0075ca"
gh label create "cross-platform" --description "Multi-platform issue" --color "0e8a16"

# Community
gh label create "help-wanted" --description "Extra attention is needed" --color "008672"
gh label create "question" --description "Further information is requested" --color "d876e3"
gh label create "discussion" --description "Needs discussion before implementation" --color "cc317c"
gh label create "wontfix" --description "This will not be worked on" --color "ffffff"
gh label create "duplicate" --description "This issue or pull request already exists" --color "cfd3d7"
gh label create "conduct" --description "Code of Conduct issue" --color "d73a4a"

# Special Programs
gh label create "hacktoberfest" --description "Hacktoberfest eligible" --color "ff6b6b"
gh label create "good-for-demo" --description "Good candidate for demo/presentation" --color "fef2c0"
gh label create "caro-the-shiba" --description "Caro mascot related" --color "f9d0c4"

# Status
gh label create "in-progress" --description "Work is actively being done" --color "0e8a16"
gh label create "blocked" --description "Blocked by another issue" --color "d93f0b"
gh label create "needs-review" --description "Ready for review" --color "fbca04"
gh label create "needs-testing" --description "Needs testing on specific platforms" --color "d4c5f9"
gh label create "needs-docs" --description "Needs documentation update" --color "0075ca"

# Lane Lead Applications
gh label create "lane-lead-application" --description "Application to lead a contribution lane" --color "5319e7"
```

Make the script executable and run it:
```bash
chmod +x create-labels.sh
./create-labels.sh
```

---

## Label Categories

### Contribution Lanes (Primary Organization)

| Label | Color | Description | Use For |
|-------|-------|-------------|---------|
| `lane/security` | Red | Security, guardrails, policies | Safety validation, red-team tests, policy engine |
| `lane/runtime` | Blue | Tokio, streaming, orchestration | Async work, backend management, cancellation |
| `lane/inference` | Purple | Performance, ML optimization | Quantization, benchmarks, model loading |
| `lane/ux` | Light Blue | Ratatui, TUI design | Confirmations, plan flow, terminal UI |
| `lane/ecosystem` | Green | MCP, IDE integration | MCP server, VS Code, plugins |
| `lane/distribution` | Pink | Packaging, signing, bundles | Nix, Homebrew, offline installs |

**Usage:** Every technical issue should have exactly ONE lane label.

---

### Experience Level

| Label | Color | Description | Target |
|-------|-------|-------------|--------|
| `good-first-issue` | Purple | Good for newcomers | Absolute beginners, clear scope |
| `first-time-contributor` | Green | Specially for first-timers | Detailed guides, high support |
| `advanced` | Red | Requires experience | Multi-module changes, complex |
| `expert` | Dark Red | Expert-level | Architecture changes, security-critical |

**Usage:** Combine with lane labels to indicate difficulty within that lane.

---

### Priority

| Label | Color | Description | SLA |
|-------|-------|-------------|-----|
| `critical` | Red | Blocking release | Review within 24h |
| `high-priority` | Orange | Important | Review within 3 days |
| `medium-priority` | Yellow | Normal | Review within 1 week |
| `low-priority` | Green | Nice to have | Review when available |

**Usage:** Maintainers assign based on release timeline and user impact.

---

### Type of Work

| Label | Color | Description |
|-------|-------|-------------|
| `bug` | Red | Something isn't working |
| `enhancement` | Light Blue | New feature |
| `documentation` | Blue | Docs improvement |
| `refactor` | Light Purple | Code cleanup |
| `performance` | Purple | Speed optimization |
| `testing` | Dark Blue | Test coverage |
| `tooling` | Gray | Dev infrastructure |

---

### Technical Areas

| Label | Color | Description |
|-------|-------|-------------|
| `safety` | Red | Command safety |
| `backend` | Dark Blue | LLM backends |
| `agent` | Purple | Agent logic |
| `ui` | Pink | User interface |
| `cli` | Green | CLI arguments |
| `config` | Yellow | Configuration |
| `cache` | Teal | Model cache |
| `execution` | Red | Command execution |

---

### Technology-Specific

| Label | Use For |
|-------|---------|
| `tokio` | Async runtime issues |
| `ratatui` | Terminal UI framework |
| `mcp` | Model Context Protocol |
| `mlx` | Apple Silicon backend |
| `candle` | Candle ML framework |
| `packaging` | Package managers |

---

### Platform

| Label | Use For |
|-------|---------|
| `macos` | macOS-only issues |
| `linux` | Linux-only issues |
| `windows` | Windows-only issues |
| `cross-platform` | All platforms |

---

## Label Combinations (Examples)

### Security Lane Examples
```
lane/security + good-first-issue + safety
- Add Windows PowerShell dangerous patterns

lane/security + advanced + critical
- Implement policy engine with rule language

lane/security + expert + testing
- Build adversarial prompt injection test suite
```

### Runtime Lane Examples
```
lane/runtime + good-first-issue + tokio
- Add cancellation handling for long operations

lane/runtime + advanced + performance
- Implement streaming response display

lane/runtime + expert + refactor
- Redesign backend abstraction trait system
```

### Inference Lane Examples
```
lane/inference + good-first-issue + tooling
- Create benchmark harness for TTFT measurement

lane/inference + advanced + performance + mlx
- Optimize MLX model loading with mmap

lane/inference + expert + candle
- Implement CPU feature detection (AVX2/AVX512/NEON)
```

### UX Lane Examples
```
lane/ux + good-first-issue + ui
- Add syntax highlighting to command output

lane/ux + advanced + ratatui
- Build interactive confirmation UI with risk indicators

lane/ux + expert + enhancement
- Implement plan/review/apply flow with diff preview
```

### Ecosystem Lane Examples
```
lane/ecosystem + good-first-issue + documentation
- Write Claude Desktop integration guide

lane/ecosystem + advanced + mcp
- Build MCP server exposing command generation

lane/ecosystem + expert + integration
- Create VS Code extension with inline suggestions
```

### Distribution Lane Examples
```
lane/distribution + good-first-issue + packaging
- Create Nix flake for reproducible builds

lane/distribution + advanced + security
- Implement GPG signing for release artifacts

lane/distribution + expert + tooling
- Build offline bundle creator with embedded models
```

---

## Label Workflow

### For New Issues

1. **Contributor creates issue**
2. **Maintainer adds labels:**
   - One `lane/*` label (which lane owns this?)
   - One difficulty label (`good-first-issue`, etc.)
   - One type label (`bug`, `enhancement`, etc.)
   - Optional: technical area, platform, technology
3. **Lane lead reviews** (if lane lead exists)
4. **Issue is ready for assignment**

### For PRs

1. **Contributor opens PR**
2. **Maintainer adds labels:**
   - Same `lane/*` as related issue
   - `needs-review` when ready
   - `needs-testing` if platform verification needed
   - `needs-docs` if documentation update required
3. **Lane lead reviews** (or maintainer if no lane lead)
4. **Merge when approved**

---

## Special Label Meanings

### `help-wanted`
- Extra visibility for important issues
- Combined with lane labels to recruit for specific lanes
- Maintainers actively seeking contributors

### `good-for-demo`
- Feature would make a good demo
- Use for selection of demo content
- Helps prioritize user-visible features

### `caro-the-shiba`
- Anything related to our Shiba mascot!
- ASCII art, fun features, mascot design
- Keeps things light and fun

### `lane-lead-application`
- Applications to become a lane lead
- Not for regular issues
- Maintainers review and interview candidates

### `hacktoberfest`
- Eligible for Hacktoberfest contribution counting
- Should also have difficulty + lane labels
- Added in September/October

---

## Maintaining Labels

### Annual Review
- January: Review all labels for usage
- Remove unused labels
- Add new labels as needed
- Update this documentation

### Label Hygiene
- Maintainers should label all new issues within 48h
- Remove incorrect labels promptly
- Update labels as issue scope changes
- Keep lane labels mutually exclusive (one lane per issue)

---

## Questions?

- **Label confusion?** Ask in GitHub Discussions
- **New label needed?** Open an issue with `label: meta` in title
- **Wrong label?** Comment on the issue or PR

---

**Last updated:** 2025-01-15
