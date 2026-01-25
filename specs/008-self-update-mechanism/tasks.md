# Self-Update Implementation Tasks

## Phase 1: Foundation (v1.1.0)

### P1-1: Add self_update dependency

**Status**: `pending`

**Description**: Add the `self_update` crate as an optional dependency with appropriate feature flags.

**Files**:
- `Cargo.toml`

**Acceptance Criteria**:
- [ ] `self_update = "0.42"` added with correct features
- [ ] `self-update` feature flag created
- [ ] Feature added to `full` feature set
- [ ] `cargo check --features self-update` passes

**Implementation**:
```toml
[dependencies.self_update]
version = "0.42"
optional = true
default-features = false
features = ["archive-tar", "archive-zip", "compression-flate2", "rustls"]

[features]
self-update = ["dep:self_update"]
full = ["remote-backends", "embedded-mlx", "embedded-cpu", "self-update"]
```

---

### P1-2: Add can_self_update() to version module

**Status**: `pending`

**Description**: Extend the version module to detect if self-update is supported for the current build type.

**Files**:
- `src/version.rs`

**Acceptance Criteria**:
- [ ] `can_self_update()` returns `true` only for official binary releases
- [ ] `update_instructions()` returns appropriate message per build type
- [ ] Unit tests cover all three build types

**Implementation**:
```rust
impl VersionInfo {
    /// Check if self-update is supported for this build
    pub fn can_self_update(&self) -> bool {
        self.build_type() == "binary (official release)"
    }

    /// Get update instructions appropriate for build type
    pub fn update_instructions(&self) -> &'static str {
        match self.build_type() {
            "binary (official release)" => "Run 'caro update' to update",
            "source (cargo install)" => "Run 'cargo install caro --force' to update",
            "dev (local build)" => "Rebuild from source to update",
            _ => "Unknown build type",
        }
    }
}
```

---

### P1-3: Create update module structure

**Status**: `pending`

**Description**: Create the update module with version checking functionality.

**Files**:
- `src/update/mod.rs` (new)
- `src/update/checker.rs` (new)
- `src/lib.rs` (add module)

**Acceptance Criteria**:
- [ ] Module compiles with `--features self-update`
- [ ] Module is behind `#[cfg(feature = "self-update")]`
- [ ] `UpdateChecker` struct can fetch latest version from GitHub

**Implementation**:
```rust
// src/update/mod.rs
#[cfg(feature = "self-update")]
mod checker;

#[cfg(feature = "self-update")]
pub use checker::UpdateChecker;
```

---

### P1-4: Implement version checking

**Status**: `pending`

**Description**: Implement the `UpdateChecker` that queries GitHub for the latest release.

**Files**:
- `src/update/checker.rs`

**Acceptance Criteria**:
- [ ] `check_for_update()` returns `Some(version)` if newer available
- [ ] `check_for_update()` returns `None` if up-to-date
- [ ] Proper error handling for network failures
- [ ] Semver comparison handles edge cases

**Implementation**:
```rust
use anyhow::Result;
use self_update::backends::github::ReleaseList;

pub struct UpdateChecker {
    current_version: semver::Version,
}

impl UpdateChecker {
    pub fn new() -> Result<Self> {
        Ok(Self {
            current_version: semver::Version::parse(env!("CARGO_PKG_VERSION"))?,
        })
    }

    pub fn check_for_update(&self) -> Result<Option<String>> {
        let releases = ReleaseList::configure()
            .repo_owner("wildcard")
            .repo_name("caro")
            .build()?
            .fetch()?;

        if let Some(latest) = releases.first() {
            let latest_str = latest.version.trim_start_matches('v');
            let latest_version = semver::Version::parse(latest_str)?;

            if latest_version > self.current_version {
                return Ok(Some(latest_str.to_string()));
            }
        }
        Ok(None)
    }
}
```

---

### P1-5: Add update CLI subcommand

