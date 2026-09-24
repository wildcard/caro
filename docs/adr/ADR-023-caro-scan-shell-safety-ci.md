# ADR-023 — `caro scan`: Shell Command & Script Safety Analysis for CI/CD

**Date**: 2026-06-12
**Status**: Proposed
**Authors**: `caro-research--scoping-process` (automated research/scoping agent)
**Relates to**: ADR-015 (MCP Safety Server), ADR-022 (caro-safety library)
**Deferred from**: ADR-015 out-of-scope item: "`caro-check` standalone RAMPART integration"
**Issue**: ROADMAP.md v2.0.0 "Advanced Tool Use Patterns (#180)"

---

## Context

### The Gap: Caro Has No CI-First Entry Point

ADR-015 scoped `--output json` on `caro --dry-run` as the RAMPART/CI integration
path, but explicitly deferred the "standalone RAMPART integration" to the next
sprint. That sprint is now.

The current `caro` CLI is optimised for interactive use: it prompts, colours
output, and exits with a single boolean (executed/not). A CI pipeline needs:

1. A subcommand that accepts a command string **or** a script file **or** stdin.
2. Machine-readable output that integrates with existing tooling (shellcheck
   JSON format, SARIF for GitHub Advanced Security).
3. Deterministic exit codes that `make`, GitHub Actions `if:` conditions, and
   RAMPART assertions can gate on without parsing stdout.
4. Batch operation: check an entire CI step's command list in one subprocess call.

`caro check` already exists as the CaroML `.caro` task-file validator
(`src/caroml/mod.rs::check_file`). Rather than overload that subcommand with a
second meaning, this ADR introduces **`caro scan`** as the dedicated shell-safety
entry point.

### Phase 1 Research — The Competitive Landscape

#### ShellCheck (koalaman/shellcheck, ~36 K stars, Haskell)

ShellCheck is the dominant static analysis tool for shell scripts. It finds
syntax errors, deprecated constructs, quoting bugs, and portability issues.

**What it gets right (to replicate):**
- Exit 0 = clean, exit 1 = issues found — universally understood by CI runners.
- Reads from a file path, stdin (`-`), or inline string (`--`).
- Multiple output formats: `-f gcc`, `-f json1`, `-f checkstyle`. JSON1 format:

```json
[
  {
    "file": "test.sh",
    "line": 3,
    "endLine": 3,
    "column": 7,
    "endColumn": 16,
    "level": "warning",
    "code": 2086,
    "message": "Double quote to prevent globbing and word splitting."
  }
]
```

- SARIF output via `shellcheck-sarif` pipe (GitHub Marketplace action).

**Their bugs / design gaps we design around:**
1. **No intent/safety analysis.** ShellCheck catches `rm -rf /` as valid
   syntax. It has zero awareness of destructive intent, privilege escalation,
   or supply-chain attack patterns. This is the gap Caro fills.
2. **No risk-level taxonomy.** ShellCheck levels are `error/warning/info/style` —
   inadequate for a security tool. Caro's `Critical/High/Moderate/Low` maps
   directly to CI gate severity.
3. **No suggested-routing.** ShellCheck tells you what's wrong; it does not
   tell you whether to block, human-gate, or log. Caro does.
4. **Haskell binary.** Not embeddable in Rust agent frameworks. `caro scan`
   is a native Rust subprocess callable by any language.
5. **No `command_hash`.** Shellcheck results cannot be correlated with audit
   records. `SafetyAssessmentOutput.command_hash` (ADR-015) solves this.

#### RAMPART (Microsoft, May 2026, Python)

RAMPART establishes safety-in-CI as a developer expectation. Its approach:
write test assertions against agent behavior *before* running the agent in
production.

**What it gets right:**
- CI-first design: assertions fail the build, not post-hoc alerts.
- Structured test cases with known inputs → expected outputs.

**Their gaps we solve:**
- Python-native only. No Rust SDK, no cross-language subprocess contract.
- No shell-specific intelligence. RAMPART tests agent decisions; it does not
  validate the shell commands those agents emit.
- 53ms median for the OAP interceptor (from ADR-022 research). Caro's regex
  engine is sub-2ms.

