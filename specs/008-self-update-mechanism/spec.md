# Caro Self-Update Mechanism - Product Requirements Document

## Executive Summary

This specification defines the product requirements for adding self-update functionality to Caro, enabling users to update their installation with a single command. The feature uses the `self_update` crate to check GitHub Releases and replace the binary in-place, providing a seamless upgrade experience.

**Key Value Proposition**: Users can stay current with security patches and new features without manual download/install steps.

## Goals

### Primary Goals

1. **One-Command Updates**: `caro update` checks for and installs the latest version
2. **Security Velocity**: Get security patches to users within minutes of release
3. **Zero Configuration**: Works out-of-the-box for GitHub Release installations
4. **Cross-Platform**: Consistent experience on macOS, Linux, and Windows
5. **User Control**: Updates are always user-initiated, never automatic

### Secondary Goals

1. **Version Awareness**: Users can easily check if updates are available
2. **Rollback Information**: Clear guidance if update causes issues
3. **Progress Feedback**: Download progress and status during update
4. **Offline Graceful**: Clear error messages when offline

### Non-Goals

1. **Automatic Background Updates**: We don't update without user consent
2. **Delta/Patch Updates**: Full binary replacement only (simplicity over bandwidth)
3. **Version Pinning**: Users can't pin to specific versions via this mechanism
4. **Downgrade Support**: Only upgrades, not downgrades
5. **cargo install Updates**: Source-based installations use `cargo install --force`

## User Stories

### US-1: Check for Updates

**As a** Caro user
**I want to** check if a new version is available
**So that** I know when to update without actually updating

**Acceptance Criteria:**
- [ ] `caro update --check` shows current vs latest version
- [ ] Exit code 0 if up-to-date, exit code 1 if update available
- [ ] Works without network shows clear offline error
- [ ] Completes in < 5 seconds on normal network

**Example Output:**
```
$ caro update --check
Current version: 1.0.3
Latest version:  1.0.5

Update available! Run 'caro update' to install.
```

### US-2: Install Update

**As a** Caro user
**I want to** update to the latest version with one command
**So that** I get new features and security fixes quickly

**Acceptance Criteria:**
- [ ] `caro update` downloads and installs latest version
- [ ] Shows download progress with percentage and speed
- [ ] Verifies download integrity before replacing
- [ ] Preserves executable permissions after update
- [ ] Shows success message with new version number

**Example Output:**
```
$ caro update
Checking for updates...
Current version: 1.0.3
Latest version:  1.0.5

Downloading caro v1.0.5...
[████████████████████████████████████████] 100% (12.5 MB, 2.1 MB/s)

Verifying download...
Installing update...

Updated successfully!
caro 1.0.5 (abc1234 2026-01-15)

Restart your shell or run 'caro --version' to verify.
```

### US-3: Already Up-to-Date

**As a** Caro user
**I want to** know when I'm already on the latest version
**So that** I don't waste time or bandwidth

**Acceptance Criteria:**
- [ ] `caro update` shows "already up-to-date" if current
- [ ] No download occurs when already current
- [ ] Exit code 0 for success

**Example Output:**
```
$ caro update
Checking for updates...

You're already running the latest version!
caro 1.0.5 (abc1234 2026-01-15)
```

### US-4: Force Reinstall

**As a** Caro user
**I want to** force reinstall even if up-to-date
**So that** I can repair a corrupted installation

**Acceptance Criteria:**
- [ ] `caro update --force` downloads even if current version
- [ ] Confirmation prompt before force update
- [ ] Same download/install flow as normal update

**Example Output:**
```
$ caro update --force
You are already running the latest version (1.0.5).

Force reinstall anyway? [y/N] y

Downloading caro v1.0.5...
[████████████████████████████████████████] 100%

Reinstalled successfully!
```

### US-5: Source Installation Guidance

**As a** developer who installed via `cargo install`
**I want to** know the correct update command for my installation type
**So that** I don't break my setup

**Acceptance Criteria:**
- [ ] Detects source-based installation
- [ ] Shows `cargo install caro --force` guidance
- [ ] Does not attempt binary replacement for source installs

