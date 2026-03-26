# v1.2.0 Technical Specification: Delivering on the Promise

**Version**: 1.2.0
**Date**: March 26, 2026
**Status**: Draft — awaiting review
**Issues**: #790 (README gap analysis), #791 (website gap analysis)
**Milestone**: [v1.2.0](https://github.com/wildcard/caro/milestone/13)

---

## 1. Executive Summary

Caro v1.2.0 has two mandates:

1. **Close the credibility gap**: The website, docs, and marketing materials describe features that don't exist. Issues #790 and #791 catalog 40+ verified discrepancies. v1.2 must implement the missing features or remove the false claims — we cannot ship both.

2. **Ship the website**: The original v1.2.0 scope (Astro Starlight docs, landing page, SEO) is a marketing prerequisite. This depends on the code matching the claims first.

**Target**: April 30, 2026 (per OPERATOR_TRIAGE.md deadline extension)

---

## 2. Gap Analysis Summary

The following tables aggregate all verified gaps from #790 and #791, organized by implementation priority.

### 2.1 Tier 0 — Must Implement (Users will get immediate errors)

These features are documented with specific flag names/subcommands that users will try and get hard errors.

| ID | Gap | Source(s) | Affected File(s) | Est. Effort |
|----|-----|-----------|-------------------|-------------|
| T0-01 | `--quiet` flag | FAQ, docs | `src/cli/mod.rs` | S |
| T0-02 | `-e` / `--execute` short flag | FAQ | `src/cli/mod.rs` | XS |
| T0-03 | `--no-telemetry` flag | Telemetry page, FAQ | `src/cli/mod.rs` | S |
| T0-04 | `caro telemetry show` subcommand | Telemetry page | `src/main.rs` (uncomment + fix) | M |
| T0-05 | `caro telemetry export -o <file>` | Telemetry page | `src/main.rs` (uncomment + fix) | M |
| T0-06 | `telemetry.air_gapped` config key | Telemetry page | `src/config/mod.rs` | S |
| T0-07 | `CARO_TELEMETRY_ENABLED` env var | FAQ, docs | `src/config/mod.rs` | S |
| T0-08 | `--backend-info` flag | Skill docs | `src/cli/mod.rs` | S |
| T0-09 | Config TOML: `safety.level`, `safety.require_confirmation` | Skill docs | `src/config/mod.rs` | M |
| T0-10 | Config TOML: `backend.primary`, `backend.enable_fallback`, backend-specific settings | Skill docs | `src/config/mod.rs` | L |
| T0-11 | Config TOML: `output.format`, `output.color` | Skill docs | `src/config/mod.rs` | M |
| T0-12 | Installation checker script `cargo install Caro` → `cargo install caro` | Skill script | `.claude/skills/caro-shell-helper/scripts/` | XS |

### 2.2 Tier 1 — Must Implement (Misleading behavior)

These features exist in partial form but don't behave as advertised.

| ID | Gap | Source(s) | Affected File(s) | Est. Effort |
|----|-----|-----------|-------------------|-------------|
| T1-01 | **Safer alternatives when blocking**: When safety blocks a command, suggest a safer variant | Quick Start, Skill blog, #791 | `src/safety/`, `src/backends/static_matcher.rs`, `src/main.rs` | L |
| T1-02 | **Interactive confirmation y/n**: Only works in TTY; non-interactive silently skips | Quick Start, Skill SKILL.md | `src/execution/`, `src/main.rs` | M |
| T1-03 | **4-tier color-coded output**: Risk levels exist in code but no emoji/color in output | Skill SKILL.md, README | `src/main.rs` output formatting | M |
| T1-04 | **Claude backend accessible by default**: Behind `--features remote-backends` gate | Skill SKILL.md | `Cargo.toml`, `src/backends/mod.rs` | M |
| T1-05 | **`--explain` flag**: Used in examples, is in code but broken | Skill SKILL.md, #791 | `src/cli/mod.rs`, `src/main.rs` | S |
| T1-06 | **LLM quality**: `ls -la` / `echo 'Unable to generate command'` on 3/5 queries | #790 | Embedded model, prompt engineering | L |
| T1-07 | **`--shell powershell`**: Generates POSIX `ls -la` instead of Windows commands | Landing, FAQ | `src/prompts/`, static matcher | M |
| T1-08 | **`confidence_score` in JSON output**: Verbose shows it, JSON does not | Docs mentioning confidence | `src/cli/mod.rs` output serialization | S |

### 2.3 Tier 2 — Should Fix (Accuracy)

| ID | Gap | Source(s) | Affected File(s) | Est. Effort |
|----|-----|-----------|-------------------|-------------|
| T2-01 | MLX inference <2s claim: actual is ~4s | Explore page, docs intro | Performance: not just a claim fix | N/A |
| T2-02 | Memory ~2GB with 7B model: default is 1.5B (~1.1GB) | Explore page | Documentation-only fix | XS |
| T2-03 | "50+" vs "52+" dangerous patterns inconsistency | Explore page vs README | Documentation-only fix | XS |
| T2-04 | MCP server listed as current integration on /explore | Explore page | Website copy update | XS |
| T2-05 | Customizable safety rules: `custom_patterns` not exposed to CLI | Compare page | `src/cli/mod.rs`, config | M |
| T2-06 | `caro assess` / `caro knowledge` / `caro profile`: commented out / gated | Docs, roadmap | Feature decision needed | M |
| T2-07 | BSD vs GNU awareness claim untested | Compare, FAQ, Docs | Would need Linux testing | N/A |

### 2.4 Tier 3 — Website/Docs Cleanup (No Code Changes)

| ID | Gap | Source(s) | Est. Effort |
|----|-----|-----------|-------------|
| T3-01 | Skill docs show fabricated config TOML schema | Skill SKILL.md, README.md, examples | S |
| T3-02 | Skill docs show keyboard shortcuts `e`/`s` during confirmation | Skill basic-usage.md | XS |
| T3-03 | Skill docs show ambiguous request disambiguation UX | Skill basic-usage.md | XS |
| T3-04 | Skill docs show tool-not-found suggestions | Skill basic-usage.md | XS |
| T3-05 | Roadmap: 60 items at 0%, v1.1.3 released but not reflected | caro.sh/roadmap | XS |
| T3-06 | FAQ references `--model` instead of `--model-name` | FAQ | XS |

---

## 3. Implementation Plan

### Phase 1: Fix Immediately Broken Features (Weeks 1-2)

**Goal**: Eliminate all user-facing errors when following documentation.

#### 3.1 CLI Flag Hardening

**File**: `src/cli/mod.rs`

Implement the following flags that are documented but missing:

```rust
// Add to Cli struct:
#[arg(long, short = 'e', help = "Execute command directly (alias for --execute)")]
execute_short: bool,

#[arg(long, help = "Suppress non-essential output")]
quiet: bool,

#[arg(long, help = "Disable telemetry for this session")]
no_telemetry: bool,

#[arg(long, help = "Show backend status and available backends")]
backend_info: bool,
```

**Acceptance criteria**:
- `caro -e "list files"` executes the command (equivalent to `--execute`)
- `caro --quiet "list files"` suppresses timing/debug output
- `caro --no-telemetry "list files"` disables telemetry collection for this session
- `caro --backend-info` lists all backends and their availability status
- `caro --explain "list files"` shows detailed breakdown of the generated command

#### 3.2 Telemetry Subcommand Restoration

**File**: `src/main.rs`

Uncomment and fix the telemetry subcommand at lines 447-451 and 1893-1896. The code is already written but commented out. Key implementation:

1. Uncomment `Commands::Telemetry { command }` match arm
2. Uncomment the `TelemetryCommands` enum import
3. Wire `handle_telemetry()` to the storage path
4. Add `TelemetryCommands` subcommand with `Show` and `Export` variants
5. Implement `show` (print local telemetry stats) and `export` (write JSON to file)

**Acceptance criteria**:
- `caro telemetry show` prints local telemetry stats from SQLite
- `caro telemetry export -o telemetry-data.json` writes telemetry data to JSON
- `caro telemetry show` works without network access

#### 3.3 Configuration Key Expansion

**File**: `src/config/mod.rs`

Expand the config key parser to accept all documented keys. Current valid keys: `backend`, `model-name`, `shell`, `safety`. Must add:

```
telemetry.enabled          → UserConfig.telemetry.enabled
telemetry.level            → UserConfig.telemetry.level
telemetry.air_gapped       → UserConfig.telemetry.air_gapped
safety.level               → UserConfig.safety.level
safety.require_confirmation → UserConfig.safety.require_confirmation
output.format              → UserConfig.output.format
output.color               → UserConfig.output.color
backend.primary            → UserConfig.backend.primary
backend.enable_fallback    → UserConfig.backend.enable_fallback
backend.ollama.base_url    → UserConfig.backend.ollama.base_url
backend.ollama.model_name  → UserConfig.backend.ollama.model_name
backend.vllm.base_url      → UserConfig.backend.vllm.base_url
backend.vllm.model_name    → UserConfig.backend.vllm.model_name
```

**Acceptance criteria**:
- All keys in the documented TOML examples work without errors
- `caro config set <key> <value>` validates against the full schema
- `caro config get` shows all configured values

#### 3.4 Environment Variable Support

**File**: `src/config/mod.rs`

Add env var override support:
- `CARO_TELEMETRY_ENABLED=false` disables telemetry
- `CARO_BACKEND=ollama` selects backend
- `CARO_SAFETY=strict` sets safety level

Read these after config file but before CLI args (CLI args take precedence).

**Acceptance criteria**:
- `CARO_TELEMETRY_ENABLED=false caro "list files"` respects the env var
- Priority: CLI flags > env vars > config file > defaults

---

### Phase 2: Improve Command Generation Quality (Weeks 2-3)

**Goal**: Get command generation from "fails 60% of the time" to "works 80%+ of the time."

#### 3.5 Static Matcher Expansion

**File**: `src/backends/static_matcher.rs`

The current static matcher returns `ls -la` or `find . -type f -name '*.txt' | sort -nrk 3,3` for too many queries regardless of input. Expand patterns for:

- "delete all log files" → `find . -name "*.log" -type f -mtime +30 -delete` (not `echo 'Unable to generate'`)
- "check disk space" → `df -h` (currently returns `ls -la`)
- "show processes by memory" → `ps aux --sort=-%mem | head -11`
- "find large files" → `find . -type f -size +100M`
- "count files by type" → `find . -type f | sed 's/.*\.//' | sort | uniq -c | sort -rn`

Add 30-50 high-traffic query patterns covering the most common shell tasks.

**Acceptance criteria**:
- Pass rate on eval test suite improves from 31% baseline to 60%+
- "delete all log files" no longer returns `echo 'Unable to generate command'`
- "check disk space" returns `df -h`

#### 3.6 Prompt Engineering for Embedded Model

**File**: `src/prompts/` (relevant prompt templates)

The embedded model (qwen2.5-coder-1.5b) generates `ls -la` too frequently. Improve the system prompt to:
- Reject `ls -la` as a default/fallback response
- Force single-line POSIX command output
- Provide few-shot examples for common operations
- Add negative examples (what NOT to generate)

**Acceptance criteria**:
- Embedded model pass rate improves by 15%+ on eval suite
- `ls -la` generation rate drops below 20% of queries

#### 3.7 PowerShell/Windows Command Generation

**File**: `src/prompts/`, `src/backends/static_matcher.rs`

When `--shell powershell` is passed, generate Windows commands:
- `ls` → `Get-ChildItem` or `dir`
- `find` → `Get-ChildItem -Recurse | Where-Object {...}`
- `cat` → `Get-Content`
- `rm` → `Remove-Item`

Static matcher should have a PowerShell variant for common operations.

**Acceptance criteria**:
- `caro --shell powershell "list all files"` returns PowerShell syntax
- `caro --shell cmd "list all files"` returns cmd syntax

---

### Phase 3: Safety & UX Improvements (Weeks 3-4)

**Goal**: Make the safety system match the 4-tier educational UX described in marketing.

#### 3.8 Safer Alternatives Implementation

**Files**: `src/safety/`, `src/backends/static_matcher.rs`, `src/main.rs`

When a command is blocked by safety, generate a safer alternative:

| Dangerous Pattern | Block Message | Suggested Alternative |
|---|---|---|
| `rm -rf /` | CRITICAL: Full filesystem destruction | Try `rm -rf ./specific-directory` instead |
| `rm -rf ~` | CRITICAL: Home directory deletion | Try `rm -rf ~/specific-project` instead |
| `rm -rf *` in root | HIGH: Mass recursive deletion | Preview first: `ls -la` then `rm -rf ./specific-dir` |
| `chmod 777 /` | CRITICAL: Root privilege escalation | `chmod 755` for directories, `chmod 644` for files |
| `dd if=/dev/zero` to disk | CRITICAL: Disk destruction | Not recommended — use proper partitioning tools |

Wire this into the `alternatives` field of `GeneratedCommand` (currently always `vec![]`).

**Acceptance criteria**:
- `caro "delete everything in root directory"` shows CRITICAL warning with suggestion
- `caro "delete all log files"` (non-critical) shows safer approach with preview command
- `alternatives` field in JSON output is populated for blocked/warned commands

#### 3.9 Color-Coded Output

**File**: `src/main.rs` (output formatting)

Currently all output is plain text. Add ANSI color codes:

```
Safe (Green):     \x1b[32m✅ Safe\x1b[0m
Moderate (Yellow): \x1b[33m⚠️ Moderate\x1b[0m
High (Orange):    \x1b[33m🔴 High\x1b[0m (orange ANSI is bright yellow)
Critical (Red):   \x1b[31m🔴 Critical\x1b[0m
```

Respect `--no-color` or TTY detection. JSON output unchanged.

**Acceptance criteria**:
- Interactive terminal shows colored safety levels with emoji
- Piped/redirected output uses plain text
- `--output json` unchanged

#### 3.10 Interactive Confirmation UX

**File**: `src/main.rs`, `src/execution/`

The current behavior in non-interactive mode silently skips confirmation. Improve:

- In TTY: Show `? Execute this command? (y/n)` with y/n input
- In non-TTY (CI, piped): Show the command and suggest `--execute/-x` flag
- Add `--confirm` / `-y` to skip confirmation (exists but behavior should match docs)

**Acceptance criteria**:
- In terminal: Shows `Execute this command? (y/N)` prompt
- In CI/piped: Shows clear guidance on how to execute
- `-y`/`--confirm` auto-executes without prompting

---

### Phase 4: Website & Docs Alignment (Weeks 4-5)

**Goal**: Update all marketing materials to match reality — remove false claims, correct descriptions, add "coming soon" badges.

#### 3.11 Skill Documentation Overhaul

**Files**: `.claude/skills/caro-shell-helper/SKILL.md`, `README.md`, `QUICK_START.md`, `examples/basic-usage.md`

Rewrite all skill docs to match actual CLI behavior:
- Replace fabricated config TOML with actual valid keys
- Remove keyboard shortcuts that don't exist
- Remove disambiguation UX that doesn't exist
- Remove tool-not-found suggestions
- Fix `caro install Caro` → `caro install caro` in check script
- Add "Coming Soon" badges for features in development

#### 3.12 Website Copy Audit

Update all website pages flagged in #791:
- `/faq`: Remove `--quiet`, `-e`, `--no-telemetry`, `--model`, `CARO_TELEMETRY_ENABLED` — OR mark as "coming in v1.2"
- `/telemetry`: Remove or gate subcommand/flag/config examples behind v1.2 release
- `/roadmap`: Reflect actual completion status of v1.1.x items
- `/explore`: Fix MCP server status (currently "Available", should be "Coming Soon"), fix performance claims
- `/compare`: Mark "Customizable safety rules" as "Planned" until CLI support lands

#### 3.13 Documentation Site (Astro Starlight)

**PR**: #139, #639

- Create `docs.caro.sh` with accurate documentation
- Remove all features from docs that don't exist in current release
- Gate future features behind v1.2/v1.3 tabs

---

### Phase 5: Infrastructure & Release (Week 5)

#### 3.14 Resolve Merge Conflicts (P0 Blocker)

**Issue**: #681

Resolve merge conflicts for the 23 v1.2.0 PRs that are blocked. This is a prerequisite for any PR merges.

#### 3.15 i18n Completion

**Issues**: #687, #746, #745, #688, #689, #690

Current state: 61.8% average translation coverage across 14 locales. Target: Tier 1 ≥95%, Tier 2 ≥85%.

Complete WP07 (translation automation) and WP08 (language switcher polish) from the i18n spec.

#### 3.16 v1.2.0 Release Preparation

- Run full `cargo test` suite — all tests must pass
- Run `cargo clippy -- -D warnings` — zero warnings
- Run `cargo fmt --all --check` — zero formatting issues
- Run `cargo audit` — no known vulnerabilities
- Update `CHANGELOG.md` with v1.2.0 release notes
- Update `website/src/config/site.ts` version
- Run release preparation: `/caro.release.prepare`

---

## 4. Architecture Decisions Required

| Decision | Impact | Recommendation |
|----------|--------|----------------|
| Implement `--quiet`/`--no-telemetry`/`-e` or remove from docs? | Tier 0 features | **Implement** — these are simple flags, easier than updating 6+ doc pages |
| Enable Claude backend by default or remove from skill docs? | Tier 1 feature | **Gate in skill docs** — `--features remote-backends` should be opt-in, not default |
| Fix `caro assess`/`knowledge`/`profile` or mark experimental? | Tier 2 feature | **Mark experimental** — behind feature flags with clear docs |
| Fix MLX <2s claim or update to ~4s? | Tier 2 accuracy | **Update to ~4s** — performance is what it is |
| Fix MCP status on /explore or /landing? | Tier 2 accuracy | **Remove from /explore** — landing page's "Coming Soon" is the correct state |
| Gate or rewrite Skill examples? | Skill docs | **Rewrite** — skill should only reference features that work today |

---

## 5. Test Plan

### 5.1 Regression Tests

Every new flag must have a unit test:

```
test --quiet suppresses timing output
test -e executes command
test --no-telemetry disables telemetry for session
test --backend-info shows available backends
test telemetry show displays stats
test telemetry export writes JSON file
test telemetry.air_gapped config key accepted
test CARO_TELEMETRY_ENABLED env var respected
test safety.level config key accepted
test output.format config key accepted
test backend.primary config key accepted
```

### 5.2 Integration Tests

```
test "safer alternative suggested when blocking" — dangerous command → blocked with suggestion
test "color output in TTY" — check ANSI escape codes present in output
test "non-TTY shows guidance" — piped output shows --execute flag suggestion
test "powershell generates Windows commands" — --shell powershell → Get-ChildItem
```

### 5.3 Manual QA Checklist

- [ ] Follow all examples on caro.sh/faq — none produce errors
- [ ] Follow all examples on caro.sh/telemetry — none produce errors
- [ ] Follow all examples in skill SKILL.md — config keys work
- [ ] `caro "delete all log files"` returns useful command (not `echo 'Unable to'`)
- [ ] `caro "check disk space"` returns `df -h` (not `ls -la`)
- [ ] `caro --shell powershell "list files"` returns PowerShell syntax
- [ ] `caro "delete everything in root directory"` shows CRITICAL with alternative
- [ ] In interactive terminal: see `Execute this command? (y/N)` prompt
- [ ] In CI/piped: see guidance text
- [ ] JSON output includes populated `alternatives` field when safety blocks

---

## 6. Dependencies & Risks

### Dependencies
- #681 must be resolved first (merge conflicts blocker)
- #790/#791 gap analyses define the acceptance criteria
- Website PRs (#130, #639) depend on content being correct first

### Risks
| Risk | Impact | Mitigation |
|------|--------|------------|
| Embedded model quality still poor after prompt changes | High | Add static matcher expansion as fallback; ship best-of-both |
| Telemetry subcommands require more work than "uncomment" | Medium | Est. 2-3 days if clean; spike first |
| Config expansion breaks existing user configs | Medium | Add migration logic, maintain backward compat |
| 23 blocked PRs create merge order complexity | High | Resolve #681 first, then merge in dependency order |

---

## 7. Out of Scope (Deferred to v1.3.0)

The following are explicitly deferred and should NOT be in v1.2.0:

- Interactive TUI welcome screen (#641, #672)
- Proactive suggested queries (#643, #674)
- Enhanced context with Starship (#636, #670)
- Request memory tracking (#647, #671)
- User feedback system (#578, #673)
- Capability boundaries (#644)
- ShellCheck integration (#619)
- Knowledge/profile features (behind feature gates)
- Shell integration daemon (007 spec, v1.0)
- Dogma rule engine
- Karo distributed intelligence
- Voice synthesis