#### GitHub Advanced Security / CodeQL Shell Queries

CodeQL can flag some dangerous shell patterns in Actions workflows, but only
within `.github/workflows/` YAML — not for arbitrary scripts or inline
commands. SARIF output is the integration format.

**Caro's unique value:** SARIF output from `caro scan --format sarif` slots
directly into the same GitHub Code Scanning dashboard, covering the gap that
CodeQL leaves.

---

## Decision

Add a `caro scan` subcommand to the existing `caro` CLI binary. No new binary,
no new feature flags, no new dependencies. The subcommand reuses
`SafetyValidator` from `src/safety/` directly.

### Command surface

```
USAGE:
    caro scan [OPTIONS] [COMMAND]

ARGS:
    [COMMAND]    Shell command to validate. Omit to read from stdin.

OPTIONS:
    -f, --file <FILE>           Script file to validate (reads all commands)
    --format <FORMAT>           Output format [default: text] [possible: text, json, sarif]
    --safety-level <LEVEL>      Safety policy [default: moderate] [possible: strict, moderate, permissive]
    --shell <SHELL>             Target shell [default: auto-detect] [possible: bash, sh, zsh, fish]
    --fail-on <LEVEL>           Exit non-zero at this risk level or above [default: high]
                                [possible: critical, high, moderate, low]
    -q, --quiet                 Suppress text output; rely on exit code only

EXAMPLES:
    caro scan "rm -rf /tmp/build"
    caro scan --file deploy.sh --format json
    cat pipeline.sh | caro scan -
    caro scan --file ci.sh --format sarif > results.sarif
```

### What it does

