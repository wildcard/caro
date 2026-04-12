# Caro Testing Strategy

**Version**: 1.0
**Last updated**: 2026-04-12

## Purpose

This document describes a progressive, phased approach to caro's test
suite. It is inspired by the 7-phase validation strategy used by the
[OpenEndpointSecurity](https://github.com/5BSD/OpenEndpointSecurity)
project, where each phase gates the next: Build -> Device -> NOTIFY ->
AUTH -> Restrictions -> Stress -> Security.

The motivation is simple: **not all tests are equally important, and
not all failures should block merges the same way**. Phased testing
gives fast feedback for common failure modes while reserving expensive
and flaky tests for later stages.

## Guiding Principles

1. **Fast first**. The phases that run most frequently must be the
   fastest and most reliable.
2. **Cheap before expensive**. Compile errors and lint failures should
   block in seconds, not after a 10-minute integration run.
3. **Deterministic before stochastic**. Unit tests run before property
   tests; property tests run before benchmarks.
4. **Local before network**. Tests that require downloads, APIs, or
   external services run last.
5. **Gates, not suggestions**. Each phase must pass before the next
   starts. A Phase 2 failure means Phase 3 does not run.

## The Seven Phases

### Phase 1 -- Compile + Lint

**Purpose**: Catch syntax errors, missing imports, lint violations, and
formatting issues. The fastest possible signal that something is wrong.

**Commands**:

```bash
cargo fmt -- --check
cargo check --all-features
cargo clippy --all-features -- -D warnings
```

**Runs**: On every local save (via IDE), every commit (via pre-commit
hook), every push (via CI), every PR open/update.

**Blocks**: All subsequent phases.

**Typical duration**: 5-30 seconds for `cargo check` after warm cache.

---

### Phase 2 -- Unit Tests

**Purpose**: Validate individual functions and types in isolation.
Fast, deterministic, no I/O.

**Commands**:

```bash
cargo test --lib
```

**What's included**:

- `src/safety/patterns.rs::tests` -- pattern compilation, filtering
- `src/safety/expansion.rs::tests` -- shell expansion detection (20+ tests)
- `src/safety/cache.rs::tests` -- LRU eviction, TTL expiry
- `src/platform/mod.rs::tests` -- OS/shell detection
- `src/models/mod.rs::tests` -- data type invariants
- `src/backends/*::tests` -- backend trait implementations (mocked)
- `src/telemetry/*::tests` -- redaction and event serialization

**Runs**: On every push, every PR.

**Blocks**: Phase 3 onward.

**Typical duration**: Under 10 seconds.

---

### Phase 3 -- Safety Contract Tests

**Purpose**: Validate the *security-critical* contract between the
safety validator and its callers. This is caro's equivalent of OES's
AUTH-mode testing.

**Commands**:

```bash
cargo test --test safety_validator_contract
cargo test --test property_tests
```

**What's included**:

- `tests/safety_validator_contract.rs` -- 26+ contract tests covering:
  - Dangerous command detection across all risk levels
  - Safe command allowance
  - Borderline command handling
  - Custom pattern addition
  - Allowlist behavior
  - **OES-inspired**: Recursion detection, expansion detection, timeout
    fail-closed, cache consistency
- `tests/property_tests.rs` -- proptest-based randomized validation

**Runs**: On every push, every PR. **This phase is mandatory** -- a
failure here indicates a safety regression.

**Blocks**: Phase 4 onward.

**Typical duration**: Under 30 seconds.

---

### Phase 4 -- Integration Tests

**Purpose**: Validate that components work together: backend + safety +
CLI dispatch + telemetry.

**Commands**:

```bash
cargo test --test integration_tests
cargo test --test cli_interface_contract
cargo test --test backend_trait_contract
cargo test --test embedded_backend_contract
cargo test --test logging_contract
```

**What's included**:

- Full generate -> validate -> display pipeline with mocked backends
- CLI flag parsing and subcommand dispatch
- Backend trait implementations called from CLI
- Telemetry event emission across the pipeline

**Runs**: On every PR after Phases 1-3 pass.

**Blocks**: Phase 5 onward.

**Typical duration**: 1-5 minutes.

---

### Phase 5 -- Platform Compatibility

**Purpose**: Catch platform-specific issues before they reach users.

**Commands**:

```bash
cargo test --test platform_detection_contract
cargo test --test execution_contract
cargo test --test execution_prompt_behavior
```

**What's included**:

- Shell detection across bash/zsh/fish/PowerShell/cmd
- BSD vs GNU flag handling
- Platform-specific file paths and line endings
- Shell-specific command execution

**Runs**: On every merge to main via matrix build
(Linux + macOS + Windows).

**Blocks**: Phase 6 onward.

**Typical duration**: 2-10 minutes per platform.

---

### Phase 6 -- Stress & Performance

**Purpose**: Detect performance regressions and validate behavior under
load.

**Commands**:

```bash
cargo bench
# Optional stress harness:
cargo test --test system_integration --release
```

**What's included**:

- `benches/` -- criterion-based microbenchmarks
- Safety validator cache hit/miss latency
- Concurrent validation (N parallel requests)
- Memory footprint under sustained load

**Runs**: Nightly on main, on release candidate PRs.

**Blocks**: Release (not individual PRs).

**Typical duration**: 5-30 minutes.

---

### Phase 7 -- Security Regression

**Purpose**: The final safety net. This phase contains every known
attack pattern, bypass attempt, and edge case discovered over the life
of the project. It must never regress.

**Commands**:

```bash
cargo test --test beta_regression
cargo test --test security_regression  # (future)
```

**What's included**:

- Every previously-discovered bypass attempt
- Encoding tricks (hex escapes, Unicode homoglyphs)
- Quoting edge cases
- Expansion nesting
- Platform-specific dangerous commands
- Cross-references each test to a specific threat in
  [`THREAT_MODEL.md`](THREAT_MODEL.md)

**Runs**: On every PR (fast subset) and before every release (full).

**Blocks**: Release.

**Growth rule**: Every security bug fix **must** add a regression test
to this phase before the fix is merged.

---

## CI Gating Summary

| Phase | On PR open | On PR push | On merge | Nightly | Pre-release |
|---|---|---|---|---|---|
| 1. Compile + Lint | ✅ | ✅ | ✅ | ✅ | ✅ |
| 2. Unit Tests | ✅ | ✅ | ✅ | ✅ | ✅ |
| 3. Safety Contract | ✅ | ✅ | ✅ | ✅ | ✅ |
| 4. Integration | ✅ | ✅ | ✅ | ✅ | ✅ |
| 5. Platform Compat | -- | -- | ✅ | ✅ | ✅ |
| 6. Stress + Perf | -- | -- | -- | ✅ | ✅ |
| 7. Security Regression | Fast subset | Fast subset | Full | Full | Full |

## What Phase Am I In?

Rule of thumb when writing a new test:

- **Tests a pure function?** Phase 2 (inline `#[cfg(test)] mod tests`).
- **Tests safety validation behavior?** Phase 3 (`tests/safety_validator_contract.rs`).
- **Tests full CLI flow?** Phase 4 (`tests/integration_tests.rs` or similar).
- **Tests platform-specific behavior?** Phase 5 (`tests/platform_detection_contract.rs`).
- **Measures performance?** Phase 6 (`benches/`).
- **Reproduces a security bug?** Phase 7 (`tests/security_regression.rs`).

## See Also

- [`THREAT_MODEL.md`](THREAT_MODEL.md) -- What Phase 7 protects against
- [`SECURITY.md`](../SECURITY.md) -- Vulnerability disclosure
- [OES TESTING.md](https://github.com/5BSD/OpenEndpointSecurity) -- Source of the phased approach
