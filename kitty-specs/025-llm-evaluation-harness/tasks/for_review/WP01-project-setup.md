---
work_package_id: "WP01"
subtasks:
  - "T001"
  - "T002"
  - "T003"
  - "T004"
  - "T005"
title: "Project Setup & Directory Structure"
phase: "Phase 0 - Setup"
lane: "for_review"
assignee: ""
agent: "claude"
shell_pid: "16951"
review_status: ""
reviewed_by: ""
history:
  - timestamp: "2026-01-09T00:00:00Z"
    lane: "planned"
    agent: "system"
    shell_pid: ""
    action: "Prompt generated via /spec-kitty.tasks"
  - timestamp: "2026-01-09T21:45:00Z"
    lane: "doing"
    agent: "claude"
    shell_pid: "16951"
    action: "Completed implementation: Created tests/evaluation/ directory structure, mod.rs with module declarations, stub files (dataset.rs, harness.rs, validators.rs, reporter.rs), verified cargo build succeeds. toml dependency already present at v0.9."
---

# Work Package Prompt: WP01 – Project Setup & Directory Structure

## ⚠️ IMPORTANT: Review Feedback Status

**Read this first if you are implementing this task!**

- **Has review feedback?**: Check the `review_status` field above. If it says `has_feedback`, scroll to the **Review Feedback** section immediately (right below this notice).
- **You must address all feedback** before your work is complete. Feedback items are your implementation TODO list.
- **Mark as acknowledged**: When you understand the feedback and begin addressing it, update `review_status: acknowledged` in the frontmatter.
- **Report progress**: As you address each feedback item, update the Activity Log explaining what you changed.

---

## Review Feedback

> **Populated by `/spec-kitty.review`** – Reviewers add detailed feedback here when work needs changes. Implementation must address every item listed below before returning for re-review.

*[This section is empty initially. Reviewers will populate it if the work is returned from review. If you see feedback here, treat each item as a must-do before completion.]*

---

## Objectives & Success Criteria

**Goal**: Establish evaluation harness directory structure and add toml dependency.

**Success Criteria**:
- `cargo build` succeeds with new toml dependency
- `tests/evaluation/` directory exists with proper module structure
- All module files (`mod.rs`, `dataset.rs`, `harness.rs`, `validators.rs`, `reporter.rs`) present
- Module imports work correctly

## Context & Constraints

**References**:
- [plan.md](../../plan.md) - Project structure at lines 104-133
- [data-model.md](../../data-model.md) - Data structures to be implemented
- `.specify/memory/constitution.md` - Principle I (Simplicity), Principle II (Library-First)

**Constraints**:
- This IS testing infrastructure (not production code) - lives in `tests/` directory
- Zero changes to `src/` production code (reuse existing modules)
- Follow existing `tests/` directory patterns
- Use `[dev-dependencies]` for toml crate

**Architectural Decision** (from plan.md):
> Evaluation harness lives in `tests/evaluation/` as integration test code, reusing existing `src/` exports. This is the simplest approach that requires zero changes to production code.

## Subtasks & Detailed Guidance

### Subtask T001 – Add toml dependency to Cargo.toml

**Purpose**: Enable TOML test dataset parsing using the toml crate.

**Steps**:
1. Open `Cargo.toml` at project root
2. Locate the `[dev-dependencies]` section (create if missing)
3. Add line: `toml = "0.8"`
4. Run `cargo update toml` to fetch dependency

**Files**:
- `/Users/kobik-private/workspace/caro/.worktrees/025-llm-evaluation-harness/Cargo.toml`

**Notes**:
- Use `[dev-dependencies]` because evaluation is test-only code
- Version `0.8` matches plan.md specification
- No changes to `[dependencies]` (production dependencies)

### Subtask T002 – Create tests/evaluation/ directory structure

**Purpose**: Establish the directory where all evaluation harness code will live.

**Steps**:
1. Navigate to project root
2. Create directory: `mkdir -p tests/evaluation`
3. Verify directory exists: `ls tests/`

**Files**:
- Create: `tests/evaluation/` (directory)

**Notes**:
- Existing `tests/` directory already present (integration tests, unit tests)
- New `evaluation/` subdirectory follows existing pattern

