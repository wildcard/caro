# Tasks: v1.2.0 Close the Credibility Gap

## Work Packages Overview

| WP | Title | Status | Priority | Effort |
|----|-------|--------|----------|--------|
| WP01 | CLI Flag Implementation | Planned | P0 | 1 day |
| WP02 | Telemetry Subcommands | Planned | P0 | 1-2 days |
| WP03 | Config Key Expansion + Env Vars | Planned | P0 | 1-2 days |
| WP04 | Safer Alternatives | Planned | P1 | 1-2 days |
| WP05 | Static Matcher Expansion | Planned | P1 | 2-3 days |
| WP06 | Embedded Model Prompt + Fallback | Planned | P1 | 2-3 days |
| WP07 | PowerShell Command Generation | Planned | P1 | 1 day |
| WP08 | Color Output + Confidence Score | Planned | P1 | 1 day |
| WP09 | Skill Documentation Rewrite | Planned | P2 | 1 day |
| WP10 | Website Copy Audit | Planned | P2 | 0.5 day |
| WP11 | Test Suite & Validation | Planned | P2 | 1 day |
| WP12 | Release Preparation | Planned | P3 | 0.5 day |

**Total estimated effort**: 12-16 days

---

## WP01: CLI Flag Implementation

**Status**: Planned
**Priority**: P0
**Estimated Effort**: 1 day
**GitHub Issue**: #793

### Description

Add 5 documented CLI flags that produce "unexpected argument" errors today.

### Acceptance Criteria

- [ ] `--quiet` suppresses timing/debug output
- [ ] `-e` executes command directly (alias for `--execute`)
- [ ] `--no-telemetry` disables telemetry for session
- [ ] `--backend-info` lists available backends
- [ ] `--explain` shows detailed command breakdown
- [ ] All flags appear in `--help` output
- [ ] Unit tests for each flag

### Files to Create/Modify

- `src/cli/mod.rs` — Add flag fields to `Cli` struct
- `src/main.rs` — Wire flags into behavior

### Detailed Tasks

- [ ] T01: Add `quiet: bool` to `Cli` struct with `#[arg(long)]`
- [ ] T02: Add `execute_e: bool` to `Cli` struct with `#[arg(short = 'e')]`
- [ ] T03: Add `no_telemetry: bool` to `Cli` struct with `#[arg(long)]`
- [ ] T04: Add `backend_info: bool` to `Cli` struct with `#[arg(long)]`
- [ ] T05: Add `explain: bool` to `Cli` struct with `#[arg(long)]`
- [ ] T06: Wire `--quiet` in main.rs (suppress timing output)
- [ ] T07: Wire `-e` in main.rs (set `execute = true`)
- [ ] T08: Wire `--no-telemetry` in main.rs (disable telemetry config)
- [ ] T09: Wire `--backend-info` in main.rs (print backends, exit)
- [ ] T10: Wire `--explain` in main.rs (enable explanation output)
- [ ] T11: Add unit tests for all 5 flags

---

## WP02: Telemetry Subcommands

**Status**: Planned
**Priority**: P0
**Estimated Effort**: 1-2 days
**GitHub Issue**: #794

### Description

Uncomment and wire the telemetry subcommands (`show`, `export`) that are documented on caro.sh/telemetry but silently fall through to command generation.

### Acceptance Criteria

- [ ] `caro telemetry show` displays human-readable telemetry summary
- [ ] `caro telemetry export -o <file>` writes valid JSON
- [ ] Works without network access (reads from local SQLite)
- [ ] `caro telemetry --help` shows subcommands
- [ ] Unit tests for show and export

### Files to Create/Modify

- `src/main.rs` — Uncomment telemetry subcommand enum and match arm
- `src/cli/telemetry.rs` — Verify/handle_telemetry implementation

### Detailed Tasks

- [ ] T01: Uncomment `Telemetry` variant in `Commands` enum (line ~447)
- [ ] T02: Uncomment telemetry match arm (line ~1893)
- [ ] T03: Verify `TelemetryCommands` enum exists with Show/Export variants
- [ ] T04: Implement `show` formatter (read SQLite, format summary)
- [ ] T05: Implement `export` to JSON (read SQLite, serialize, write file)
- [ ] T06: Wire to storage path from main.rs
- [ ] T07: Add unit tests