**Status**: `pending`

**Description**: Add the `update` subcommand to the CLI with `--check` flag.

**Files**:
- `src/cli/mod.rs`
- `src/cli/update.rs` (new)
- `src/main.rs` (add handler)

**Acceptance Criteria**:
- [ ] `caro update --check` works and shows version info
- [ ] `caro update --help` shows proper help text
- [ ] Exit codes match spec (0 = current, 1 = update available)
- [ ] Build type detection prevents update on wrong builds

---

### P1-6: Write tests for Phase 1

**Status**: `pending`

**Description**: Add unit and integration tests for version checking.

**Files**:
- `src/update/checker.rs` (add tests)
- `src/version.rs` (add tests)
- `tests/update_check.rs` (new, optional)

**Acceptance Criteria**:
- [ ] Version comparison tests pass
- [ ] Build type detection tests pass
- [ ] `cargo test --features self-update` passes

---

## Phase 2: Self-Update (v1.2.0)

### P2-1: Implement download and install

**Status**: `pending`

**Description**: Add the ability to download and install updates.

**Files**:
- `src/update/installer.rs` (new)
- `src/update/mod.rs` (add export)

**Acceptance Criteria**:
- [ ] Downloads correct binary for target platform
- [ ] Shows progress bar during download
- [ ] Extracts binary from archive
- [ ] Replaces current binary atomically

**Implementation**:
```rust
use self_update::backends::github::Update;

pub fn install_update() -> Result<self_update::Status> {
    Update::configure()
        .repo_owner("wildcard")
        .repo_name("caro")
        .bin_name("caro")
        .show_download_progress(true)
        .show_output(true)
        .current_version(env!("CARGO_PKG_VERSION"))
        .build()?
        .update()
}
```

---

### P2-2: Add progress bar integration

**Status**: `pending`

**Description**: Create custom progress bar using indicatif for better UX.

**Files**:
- `src/update/progress.rs` (new)
- `src/update/installer.rs` (integrate)

**Acceptance Criteria**:
- [ ] Progress bar shows percentage, bytes, speed
- [ ] Matches Caro's visual style
- [ ] Works in quiet mode (hidden)

---

### P2-3: Implement force reinstall

**Status**: `pending`

**Description**: Add `--force` flag to reinstall even if up-to-date.

**Files**:
- `src/cli/update.rs`

**Acceptance Criteria**:
- [ ] `--force` flag works
- [ ] Confirmation prompt before force reinstall
- [ ] `--yes` skips confirmation

---

### P2-4: Handle edge cases

**Status**: `pending`

**Description**: Handle platform-specific edge cases and errors.

**Files**:
- `src/update/installer.rs`
- `src/cli/update.rs`

**Acceptance Criteria**:
- [ ] Windows binary replacement works
- [ ] Permission errors show helpful message
- [ ] Network errors show retry guidance
- [ ] Partial downloads are cleaned up

---

### P2-5: Cross-platform testing

**Status**: `pending`

**Description**: Test full update cycle on all platforms.

**Acceptance Criteria**:
- [ ] macOS ARM64 tested
- [ ] macOS x86_64 tested
- [ ] Linux x86_64 tested
- [ ] Windows x86_64 tested (best effort)

---

## Phase 3: Polish (v1.3.0)

### P3-1: Add quiet mode

**Status**: `pending`

**Description**: Implement `--quiet` flag for script usage.

**Files**:
- `src/cli/update.rs`

**Acceptance Criteria**:
- [ ] `--quiet` outputs only essential info
- [ ] No colors or progress bars in quiet mode
- [ ] Exit codes still work for script logic

---

### P3-2: Optional startup check

**Status**: `pending`

**Description**: Add config option to check for updates on startup.

**Files**:
- `src/config/mod.rs`
- `src/main.rs`

