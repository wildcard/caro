# Distribution and Installation Pipeline Setup Guide

This document provides step-by-step instructions for maintainers to set up the complete distribution pipeline for cmdai.

## Overview

cmdai now supports multiple installation methods inspired by best-in-class tools like [Starship](https://starship.rs) and [fnm](https://github.com/Schniz/fnm):

- **One-line installer** for macOS/Linux
- **Package managers**: Homebrew, Cargo, Scoop, Chocolatey, Winget
- **Manual downloads** with pre-built binaries
- **Automated release workflow** that generates all package manifests

## Setup Checklist

### 1. GitHub Repository Setup

- [x] Release workflow configured (`.github/workflows/release.yml`)
- [x] Multi-platform binary builds
- [x] Automatic checksum generation
- [x] Installer manifest generation
- [ ] Configure `CARGO_TOKEN` secret for crates.io publishing

### 2. Create Homebrew Tap

Create a new repository: `wildcard/homebrew-tap`

```bash
# Create the tap repository
gh repo create wildcard/homebrew-tap --public --description "Homebrew tap for cmdai"

# Clone and set up
git clone https://github.com/wildcard/homebrew-tap
cd homebrew-tap
mkdir -p Formula

# Copy the formula template (will be auto-updated by release workflow)
cp /path/to/cmdai/.install/homebrew-formula.rb Formula/cmdai.rb

# Create README
cat > README.md << 'EOF'
# Homebrew Tap for cmdai

## Installation

```bash
brew install wildcard/tap/cmdai
```

Or:

```bash
brew tap wildcard/tap
brew install cmdai
```

## Available Formulae

- **cmdai** - Convert natural language to safe POSIX shell commands using local LLMs
EOF

git add .
git commit -m "Initial tap setup with cmdai formula"
git push
```

**Post-Release Process:**
After each cmdai release, update the formula:
1. Download `installers.tar.gz` from the release
2. Extract and copy `homebrew-formula.rb` to `Formula/cmdai.rb`
3. Commit and push: `git commit -am "Update cmdai to v${VERSION}" && git push`

### 3. Create Scoop Bucket

Create a new repository: `wildcard/scoop-bucket`

```bash
# Create the bucket repository
gh repo create wildcard/scoop-bucket --public --description "Scoop bucket for cmdai"

# Clone and set up
git clone https://github.com/wildcard/scoop-bucket
cd scoop-bucket

# Copy the manifest template
cp /path/to/cmdai/.install/scoop-manifest.json cmdai.json

# Create README
cat > README.md << 'EOF'
# Scoop Bucket for cmdai

## Installation

```powershell
scoop bucket add wildcard https://github.com/wildcard/scoop-bucket
scoop install cmdai
```

## Available Apps

- **cmdai** - Convert natural language to safe POSIX shell commands using local LLMs
EOF

git add .
git commit -m "Initial bucket setup with cmdai manifest"
git push
```

**Post-Release Process:**
After each cmdai release:
1. Download `installers.tar.gz` from the release
2. Extract and copy `scoop-manifest.json` to `cmdai.json`
3. Commit and push: `git commit -am "Update cmdai to v${VERSION}" && git push`

### 4. Submit to Chocolatey

**One-time setup:**
1. Create account at https://community.chocolatey.org/account/Register
2. Generate API key from your profile
3. Save API key: `choco apikey --key YOUR_KEY --source https://push.chocolatey.org/`

**For each release:**
```bash
# Download installers.tar.gz from release
tar -xzf installers.tar.gz

# Create package structure
mkdir -p cmdai-choco/tools
cp .install/chocolatey-install.ps1 cmdai-choco/tools/
cp .install/chocolatey-package.nuspec cmdai-choco/

# Package and push
cd cmdai-choco
choco pack
choco push cmdai.{version}.nupkg --source https://push.chocolatey.org/

# Wait for moderation approval (first package only)
```

**Automated Publishing (Future):**
Consider adding to release workflow with `CHOCO_API_KEY` secret.

### 5. Submit to Winget

**For each release:**
1. Fork https://github.com/microsoft/winget-pkgs
2. Download `installers.tar.gz` from cmdai release
3. Extract `winget-manifest.yaml`
4. Create directory structure:
   ```bash
   mkdir -p manifests/w/wildcard/cmdai/{version}
   cp .install/winget-manifest.yaml manifests/w/wildcard/cmdai/{version}/wildcard.cmdai.yaml
   ```
5. Commit and create PR to microsoft/winget-pkgs
6. Wait for automated validation and merge

**Tools:**
Use `wingetcreate` to simplify this:
```powershell
winget install Microsoft.WingetCreate
wingetcreate new https://github.com/wildcard/cmdai/releases/download/v{version}/cmdai-windows-amd64.exe
```

### 6. Publish to crates.io

**One-time setup:**
1. Create account at https://crates.io
2. Generate API token from account settings
3. Add to GitHub secrets: `CARGO_TOKEN`

**Automated Publishing:**
The release workflow automatically publishes to crates.io when a version tag is pushed.

**Manual Publishing:**
```bash
cargo login
cargo publish
```

### 7. Update Main Repository

After setting up tap/bucket repositories, update installation URLs:

**In `install.sh`:**
```bash
# Already uses main repository releases - no changes needed
```

**In `INSTALL.md`:**
```bash
# URLs are already correct - no changes needed
```

**In `README.md`:**
Verify the installation commands reference the correct tap/bucket names.

## Release Process

### Creating a New Release

1. **Update Version:**
   ```bash
   # Update Cargo.toml
   vim Cargo.toml  # Change version = "0.1.0" to "0.2.0"

   # Commit version bump
   git add Cargo.toml
   git commit -m "Bump version to 0.2.0"
   git push
   ```

2. **Create and Push Tag:**
   ```bash
   git tag -a v0.2.0 -m "Release v0.2.0"
   git push origin v0.2.0
   ```

3. **Automated Workflow:**
   The release workflow will automatically:
   - Create GitHub release
   - Build binaries for all platforms
   - Generate checksums
   - Update package manager manifests
   - Upload all artifacts
   - Publish to crates.io

4. **Update Package Managers:**
   - **Homebrew**: Download `installers.tar.gz`, extract formula, update tap repo
   - **Scoop**: Download `installers.tar.gz`, extract manifest, update bucket repo
   - **Chocolatey**: Download installers, package, and push
   - **Winget**: Create PR to microsoft/winget-pkgs with updated manifest

### Testing Releases

Before creating the tag:

```bash
# Test the workflow locally (requires act)
act -j create-release
act -j build-and-upload
act -j generate-installers

# Or create a release candidate tag
git tag -a v0.2.0-rc1 -m "Release candidate 0.2.0-rc1"
git push origin v0.2.0-rc1
```

## Monitoring and Maintenance

### Package Manager Status Dashboard

Track installation sources:

| Source | Status | Auto-Update | Maintainer Action Required |
|--------|--------|-------------|----------------------------|
| GitHub Releases | ✅ Automated | Yes | None |
| crates.io | ✅ Automated | Yes | Verify `CARGO_TOKEN` secret |
| Homebrew | 🔄 Semi-automated | No | Update tap after release |
| Scoop | 🔄 Semi-automated | Manifest supports | Update bucket after release |
| Chocolatey | ⚠️ Manual | No | Package and push after release |
| Winget | ⚠️ Manual | No | Create PR after release |

### Analytics and Downloads

Monitor distribution:
- GitHub release downloads
- crates.io statistics: https://crates.io/crates/cmdai/stats
- Homebrew analytics (if enabled)

### User Support

Common installation issues:

1. **curl | bash concerns**: Point users to INSTALL.md for manual download
2. **Homebrew not finding formula**: Check tap repository is public and accessible
3. **Checksum mismatches**: Verify release artifacts weren't corrupted
4. **Platform not supported**: Check release workflow matrix includes the platform

## Future Enhancements

### Planned Improvements

- [ ] **Automated Homebrew tap updates** via GitHub Actions
- [ ] **Automated Scoop bucket updates** via GitHub Actions
- [ ] **Automated Chocolatey publishing** with API key in secrets
- [ ] **Shell completions** generation and distribution
- [ ] **APT repository** for Debian/Ubuntu
- [ ] **YUM repository** for RHEL/Fedora
- [ ] **AUR package** for Arch Linux
- [ ] **Nix package** for NixOS
- [ ] **Docker images** on Docker Hub
- [ ] **Snap package** for Linux
- [ ] **Flatpak package** for Linux

### Contribution Opportunities

Community members can help by:
- Maintaining package manager repositories (Homebrew tap, Scoop bucket)
- Creating packages for additional package managers (AUR, Nix, Snap)
- Improving installation documentation
- Testing installation methods on different platforms
- Reporting and fixing installation issues

## References

- [Starship Installation](https://starship.rs/installing/) - Inspiration for multi-platform distribution
- [fnm Installation](https://github.com/Schniz/fnm#installation) - Inspiration for package manager support
- [Homebrew Formula Cookbook](https://docs.brew.sh/Formula-Cookbook)
- [Scoop Manifest Reference](https://github.com/ScoopInstaller/Scoop/wiki/App-Manifests)
- [Chocolatey Package Creation](https://docs.chocolatey.org/en-us/create/create-packages)
- [Winget Package Manifest](https://docs.microsoft.com/en-us/windows/package-manager/package/)

## Support

For distribution/installation issues:
- Create issue: https://github.com/wildcard/cmdai/issues
- Tag with: `distribution`, `installation`
- Provide: OS, installation method, error output