---

## WP03: Config Key Expansion + Env Vars

**Status**: Planned
**Priority**: P0
**Estimated Effort**: 1-2 days
**GitHub Issue**: #795

### Description

Expand config key parser from 4 valid keys to 14+ documented keys. Add environment variable override support.

### Acceptance Criteria

- [ ] All 14 documented config keys accepted by `caro config set`
- [ ] `caro config get` shows nested key values
- [ ] `CARO_TELEMETRY_ENABLED=false` overrides config file
- [ ] `CARO_BACKEND=ollama` overrides config file
- [ ] `CARO_SAFETY=strict` overrides config file
- [ ] Priority: CLI flags > env vars > config file > defaults
- [ ] Backward compatible (old 4 keys still work)
- [ ] Unit tests for all new keys and env vars

### Keys to Add

```
telemetry.enabled, telemetry.level, telemetry.air_gapped
safety.level, safety.require_confirmation
output.format, output.color
backend.primary, backend.enable_fallback
backend.ollama.base_url, backend.ollama.model_name
backend.vllm.base_url, backend.vllm.model_name
```

### Files to Create/Modify

- `src/config/mod.rs` — Expand key parser, add struct fields, add env var support

### Detailed Tasks

- [ ] T01: Add `TelemetryConfig` struct with `enabled`, `level`, `air_gapped`
- [ ] T02: Add `OutputConfig` struct with `format`, `color`
- [ ] T03: Expand `BackendConfig` with `primary`, `enable_fallback`, `ollama`, `vllm` sub-structs
- [ ] T04: Expand config key match for all 14 keys
- [ ] T05: Add env var reader (CARO_TELEMETRY_ENABLED, CARO_BACKEND, CARO_SAFETY)
- [ ] T06: Implement config precedence chain (CLI > env > file > default)
- [ ] T07: Update `caro config get` for nested keys
- [ ] T08: Ensure backward compat (old flat keys still work)
- [ ] T09: Add unit tests

---

## WP04: Safer Alternatives

**Status**: Planned
**Priority**: P1
**Estimated Effort**: 1-2 days
**GitHub Issue**: #796

### Description

When safety validation blocks a command, generate and display a safer alternative. Currently `alternatives` is always `vec![]`.

### Acceptance Criteria

- [ ] Blocked commands show a safer alternative in human-readable output
- [ ] `alternatives` field in JSON output is populated for blocked commands
- [ ] 15-20 common dangerous patterns have mapped alternatives
- [ ] Preview commands shown for destructive operations
- [ ] Unit tests

### Files to Create/Modify

- `src/safety/alternatives.rs` (new) — Pattern-to-alternative mappings
- `src/safety/validator.rs` — Return alternatives on block
- `src/safety/mod.rs` — Export new module
- `src/main.rs` — Display alternatives in output

### Detailed Tasks

- [ ] T01: Create `src/safety/alternatives.rs` with SafetyAlternative struct
- [ ] T02: Define 15-20 pattern-to-alternative mappings
- [ ] T03: Integrate lookup into `validate()` function
- [ ] T04: Wire into main.rs output formatting
- [ ] T05: Populate `alternatives` field in GeneratedCommand serialization
- [ ] T06: Add unit tests

---

## WP05: Static Matcher Expansion

**Status**: Planned
**Priority**: P1
**Estimated Effort**: 2-3 days
**GitHub Issue**: #797

### Description

Add 30-50 new regex patterns to the static matcher covering common shell tasks. Currently returns `ls -la` or `echo 'Unable to generate command'` for most queries.

### Acceptance Criteria

- [ ] "delete all log files" → `find . -name "*.log" -type f -mtime +30 -delete`
- [ ] "check disk space" → `df -h`
- [ ] "show top processes by memory" → `ps aux --sort=-%mem | head -20`
- [ ] "count files" → `find . -type f | wc -l`
- [ ] "find large files" → works (already works, keep)
- [ ] "check uptime" → `uptime`
- [ ] Eval pass rate improves to 60%+
- [ ] Unit tests for each new pattern

### Files to Create/Modify

- `src/backends/static_matcher.rs` — Add patterns

