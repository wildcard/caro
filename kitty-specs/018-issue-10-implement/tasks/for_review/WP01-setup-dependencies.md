---
work_package_id: "WP01"
subtasks: ["T001", "T002", "T003", "T004", "T005", "T006"]
title: "Setup & Dependencies"
phase: "Phase 1 - Foundation"
lane: "for_review"
assignee: ""
agent: "claude"
shell_pid: "13317"
review_status: ""
reviewed_by: ""
history:
  - timestamp: "2026-01-08T00:00:00Z"
    lane: "planned"
    agent: "system"
    shell_pid: ""
    action: "Prompt generated via /spec-kitty.tasks"
---

# Work Package Prompt: WP01 – Setup & Dependencies

## Objectives & Success Criteria

**Goal**: Add required Rust dependencies to Cargo.toml and verify clean build.

Add 5 production dependencies and 1 dev dependency:
- reqwest 0.11 with stream and json features (HTTP client)
- indicatif 0.17 (progress bars)
- sha2 0.10 (SHA256 checksums)
- fd-lock 4.0 (manifest file locking)
- wiremock 0.5 (HTTP mocking for tests)

**Related Documents**:
- Spec: [Issue #10](https://github.com/wildcard/caro/issues/10)
- Plan: kitty-specs/018-issue-10-implement/plan.md
- Tasks: kitty-specs/018-issue-10-implement/tasks.md

## Implementation Guidance

### T001: Add reqwest 0.11

Add to `[dependencies]` section:
```toml
reqwest = { version = "0.11", features = ["stream", "json"] }
```

**Why these features**:
- `stream`: Required for streaming downloads (async Bytes stream)
- `json`: Required for parsing HF Hub API responses

### T002: Add indicatif 0.17

Add to `[dependencies]` section:
```toml
indicatif = "0.17"
```

**Purpose**: Progress bars with download speed and ETA calculation.

### T003: Add sha2 0.10

Add to `[dependencies]` section:
```toml
sha2 = "0.10"
```

**Purpose**: SHA256 checksum validation during streaming downloads.

### T004: Add fd-lock 4.0

Add to `[dependencies]` section:
```toml
fd-lock = "4.0"
```

**Purpose**: File-based locking for atomic manifest updates (prevents corruption from concurrent downloads).

### T005: Add wiremock 0.5

Add to `[dev-dependencies]` section:
```toml
wiremock = "0.5"
```

**Purpose**: HTTP mocking for unit tests (simulates HF Hub API responses).

### T006: Verify clean build

Run the following commands to verify:
```bash
cargo check   # Verify dependencies resolve and code compiles
cargo build   # Full build
cargo test    # Run existing tests (should still pass)
```

Expected output: No errors, all existing tests pass.

## Definition of Done

- [ ] All 5 production dependencies added to Cargo.toml
- [ ] wiremock 0.5 added to dev-dependencies
- [ ] `cargo check` passes without errors
- [ ] `cargo build` completes successfully
- [ ] `cargo test` runs and all existing tests pass
- [ ] No dependency version conflicts reported
- [ ] Cargo.lock updated and committed

## Testing Strategy

**Validation**:
1. Run `cargo tree` to verify dependency graph is clean
2. Check for duplicate versions of crates (should be none)
3. Verify all features are enabled correctly

**No new tests required** for this work package (dependency setup only).

## Risks & Mitigation

| Risk | Mitigation |
|------|------------|
| Version conflicts with existing dependencies | Use compatible versions (all are widely used crates) |
| Feature flags incorrect | Verify with cargo doc that features are available |
| Build failures on CI | Test locally first with cargo check/build/test |

## Notes for Reviewers

- Verify all version numbers match plan.md specifications
- Check that feature flags are exactly as specified (`stream` and `json` for reqwest)
- Confirm no extraneous dependencies were added
- Ensure Cargo.lock is included in commit

## Activity Log

- 2026-01-08T00:00:00Z – system – lane=planned – Prompt created
- 2026-01-08T15:46:47Z – claude – shell_pid=13317 – lane=doing – Started WP01: Setup & Dependencies
- 2026-01-08T15:51:32Z – claude – shell_pid=13317 – lane=doing – Completed implementation: All dependencies added, tests passing
- 2026-01-08T15:52:30Z – claude – shell_pid=13317 – lane=for_review – Ready for review: All dependencies added, builds clean, tests pass
