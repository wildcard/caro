# Self-Update Implementation Plan

## Overview

This document outlines the implementation strategy for adding self-update functionality to Caro using the `self_update` crate.

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         caro update                              │
└───────────────────────────────┬─────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Build Type Detection                          │
│  ┌─────────────────┬─────────────────┬─────────────────────┐    │
│  │ Official Binary │  Cargo Install  │    Dev Build        │    │
│  │ (CARO_RELEASE=1)│  (release mode) │  (debug/unknown)    │    │
│  └────────┬────────┴────────┬────────┴──────────┬──────────┘    │
│           │                 │                   │               │
│           ▼                 ▼                   ▼               │
│      Self-Update       Show cargo          Refuse with         │
│      Enabled           install hint        warning             │
└───────────────────────────────┬─────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                    GitHub Release Check                          │
│  • GET api.github.com/repos/wildcard/caro/releases/latest       │
│  • Parse version from tag (v1.0.5 → 1.0.5)                      │
│  • Compare with current version (semver)                        │
└───────────────────────────────┬─────────────────────────────────┘
                                │
                    ┌───────────┴───────────┐
                    │                       │
                    ▼                       ▼
            ┌───────────────┐       ┌───────────────┐
            │ Up-to-date    │       │ Update Avail  │
            │ Exit(0)       │       │ Continue      │
            └───────────────┘       └───────┬───────┘
                                            │
                                            ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Download Binary                               │
│  • Select asset by target triple                                 │
│  • Show progress bar with speed                                  │
│  • Write to temp file                                            │
└───────────────────────────────┬─────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Verify & Extract                              │
│  • Checksum verification (future)                                │
│  • Extract from tar.gz/zip                                       │
│  • Verify extracted binary exists                                │
└───────────────────────────────┬─────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Replace Binary                                │
│  • Use self-replace crate                                        │
│  • Atomic replacement                                            │
│  • Handle Windows special cases                                  │
└───────────────────────────────┬─────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Success                                       │
│  • Print new version info                                        │
│  • Suggest shell restart                                         │
│  • Exit(0)                                                       │
└─────────────────────────────────────────────────────────────────┘
```

## Module Structure

```
src/
├── cli/
│   ├── mod.rs              # Add UpdateArgs
│   └── update.rs           # NEW: Update command implementation
├── update/                 # NEW: Update module
│   ├── mod.rs             # Module exports
│   ├── checker.rs         # Version checking logic
│   ├── installer.rs       # Download and install logic
│   └── progress.rs        # Progress bar rendering
└── version.rs             # Existing, add can_self_update()
```

## Implementation Phases

### Phase 1: Foundation (v1.1.0)

**Goal**: Implement `caro update --check` and build type detection.

**Files to Create/Modify**:

1. **Cargo.toml**: Add optional `self_update` dependency
2. **src/cli/mod.rs**: Add `UpdateArgs` struct and subcommand
3. **src/update/mod.rs**: Module root
4. **src/update/checker.rs**: Version check logic
5. **src/version.rs**: Add `can_self_update()` method

**Key Code**:

```rust
// src/update/checker.rs
use self_update::backends::github::ReleaseList;

pub struct UpdateChecker {
    current_version: String,
}

impl UpdateChecker {
    pub fn new() -> Self {
        Self {
            current_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    pub async fn check_for_update(&self) -> Result<Option<String>> {
        let releases = ReleaseList::configure()
            .repo_owner("wildcard")
            .repo_name("caro")
            .build()?
            .fetch()?;

        if let Some(latest) = releases.first() {
            let latest_version = latest.version.trim_start_matches('v');
            if is_newer(latest_version, &self.current_version)? {
                return Ok(Some(latest_version.to_string()));
            }
        }
        Ok(None)
    }
}
```

### Phase 2: Self-Update (v1.2.0)

**Goal**: Full download and install functionality.

**Files to Create/Modify**:

1. **src/update/installer.rs**: Download and install logic
2. **src/update/progress.rs**: Progress bar with indicatif
3. **src/cli/update.rs**: Full update command handler

**Key Code**:

```rust
// src/update/installer.rs
use self_update::backends::github::Update;

pub async fn install_update(target_version: &str, force: bool) -> Result<()> {
    let status = Update::configure()
        .repo_owner("wildcard")
        .repo_name("caro")
        .bin_name("caro")
        .show_download_progress(true)
        .show_output(true)
        .no_confirm(false)
        .current_version(env!("CARGO_PKG_VERSION"))
        .build()?
        .update()?;

    match status {
        self_update::Status::Updated(v) => {
            println!("\nUpdated successfully!");
            println!("caro {} - restart your shell to use the new version", v);
        }
        self_update::Status::UpToDate(v) => {
            println!("\nAlready running the latest version: {}", v);
        }
    }

    Ok(())
}
```

### Phase 3: Polish (v1.3.0)

**Goal**: Quiet mode, optional startup check, better UX.

**Additions**:

1. **Config option**: `check_updates_on_startup: bool`
2. **Quiet mode**: `--quiet` flag for scripting
3. **Startup hint**: Show "update available" at startup (optional)

### Phase 4: Security (v2.0.0)

**Goal**: Signature verification.

**Additions**:

1. Enable `signatures` feature in self_update
2. Generate Ed25519 keypair for releases
3. Sign releases with `zipsign`
4. Verify signatures before install

## CLI Integration

### Clap Argument Definition

```rust
// src/cli/mod.rs

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Check for and install updates
    #[cfg(feature = "self-update")]
    Update(UpdateArgs),
    // ... existing commands
}

#[derive(Args)]
#[cfg(feature = "self-update")]
pub struct UpdateArgs {
    /// Check for updates without installing
    #[arg(short, long)]
    pub check: bool,