### Categories to Cover

- File operations (delete logs, delete temp, delete old, clean downloads)
- System info (disk space, processes, CPU, network, memory, uptime, env vars)
- Text processing (count files, search text, largest files, compare files)
- Process management (kill process, list services)
- Git operations (status, log, diff)

### Detailed Tasks

- [ ] T01: Add file operation patterns (8-10 patterns)
- [ ] T02: Add system info patterns (8-10 patterns)
- [ ] T03: Add text processing patterns (5-8 patterns)
- [ ] T04: Add process management patterns (3-5 patterns)
- [ ] T05: Add git operation patterns (3-5 patterns)
- [ ] T06: Add preview commands for destructive operations
- [ ] T07: Add unit tests for all new patterns
- [ ] T08: Run eval suite and verify pass rate improvement

---

## WP06: Embedded Model Prompt + Fallback

**Status**: Planned
**Priority**: P1
**Estimated Effort**: 2-3 days
**GitHub Issue**: #798

### Description

Improve the embedded model's command generation quality through prompt engineering and implement a fallback chain when the model produces generic outputs.

### Acceptance Criteria

- [ ] System prompt includes few-shot examples and negative constraints
- [ ] Model never outputs `echo 'Unable to generate command'` (falls back to static matcher)
- [ ] `ls -la` generation rate < 20% for non-listing queries
- [ ] Fallback chain: embedded model → static matcher → error with guidance
- [ ] Eval pass rate for embedded model improves to 45%+

### Files to Create/Modify

- `src/prompts/` (relevant templates) — Rewrite system prompt
- `src/backends/embedded/embedded_backend.rs` — Add post-processing + fallback

### Detailed Tasks

- [ ] T01: Rewrite embedded model system prompt with examples
- [ ] T02: Add negative examples (what NOT to generate)
- [ ] T03: Implement post-processing filter in embedded_backend.rs
- [ ] T04: Add fallback chain (reject generic → static matcher → error)
- [ ] T05: Tune temperature/top-p sampling parameters
- [ ] T06: Run eval suite and verify improvement
- [ ] T07: Add unit tests for fallback behavior

---

## WP07: PowerShell Command Generation

**Status**: Planned
**Priority**: P1
**Estimated Effort**: 1 day
**GitHub Issue**: #800

### Description

When `--shell powershell` is passed, generate Windows-appropriate commands instead of POSIX.

### Acceptance Criteria

- [ ] `caro --shell powershell "list all files"` → `Get-ChildItem` or `dir`
- [ ] Static matcher has PowerShell equivalents for common operations
- [ ] Embedded model prompt modifies based on shell type
- [ ] Unit tests

### Files to Create/Modify

- `src/backends/static_matcher.rs` — PowerShell pattern variant
- `src/prompts/` — Shell-conditional prompt

### Detailed Tasks

- [ ] T01: Add POSIX-to-PowerShell mapping table in static matcher
- [ ] T02: Modify embedded model prompt when `--shell powershell`
- [ ] T03: Add PowerShell-specific test patterns
- [ ] T04: Add unit tests

---

## WP08: Color Output + Confidence Score

**Status**: Planned
**Priority**: P1
**Estimated Effort**: 1 day
**GitHub Issue**: #799

### Description

Add ANSI color-coded safety levels for TTY and populate `confidence_score` in JSON output.

### Acceptance Criteria

- [ ] TTY: colored safety levels with emoji (🟢 Safe, 🟡 Moderate, 🟠 High, 🔴 Critical)
- [ ] Piped: plain text without ANSI codes
- [ ] JSON: `confidence_score` field present with numeric value
- [ ] Unit tests

### Files to Create/Modify

- `src/main.rs` — Output formatting logic
- `src/cli/mod.rs` — Confidence score in JSON serialization

### Detailed Tasks

- [ ] T01: Add output mode detection (Colored vs Plain vs Json)
- [ ] T02: Implement ANSI color formatting for safety levels
- [ ] T03: Add `confidence_score` to JSON output struct
- [ ] T04: Ensure non-TTY gets plain text
- [ ] T05: Add unit tests

---

## WP09: Skill Documentation Rewrite

