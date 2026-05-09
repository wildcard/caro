# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **CaroML preview** — a meta-language for intent-tracked shell tasks. A
  `.caro` file describes a task as a sequence of natural-language `DO` lines
  in an eight-keyword line-keyword DSL (TASK / WHY / NEED / ON / LET / DO /
  NOTE / REM). Caro generates a per-platform `.caro.lock` (schema_version=2)
  with active variants, A/B candidates, validation outcomes, and a
  history-trail; an on-disk `.<platform>.sh` runbook can be regenerated
  from the lock and `bash`-executed without Caro installed. New CLI verbs:
  `caro check / generate / run / export / list / new / jobs / do /
  experiment / adopt / history / why / render / skill install`. Carofile
  orchestration (`caro do <job>`) lets a single project file index native
  CaroML tasks alongside external commands. The bundled `caro-scaffold`
  Claude-Code-style skill (under `.claude/skills/caro-scaffold/`) is
  installable via `caro skill install`. Multi-angle validator chain
  (safety + platform + secrets + side_effects) drives a per-step repair
  loop. Project memory: per-user run journal at
  `~/.caro/state/<intent_hash>/journal.jsonl`, A/B challenger lifecycle
  via `caro experiment` / `caro adopt`. CaroML preview ships behind the
  same `cargo install caro` path; comprehensive E2E test
  (`tests/caroml_e2e.rs`) exercises all 16 phases of the pipeline.
  ([#893](https://github.com/wildcard/caro/pull/893),
   [#904](https://github.com/wildcard/caro/pull/904),
   [#905](https://github.com/wildcard/caro/pull/905),
   [#906](https://github.com/wildcard/caro/pull/906),
   [#907](https://github.com/wildcard/caro/pull/907),
   [#908](https://github.com/wildcard/caro/pull/908),
   [#909](https://github.com/wildcard/caro/pull/909),
   [#911](https://github.com/wildcard/caro/pull/911),
   [#912](https://github.com/wildcard/caro/pull/912),
   [#913](https://github.com/wildcard/caro/pull/913))
- **CaroML voice** — pager-era epilogue codes (143/371/607/42/111111) on
  success messages, opt-out via `CARO_NO_EGGS`. Documented at
  `docs/caroml/voice.md`.
- **CaroML examples library** at `examples/library/system/` plus a sample
  `Carofile`. Walk-through documentation under `docs/caroml/`.
- **`BsdFlavor` sub-classification** in `src/platform/mod.rs`: identifies the
  underlying BSD-family OS (`FreeBsd`, `OpenBsd`, `NetBsd`, `MacOs`,
  `DragonFlyBsd`, `Unknown`) independently of the userland `UtilityType`. A
  macOS host with Homebrew GNU coreutils now correctly reports
  `bsd_flavor() == Some(MacOs)` while `utility_type() == Gnu`. Surfaces
  through `to_prompt_string()` and adds flavor-specific notes in
  `platform_notes()` (pkg/jail/gpart for FreeBSD, pf/doas for OpenBSD,
  pkgsrc for NetBSD).
- **`PlatformContext::is_bsd_family()`** convenience getter on the public
  API.
- **`PlatformContextBuilder::bsd_flavor()`** for explicit builder
  configuration in tests and non-async contexts.
- **`docs/SAFETY_PHILOSOPHY.md`**: new doctrine document explaining the
  kernel-driver mindset behind caro's defense-in-depth layering, citing
  the FreeBSD Device Driver Book's Ch 29 (Portability), Ch 31 (Security
  Best Practices), and Ch 37 (Submitting to FreeBSD). Linked from
  `SECURITY.md` and `CONTRIBUTING.md`.

### Changed

- **`detect_os()`** now recognizes `freebsd`, `openbsd`, `netbsd`, and
  `dragonfly` build targets (previously fell through to `"unknown"` on
  those platforms despite the cross-platform CI matrix building for them).
- **`is_posix_compliant()`** extended to mark FreeBSD/OpenBSD/NetBSD/
  DragonFly as POSIX-compliant.
- **`EmbeddedModelBackend::with_safety_config`** now returns
  `Result<Self, GeneratorError>` instead of `Self`. **Breaking API
  change**: callers must propagate the error. Previously the method
  panicked via `.expect()` if the supplied `SafetyConfig` was malformed
  (e.g. `max_command_length == 0`); now it returns
  `GeneratorError::ConfigError`, allowing the CLI to surface a clear
  error or fall back to a different backend. The single in-tree caller
  in `src/cli/mod.rs:301` is updated. Aligns with FDD-book ch. 31
  (Security Best Practices) — never panic at a privilege boundary.
- **`EmbeddedModelBackend::new` / `with_variant_and_path`** internal
  initialization of the default `SafetyValidator` now propagates errors
  via `?` instead of panicking via `.expect()`. No public API change
  (the constructor already returned `Result`).

### Removed

- **`impl Default for EmbeddedModelBackend`**: the `default()` impl
  previously called `Self::new().expect(...)` and was unused
  (`git grep` finds no callers in the workspace). Removed rather than
  kept as a hidden panic surface.

### Fixed

### Security

- **BSD-family safety patterns — round 2**: Added 4 more `DangerPattern`
  entries surfaced by a post-fix `safety-pattern-auditor` pass on the
  initial 10. Pattern total grows 62 → 66.
  - `zpool destroy` / `zpool labelclear` (Critical) — closes the pool-level
    destruction gap left by the dataset-only `zfs destroy -r/-R/-f` pattern
  - `bectl destroy` (High) — FreeBSD Boot Environment removal forecloses
    rollback recovery
  - `gmirror destroy` / `gmirror clear` (Critical) — GEOM mirror destruction
    or metadata wipe
  - `bsdlabel -w` / `disklabel -w` (Critical) — destroys partition table on
    FreeBSD/OpenBSD; the `(?:\S+\s+)*?` shape allows interleaved flags
- **`bsdinstall` anchor robustness**: changed
  `(^|[;&|]\s*)(sudo\s+)?bsdinstall\b` to
  `(^\s*|[;&|]+\s*)(sudo\s+)?bsdinstall\b` to close two bypasses caught
  by the auditor — leading whitespace (`   bsdinstall`) and `&&`/`||`
  chaining (`cd /tmp && bsdinstall`). `man bsdinstall` and
  `which bsdinstall` remain unflagged.
- **BSD-family safety patterns**: Added 10 new `DangerPattern` entries covering
  destructive utilities specific to FreeBSD/OpenBSD/NetBSD/macOS that the
  original GNU/Linux-flavored set did not catch:
  - `gpart destroy/delete` — partition table destruction (Critical)
  - `zfs destroy -r/-R/-f` — recursive/forced ZFS dataset wipe (Critical)
  - `dd`/`mkfs.*`/`newfs`/`>` redirects targeting `/dev/da*`, `/dev/ada*`,
    `/dev/nvd*`, `/dev/md*` — BSD device naming (Critical)
  - `pkg delete -f` — forced package removal bypassing dependency checks (High)
  - `bsdinstall` invoked at start-of-statement — destructive outside install
    media; anchored so `man bsdinstall` and `which bsdinstall` are not flagged
    (High)
  - `chflags noschg` on `/etc`, `/bin`, `/sbin`, `/boot`, `/usr/bin`,
    `/usr/sbin` — immutability bypass / security regression (High)
  - `jail -r <name>` — running-jail removal (Moderate)
  - New TDD-driven contract tests in `tests/safety_validator_contract.rs`
    cover positive matches and false-positive prevention for read-only
    variants (`gpart show`, `zfs list`, `pkg info`, etc.). Pattern total
    grows from 52 → 62.


## [1.3.2] - 2026-05-09

### Added

- **Static matcher: date arithmetic** — added patterns for future date (`date -v+Nd`
  / `date -d '+N days'`) and past date (`date -v-Nd`) that correctly distinguish
  future from past queries; also added Unix timestamp → human-readable conversion
  (`date -r <epoch>` BSD / `date -d @<epoch>` GNU). Closes #955.
- **Static matcher: `nc -zv` port check** — added pattern for "check if port is open"
  queries that produces `nc -zv host 80`, which is more widely available than `nmap`
  and doesn't require elevated privileges. Closes #1003.
- **Static matcher: wget patterns** — added basic download (`wget <url>`), save-as
  (`wget -O file url`), and recursive download (`wget -r --no-parent url`). Closes #952.
- **Static matcher: `basename` / `dirname` / `head`** — added path-manipulation helpers
  and a first-N-lines pattern. Closes #983.
- **Static matcher: `zcat`** — added pattern for viewing compressed files without
  extracting. Closes #998 (partial).
- **Static matcher: `docker network ls` / `docker network inspect`** — added two
  Docker network subcommand patterns. Closes #999 (partial).

### Fixed

- **LLM prompt BSD date hint** — the system prompt previously showed only
  `date -v-7d` (past) to the embedded LLM, causing it to copy the minus sign even for
  future-date queries. Now both signs and the `date -r` timestamp form are shown. Closes #955.

## [1.3.1] - 2026-05-09

### Fixed

- **P0 safety regression**: `chmod -R 777 /` and `chmod -R <mode> /` variants now
  correctly match the dangerous-pattern list. The original `chmod 777 /` pattern
  did not account for the `-R` (recursive) flag, allowing world-writable
  root-directory commands to pass through `--safety strict`. Two patterns added:
  one that handles optional flag prefixes before the mode (`chmod (-flags)* 777 /`)
  and one that broadly catches any recursive chmod on the root
  (`chmod -R <mode> /`). Closes #1034.
- **P1 exit-code regression**: Unsafe commands detected by the static matcher
  now propagate as `Err` immediately instead of silently falling through to the
  LLM backend. Previously, `GeneratorError::Unsafe` was caught by the
  `BackendUnavailable` fall-through arm in the agent loop, causing the LLM to
  re-attempt a command that was already known dangerous, and the final exit code
  to be 0 even when an error was printed to stderr. Closes #1035.
- **P1 non-asserting safety test**: `test_example_safety_002_blocks_chmod_777` in
  `tests/website_claims.rs` printed `WARNING: not blocked` instead of failing the
  test. The `println!` has been replaced with `assert!`, turning the silent watch
  into an enforced regression guard. Closes #1037.
- **P2 CLAUDE.md version banner**: `CLAUDE.md` incorrectly displayed version
  `1.1.0 (GA)` since that file was not part of the release version checklist.
  Updated to `1.3.0`. Closes #1044.
## [1.3.0] - 2026-04-20

### Added

- **`caro ai` conversational command generation** (#861): Atuin-AI-style once-mode
  AI invocation that resumes or creates a session, runs the prompt through the
  configured backend, validates the result via the existing 52-pattern
  `SafetyValidator`, and persists the turn.
  - `caro ai --once "<prompt>"` for scripted use
  - `caro ai --continue-session` for shell-widget invocation
  - TTL-based session resume via `session_continue_minutes`
- **`caro shell-init <bash|zsh|fish>`** (#861): emits a shell integration script
  that binds `?` on an empty prompt to `caro ai`. Literal `?` is preserved when
  the prompt already has characters (globs etc.). Fish output is properly quoted
  to preserve multi-word commands.
- **`[ai]` config block** (#861): strict opt-in privacy gates — `opening.send_cwd`,
  `opening.send_last_command`, `capabilities.enable_history_search` all default
  to `false`. With defaults, only the OS + shell name leaves the process.
- **Off-host context warning** (#861): when a remote backend (ollama, vllm, exo,
  claude) is configured alongside any opt-in context toggle and an explicit
  endpoint, stderr surfaces a warning before the generation happens.
- **`CliApp::backend_arc()` accessor** (#861): exposes the constructed backend
  for reuse without re-instantiation.

### Security

- Every command produced by `caro ai` flows through the same 52-pattern
  `SafetyValidator` used for `caro <prompt>`; `rm -rf /` at `SafetyLevel::Moderate`
  is unconditionally blocked (covered by a new strict regression test in
  `src/ai/runner.rs`).
- Session history is only included in LLM context when
  `capabilities.enable_history_search` is true, closing the leak that could have
  bypassed the opening-context opt-in toggles.

### Internal

- New `src/ai/` module: `privacy`, `session`, `store`, `runner`, `shell_init`
  (23 unit tests covering privacy toggle combinations, session TTL, shell-init
  snapshots, and safety integration).
- Collapsed 3 pre-existing clippy `collapsible_match` warnings in
  `src/evaluation/models.rs` to unblock `-D warnings` on the lint CI job.

## [1.2.0] - 2026-03-26

### Added

- **Interactive Terminal Demo** (#130): Animated terminal landing page demo on caro.sh
  - Real-time typewriter animation showing caro converting natural language to shell commands
  - Showcases multiple example queries with syntax-highlighted output
  - Responsive design optimized for desktop and mobile

- **Homebrew Tap** (#573, #595): Install caro via `brew install wildcard/tap/caro`
  - Official Homebrew formula at `homebrew-tap/Formula/caro.rb`
  - Automated formula update workflow on each release via `.github/workflows/update-homebrew.yml`
  - SHA256 checksum verification for all release binaries

- **Setup Wizard** (#639): Interactive `caro init` command for first-time configuration
  - Guided setup for shell, safety level, log level, and model preferences
  - Auto-detects current shell and suggests sensible defaults
  - Runs automatically on first launch when no config exists
  - `--minimal` flag for non-interactive environments
  - `--force` flag to reconfigure existing installations

- **JSON Schema for Configuration** (#11): Auto-generated JSON schema for TOML configuration
  - Added `generate-schema` binary to generate `.vscode/caro-config.schema.json`
  - VS Code autocomplete and validation for `config.toml`
  - Schema includes all configuration options with descriptions
  - Example configuration file with schema reference

- **Enhanced Prompt Validation** (#462): User notifications for malformed queries
  - Warning messages for flags-only or operator-only prompts
  - Hint messages for very short or ambiguous prompts (verbose mode)
  - Helps users provide better queries before sending to backend
  - 8 new test cases for validation scenarios

- **Rustdoc Examples** (#7): Comprehensive documentation examples for public APIs
  - Cache module: CacheManager methods with async/await examples
  - Execution module: ExecutionContext and ShellDetector examples
  - Logging module: Logger initialization and redaction examples
  - Config module: ConfigManager methods with builder pattern examples
  - All examples compile successfully with `cargo test --doc`

### Changed

- **Improved Installation Docs** (#573): Streamlined README installation section
  - Homebrew as primary installation method for macOS
  - Cleaner per-OS sections (macOS, Linux & BSD, Windows)
  - Removed verbose manual download tables in favor of releases page link

### Fixed

- **Documentation Warnings**: Fixed all `cargo doc` warnings
  - Fixed unresolved link to feature-gated `knowledge` module
  - Fixed URL formatting in ChromaDB documentation

- **Version Header & Security Notes** (#639): `caro --version` now shows version prominently
  - Added security disclaimer on first run and `caro init`
  - Links to security documentation at caro.sh/docs/security

### Security

## [1.1.3] - 2026-01-15

### Fixed

- **Safety Flag Wiring** (#461): The `--safety` CLI flag now properly configures backend validators
  - Added `SafetyConfig::from_level()` to convert CLI SafetyLevel to backend SafetyConfig
  - Added `with_safety_config()` builder methods to `StaticMatcher` and `EmbeddedModelBackend`
  - Safety level priority: CLI flag → config file → default (moderate)
  - `--safety strict` blocks High and Critical risk commands
  - `--safety moderate` blocks Critical only (default)
  - `--safety permissive` warns but allows all

## [1.1.2] - 2026-01-15

### Added

- **Hugging Face Model Download** (#399): Full-featured model download with resumable downloads, checksum validation, and progress tracking
  - Streaming downloads with automatic retry on network failures
  - Resume interrupted downloads from where they left off
  - SHA256 checksum validation ensures file integrity
  - File locking for concurrent download safety
  - Comprehensive integration tests with mock HTTP server

- **Benchmark Suite** (#397): Performance validation and regression testing infrastructure
  - Comprehensive benchmark suite for command generation performance
  - `benchmark-compare.py` script for comparing results across runs
  - Integration with CI for automated performance tracking
  - Baseline measurements for all major operations

- **Backend Preference Configuration** (#455): Environment variable support for backend selection
  - New `CARO_BACKEND` environment variable to specify preferred backend
  - Priority order: CLI flag > env var > config file > auto-detect
  - Supports: `embedded`, `mlx`, `ollama`, `exo`, `vllm`
  - Example: `CARO_BACKEND=mlx caro "list files"`

- **CLI Startup Optimization** (#454): CapabilityProfile caching for 99.9% faster startup
  - Cache detected capabilities to `~/.cache/caro/capabilities-{version}.json`
  - First run: ~650ms detection, subsequent runs: <1ms cache hit
  - Cache automatically invalidates on version change
  - Benchmark shows 650x improvement in startup time

- **Property-Based Tests for LRU Cache** (#398): Comprehensive test coverage
  - QuickCheck property tests for cache eviction behavior
  - Tests verify: FIFO order, capacity limits, update semantics
  - Increases confidence in cache correctness under edge cases

### Fixed

- **Install Script Improvements**: Better handling of existing installations
  - Detect and warn about multiple caro binaries in PATH
  - Add `--force` flag to setup.sh for reinstallation
  - Fix SAFETY_LEVEL unbound variable in non-interactive mode
  - Show git hash when installed from crates.io (reads .cargo_vcs_info.json)

- **Issue #275**: Unquoted CLI argument handling verified working
  - All 45 subtasks from spec 002-unquoted-cli-arguments complete
  - Tests pass: `e2e_unquoted_prompt_basic`, `e2e_multi_word_unquoted_prompt`

## [1.1.1] - 2026-01-14

### Added

- **Edit Option in Confirmation Prompt**: New `(e)dit` option alongside Yes/No when confirming commands
  - Selecting Edit places the generated command directly into your shell's readline buffer for editing
  - Works like atuin/zoxide - command appears as if you typed it, ready to modify and execute
  - Requires shell integration: `eval "$(caro init <shell>)"` (zsh, bash, fish supported)
  - Fallback: Without shell integration, copies command to clipboard

- **Shell Integration Command**: `caro init <shell>` generates shell wrapper functions
  - Supports zsh, bash, and fish shells
  - Enables the Edit feature by properly routing stdout/stderr
  - Uses exit code 201 to signal edit mode to the wrapper

### Changed

- **Confirmation Prompt**: Changed from `[y/N]` to `(Y)es / (n)o / (e)dit` using `dialoguer::Select`
- **Output Routing**: Display output now goes to stderr when running through shell wrapper (`CARO_WRAPPER=1`), keeping stdout clean for edit mode

## [1.1.0] - 2026-01-12

### 🎉 General Availability Release

Caro v1.1.0 is now generally available! This release represents the culmination of extensive beta testing with 93.1% command generation accuracy and zero false positives in safety validation.

### Changed

- **Telemetry Default**: Now opt-in (disabled by default) - your privacy, your choice
- **Consent Message**: Updated to reflect GA status

### Highlights from Beta

- 93.1% pass rate on comprehensive test suite (exceeds 86% target)
- 52 safety patterns with 0% false positive rate
- Platform-aware command generation (BSD vs GNU)
- System assessment and health check commands (`caro assess`, `caro doctor`)
- Privacy-first telemetry with transparent consent

### Fixed

- **Issue #411**: Platform-specific command syntax incompatibility (P2 Blocker)
  - Root cause: Platform profile hardcoded to Ubuntu instead of detecting actual platform
  - Fix:
    - Updated `AgentLoop::new()` to accept detected `CapabilityProfile` parameter
    - Added `CapabilityProfile::detect().await` in `CliApp::with_config()` and `run_evaluation_tests`
    - Enhanced `select_command()` to check `profile_type` and return BSD commands on BSD platforms
  - Impact: Commands now use correct syntax for each platform:
    - BSD syntax on macOS/FreeBSD (`du -h -d 1`)
    - GNU syntax on Linux (`du -h --max-depth=1`)
  - Testing: Added 5 platform-specific tests; File Management pass rate: 80% → 100%
  - Verification: Runtime tested on macOS, all 153 unit tests passing

For detailed beta changelog, see v1.1.0-beta.1 and v1.1.0-beta.2 entries below.

## [1.1.0-beta.2] - 2026-01-09

### 🔥 Critical Fixes (P0 Issues from Beta.1 Testing)

This release fixes **5 critical P0 issues** identified during v1.1.0-beta.1 comprehensive beta testing that were blocking GA release.

#### Fixed

- **Issue #402**: Telemetry consent prompt appearing on every command invocation
  - Root cause: Consent result was never persisted to config file
  - Fix: Added config persistence after consent prompt
  - Impact: Eliminates 28-line prompt spam and 2-second overhead on every command

- **Issue #403**: Telemetry cannot be disabled despite config setting
  - Root cause: Same as #402 - first_run flag never updated
  - Fix: Properly updates `telemetry.enabled` and `telemetry.first_run` in config
  - Verification: `caro config set telemetry.enabled false` now persists correctly

- **Issue #404**: `--output json` produces invalid JSON (telemetry prompt pollutes stdout)
  - Root cause: Interactive consent prompt writes to stdout, breaking JSON format
  - Fix: Skip interactive consent for non-human output formats (json, yaml)
  - Verification: `caro --output json "list files" | jq '.'` now works correctly

- **Issue #405**: Documentation mismatch - commands claimed missing but actually work
  - Root cause: Beta testing instructions incorrectly stated `caro assess` and `caro telemetry` don't exist
  - Fix: Updated `.claude/releases/BETA-TESTING-INSTRUCTIONS.md` with correct information
  - Impact: Eliminates tester confusion about available commands

- **Issue #406**: Command generation quality below target (40% vs 95% for File Management)
  - Root cause: Missing static patterns for common queries
  - Fixes:
    - Added pattern for "show disk space by directory" (simple variant without "sorted" requirement)
    - Updated python files pattern to handle "from" in addition to "modified"
    - Verified "list hidden files" pattern works correctly
  - Impact: File Management pass rate increased from 40% (2/5) to 100% (5/5)

#### Testing

- Added `tests/beta_regression.rs` with 5 regression tests to prevent future breakage
- All tests passing: 5/5 regression tests + 148 library tests
- Verified fixes manually per beta testing protocol

#### Documentation

- Updated beta testing instructions to reflect actual command availability
- Added detailed troubleshooting guidance for telemetry configuration

### 📊 Quality Metrics

- **Command Quality**: File Management category improved from 40% to 100%
- **Telemetry UX**: Eliminated consent prompt spam (appears once, persists correctly)
- **JSON Output**: Now 100% spec-compliant (no pollution from interactive prompts)
- **Test Coverage**: +5 regression tests covering all P0 fixes

## [1.1.0-beta.1] - 2026-01-08

### 🎯 Release Highlights

This is a **major quality and capability release** that dramatically improves command generation accuracy, safety validation, and system assessment. Beta testing shows **93.1% pass rate** (up from 30% baseline), exceeding our 86% target.

### ✨ Added

#### System Assessment & Recommendations
- **System resource assessment** (`caro assess`) - Analyzes CPU, GPU, memory, and provides model recommendations
  - Apple Silicon GPU detection with Metal API support
  - NVIDIA GPU detection with CUDA capability assessment
  - CPU architecture and core count detection
  - Memory capacity analysis with smart recommendation thresholds
  - Recommends optimal models based on available resources
  - Multiple output formats: human-readable, JSON, markdown
- **Health check system** (`caro doctor`) - Comprehensive diagnostics for troubleshooting
  - Validates model availability and accessibility
  - Checks system requirements and dependencies
  - Provides actionable troubleshooting steps

#### Command Generation Quality
- **Static matcher expansion** - 50+ high-confidence command patterns (up from 4)
  - File management operations (find by date, size, type)
  - System monitoring (processes, disk usage, network)
  - Git operations (status, log, diff)
  - DevOps commands (kubectl, docker basics)
  - Text processing (grep, search patterns)
  - Log analysis patterns
- **Chain-of-thought prompting** - Models now reason through command generation step-by-step
- **Negative examples in prompts** - Teaches models what NOT to generate
- **Platform-specific prompt optimization** - Tailored examples for macOS/Linux/BSD
- **Expanded few-shot examples** - 15-20 examples per command category (up from 4-8)

#### Safety & Validation
- **52 dangerous command patterns** with 0% false positive rate
  - Recursive deletion detection
  - Privilege escalation warnings
  - Data destruction prevention
  - System-wide operation blocking
- **Validation-triggered retry** - Auto-repairs invalid commands
- **Confidence-based refinement** - Re-generates low-confidence outputs
- **Safety level configuration** - Strict, balanced, permissive modes

#### Testing & Quality Assurance
- **Beta test suite** - 75 YAML-driven test cases across 8 categories
  - file_management: 19 tests (94.7% pass rate)
  - system_monitoring: 7 tests (100% pass rate)
  - git_version_control: 3 tests (100% pass rate)
  - log_analysis: 4 tests (100% pass rate)
  - network_operations: 5 tests (100% pass rate)
  - devops_kubernetes: 5 tests (100% pass rate)
  - text_processing: 7 tests (100% pass rate)
  - dangerous_commands: 8 tests (safety validation)
- **10 beta tester profiles** - Simulates diverse user personas (novice to expert)
- **Regression test suite** - Prevents re-introduction of fixed bugs
- **Assessment integration tests** - Validates system detection accuracy
- **Contract tests** - Ensures safety validator behavior consistency

#### Telemetry & Privacy
- **Privacy-focused telemetry** - Anonymous usage data to improve quality
  - Session timing and performance metrics
  - Platform info (OS, shell type)
  - Error categories and safety events
  - **NEVER collects**: commands, prompts, file paths, personal data
- **Transparent consent** - Clear notice on first run with easy opt-out
- **Local storage** - All data stored locally until explicit upload
- **Redaction system** - Automatically strips sensitive data (IPs, paths, credentials)

### 🔄 Changed

#### Agent & Backend Improvements
- **Temperature tuning** - Reduced from 0.7 to 0.1 for more deterministic outputs
- **Prompt unification** - Consistent prompting across embedded and cloud backends
- **Agent loop enhancement** - Better error recovery and retry logic
- **Backend configuration** - More flexible model selection and parameters

#### Performance & Reliability
- **Command generation latency** - < 1 second for most queries
- **Safety validation** - Instant pattern matching
- **Binary startup time** - < 100ms
- **Test execution** - Full suite runs in ~140 seconds
- **Build time** - Release build in ~48 seconds

#### User Experience
- **Telemetry notice** - Clear, informative first-run experience
- **Help output** - Improved clarity with subcommand descriptions
- **Error messages** - More actionable with specific guidance
- **Version display** - Shows build info and commit hash

### 🐛 Fixed

- **Issue #161** - Unquoted CLI argument parsing (7 regression tests added)
- **Platform detection** - Correct BSD vs GNU command generation
- **JSON parsing** - Handles malformed LLM responses gracefully
- **Memory leaks** - Fixed in assessment module initialization
- **Temperature configuration** - Consistent across all backends

### 📊 Quality Metrics

**QA Validation Results** (2026-01-08):
- ✅ **93.1% pass rate** on 58 comprehensive beta test cases (exceeds 86% target)
- ✅ **100% pass rate** on all 7 safe command categories
- ✅ **0% false positive rate** in safety validation
- ✅ **0 P0/P1 bugs** discovered in testing
- ✅ **146/146 library tests passing**
- ✅ **58/58 website claims validated**
- ✅ **7/7 assessment tests passing**
- ✅ **7/7 regression tests passing**

**Performance Benchmarks**:
- Static matcher: < 50ms
- Embedded backend: < 1000ms
- Agent loop: < 2000ms
- Binary startup: < 100ms

### 🔐 Security

- **Enhanced safety patterns** - 52 dangerous command patterns (up from ~20)
- **Zero false positives** - Safe commands never blocked incorrectly
- **Validation hardening** - Catches edge cases and obfuscation attempts
- **Privacy-first telemetry** - No sensitive data collection, local-first storage

### 📚 Documentation

- **164 release planning documents** - Comprehensive guides for:
  - Beta testing strategy and execution
  - Security audit and vulnerability management
  - Performance benchmarking methodology
  - Deployment and distribution
  - User documentation system
  - Contributor onboarding
  - Testing strategy and QA processes

### 🙏 Contributors

This release includes improvements from 12 beta testing cycles with contributions from:
- Static pattern analysis and expansion
- Prompt engineering and optimization
- Safety validation enhancement
- Test infrastructure development
- Documentation and planning

### 🚀 Migration Notes

**Breaking Changes**: None - 100% backward compatible with v1.0.x

**New Features to Try**:
```bash
# System assessment
caro assess

# Health diagnostics
caro doctor

# Beta test suite
caro test

# Check telemetry settings
caro telemetry status
```

**Recommended Actions**:
1. Review telemetry settings: `caro telemetry status`
2. Run system assessment: `caro assess`
3. Verify installation: `caro doctor`

### 📝 Notes

- This is a **beta release** ready for daily use by early adopters
- Telemetry is **opt-in by default** with clear disclosure
- MLX backend (Apple Silicon GPU) requires `cmake` to build
- All safe command categories achieve 100% pass rate
- Dangerous commands correctly blocked with 0% false positives

For detailed QA validation results, see:
- `.claude/beta-testing/cycles/v1.1.0-qa-validation.md`
- `.claude/beta-testing/v1.1.0-test-evidence.md`

## [1.0.3] - 2025-12-31

### Added
- **Version information display**: Comprehensive version output with build metadata
  - Basic version: `caro --version` shows `caro 1.0.2 (abc1234 2025-01-15)` (scriptable, single-line)
  - Verbose version: `caro --version --verbose` shows detailed build information with Caro's personality
  - Build type detection: Distinguishes between dev builds, source installs, and official releases
  - Compile-time metadata capture: Git commit hash, build date, rustc version, target platform
- **Unquoted CLI prompts**: Natural language prompts without quotes (e.g., `caro list files`)
  - Maintains 100% backward compatibility with quoted prompts (e.g., `caro "list files"`)
  - Supports multi-word prompts: `caro find large files in current directory`
  - Shell operators detected and handled correctly: `>`, `|`, `<`, `>>`, `2>`, `&`, `;`
- **-p/--prompt flag**: Explicit prompt specification for non-interactive mode
  - Example: `caro -p "list files"`
  - Highest priority in prompt resolution
- **stdin input support**: Pipe prompts from other commands
  - Example: `echo "list files" | caro`
  - Medium priority in prompt resolution (after -p flag)
- **Help display for empty input**: Shows usage examples instead of error
  - `caro` (no args) displays helpful usage information with exit code 0
  - Whitespace-only input also shows help

### Changed
- **Argument parsing**: Accepts trailing unquoted words as prompt
  - Uses clap's `trailing_var_arg` feature for flexible argument handling
  - Flags must appear before trailing arguments (e.g., `--verbose list files`)
- **Input prioritization**: Flag > stdin > trailing arguments
  - -p/--prompt flag takes highest priority
  - Piped stdin takes medium priority
  - Trailing arguments take lowest priority
- **Validation behavior**: Empty/whitespace prompts show help instead of error

### Technical Details
- **Architecture**: Library-First design with pure functions
  - `resolve_prompt()`: Priority-based prompt resolution
  - `validate_prompt()`: Empty/whitespace validation
  - `truncate_at_shell_operator()`: POSIX operator detection
- **Performance**: Argument parsing overhead < 10ms
- **Testing**: 193 tests passing (12 unit tests for new features, 31 E2E tests)

### Success Criteria Validated
- ✅ SC-001: 100% accuracy for 2-5 word prompts
- ✅ SC-002: Backward compatibility maintained
- ✅ SC-003: Cross-platform tests passing
- ✅ SC-004: Help display for empty input
- ✅ SC-005: Non-interactive mode with -p flag
- ✅ SC-006: Stdin processing works
- ✅ SC-007: Shell operator detection 100% accurate

## [1.0.2] - 2025-12-28

### Fixed

#### Cross-Platform Binary Distribution
- **OpenSSL dependency removed**: Switched `hf-hub` and `tokenizers` from `native-tls` to `rustls-tls`
  - Eliminates system OpenSSL dependency for cross-compilation
  - Enables successful ARM64 Linux builds without OpenSSL headers
  - Pure Rust TLS stack works across all platforms without system dependencies
  - Fixes failed v1.0.1 release where no binaries were attached to GitHub release

#### CI/CD Improvements
- **Release workflow resilience**: Added `fail-fast: false` to build matrix
  - Platform builds now run independently
  - One platform failure doesn't cancel other builds
  - Ensures maximum binary availability even if individual platforms fail

### Technical Details
- **Dependency changes**:
  - `hf-hub`: `default-features = false, features = ["tokio", "rustls-tls"]`
  - `tokenizers`: `default-features = false, features = ["http", "rustls-tls", "onig"]`
- **Platform compatibility**: Binaries work on Ubuntu, Debian, Fedora, Arch, Alpine, WSL without OpenSSL
- **Binary size**: No impact, rustls is similar size to native-tls when statically linked

## [1.0.1] - 2025-12-25

### Changed

#### Dependencies
- **Major Updates**:
  - `thiserror`: 1.0.69 → 2.0.16 - Updated error handling macros
  - `sysinfo`: 0.29.11 → 0.37.2 - System information library API updates
  - `which`: 4.4.2 → 8.0.0 - Executable path detection with new Sys trait
  - `directories`: 5.0.1 → 6.0.0 - Platform directory utilities
  - `criterion`: 0.5.1 → 0.8.1 - Benchmarking framework updates
  - `dialoguer`: 0.11.0 → 0.12.0 - Interactive prompt improvements

- **Minor/Patch Updates** (rust-minor-patch group):
  - Updated 12 dependencies including: `clap`, `tokio`, `serde`, `regex`, and other core libraries
  - All updates maintain API compatibility

- **GitHub Actions Updates**:
  - Updated 10 GitHub Actions to latest versions for improved CI/CD reliability
  - Includes: `actions/checkout@v6`, `dtolnay/rust-toolchain@v1`, and other workflow actions

### Fixed
- Replace deprecated `criterion::black_box()` with `std::hint::black_box()` in benchmarks
  - Resolves clippy warnings after criterion 0.8.1 upgrade
  - Maintains benchmark functionality with standard library function

## [1.0.0] - 2025-12-24

### Changed - Project Rename

**BREAKING CHANGE**: Project renamed from `caro` to `caro`
- Binary name: `caro` → `caro`
- Crate name: `caro` → `caro`
- Package name on crates.io: `caro`
- All imports updated: `use caro::*` → `use caro::*`
- Repository and documentation updated throughout

**Migration Guide**:
```bash
# Uninstall old version
cargo uninstall caro

# Install new version
cargo install caro

# Remove any shell aliases pointing to caro
# Check ~/.zshrc, ~/.bashrc for: alias caro='caro'
```

### Added - Feature 004: Embedded Model + Remote Backend Support

#### Embedded Model Backend (`src/backends/embedded/`)
- **EmbeddedModelBackend**: Primary inference backend with platform-specific optimizations
  - MLX backend for Apple Silicon (M1/M2/M3) with GPU acceleration
  - CPU backend using Candle framework for cross-platform support
  - Lazy model loading with <2s initialization time
  - Qwen2.5-Coder-1.5B-Instruct model with Q4_K_M quantization (~1.1GB)
  - JSON response parsing with multiple fallback strategies
  - Simulated inference for testing (~500ms MLX, ~800ms CPU)

#### Remote Backends (`src/backends/remote/`)
- **OllamaBackend**: Local Ollama server integration
  - HTTP API client with configurable timeout
  - Automatic fallback to embedded backend on failure
  - JSON request/response handling with robust parsing
  - Model selection and temperature control
- **VllmBackend**: OpenAI-compatible vLLM server support
  - Bearer token authentication for API access
  - Chat completion endpoint integration
  - Embedded backend fallback on connection failure
  - Configurable model and inference parameters

#### CLI Integration (`src/cli/`)
- **CliApp**: Enhanced with backend selection and user interaction
  - Configuration-driven backend selection
  - Interactive confirmation for dangerous commands
  - Non-terminal environment detection with graceful fallback
  - Multiple output formats (JSON, YAML, Plain text)
  - Verbose mode with timing and debug information
- **Backend Integration**: Automatic backend selection
  - Debug builds use mock backend for testing
  - Release builds use embedded backend with remote fallbacks
  - Availability checking with automatic fallback chain

#### Configuration System (`src/config/`)
- **Enhanced ConfigManager**: Backend configuration support
  - User preferences for primary backend selection
  - Remote backend URL and authentication settings
  - Safety level configuration (strict, moderate, permissive)
  - TOML-based persistence with validation

#### Safety System Integration
- **Risk Assessment**: Command safety validation
  - Critical commands blocked with explanatory messages
  - Moderate/high risk commands require confirmation
  - Permissive mode for advanced users
  - Custom dangerous pattern definitions

#### User Interaction
- **Interactive Confirmations**: Safe command execution
  - Color-coded risk indicators (green/yellow/red)
  - Terminal detection for interactive prompts
  - `--confirm/-y` flag for automation
  - Helpful guidance in non-interactive environments

### Performance
- Embedded model initialization: <2s (target met) ✅
- Command generation: <1s typical (500-800ms) ✅
- Remote backend fallback: <5s timeout ✅
- CLI startup: <100ms (debug), <50ms (release) ✅

### Testing
- 44 library unit tests passing
- 9 system integration tests passing
- 9 embedded backend integration tests passing
- Remote backend fallback scenarios validated
- Safety validation comprehensive test coverage
- Multi-platform CI/CD pipeline configured

### Build & Distribution
- **Multi-platform builds**: Linux, macOS, Windows
- **Architecture support**: x86_64, aarch64
- **Feature flags**: 
  - `embedded-cpu`: CPU backend (default)
  - `embedded-mlx`: Apple Silicon MLX backend
  - `remote-backends`: Ollama/vLLM support
- **GitHub Actions CI**: Quality checks, testing, and release automation

### Dependencies Added
- `mlx-rs = "0.25"` - Apple Silicon MLX bindings (optional)
- `candle-core = "0.9"` - Neural network inference (optional)
- `candle-transformers = "0.9"` - Transformer models (optional)
- `tokenizers = "0.15"` - Fast tokenization
- `reqwest = "0.11"` - HTTP client for remote backends (optional)
- `async-trait = "0.1"` - Async trait support
- `serde_yaml = "0.9"` - YAML output format
- `atty = "0.2"` - Terminal detection
- `dialoguer = "0.11"` - Interactive confirmations

### Added - Feature 003: Core Infrastructure Modules

#### Cache Module (`src/cache/`)
- **CacheManager**: Model caching with Hugging Face integration
  - LRU eviction algorithm for cache size management
  - SHA256 checksum validation for model integrity
  - Offline-first operation with manifest persistence
  - XDG Base Directory compliance for cross-platform support
- **ManifestManager**: JSON-based cache metadata management
  - Automatic manifest creation and persistence
  - Cache statistics tracking (total size, model count)
  - Integrity validation and corruption detection

#### Config Module (`src/config/`)
- **ConfigManager**: TOML-based configuration management
  - Load/save user preferences with validation
  - CLI argument override support (`merge_with_cli`)
  - Environment variable override support (`merge_with_env`)
  - Schema validation with deprecated key warnings
- **ConfigSchema**: Configuration validation logic
  - Known keys/sections tracking
  - Deprecated key migration support

#### Execution Module (`src/execution/`)
- **ExecutionContext**: System context capture for LLM prompts
  - Current directory, shell type, platform detection
  - Environment variable capture with sensitive data filtering
  - Username/hostname detection (cross-platform)
  - Serialization for LLM prompt integration
- **ShellDetector**: Shell and platform detection utilities
  - Auto-detection from environment ($SHELL)
  - Fallback to POSIX sh for unknown shells
  - Platform-specific detection (Linux, macOS, Windows)

#### Logging Module (`src/logging/`)
- **Logger**: Structured logging with tracing integration
  - JSON and plain text format support
  - Log level configuration (Debug, Info, Warn, Error)
  - File and stdout output options
  - Operation span tracking for performance monitoring
- **Redaction**: Sensitive data filtering
  - Pattern-based redaction of API_KEY, TOKEN, PASSWORD, SECRET
  - Regex-based sensitive data detection

#### Infrastructure Models (`src/models/mod.rs`)
- Added infrastructure-specific types:
  - `Platform`: Operating system detection (Linux/macOS/Windows)
  - `SafetyLevel`: Command safety configuration (Strict/Moderate/Permissive)
  - `LogLevel`: Logging severity levels
  - `UserConfiguration`: User preferences with builder pattern
  - `ExecutionContext`: Complete execution environment model
  - `ConfigSchema`: Configuration schema validation
  - `CacheManifest`: Cache metadata structure

### Performance
- Context capture: <50ms (NFR-003) ✅
- Config loading: <100ms (NFR-002) ✅
- Cache operations: <5s for <1GB models (NFR-001) ✅
- Logging: Non-blocking with async I/O (NFR-004) ✅

### Testing
- 40 passing integration tests across all modules
- Comprehensive contract tests for each infrastructure component
- Cross-module integration scenarios validated
- Performance requirements verified in automated tests

### Dependencies Added
- `directories = "5"` - XDG directory resolution
- `dirs = "5"` - Platform-specific directories
- `toml = "0.8"` - TOML parsing for configuration
- `tracing = "0.1"` - Structured logging framework
- `tracing-subscriber = "0.3"` - Tracing subscriber implementation
- `tracing-appender = "0.2"` - Log file rotation support
- `sha2 = "0.10"` - SHA256 checksums for integrity validation

### Security

This is the first stable release of caro with comprehensive security controls:

**Release Security**:
- Controlled release process with verified maintainers only
- GPG-signed tags required for all releases
- Automated CI/CD security checks (cargo audit, clippy)
- crates.io publish tokens with minimal scope (publish-update only)
- Multi-step verification before publication

**Command Safety**:
- Comprehensive dangerous command pattern detection
- Risk level assessment (Safe, Moderate, High, Critical)
- Interactive confirmation for potentially dangerous operations
- Blocked commands with clear explanatory messages
- POSIX compliance validation

**Dependency Security**:
- All dependencies vetted for security vulnerabilities
- Minimal dependency tree to reduce attack surface
- Regular security audits via `cargo audit`
- Pinned versions for reproducible builds

**Development Security**:
- 2FA required for all maintainer accounts
- Signed commits for release-related changes
- Branch protection on main branch
- Required code reviews for all changes
- Automated security scanning in CI/CD

See `docs/RELEASE_PROCESS.md` for complete security procedures.

### Notes

This release marks the transition from `caro` to `caro` and establishes the foundation for a security-critical CLI tool. We follow BSD/GNU-level security practices to ensure user trust.

**First Release Highlights**:
- ✅ Single binary under 50MB (without embedded model)
- ✅ Startup time < 100ms
- ✅ First inference < 2s on Apple Silicon
- ✅ Comprehensive safety validation
- ✅ Multi-backend support (MLX, Ollama, vLLM)
- ✅ Cross-platform support (Linux, macOS, Windows)
- ✅ Security-first development process

**Known Limitations**:
- ARM64 Linux binary builds may fail due to OpenSSL cross-compilation issues (users can compile from source)
- Embedded models require manual download and caching
- MLX backend requires Apple Silicon hardware

**Upgrade Path**:
If you previously installed `caro`, please uninstall it and install `caro`:
```bash
cargo uninstall caro
cargo install caro
```

**Breaking Changes**:
This is the first stable release. All previous versions were development previews and are not supported.
