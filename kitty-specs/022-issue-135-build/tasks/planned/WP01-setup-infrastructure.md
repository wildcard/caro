---
work_package_id: "WP01"
subtasks:
  - "T001"
  - "T002"
  - "T003"
  - "T004"
  - "T005"
  - "T006"
title: "Setup & Infrastructure"
phase: "Phase 1 - Foundation"
lane: "planned"
assignee: ""
agent: ""
shell_pid: ""
review_status: ""
reviewed_by: ""
history:
  - timestamp: "2026-01-08T00:00:00Z"
    lane: "planned"
    agent: "system"
    shell_pid: ""
    action: "Prompt generated via /spec-kitty.tasks"
---

# Work Package Prompt: WP01 – Setup & Infrastructure

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

## Markdown Formatting
Wrap HTML/XML tags in backticks: `` `<div>` ``, `` `<script>` ``
Use language identifiers in code blocks: ````python`, ````bash`

---

## Objectives & Success Criteria

**Goal**: Establish evaluation harness project structure, dependencies, and foundational types.

**Success Criteria**:
- Project structure exists at `tests/evaluation/`
- Dependencies compile successfully with `cargo build`
- Basic types are defined in `src/lib.rs`
- Results directory is properly gitignored
- Dataset directory structure is created
- README.md provides clear usage documentation

**Independent Test**: Run `cargo build` from `tests/evaluation/` directory, verify all directories exist, confirm README documents prerequisites.

## Context & Constraints

**Prerequisites**: None (this is the starting package)

**Supporting Documents**:
- **Spec**: `kitty-specs/022-issue-135-build/spec.md` - User stories and requirements
- **Plan**: `kitty-specs/022-issue-135-build/plan.md` - Technical architecture and stack decisions
- **Data Model**: `kitty-specs/022-issue-135-build/data-model.md` - Entity definitions
- **Tasks**: `kitty-specs/022-issue-135-build/tasks.md` - Complete task breakdown

**Key Architectural Decisions**:
- Single project structure with `tests/evaluation/` subdirectory (follows existing caro test organization)
- Separate Cargo.toml for evaluation harness to avoid workspace conflicts
- Shellcheck as external dependency (must be documented in README)
- Dataset organization by category: correctness, safety, posix, backend_comparison

**Constraints**:
- Must not affect production caro binary
- Results directory must be gitignored
- Shellcheck installation required (document in README)

## Subtasks & Detailed Guidance

### T001 – Create `tests/evaluation/` directory structure

**Purpose**: Establish the root directory for the evaluation harness.

**Steps**:
1. Create directory: `tests/evaluation/`
2. Verify it's at the correct location (parallel to existing `tests/integration/` and `tests/unit/`)

**Files**:
- `tests/evaluation/` (directory)

**Parallel?**: No (must complete first)

### T002 – Create `tests/evaluation/Cargo.toml` with dependencies

**Purpose**: Define evaluation harness dependencies independently from main caro project.

**Steps**:
1. Create `tests/evaluation/Cargo.toml`
2. Add package metadata:
   ```toml
   [package]
   name = "caro-evaluation"
   version = "0.1.0"
   edition = "2021"
   ```
3. Add dependencies:
   ```toml
   [dependencies]
   serde = { version = "1.0", features = ["derive"] }
   serde_json = "1.0"
   tokio = { version = "1.0", features = ["full"] }
   clap = { version = "4.0", features = ["derive"] }
   indicatif = "0.17"
   similar = "2.2"
   chrono = { version = "0.4", features = ["serde"] }
   ```

**Files**:
- `tests/evaluation/Cargo.toml` (create)

**Parallel?**: No (required before other setup tasks)

**Notes**: Dependencies align with plan.md specifications. Version numbers use semantic versioning with caret requirements.

### T003 – Create `tests/evaluation/src/lib.rs` with module exports

**Purpose**: Establish the library entry point and module structure.

**Steps**:
1. Create `tests/evaluation/src/` directory
2. Create `lib.rs` with module declarations:
   ```rust
   pub mod dataset;
   pub mod executor;
   pub mod evaluator;
   pub mod safety_validator;
   pub mod posix_checker;
   pub mod reporter;
   ```
3. Add top-level documentation comment explaining the harness purpose

**Files**:
- `tests/evaluation/src/lib.rs` (create)

**Parallel?**: No (foundation for other modules)

### T004 – Create `.gitignore` entry for `tests/evaluation/results/`

**Purpose**: Prevent timestamped evaluation results from being committed to version control.

**Steps**:
1. Open project root `.gitignore`
2. Add entry:
   ```
   # Evaluation harness results
   tests/evaluation/results/*.json
   tests/evaluation/results/*.md
   ```
3. Ensure `.gitkeep` is NOT ignored so the directory structure is preserved

**Files**:
- `.gitignore` (modify)