**Status**: Planned
**Priority**: P2
**Estimated Effort**: 1 day
**GitHub Issue**: #801

### Description

Rewrite all caro-shell-helper skill documentation to match actual CLI behavior. Remove fabricated config keys, UX patterns, and keyboard shortcuts.

### Acceptance Criteria

- [ ] Every config key in SKILL.md works with `caro config set`
- [ ] No keyboard shortcuts documented that don't exist
- [ ] No UX patterns shown that aren't implemented
- [ ] Check script uses correct crate name
- [ ] All examples produce described behavior

### Files to Create/Modify

- `.claude/skills/caro-shell-helper/SKILL.md`
- `.claude/skills/caro-shell-helper/README.md`
- `.claude/skills/caro-shell-helper/QUICK_START.md`
- `.claude/skills/caro-shell-helper/examples/basic-usage.md`
- `.claude/skills/caro-shell-helper/scripts/check-caro-installed.sh`

### Detailed Tasks

- [ ] T01: Rewrite SKILL.md config sections (use only working keys)
- [ ] T02: Remove fabricated UX patterns from SKILL.md
- [ ] T03: Rewrite README.md
- [ ] T04: Rewrite QUICK_START.md
- [ ] T05: Rewrite examples/basic-usage.md
- [ ] T06: Fix `check-caro-installed.sh` (Caro → caro)

---

## WP10: Website Copy Audit

**Status**: Planned
**Priority**: P2
**Estimated Effort**: 0.5 day
**GitHub Issue**: #802

### Description

Audit website copy against implementation. This is a docs-only task — updating the actual website source requires separate work (the website repo is separate).

### Acceptance Criteria

- [ ] All flags referenced in FAQ exist in CLI
- [ ] All telemetry examples in /telemetry work
- [ ] Performance claims match real measurements
- [ ] MCP status clarified on /explore

### Deliverables

- A findings report documenting what needs to change on each page
- Aligned with implementation in this PR

---

## WP11: Test Suite & Validation

**Status**: Planned
**Priority**: P2
**Estimated Effort**: 1 day

### Description

Create a comprehensive validation test suite that runs all the gap analysis tests from #790 and #791 as automated checks.

### Acceptance Criteria

- [ ] Test: `--quiet` works
- [ ] Test: `-e` works
- [ ] Test: `--no-telemetry` works
- [ ] Test: `--backend-info` works
- [ ] Test: `--explain` works
- [ ] Test: `telemetry show` works
- [ ] Test: `telemetry export` works
- [ ] Test: All 14 config keys accepted
- [ ] Test: `alternatives` populated on block
- [ ] Test: "delete all log files" generates command
- [ ] Test: "check disk space" returns `df -h`
- [ ] Test: PowerShell generates Windows syntax
- [ ] Test: Color output in TTY
- [ ] Test: JSON includes confidence_score
- [ ] All existing tests pass

### Files to Create/Modify

- `tests/website_claims/` — New test module

### Detailed Tasks

- [ ] T01: Create `tests/website_claims/mod.rs`
- [ ] T02: Create `tests/website_claims/flag_tests.rs`
- [ ] T03: Create `tests/website_claims/telemetry_tests.rs`
- [ ] T04: Create `tests/website_claims/config_tests.rs`
- [ ] T05: Create `tests/website_claims/safety_tests.rs`
- [ ] T06: Create `tests/website_claims/quality_tests.rs`
- [ ] T07: Run `cargo test` — all pass
- [ ] T08: Run `cargo clippy -- -D warnings` — zero warnings

---

## WP12: Release Preparation

**Status**: Planned
**Priority**: P3
**Estimated Effort**: 0.5 day

### Description

Prepare the PR for merge and release.

### Acceptance Criteria

- [ ] CHANGELOG.md updated with v1.2.0 entries
- [ ] `cargo test` passes
- [ ] `cargo clippy -- -D warnings` passes
- [ ] `cargo audit` clean
- [ ] PR description complete with all changes listed
- [ ] Branch up to date with main

### Detailed Tasks

- [ ] T01: Update CHANGELOG.md
- [ ] T02: Run full test suite
- [ ] T03: Run clippy + audit
- [ ] T04: Write PR description
- [ ] T05: Rebase on main