    /// Force reinstall even if up-to-date
    #[arg(short, long)]
    pub force: bool,

    /// Minimal output for scripts
    #[arg(short, long)]
    pub quiet: bool,

    /// Skip confirmation prompts
    #[arg(short, long)]
    pub yes: bool,
}
```

### Command Handler

```rust
// src/cli/update.rs

pub async fn handle_update(args: UpdateArgs) -> Result<i32> {
    // Check build type
    let version_info = crate::version::info();

    match version_info.build_type() {
        "dev (local build)" => {
            eprintln!("Self-update is disabled for development builds.");
            return Ok(5);
        }
        "source (cargo install)" => {
            eprintln!("You installed from source. To update, run:");
            eprintln!("    cargo install caro --force");
            return Ok(5);
        }
        _ => {} // Continue for binary releases
    }

    let checker = UpdateChecker::new();

    match checker.check_for_update().await? {
        Some(new_version) => {
            if args.check {
                if !args.quiet {
                    println!("Update available: {} -> {}",
                             version_info.version, new_version);
                } else {
                    println!("{} {}", version_info.version, new_version);
                }
                return Ok(1); // Update available
            }

            // Proceed with update
            install_update(&new_version, args.force).await?;
            Ok(0)
        }
        None => {
            if !args.quiet {
                println!("Already running the latest version: {}",
                         version_info.version);
            }
            Ok(0)
        }
    }
}
```

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_comparison() {
        assert!(is_newer("1.0.5", "1.0.3").unwrap());
        assert!(!is_newer("1.0.3", "1.0.5").unwrap());
        assert!(!is_newer("1.0.3", "1.0.3").unwrap());
    }

    #[test]
    fn test_build_type_detection() {
        // Test via version module
    }
}
```

### Integration Tests

```rust
// tests/update_integration.rs

#[tokio::test]
#[ignore] // Requires network
async fn test_check_for_update() {
    let checker = UpdateChecker::new();
    let result = checker.check_for_update().await;
    assert!(result.is_ok());
}
```

### Manual Testing Checklist

- [ ] macOS ARM64: Full update cycle
- [ ] macOS x86_64: Full update cycle
- [ ] Linux x86_64: Full update cycle
- [ ] Windows x86_64: Full update cycle
- [ ] Offline behavior: Clear error message
- [ ] Permission denied: Helpful message
- [ ] Already up-to-date: Correct output
- [ ] Force reinstall: Works correctly
- [ ] Quiet mode: Minimal output
- [ ] Dev build: Refuses with warning
- [ ] Source install: Shows cargo command

## Dependencies

### Added to Cargo.toml

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

[features]
self-update = ["dep:self_update"]
full = ["remote-backends", "embedded-mlx", "embedded-cpu", "self-update"]
```

### Transitive Dependencies

- `reqwest` (already in project via remote-backends)
- `flate2` (compression)
- `tar` (archive extraction)
- `zip` (Windows archives)
- `self-replace` (atomic binary replacement)
- `semver` (version comparison)

## Binary Size Impact

Estimated increase: **500KB - 1MB** with self_update features.

This is acceptable given the feature value.

## CI/CD Changes

### Release Workflow Updates

1. Ensure asset naming matches expected pattern
2. Add checksums to release notes
3. Future: Add signature generation

### Test Workflow

```yaml
# .github/workflows/test.yml
- name: Test self-update feature
  run: cargo test --features self-update
```

## Rollback Plan

If self-update causes issues:

1. Users can always `cargo install caro --force`
2. Previous releases remain on GitHub
3. Feature can be disabled in build

## Timeline

| Phase | Version | Estimated Effort |
|-------|---------|------------------|
| Phase 1: Foundation | v1.1.0 | 2-3 days |
| Phase 2: Self-Update | v1.2.0 | 3-4 days |
| Phase 3: Polish | v1.3.0 | 2-3 days |
| Phase 4: Security | v2.0.0 | 4-5 days |

---

*Implementation plan created January 2026*