**Acceptance Criteria**:
- [ ] `check_updates_on_startup: bool` config option
- [ ] Shows non-intrusive message if update available
- [ ] Respects rate limiting (once per day max)
- [ ] Doesn't block startup

---

### P3-3: Improve error messages

**Status**: `pending`

**Description**: Enhance error messages with actionable guidance.

**Files**:
- `src/update/installer.rs`
- `src/cli/update.rs`

**Acceptance Criteria**:
- [ ] Network errors suggest checking connection
- [ ] Permission errors suggest sudo or path check
- [ ] Rate limit errors suggest waiting

---

### P3-4: Add update to help text

**Status**: `pending`

**Description**: Ensure update command is visible in main help.

**Files**:
- `src/cli/mod.rs`

**Acceptance Criteria**:
- [ ] `caro --help` shows update command
- [ ] Help text explains what update does

---

## Phase 4: Security (v2.0.0)

### P4-1: Generate signing keypair

**Status**: `pending`

**Description**: Generate Ed25519 keypair for release signing.

**Acceptance Criteria**:
- [ ] Private key securely stored in GitHub Secrets
- [ ] Public key embedded in binary
- [ ] Key rotation procedure documented

---

### P4-2: Sign releases

**Status**: `pending`

**Description**: Integrate signing into release workflow.

**Files**:
- `.github/workflows/release.yml`

**Acceptance Criteria**:
- [ ] Each release asset is signed with zipsign
- [ ] Signatures published alongside assets

---

### P4-3: Verify signatures on update

**Status**: `pending`

**Description**: Verify Ed25519 signatures before installing updates.

**Files**:
- `src/update/installer.rs`
- `Cargo.toml` (add signatures feature)

**Acceptance Criteria**:
- [ ] `signatures` feature enabled
- [ ] Invalid signatures abort update
- [ ] Clear error message on signature mismatch

---

### P4-4: Add checksum verification

**Status**: `pending`

**Description**: Verify SHA256 checksums from release notes.

**Files**:
- `src/update/installer.rs`

**Acceptance Criteria**:
- [ ] Parse checksums from release body
- [ ] Verify downloaded file matches
- [ ] Fail with clear error on mismatch

---

## Documentation Tasks

### D-1: Update README

**Status**: `pending`

**Description**: Document the update command in README.

**Files**:
- `README.md`

**Acceptance Criteria**:
- [ ] `caro update` documented
- [ ] Installation method differences explained

---

### D-2: Add UPDATING.md

**Status**: `pending`

**Description**: Create dedicated updating documentation.

**Files**:
- `docs/UPDATING.md` (new)

**Acceptance Criteria**:
- [ ] Explains all update methods
- [ ] Covers troubleshooting
- [ ] Documents rollback procedure

---

### D-3: Update man page

**Status**: `pending`

**Description**: Add update command to man page if applicable.

---

## CI/CD Tasks

### C-1: Verify asset naming in release

**Status**: `pending`

**Description**: Ensure release workflow produces correctly named assets.

**Files**:
- `.github/workflows/release.yml`

**Acceptance Criteria**:
- [ ] Asset names match `caro-{version}-{target}.{ext}`
- [ ] All platforms produce assets

---

### C-2: Add self-update feature to CI

**Status**: `pending`

**Description**: Test self-update feature in CI.

**Files**:
- `.github/workflows/ci.yml`

**Acceptance Criteria**:
- [ ] `cargo test --features self-update` runs in CI
- [ ] Feature compiles on all platforms

---

## Summary

| Phase | Tasks | Priority |
|-------|-------|----------|
| Phase 1 | P1-1 through P1-6 | High |
| Phase 2 | P2-1 through P2-5 | High |
| Phase 3 | P3-1 through P3-4 | Medium |
| Phase 4 | P4-1 through P4-4 | Low |
| Docs | D-1 through D-3 | Medium |
| CI/CD | C-1 through C-2 | Medium |

---

*Task list created January 2026*