### Subtask T003 – Create tests/evaluation/mod.rs with module declarations

**Purpose**: Define module structure for evaluation harness.

**Steps**:
1. Create file `tests/evaluation/mod.rs`
2. Add module declarations:
   ```rust
   // Evaluation harness modules
   pub mod dataset;
   pub mod harness;
   pub mod validators;
   pub mod reporter;
   ```
3. Save file

**Files**:
- Create: `tests/evaluation/mod.rs`

**Notes**:
- `pub mod` makes modules accessible to test entry point
- Module structure from plan.md lines 108-114

### Subtask T004 – Create empty stub files

**Purpose**: Create placeholder files for each module to verify structure.

**Steps**:
1. Create `tests/evaluation/dataset.rs` with content:
   ```rust
   // Dataset loading and validation logic
   // TODO: Implement TestCase, Category, TestDataset structs (WP02)
   ```

2. Create `tests/evaluation/harness.rs` with content:
   ```rust
   // Core evaluation runner logic
   // TODO: Implement run_evaluation() function (WP05)
   ```

3. Create `tests/evaluation/validators.rs` with content:
   ```rust
   // Command normalization, safety, and POSIX validators
   // TODO: Implement normalize_command(), validate_safety(), is_posix_compliant() (WP03, WP04)
   ```

4. Create `tests/evaluation/reporter.rs` with content:
   ```rust
   // JSON and console output formatting
   // TODO: Implement output_json(), output_console() (WP07)
   ```

**Files**:
- Create: `tests/evaluation/dataset.rs`
- Create: `tests/evaluation/harness.rs`
- Create: `tests/evaluation/validators.rs`
- Create: `tests/evaluation/reporter.rs`

**Notes**:
- Empty stubs prevent compile errors from mod.rs declarations
- TODO comments reference future work packages

### Subtask T005 – Verify cargo build succeeds

**Purpose**: Validate that project structure and new dependency are correct.

**Steps**:
1. Run `cargo build` from project root
2. Verify output shows "Compiling toml v0.8.x"
3. Verify build succeeds with no errors
4. Check that `tests/evaluation/` modules compile

**Files**:
- No file changes (verification only)

**Expected Output**:
```
   Compiling toml v0.8.x
   Compiling caro v1.1.0 (/path/to/caro)
    Finished dev [unoptimized + debuginfo] target(s) in 3.45s
```

**Notes**:
- If build fails, check `mod.rs` declarations match file names
- Verify `toml = "0.8"` in `[dev-dependencies]` section

## Risks & Mitigations

**Risk**: Module visibility issues in test directory
- **Mitigation**: Follow existing `tests/` structure; use `pub mod` declarations

**Risk**: Wrong dependency section (production vs dev)
- **Mitigation**: Verify toml is under `[dev-dependencies]`, not `[dependencies]`

**Risk**: Path issues when running from worktree
- **Mitigation**: All paths relative to project root; verify `pwd` shows worktree location

## Definition of Done Checklist

- [ ] `Cargo.toml` contains `toml = "0.8"` under `[dev-dependencies]`
- [ ] `tests/evaluation/` directory exists
- [ ] `tests/evaluation/mod.rs` declares 4 modules (dataset, harness, validators, reporter)
- [ ] All 4 stub files exist with TODO comments
- [ ] `cargo build` succeeds with no errors
- [ ] Output shows toml crate compilation

## Review Guidance

**Acceptance Checkpoints**:
1. Verify `git diff Cargo.toml` shows toml added to dev-dependencies
2. Verify `tree tests/evaluation/` shows 5 files (mod.rs + 4 modules)
3. Run `cargo build` and confirm success
4. Check `cargo tree | grep toml` shows toml as dev dependency

**Questions for Reviewer**:
- Does directory structure match plan.md specification?
- Are all modules properly declared in mod.rs?
- Does cargo build succeed without errors?

## Activity Log

- 2026-01-09T00:00:00Z – system – shell_pid= – lane=planned – Prompt created via /spec-kitty.tasks
- 2026-01-09T09:41:56Z – claude – shell_pid=16951 – lane=doing – Started implementation
- 2026-01-09T09:47:15Z – claude – shell_pid=16951 – lane=for_review – Ready for review