**Parallel?**: Yes (can proceed in parallel with T005 and T006 after T001)

### T005 – Create dataset directory structure under `tests/evaluation/datasets/`

**Purpose**: Organize test datasets by category for clarity and maintainability.

**Steps**:
1. Create directories:
   ```
   tests/evaluation/datasets/
   ├── correctness/
   ├── safety/
   ├── posix/
   └── backend_comparison/
   ```
2. Add `.gitkeep` files to each subdirectory to preserve structure in git

**Files**:
- `tests/evaluation/datasets/correctness/` (directory + `.gitkeep`)
- `tests/evaluation/datasets/safety/` (directory + `.gitkeep`)
- `tests/evaluation/datasets/posix/` (directory + `.gitkeep`)
- `tests/evaluation/datasets/backend_comparison/` (directory + `.gitkeep`)

**Parallel?**: Yes (can proceed in parallel with T004 and T006 after T001)

### T006 – Create `tests/evaluation/README.md` with usage documentation

**Purpose**: Provide clear instructions for running evaluations and understanding the harness.

**Steps**:
1. Create `tests/evaluation/README.md`
2. Include sections:
   - **Overview**: Purpose of the evaluation harness
   - **Prerequisites**:
     - Caro binary built (`cargo build --release`)
     - Shellcheck installed (with installation instructions for macOS/Linux)
   - **Quick Start**: Basic commands to run evaluations
   - **Running Tests**: How to execute different test categories
   - **Adding Test Cases**: Guidelines for creating new test datasets
   - **Viewing Results**: Where to find JSON and Markdown reports
3. Reference quickstart.md for detailed scenarios

**Files**:
- `tests/evaluation/README.md` (create)

**Parallel?**: Yes (can proceed in parallel with T004 and T005 after T001)

**Template Structure**:
```markdown
# Caro LLM Evaluation Harness

## Overview
Comprehensive evaluation framework for testing LLM-generated shell command quality.

## Prerequisites
- Caro binary: `cargo build --release`
- Shellcheck: `brew install shellcheck` (macOS) or `sudo apt-get install shellcheck` (Linux)

## Quick Start
\```bash
cd tests/evaluation
cargo test --test test_correctness -- --nocapture
\```

## Running Tests
- Correctness: `cargo test --test test_correctness`
- Safety: `cargo test --test test_safety`
- POSIX: `cargo test --test test_posix`
- All: `cargo test -- --nocapture`

## Adding Test Cases
See `datasets/` for examples. Each test case requires:
- Unique ID
- Natural language prompt
- Expected command
- Category and risk level

## Viewing Results
Results are written to `results/run_YYYY-MM-DD_HHMMSS.{json,md}`
```

## Risks & Mitigations

**Risk**: Cargo workspace conflicts with main caro project
**Mitigation**: Use separate Cargo.toml for evaluation harness; test compilation before proceeding

**Risk**: Shellcheck not installed on developer machines
**Mitigation**: Document prerequisite clearly in README with platform-specific installation commands; fail gracefully with error message if missing

**Risk**: Results directory accidentally committed
**Mitigation**: Add explicit gitignore entries; verify with `git status` after creating test results

## Definition of Done Checklist

- [ ] Directory structure created: `tests/evaluation/` with all subdirectories
- [ ] `Cargo.toml` created with all required dependencies
- [ ] `src/lib.rs` created with module exports
- [ ] `.gitignore` updated to exclude results
- [ ] Dataset directories created (correctness, safety, posix, backend_comparison)
- [ ] `README.md` created with prerequisites and usage instructions
- [ ] `cargo build` succeeds from `tests/evaluation/` directory
- [ ] Shellcheck installation documented in README
- [ ] `tasks.md` updated with WP01 completion status

## Review Guidance

**Key Acceptance Checkpoints**:
1. Verify all directories exist with correct structure
2. Confirm Cargo.toml compiles successfully
3. Check gitignore prevents results from being committed
4. Validate README includes shellcheck installation instructions
5. Ensure no production code is affected (isolated under tests/)

**Context for Reviewers**:
- This is foundational work; future WPs depend on this structure
- Directory organization follows plan.md specifications
- Separate Cargo.toml is intentional to avoid workspace conflicts

## Activity Log

> Append entries when the work package changes lanes. Include timestamp, agent, shell PID, lane, and a short note.

- 2026-01-08T00:00:00Z – system – lane=planned – Prompt created.

---

### Updating Metadata When Changing Lanes

1. Capture your shell PID: `echo $$` (or use helper scripts when available).
2. Update frontmatter (`lane`, `assignee`, `agent`, `shell_pid`).
3. Add an entry to the **Activity Log** describing the transition.
4. Run `.kittify/scripts/bash/tasks-move-to-lane.sh <FEATURE> <WPID> <lane>` (PowerShell variant available) to move the prompt, update metadata, and append history in one step.
5. Commit or stage the change, preserving history.