**Example Output:**
```
$ caro update
Checking for updates...

You installed Caro from source (cargo install).
To update, run:

    cargo install caro --force

Self-update only works for binary releases from GitHub.
```

### US-6: Development Build Warning

**As a** developer with a local dev build
**I want to** be warned that self-update doesn't apply
**So that** I don't accidentally overwrite my development version

**Acceptance Criteria:**
- [ ] Detects development/debug builds
- [ ] Shows warning and refuses to update
- [ ] Explains why (dev builds shouldn't be replaced)

**Example Output:**
```
$ caro update

This is a development build (dev local build).
Self-update is disabled for development versions.

To test updates, build a release version or install from GitHub.
```

### US-7: Quiet Mode for Scripts

**As a** system administrator
**I want to** run update checks in scripts without interactive prompts
**So that** I can automate update notifications

**Acceptance Criteria:**
- [ ] `caro update --check --quiet` outputs only version info
- [ ] Exit codes indicate update status (0 = current, 1 = available)
- [ ] No color codes or progress bars in quiet mode

**Example Output:**
```
$ caro update --check --quiet
1.0.3 1.0.5
$ echo $?
1
```

### US-8: Network Failure Handling

**As a** Caro user
**I want to** get clear error messages when offline
**So that** I understand why the update failed

**Acceptance Criteria:**
- [ ] Clear error message for network failures
- [ ] Retry suggestion for transient failures
- [ ] Exit code non-zero for failures

**Example Output:**
```
$ caro update
Checking for updates...

Error: Unable to reach GitHub API
  - Check your internet connection
  - GitHub status: https://githubstatus.com

No changes made to your installation.
```

## User Experience

### Command Interface

```
USAGE:
    caro update [OPTIONS]

OPTIONS:
    -c, --check     Check for updates without installing
    -f, --force     Force reinstall even if up-to-date
    -q, --quiet     Minimal output (for scripts)
    -y, --yes       Skip confirmation prompts
    -h, --help      Print help information

EXAMPLES:
    caro update              # Interactive update
    caro update --check      # Check only, don't install
    caro update --force      # Force reinstall
    caro update -y           # Auto-confirm (non-interactive)
```

### Progress Indicators

**Download Progress:**
```
Downloading caro v1.0.5...
[████████████████░░░░░░░░░░░░░░░░░░░░░░░░] 42% (5.2 MB / 12.5 MB) 1.8 MB/s
```

**Verification:**
```
Verifying download... OK
```

**Installation:**
```
Installing update... OK
```

### Error States

| State | Message | Exit Code |
|-------|---------|-----------|
| Up-to-date | "Already running latest version" | 0 |
| Update available | "Update available! Run 'caro update'" | 1 |
| Update success | "Updated successfully!" | 0 |
| Network error | "Unable to reach GitHub API" | 2 |
| Permission error | "Permission denied. Try with sudo?" | 3 |
| Checksum mismatch | "Download verification failed" | 4 |
| Unsupported install | "Self-update not supported for this installation" | 5 |

### Integration with Caro Personality

Update messages should reflect Caro's helpful personality:

**Checking:**
```
Hey! Let me check if there's a newer version of me...
```

**Up-to-date:**
```
You're all set! Running the latest version (1.0.5)
```

**Update available:**
```
Good news! Version 1.0.5 is available (you have 1.0.3)
Ready to update? [Y/n]
```

**Success:**
```
All done! I'm now running version 1.0.5
Thanks for keeping me up-to-date!
```

## Technical Requirements

### Platform Support

| Platform | Binary Format | Archive Type | Status |
|----------|--------------|--------------|--------|
| macOS ARM64 | Universal | `.tar.gz` | Required |
| macOS x86_64 | Universal | `.tar.gz` | Required |
| Linux x86_64 | ELF | `.tar.gz` | Required |
| Linux ARM64 | ELF | `.tar.gz` | Best-effort |
| Windows x86_64 | PE | `.zip` | Best-effort |

### Release Artifact Naming

GitHub Release assets must follow this convention:

```
caro-{version}-{target}.{ext}

Examples:
caro-1.0.5-aarch64-apple-darwin.tar.gz
caro-1.0.5-x86_64-apple-darwin.tar.gz
caro-1.0.5-x86_64-unknown-linux-gnu.tar.gz
caro-1.0.5-aarch64-unknown-linux-gnu.tar.gz
caro-1.0.5-x86_64-pc-windows-msvc.zip
```

### Feature Flags

```toml
[features]
default = ["embedded-mlx", "embedded-cpu"]
self-update = ["dep:self_update"]
full = ["remote-backends", "embedded-mlx", "embedded-cpu", "self-update"]
```

Self-update is opt-in to keep binary size minimal for embedded use cases.

### Dependencies

```toml
[dependencies.self_update]
version = "0.42"
optional = true
default-features = false
features = [
    "archive-tar",
    "archive-zip",
    "compression-flate2",
    "rustls",
]
```

### Build Type Detection

Update behavior varies by installation method:

| Build Type | Detection | Update Behavior |
|------------|-----------|-----------------|
| `binary (official release)` | `CARO_RELEASE=1` | Full self-update |
| `source (cargo install)` | Release profile, no flag | Suggest cargo install |
| `dev (local build)` | Debug profile or unknown git | Refuse, warn |

## Security Requirements

### SR-1: HTTPS Only

All downloads must use HTTPS. HTTP requests should fail or upgrade.

### SR-2: Version Comparison

Only allow upgrades (new version > current version). Downgrades require manual intervention.

### SR-3: Checksum Verification

Downloaded binaries must be verified against checksums from release notes.

### SR-4: User Confirmation

Default to requiring user confirmation before replacing binary. `--yes` flag for automation.

### SR-5: Atomic Replacement

Use `self-replace` crate for atomic binary replacement to prevent corruption.

### SR-6: Future: Signature Verification

Roadmap item: Ed25519 signatures for releases using `zipsign`.

## Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Update success rate | > 99% | Telemetry (opt-in) |
| Time to check | < 3 seconds | CI benchmarks |
| Time to update | < 60 seconds | Manual testing |
| User adoption | 30%+ use update command | Telemetry (opt-in) |
| Error message clarity | > 90% understand | User feedback |

## Rollout Plan

### Phase 1: v1.1.0 - Check Only

- Implement `caro update --check`
- Build type detection
- Basic error handling
- Documentation

### Phase 2: v1.2.0 - Full Self-Update

- Implement `caro update` with download/install
- Progress indicators
- Force reinstall support
- Cross-platform testing

### Phase 3: v1.3.0 - Polish

- Optional startup check (config option)
- Quiet mode for scripts
- Improved error messages
- Performance optimization

### Phase 4: v2.0.0 - Security Hardening

- Ed25519 signature verification
- Checksum validation
- Certificate pinning

## Risks and Mitigations

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Corrupted download | Low | High | Checksum verification, atomic replace |
| Permissions issues | Medium | Medium | Clear error messages, sudo guidance |
| Platform edge cases | Medium | Low | Extensive cross-platform testing |
| Rate limiting | Low | Low | Cache version checks, exponential backoff |
| Breaking changes | Low | High | Semantic versioning, changelog |

## Open Questions

1. **Startup check frequency**: How often should optional startup check run? Daily? Weekly?
2. **Rollback mechanism**: Should we provide `caro update --rollback`?
3. **Channel support**: Should we support beta/nightly channels in future?
4. **Proxy support**: How do we handle corporate proxies?

## Appendix A: Competitive Analysis

| Tool | Update Mechanism | Notes |
|------|------------------|-------|
| **rustup** | `rustup update` | Self-updates toolchain and self |
| **cargo** | Via rustup | No direct self-update |
| **gh** (GitHub CLI) | `gh upgrade` | GitHub Releases |
| **starship** | Manual/package manager | No built-in update |
| **zoxide** | Manual/package manager | No built-in update |
| **bat** | Manual/package manager | No built-in update |
| **ripgrep** | Manual/package manager | No built-in update |

Caro's approach aligns with `gh` and `rustup` patterns.

## Appendix B: Related Documents

- [ADR-004: Self-Update Mechanism](../../docs/adr/ADR-004-self-update-mechanism.md)
- [Release Process](../../docs/RELEASE_PROCESS.md)
- [Version Module](../../src/version.rs)

---

*This PRD was authored in January 2026 and defines requirements for the self-update feature.*