1. Receives one or more commands (argument, `--file`, or stdin).
2. For `--file`: extracts individual shell statements via line-by-line parsing
   (no AST — intentionally simple; avoids ADR-007's AST dependency for this
   use case). Lines starting with `#` are skipped; continuation lines (`\`)
   are joined.
3. Passes each command to `SafetyValidator::new(config).validate_command()`.
4. Produces `SafetyAssessmentOutput` per command (ADR-015 schema — already
   defined; `caro scan` is the first consumer of that schema in the CLI).
5. Exits with the code corresponding to the highest-severity finding.

### Output formats

**`--format text`** (default, human-readable):
```
BLOCKED  critical  rm -rf /
  Matched: recursive deletion of root filesystem
  Routing: block

WARN     moderate  chmod 777 /tmp/app
  Matched: world-writable permission
  Routing: async_log

OK       safe      ls -la ./dist
```

**`--format json`** (RAMPART / caro-native):
Emits a JSON array of `SafetyAssessmentOutput` objects (ADR-015 schema).
One object per input command. Identical schema to `caro --dry-run --output json`.

```json
[
  {
    "schema_version": 1,
    "command": "rm -rf /",
    "risk_level": "critical",
    "allowed": false,
    "suggested_routing": "block",
    "matched_patterns": ["recursive deletion of root filesystem"],
    "reason": "...",
    "confidence": 1.0,
    "assessed_at": "2026-06-12T...",
    "command_hash": "e3b0c44..."
  }
]
```

**`--format sarif`** (GitHub Advanced Security):
SARIF 2.1.0 output. Maps Caro risk levels to SARIF `level` values:
- `critical` → `error`
- `high` → `error`
- `moderate` → `warning`
- `low` → `note`

The `ruleId` is the matched pattern slug (e.g. `caro/recursive-root-delete`).
Results can be uploaded directly to GitHub Code Scanning with
`github/codeql-action/upload-sarif`.

---

## New Types

All new types in `src/cli/scan.rs` (new file under the existing `cli` module).
No new module; `cli/mod.rs` gains `pub mod scan;`.

### `ScanArgs`
```rust
#[derive(Debug, Parser)]
pub struct ScanArgs {
    /// Single command to validate (omit to read from stdin)
    pub command: Option<String>,
    #[arg(short = 'f', long)]
    pub file: Option<PathBuf>,
    #[arg(long, default_value = "text", value_enum)]
    pub format: ScanOutputFormat,
    #[arg(long, default_value = "moderate", value_enum)]
    pub safety_level: SafetyLevel,
    #[arg(long, default_value = "high", value_enum)]
    pub fail_on: RiskLevel,
    #[arg(short = 'q', long)]
    pub quiet: bool,
}
```

### `ScanOutputFormat`
```rust
#[derive(Debug, Clone, ValueEnum)]
pub enum ScanOutputFormat { Text, Json, Sarif }
```

### `ScanResult`  ← aggregate for batch output
```rust
#[derive(Debug, Serialize)]
pub struct ScanResult {
    pub schema_version: u8,      // 1
    pub total: usize,
    pub blocked: usize,
    pub warned: usize,
    pub safe: usize,
    pub highest_risk: Option<RiskLevel>,
    pub assessments: Vec<SafetyAssessmentOutput>,
}
```

`SafetyAssessmentOutput` is the same type defined in ADR-015. Until ADR-015
is implemented and `SafetyAssessmentOutput` lives in `src/mcp/mod.rs` or
`src/safety/`, define it in `src/cli/scan.rs` and plan a one-line re-export
when ADR-015 lands. This avoids blocking `caro scan` on ADR-015's build spike.

---

## Files That Change

| File | Change |
|------|--------|
| `src/cli/mod.rs` | Add `pub mod scan;`; add `Scan(ScanArgs)` arm to `Commands` enum; wire `Commands::Scan` in `run()` |
| `src/cli/scan.rs` | **NEW** — `ScanArgs`, `ScanOutputFormat`, `ScanResult`, `SafetyAssessmentOutput` (local copy until ADR-015 lands), `run_scan()` |
| `tests/scan_integration.rs` | **NEW** — 8 deterministic integration tests (table below) |

No changes to `src/safety/` — it is consumed as-is. No new binary. No new
feature flags (scan uses the unconditional `src/safety/` code path already
compiled into every build).

---

## Exit Code Contract

| Exit code | Meaning |
|-----------|---------|
| 0 | All commands safe at the configured `--fail-on` level |
| 1 | At least one command blocked (`suggested_routing: block`) |
| 2 | At least one command requires human gate (`suggested_routing: human_gate`) and `--fail-on` ≤ `high` |
| 3 | At least one async-log warning and `--fail-on` ≤ `moderate` |
| 10 | Input error (no command, unreadable file, unknown shell) |
| 11 | Internal validator error |

Exit codes are identical to the `caro --dry-run --output json` contract from
ADR-015. This is deliberate: CI scripts that already gate on the dry-run path
need zero changes to adopt `caro scan`.

---

## Integration Tests

`tests/scan_integration.rs` — known inputs → deterministic JSON + exit code.
All tests call `caro scan` as a subprocess (or call `run_scan()` directly with
`ScanOutputFormat::Json`). No LLM, no network.

| Test | Input | `highest_risk` | Exit |
|------|-------|----------------|------|
| `test_safe_command` | `ls -la` | `safe` | 0 |
| `test_critical_exits_1` | `rm -rf /` | `critical` | 1 |
| `test_moderate_warn_exits_0_default` | `chmod 777 /tmp` | `moderate` | 0 (default `--fail-on high`) |
| `test_moderate_warn_exits_3_when_fail_on_moderate` | `chmod 777 /tmp` + `--fail-on moderate` | `moderate` | 3 |
| `test_file_batch_mixed` | file with `ls`, `rm -rf /`, `echo hi` | `critical` | 1 |
| `test_stdin_pipe` | `echo 'rm -rf /'` piped to `caro scan -` | `critical` | 1 |
| `test_json_schema_stable` | any | `schema_version == 1` | — |
| `test_sarif_output_levels` | `rm -rf /`, `chmod 777`, `ls` | SARIF `error`/`warning`/`none` | — |

---

## Competitive Differentiation (Phase 2 Summary)

| Dimension | ShellCheck | RAMPART | caro scan |
|-----------|-----------|---------|-----------|
| Safety/intent analysis | ✗ | partial | **✓ 52+ patterns** |
| Risk taxonomy | error/warn/info | pass/fail | **Critical/High/Moderate/Low** |
| `command_hash` for audit | ✗ | ✗ | **✓** |
| Suggested routing | ✗ | ✗ | **✓ block/human-gate/async-log/auto** |
| SARIF output | via pipe | ✗ | **✓ native** |
| Offline / no network | ✓ | ✗ | **✓** |
| Rust / embeddable | ✗ (Haskell) | ✗ (Python) | **✓** |
| Sub-2ms per command | ✗ (~10ms) | ✗ (53ms) | **✓** |
| Stdin / pipe support | ✓ | ✗ | **✓** |

---

## Explicit Out-of-Scope (v1)

| Not in scope | Rationale |
|---|---|
| Full shell AST parsing | ADR-007 is a separate feature; line-by-line is sufficient for CI use case and avoids a multi-week dependency |
| ShellCheck integration (calling shellcheck as a subprocess) | Additive; valid as a follow-up that pipes both outputs into a unified SARIF report |
| `--fix` mode (auto-suggest safer alternatives) | Requires LLM inference; out-of-scope for a stateless safety scan |
| Pre-commit hook installer (`caro scan --install-hook`) | Quality-of-life addition; tracked separately |
| Windows PowerShell script support | `ShellType::PowerShell` is a future extension point |

---

## Consequences

**Positive:**
- First CI-native entry point for Caro's safety engine. Fills the slot ADR-015
  explicitly deferred.
- Zero new dependencies. No build spike needed (no new external crates).
- SARIF output enables GitHub Code Scanning integration with one workflow step.
- Exit code parity with ADR-015's dry-run contract: adopting `caro scan` is a
  one-line CI change for existing users of `--output json`.
- `SafetyAssessmentOutput` defined locally in `src/cli/scan.rs` unblocks
  implementation without waiting for ADR-015's rmcp build spike.

**Negative / risks:**
- Line-by-line parsing misses multi-line here-docs and compound statements. This
  is a known and accepted limitation for v1. The fix (ADR-007 AST parser) is
  tracked separately and composable.
- Defining `SafetyAssessmentOutput` locally creates a duplication that must be
  resolved when ADR-015 lands. Add a `TODO(ADR-015): replace with crate re-export`
  comment at definition site.

---

## Alternatives Considered

### A: Extend `caro check` (CaroML) to also handle shell commands
Rejected. `caro check` has a stable contract for task-file validation. Overloading
it with "shell safety scan" conflates two distinct operations and breaks existing
muscle memory (`caro check deploy.caro` vs `caro check "rm -rf /"`).

### B: New binary `caro-check`
Valid but heavier: adds a second binary to the release, doubles install
instructions, and provides no benefit that a subcommand does not. The
`[[bin]]` path is reserved for `caro-mcp` (ADR-015) where the MCP server
lifecycle genuinely justifies a separate process.

### C: Wait for ADR-022 caro-safety crate to land first
Unnecessary. `caro scan` calls `src/safety/` directly — the same code path
the CLI has always used. The caro-safety workspace crate (ADR-022) is the
*FFI surface* for external callers; `caro scan` is an *internal* consumer.

---

## Implementation Checklist (for the engineer picking this up)

- [ ] Add `pub mod scan;` to `src/cli/mod.rs`
- [ ] Add `Scan(scan::ScanArgs)` arm to `Commands` enum in `src/cli/mod.rs`
- [ ] Implement `src/cli/scan.rs`:
  - [ ] `ScanArgs`, `ScanOutputFormat`, `ScanResult`, local `SafetyAssessmentOutput`
  - [ ] `run_scan(args: ScanArgs) -> Result<()>` — reads input, loops validator, formats output, sets exit code
  - [ ] `format_text()`, `format_json()`, `format_sarif()` renderers
  - [ ] `extract_commands_from_script(content: &str) -> Vec<String>` — line-by-line extractor
- [ ] Wire `Commands::Scan(args) => scan::run_scan(args).await` in `cli/mod.rs::run()`
- [ ] `tests/scan_integration.rs` — 8 tests from table above
- [ ] `cargo test` passes
- [ ] `CHANGELOG.md` entry under `## [Unreleased]`
- [ ] Add `TODO(ADR-015): replace SafetyAssessmentOutput with crate re-export` comment

---

*Generated by `caro-research--scoping-process` scheduled agent · 2026-06-12*
