---
work_package_id: "WP01"
subtasks:
  - "T001"
  - "T002"
  - "T003"
  - "T004"
  - "T005"
title: "Setup & Dependencies"
phase: "Phase 0 - Foundation"
lane: "doing"
assignee: ""
agent: "claude"
shell_pid: "90455"
review_status: ""
reviewed_by: ""
history:
  - timestamp: "2026-01-08T00:00:00Z"
    lane: "planned"
    agent: "system"
    shell_pid: ""
    action: "Prompt generated via /spec-kitty.tasks"
  - timestamp: "2026-01-08T05:24:00Z"
    lane: "doing"
    agent: "claude"
    shell_pid: "90455"
    action: "Started WP01 implementation"
---

# Work Package Prompt: WP01 – Setup & Dependencies

## ⚠️ IMPORTANT: Review Feedback Status

**Read this first if you are implementing this task!**

- **Has review feedback?**: Check the `review_status` field above. If it says `has_feedback`, scroll to the **Review Feedback** section immediately.
- **You must address all feedback** before your work is complete.
- **Mark as acknowledged**: When you understand the feedback, update `review_status: acknowledged`.
- **Report progress**: Update the Activity Log as you address feedback items.

---

## Review Feedback

> **Populated by `/spec-kitty.review`** – This section is empty initially.

*No feedback yet. This section will be populated if the work is returned from review.*

---

## Markdown Formatting
Wrap HTML/XML tags in backticks: `` `<div>` ``, `` `<script>` ``
Use language identifiers in code blocks: ````rust`, ````bash`

---

## Objectives & Success Criteria

**Goal**: Configure Criterion benchmarking framework and establish project structure for comprehensive performance testing.

**Success Criteria**:
- ✅ `cargo bench --no-run` compiles successfully
- ✅ Four benchmark harnesses configured in `Cargo.toml` (cache, config, context, logging)
- ✅ `benches/` directory exists with proper structure
- ✅ Criterion version 0.5.x with `html_reports` feature enabled
- ✅ Rust toolchain verified >= 1.75.0

## Context & Constraints

**Related Documents**:
- **Spec**: `kitty-specs/017-issue-9-add/spec.md` (FR1: Benchmark Coverage, AC1: Benchmark Suite Implementation)
- **Plan**: `kitty-specs/017-issue-9-add/plan.md` (Technical Context, Project Structure)
- **Research**: `kitty-specs/017-issue-9-add/research.md` (Technology Stack Validation)
- **Data Model**: `kitty-specs/017-issue-9-add/data-model.md` (BenchmarkResult schema)

**Key Decisions from Research**:
- Criterion 0.5.x chosen for statistical analysis and HTML report generation
- Modular benchmark organization (one file per module for independent execution)
- Standard Rust benchmark structure (`benches/` directory, `harness = false`)

**Constraints**:
- Must work with existing Rust 1.75+ toolchain
- No breaking changes to production dependencies
- Benchmarks must be deterministic (no network calls, no random data)

## Subtasks & Detailed Guidance

### Subtask T001 – Add criterion dev-dependency to Cargo.toml

**Purpose**: Install Criterion benchmarking framework with required features.

**Steps**:
1. Open `Cargo.toml` at repository root
2. Add to `[dev-dependencies]` section:
   ```toml
   [dev-dependencies]
   criterion = { version = "0.5", features = ["html_reports"] }
   ```
3. Run `cargo update` to fetch the dependency
4. Verify with `cargo tree --depth 1 | grep criterion`

**Expected Output**:
```
criterion v0.5.1
```

**Files**: `Cargo.toml` (root)

**Parallel?**: No (foundation for other subtasks)

**Notes**:
- Version `0.5` is the latest stable as of this implementation
- `html_reports` feature enables HTML report generation in `target/criterion/`
- If version conflict occurs, check for transitive dependencies with `cargo tree`

---

### Subtask T002 – Create benches/ directory structure

**Purpose**: Establish standard Rust benchmark directory at repository root.

**Steps**:
1. Create `benches/` directory at repository root:
   ```bash
   mkdir -p benches
   ```
2. Verify directory exists:
   ```bash
   ls -la benches
   ```

**Expected Structure**:
```
benches/           # Created (empty for now)
├── (cache.rs will be added in WP02)
├── (config.rs will be added in WP03)
├── (context.rs will be added in WP04)
└── (logging.rs will be added in WP04)
```

**Files**: `benches/` directory

**Parallel?**: No

**Notes**:
- Rust convention: benchmarks live in top-level `benches/` directory
- Each `.rs` file in `benches/` becomes an independently runnable benchmark
- Do NOT create the individual .rs files yet (those come in WP02-WP04)

---

### Subtask T003 – Configure [[bench]] entries in Cargo.toml

**Purpose**: Register benchmark harnesses so `cargo bench` can discover them.

**Steps**:
1. Open `Cargo.toml` at repository root
2. Add `[[bench]]` sections at the end of the file (after `[dependencies]` and `[dev-dependencies]`):
   ```toml
   [[bench]]
   name = "cache"
   harness = false

   [[bench]]
   name = "config"
   harness = false

   [[bench]]
   name = "context"
   harness = false

   [[bench]]
   name = "logging"
   harness = false
   ```
