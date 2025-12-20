# Installation Package Manifests

This directory contains package manager manifests and installation scripts for distributing cmdai across multiple platforms and package managers.

## Files Overview

### Universal Install Script
- **install.sh** - One-line installer for macOS/Linux (Starship/fnm-style)
  - Usage: `curl -fsSL https://raw.githubusercontent.com/wildcard/cmdai/main/install.sh | bash`
  - Supports custom install directories
  - Automatic platform/architecture detection
  - Checksum verification
  - PATH detection and warnings

### Package Manager Manifests

#### macOS/Linux
- **homebrew-formula.rb** - Homebrew formula template
  - Multi-platform support (macOS Intel/ARM, Linux AMD64/ARM64)
  - Automatic SHA256 checksum updates during release
  - To use: Create `homebrew-tap` repository and place in `Formula/cmdai.rb`

#### Windows
- **scoop-manifest.json** - Scoop bucket manifest
  - Auto-update capability
  - Checksum verification
  - To use: Add to `scoop-bucket` repository

- **chocolatey-package.nuspec** - Chocolatey package specification
- **chocolatey-install.ps1** - Chocolatey installation script
  - To use: Submit to Chocolatey community repository

- **winget-manifest.yaml** - Windows Package Manager manifest
  - To use: Submit PR to `microsoft/winget-pkgs`

## Release Workflow Integration

The GitHub Actions release workflow (`.github/workflows/release.yml`) automatically:

1. Builds binaries for all platforms
2. Generates SHA256 checksums
3. Updates version numbers in all manifests
4. Replaces placeholder checksums with actual values
5. Creates an `installers.tar.gz` archive
6. Uploads package manifests as release artifacts
7. Generates `INSTALL_METHODS.md` with all installation options

## Setting Up Package Manager Distribution

### 1. Homebrew Tap (Recommended)

Create a new repository: `wildcard/homebrew-tap`

```bash
git clone https://github.com/wildcard/homebrew-tap
cd homebrew-tap
mkdir -p Formula
cp .install/homebrew-formula.rb Formula/cmdai.rb
git add Formula/cmdai.rb
git commit -m "Add cmdai formula"
git push
```

Users can then install with:
```bash
brew install wildcard/tap/cmdai
```

Or add the tap first:
```bash
brew tap wildcard/tap
brew install cmdai
```

### 2. Scoop Bucket (Windows)

Create a new repository: `wildcard/scoop-bucket`

```bash
git clone https://github.com/wildcard/scoop-bucket
cd scoop-bucket
cp .install/scoop-manifest.json cmdai.json
git add cmdai.json
git commit -m "Add cmdai manifest"
git push
```

Users can install with:
```bash
scoop bucket add wildcard https://github.com/wildcard/scoop-bucket
scoop install cmdai
```

### 3. Chocolatey (Windows)

1. Download the generated manifest from release artifacts
2. Extract `chocolatey-package.nuspec` and `chocolatey-install.ps1`
3. Create a `tools` directory and place `chocolatey-install.ps1` inside
4. Package: `choco pack`
5. Submit to Chocolatey community: `choco push cmdai.{version}.nupkg --source https://push.chocolatey.org/`

### 4. Winget (Windows)

1. Fork `microsoft/winget-pkgs`
2. Create directory: `manifests/w/wildcard/cmdai/{version}/`
3. Copy the generated `winget-manifest.yaml`
4. Submit PR to `microsoft/winget-pkgs`

### 5. Cargo (crates.io)

The release workflow automatically publishes to crates.io when:
- `CARGO_TOKEN` secret is configured
- Package version hasn't been published yet

Users can install with:
```bash
cargo install cmdai
```

## Installation Methods Summary

| Method | Platform | Command |
|--------|----------|---------|
| One-line script | macOS/Linux | `curl -fsSL https://raw.githubusercontent.com/wildcard/cmdai/main/install.sh \| bash` |
| Homebrew | macOS/Linux | `brew install wildcard/tap/cmdai` |
| Cargo | All | `cargo install cmdai` |
| Scoop | Windows | `scoop install cmdai` |
| Chocolatey | Windows | `choco install cmdai` |
| Winget | Windows | `winget install wildcard.cmdai` |
| Manual | All | Download from GitHub Releases |

## Updating Manifests

Manifests are automatically updated during the release process. To manually update:

1. Modify the template files in `.install/`
2. Version numbers and checksums use placeholders:
   - `0.1.0` for version (replaced with actual version)
   - `PLACEHOLDER_*_SHA256` for checksums (replaced with actual SHA256)
3. The release workflow will replace these during release

## Testing Installation Methods

### Quick Install Script
```bash
# Test with custom directory
curl -fsSL https://raw.githubusercontent.com/wildcard/cmdai/main/install.sh | bash -s -- --install-dir ~/.local/bin --no-sudo
```

### Homebrew (Local Testing)
```bash
brew install --build-from-source Formula/cmdai.rb
```

### Scoop (Local Testing)
```powershell
scoop install cmdai.json
```

## Distribution Checklist

When preparing for first release:

- [ ] Create `homebrew-tap` repository
- [ ] Create `scoop-bucket` repository
- [ ] Configure `CARGO_TOKEN` in GitHub secrets
- [ ] Test install.sh on macOS and Linux
- [ ] Submit Chocolatey package
- [ ] Submit Winget manifest PR
- [ ] Update README.md with installation instructions
- [ ] Test all installation methods

## References

This installation pipeline is inspired by:
- [Starship](https://starship.rs/installing/) - Cross-shell prompt
- [fnm](https://github.com/Schniz/fnm) - Fast Node.js version manager

Both projects demonstrate best practices for:
- Multi-platform binary distribution
- Package manager integration
- User-friendly installation experience
- Comprehensive platform support
