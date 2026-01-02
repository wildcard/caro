# ADR-004: Self-Update Mechanism

| **Status**     | Proposed                            |
|----------------|-------------------------------------|
| **Date**       | January 2026                        |
| **Authors**    | Caro Maintainers                    |
| **Target**     | Community                           |
| **Supersedes** | N/A                                 |

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Context and Problem Statement](#context-and-problem-statement)
3. [Decision Drivers](#decision-drivers)
4. [Crate Evaluation](#crate-evaluation)
5. [Recommended Decision](#recommended-decision)
6. [Implementation Strategy](#implementation-strategy)
7. [Security Considerations](#security-considerations)
8. [Consequences](#consequences)
9. [Alternatives Considered](#alternatives-considered)
10. [References](#references)

---

## Executive Summary

This ADR evaluates adding self-update functionality to Caro, enabling the CLI to check for and apply updates without requiring users to manually run `cargo install` or download binaries. After evaluating the available Rust ecosystem, **`self_update`** emerges as the recommended crate due to its maturity, GitHub release integration, cross-platform support, and active maintenance.

**Core Tenets:**
- **User convenience** through seamless update experience
- **Security-first** with signature verification and checksums
- **Optional behavior** - updates are user-initiated, never automatic
- **GitHub releases** as the primary distribution mechanism
- **Fallback to cargo install** for source-based installations

---

## Context and Problem Statement

### The Challenge

Caro is distributed through three channels:

1. **crates.io** (`cargo install caro`) - Source-based installation
2. **GitHub Releases** - Pre-compiled binaries for macOS/Linux/Windows
3. **Homebrew** (planned) - Package manager distribution

Currently, users must manually check for updates by:
- Visiting GitHub or crates.io
- Comparing versions
- Running `cargo install caro --force` or downloading new binaries
- Replacing their existing installation

This creates friction and leads to users running outdated versions with potential bugs or security issues.

### The Goal

Implement a `caro update` (or `caro --update`) command that:
1. Checks for new versions on GitHub releases
2. Downloads the appropriate binary for the user's platform
3. Verifies integrity (checksum/signature)
4. Replaces the current binary in-place
5. Reports success or failure clearly

---

## Decision Drivers

### Primary Drivers

1. **User Experience**: One-command updates reduce friction
2. **Security**: Users should be on latest security patches quickly
3. **Cross-Platform**: Must work on macOS (ARM64/x86_64), Linux, Windows
4. **GitHub Integration**: Our primary release channel is GitHub Releases
5. **Minimal Dependencies**: Caro already uses `reqwest` for remote backends

### Secondary Drivers

- Binary size impact (< 1MB increase acceptable)
- Build complexity (no new C dependencies)
- Maintenance burden (well-maintained upstream)
- Feature flag support (can be disabled for embedded builds)

---

## Crate Evaluation

### Primary Candidate: `self_update`

**Repository**: [github.com/jaemk/self_update](https://github.com/jaemk/self_update)

| Metric | Value |
|--------|-------|
| **Version** | 0.42.0 (Dec 31, 2024) |
| **Total Downloads** | 6.5M+ |
| **Recent Downloads** | 962K (last 90 days) |
| **GitHub Stars** | 910 |
| **Dependents** | 2,500+ |
| **License** | MIT |
| **Rust LOC** | 3,551 |
| **Last Updated** | Active (December 2024) |

#### Supported Backends

| Backend | Description | Caro Usage |
|---------|-------------|------------|
| **GitHub** | Fetches from `api.github.com/repos/{owner}/{repo}/releases` | **Primary** |
| **GitLab** | Equivalent functionality for GitLab instances | Not needed |
| **Amazon S3** | Cloud storage with bucket and prefix configuration | Future option |
| **Google GCS** | Cloud-based release distribution | Not needed |
| **DigitalOcean Spaces** | S3-compatible object storage | Not needed |

#### Key Features

```rust
// GitHub Release Update Pattern
self_update::backends::github::Update::configure()
    .repo_owner("wildcard")
    .repo_name("caro")
    .bin_name("caro")
    .show_download_progress(true)
    .show_output(true)
    .current_version(cargo_crate_version!())
    .build()?
    .update()?;
```

**Archive Support:**
- `archive-tar` - TAR format (used for Linux releases)
- `archive-zip` - ZIP format (used for Windows releases)

**Compression Support:**
- `compression-flate2` - gzip (`.tar.gz`)
- `compression-zip-deflate` - zip deflate
- `compression-zip-bzip2` - zip bzip2

**Security Features:**
- `signatures` - Ed25519 artifact verification via zipsign
- `rustls` - Pure Rust TLS (no OpenSSL dependency)
- Checksum validation on downloads

**Additional Features:**
- `ReleaseList` - Enumerate all available releases
- `Download` - Custom header support for private repos
- `Extract` - Archive decompression utilities
- Re-exports `self_replace` for in-place binary replacement

#### Feature Flag Configuration for Caro

```toml
[dependencies]
self_update = {
    version = "0.42",
    default-features = false,
    features = [
        "archive-tar",          # Linux releases
        "archive-zip",          # Windows releases
        "compression-flate2",   # gzip compression
        "rustls",               # Pure Rust TLS
    ],
    optional = true
}

[features]
self-update = ["self_update"]
full = ["remote-backends", "embedded-mlx", "embedded-cpu", "self-update"]
```

### Alternative Candidates

#### `self-replace` (v1.5.0)

**Repository**: [github.com/mitsuhiko/self-replace](https://github.com/mitsuhiko/self-replace)

| Metric | Value |
|--------|-------|
| **Downloads** | 6M+ |
| **Maintainer** | Armin Ronacher (@mitsuhiko) |
| **Purpose** | Low-level binary replacement only |

**Assessment**: `self-replace` is a **lower-level primitive** that handles the OS-specific complexities of replacing a running executable. It does NOT handle:
- Version checking
- Downloading releases
- GitHub/GitLab integration
- Archive extraction

**Verdict**: Not suitable as standalone solution. Note that `self_update` re-exports and uses `self-replace` internally.

#### `patchify`

**Repository**: [github.com/danwilliams/patchify](https://github.com/danwilliams/patchify)

| Metric | Value |
|--------|-------|
| **Architecture** | Client + Server components |
| **Platform Support** | Linux only (macOS untested, Windows not supported) |
| **Focus** | Self-hosted update server infrastructure |

**Features:**
- Autonomous update checking with configurable intervals
- Critical actions counter (defer updates during operations)
- Ed25519 signature verification
- SHA256 hash verification
- Status event broadcasting

**Assessment**: Patchify is designed for organizations running their own update infrastructure with a server component. It's overkill for Caro's GitHub-based release model and lacks cross-platform support.

**Verdict**: Not suitable. Too complex, limited platform support.

#### `cli-autoupdate`

**Repository**: [docs.rs/cli-autoupdate](https://docs.rs/cli-autoupdate)

| Metric | Value |
|--------|-------|
| **Ecosystem** | Less mature |
| **Documentation** | Limited |
| **Community** | Smaller |

**Assessment**: Less established than `self_update` with fewer features and smaller community.

**Verdict**: Not recommended over `self_update`.

#### `updater-lp`

**Assessment**: GitHub-focused like `self_update` but less mature and fewer downloads.

**Verdict**: `self_update` is the more established choice.

### Comparison Matrix

| Feature | self_update | self-replace | patchify | cli-autoupdate |
|---------|-------------|--------------|----------|----------------|
| **GitHub Releases** | ✅ Native | ❌ | ❌ | ✅ |
| **Version Checking** | ✅ | ❌ | ✅ | ✅ |
| **Archive Extraction** | ✅ | ❌ | ✅ | ⚠️ |
| **Signature Verification** | ✅ | ❌ | ✅ | ❌ |
| **macOS Support** | ✅ | ✅ | ⚠️ | ✅ |
| **Windows Support** | ✅ | ✅ | ❌ | ✅ |
| **Linux Support** | ✅ | ✅ | ✅ | ✅ |
| **Pure Rust TLS** | ✅ | N/A | ⚠️ | ⚠️ |
| **Active Maintenance** | ✅ | ✅ | ✅ | ⚠️ |
| **Downloads** | 6.5M+ | 6M+ | Low | Low |
| **Complexity** | Medium | Low | High | Low |

---

## Recommended Decision

**Use `self_update` crate (v0.42+) for implementing Caro's self-update mechanism.**

### Rationale

1. **Proven at Scale**: 6.5M+ downloads, 2,500+ dependents, battle-tested
2. **GitHub Native**: Built specifically for GitHub Releases workflow we already use
3. **Cross-Platform**: Handles macOS, Linux, Windows with platform-specific quirks
4. **Pure Rust**: `rustls` feature avoids OpenSSL dependency (aligns with Caro's approach)
5. **Security Ready**: Signature verification support for future hardening
6. **Minimal Footprint**: ~3,500 lines of Rust, modular feature flags
7. **Active Maintenance**: Updated December 2024, responsive maintainer

---

## Implementation Strategy

### Phase 1: Basic Update Command

```rust
// src/cli/update.rs
use self_update::backends::github::Update;
use crate::version;

pub async fn check_for_updates(auto_install: bool) -> Result<UpdateStatus> {
    let status = Update::configure()
        .repo_owner("wildcard")
        .repo_name("caro")
        .bin_name("caro")
        .show_download_progress(true)
        .no_confirm(auto_install)
        .current_version(version::info().version)
        .build()?
        .update()?;

    match status {
        self_update::Status::UpToDate(v) => {
            println!("Already running latest version: {}", v);
            Ok(UpdateStatus::Current)
        }
        self_update::Status::Updated(v) => {
            println!("Updated to version: {}", v);
            Ok(UpdateStatus::Updated(v))
        }
    }
}
```

### Phase 2: CLI Integration

```bash
# Check for updates (interactive)
caro update

# Check only, don't install
caro update --check

# Force update even if current
caro update --force

# Quiet mode for scripts
caro update --quiet
```

### Phase 3: Optional Startup Check (Future)

```rust
// Optional: Check for updates on startup (configurable)
if config.check_updates_on_startup {
    if let Some(new_version) = check_for_new_version().await? {
        eprintln!("New version {} available! Run 'caro update' to install.", new_version);
    }
}
```

### Release Artifact Naming Convention

`self_update` expects consistent artifact naming. Our GitHub releases should follow:

```
caro-{version}-{target}.tar.gz       # Linux/macOS
caro-{version}-{target}.zip          # Windows

Examples:
caro-1.0.4-aarch64-apple-darwin.tar.gz
caro-1.0.4-x86_64-unknown-linux-gnu.tar.gz
caro-1.0.4-x86_64-pc-windows-msvc.zip
```

### Build Type Awareness

The update mechanism should respect build type from `version.rs`:

| Build Type | Update Behavior |
|------------|-----------------|
| `binary (official release)` | Full self-update support |
| `source (cargo install)` | Suggest `cargo install caro --force` |
| `dev (local build)` | Disable updates, show warning |

```rust
pub fn can_self_update() -> bool {
    matches!(version::info().build_type(), "binary (official release)")
}
```

---

## Security Considerations

### Current Release Security Model

Caro's existing release process (documented in `RELEASE_PROCESS.md`) provides:

1. **GPG-signed tags** for releases
2. **SHA256 checksums** in release notes
3. **CI-based builds** (no local machine builds)
4. **Token-protected publishing** to crates.io
5. **Multi-step verification** before release

### Self-Update Security Enhancements

#### Immediate (Phase 1)

- **HTTPS-only** downloads via `rustls`
- **Version comparison** to prevent downgrades
- **GitHub API verification** (authenticated release data)
- **User confirmation** before replacing binary

#### Future (Phase 2)

- **Ed25519 signatures** via `zipsign` integration
- **Checksum verification** against release notes
- **Certificate pinning** for GitHub API

#### Implementation Example

```rust
// Prevent downgrade attacks
fn is_valid_upgrade(current: &str, new: &str) -> bool {
    let current = semver::Version::parse(current).ok()?;
    let new = semver::Version::parse(new).ok()?;
    new > current
}

// Verify download integrity
fn verify_checksum(data: &[u8], expected: &str) -> bool {
    use sha2::{Sha256, Digest};
    let hash = hex::encode(Sha256::digest(data));
    hash == expected
}
```

### Risk Mitigation

| Risk | Mitigation |
|------|------------|
| **Compromised GitHub release** | Ed25519 signatures (future), checksum verification |
| **MITM attack** | HTTPS/TLS only, rustls (no OpenSSL) |
| **Downgrade attack** | Semantic version comparison |
| **Partial download** | Checksum validation before replacement |
| **Interrupted update** | `self-replace` handles atomic replacement |

---

## Consequences

### Benefits

1. **Improved UX**: Users can update with single command
2. **Security Velocity**: Security patches reach users faster
3. **Reduced Support**: Fewer "works on latest version" issues
4. **Version Visibility**: Users know when updates are available
5. **Ecosystem Alignment**: Follows patterns from rustup, cargo

### Trade-offs

1. **Binary Size**: ~500KB-1MB increase (acceptable)
2. **New Dependency**: `self_update` crate + transitive deps
3. **Feature Complexity**: Need to handle build type awareness
4. **Testing Burden**: Need integration tests for update flow

### Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| `self_update` becomes unmaintained | Low | Medium | Fork if needed, active community |
| GitHub API rate limits | Low | Low | Cache version checks |
| Platform-specific bugs | Medium | Low | Thorough cross-platform testing |
| Update mechanism exploited | Very Low | High | Signature verification in Phase 2 |

---

## Alternatives Considered

### Alternative 1: No Self-Update

**Description**: Users continue using `cargo install` or manual downloads.

**Pros:**
- No additional complexity
- No new dependencies
- No security surface area increase

**Cons:**
- Poor user experience
- Slower security patch adoption
- More support burden

**Verdict**: Not recommended - UX is important for CLI tools.

### Alternative 2: Custom Implementation

**Description**: Build update mechanism from scratch using `reqwest` + `self-replace`.

**Pros:**
- Full control over implementation
- No external dependency on `self_update`
- Exactly what we need, nothing more

**Cons:**
- Significant development effort
- Reinventing well-tested wheel
- Maintenance burden on our team
- Likely to have bugs in edge cases

**Verdict**: Not recommended - `self_update` is mature and well-tested.

### Alternative 3: Patchify with Self-Hosted Server

**Description**: Deploy update server infrastructure using `patchify`.

**Pros:**
- More control over update distribution
- Could support delta updates
- Signature verification built-in

**Cons:**
- Requires server infrastructure
- Complex for single CLI tool
- No macOS/Windows support
- Overkill for our scale

**Verdict**: Not recommended - too complex for our needs.

---

## Implementation Notes

### Feature Flag

```toml
[features]
default = ["embedded-mlx", "embedded-cpu"]
self-update = ["dep:self_update"]
full = ["remote-backends", "embedded-mlx", "embedded-cpu", "self-update"]
```

### Conditional Compilation

```rust
#[cfg(feature = "self-update")]
mod update;

#[cfg(feature = "self-update")]
pub use update::check_for_updates;
```

### Testing Strategy

1. **Unit Tests**: Version comparison, build type detection
2. **Integration Tests**: Mock GitHub API responses
3. **Manual Testing**: Cross-platform update flows
4. **CI Verification**: Ensure release artifacts match naming convention

### Rollout Plan

1. **v1.1.0**: Basic update check command (`caro update --check`)
2. **v1.2.0**: Full self-update functionality
3. **v1.3.0**: Optional startup version check
4. **v2.0.0**: Ed25519 signature verification

---

## Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Update command works | 100% | Cross-platform CI tests |
| User adoption | 30%+ use update command | Telemetry (optional) |
| Time to update | < 30 seconds | Manual benchmarks |
| Update failure rate | < 1% | Error tracking |
| Security incidents | 0 | Incident reports |

---

## References

### Crates

- [self_update on crates.io](https://crates.io/crates/self_update)
- [self_update on GitHub](https://github.com/jaemk/self_update)
- [self-replace on crates.io](https://crates.io/crates/self-replace)
- [patchify on GitHub](https://github.com/danwilliams/patchify)

### Related Caro Documents

- [Release Process](../RELEASE_PROCESS.md)
- [ADR-001: LLM Inference Architecture](./001-llm-inference-architecture.md)
- [Version Module](../../src/version.rs)

### Prior Art

- [rustup self-update](https://github.com/rust-lang/rustup/blob/master/src/cli/self_update.rs)
- [cargo-binstall](https://github.com/cargo-bins/cargo-binstall)

### Discussions

- [Hacker News: self_update announcement](https://news.ycombinator.com/item?id=22182728)
- [Rust Forum: How to auto-update apps](https://users.rust-lang.org/t/how-to-auto-update-my-app/43100)

---

## Revision History

| Date | Author | Changes |
|------|--------|---------|
| 2026-01-02 | Caro Maintainers | Initial draft |

---

*This ADR was authored in January 2026 and recommends `self_update` v0.42+ based on the Rust ecosystem state at that time.*