3. Verify configuration:
   ```bash
   cargo bench --no-run 2>&1 | grep "Compiling"
   ```

**Expected Behavior**:
- `cargo bench --no-run` should attempt to compile (will fail because .rs files don't exist yet, which is expected)
- Error message should mention "could not find `cache` in `benches`" (confirms Cargo found the config)

**Files**: `Cargo.toml` (root)

**Parallel?**: No (depends on T001, T002)

**Notes**:
- `harness = false` is CRITICAL - tells Cargo to use Criterion's custom harness
- Benchmark names must match the .rs filenames (without .rs extension)
- Order doesn't matter, but alphabetical is conventional

---

### Subtask T004 – Add serde_json dependency (if not present)

**Purpose**: Ensure `serde_json` is available for benchmark result serialization (used by CI scripts later).

**Steps**:
1. Check if `serde_json` already exists in dependencies:
   ```bash
   grep -n "serde_json" Cargo.toml
   ```
2. **If NOT found**, add to `[dependencies]` section:
   ```toml
   [dependencies]
   serde_json = "1.0"
   ```
3. **If found**, verify version is compatible (>= 1.0)
4. Run `cargo check` to verify

**Expected Output**:
```bash
$ cargo check
    Checking caro v1.x.x
    Finished dev [unoptimized + debuginfo] target(s) in 0.5s
```

**Files**: `Cargo.toml` (root)

**Parallel?**: [P] Can proceed concurrently with T001-T003 (different section of Cargo.toml)

**Notes**:
- `serde_json` may already exist as a dependency (caro likely uses it)
- If it exists, no action needed - just verify version
- Needed for CI scripts that parse Criterion JSON output (WP05)

---

### Subtask T005 – Verify Rust toolchain version >= 1.75.0

**Purpose**: Ensure development environment meets minimum requirements for Criterion and modern Rust features.

**Steps**:
1. Check current Rust version:
   ```bash
   rustc --version
   ```
2. Expected output format:
   ```
   rustc 1.75.0 (or higher)
   ```
3. **If version < 1.75.0**, update toolchain:
   ```bash
   rustup update stable
   rustc --version  # Verify again
   ```
4. Verify Cargo version matches:
   ```bash
   cargo --version
   ```

**Expected Output**:
```
rustc 1.75.0 (or 1.76.x, 1.77.x, etc.)
cargo 1.75.0 (or matching version)
```

**Files**: None (environment check only)

**Parallel?**: No (can be first step before T001)

**Notes**:
- Criterion 0.5 requires Rust 1.70+, but caro uses 1.75+ per plan.md
- If updating via `rustup`, ensure you're using the correct toolchain profile
- Check `rust-toolchain.toml` file if present (overrides system default)
- Document any toolchain issues in Activity Log below

---

## Definition of Done Checklist

- [ ] `cargo bench --no-run` command runs (may fail because .rs files don't exist, but Cargo config is recognized)
- [ ] `Cargo.toml` contains criterion 0.5 dev-dependency with html_reports feature
- [ ] `benches/` directory exists at repository root
- [ ] Four `[[bench]]` entries in `Cargo.toml` (cache, config, context, logging) with `harness = false`
- [ ] `serde_json` dependency verified (exists or added)
- [ ] Rust toolchain confirmed >= 1.75.0 via `rustc --version`
- [ ] `cargo check` completes successfully (project still compiles)
- [ ] No new warnings introduced by dependency changes
- [ ] `tasks.md` updated with WP01 completion status

## Review Guidance

**Key Acceptance Checkpoints for `/spec-kitty.review`**:

1. **Cargo.toml validation**:
   - Criterion version exactly "0.5" (not 0.4, not 0.6)
   - `features = ["html_reports"]` present
   - All four `[[bench]]` entries have `harness = false`
   - No duplicate `[[bench]]` entries

2. **Directory structure**:
   - `benches/` exists at root (not nested under `src/` or elsewhere)
   - Empty directory is acceptable (files added in WP02-WP04)

3. **Dependency hygiene**:
   - `cargo tree` shows no version conflicts with criterion
   - `serde_json` present (don't care if it was already there or newly added)

4. **Compilation status**:
   - `cargo check` succeeds
   - No new clippy warnings from dependency changes

**Common Issues to Check**:
- ❌ Missing `harness = false` (benchmarks won't use Criterion)
- ❌ Criterion version 0.4.x (outdated, missing features)
- ❌ Benchmark names mismatch future filenames
- ❌ `benches/` created under `src/` instead of root

## Activity Log

> Append entries when the work package changes lanes. Include timestamp, agent, shell PID, lane, and a short note.

- 2026-01-08T00:00:00Z – system – lane=planned – Prompt created by /spec-kitty.tasks
- 2026-01-08T13:24:43Z – claude – shell_pid=90455 – lane=doing – Started WP01 implementation
