# Release Process

This document describes the security-controlled release process for caro. Given that caro generates and executes shell commands, we maintain strict security controls similar to BSD and GNU projects to ensure user trust and safety.

## Security Philosophy

**caro is a security-critical tool** that translates natural language into executable shell commands. A compromised release could lead to arbitrary command execution on user systems. We therefore:

- Limit release authority to verified maintainers only
- Require multi-step verification before publishing
- Maintain transparent changelog and audit trail
- Follow defense-in-depth principles
- Prioritize safety over feature velocity

## Release Authority

### Who Can Release

Only **verified maintainers** with GPG-signed commits and proven track record can trigger releases:

1. **Core Maintainers** (current: @wildcard)
   - Full release authority
   - crates.io owner status
   - GitHub repository admin access
   - GPG key on file

2. **Trusted Contributors**
   - Can propose releases via PR
   - Cannot directly publish to crates.io
   - Require core maintainer approval

### Access Controls

#### GitHub Repository Settings

**Required branch protection on `main`**:
- ✅ Require pull request reviews (minimum 1 for trusted contributors, 2+ for security changes)
- ✅ Require status checks to pass (CI, tests, clippy, security audit)
- ✅ Require signed commits for releases
- ✅ Require linear history
- ✅ Include administrators in restrictions
- ✅ Restrict who can push to matching branches (maintainers only)

**Tag protection**:
- Pattern: `v*.*.*`
- Only maintainers can create tags
- Tags are immutable once pushed

#### crates.io Permissions

**Token Management**:
- `CARGO_REGISTRY_TOKEN` stored in GitHub Secrets
- Token has `publish-update` scope only (not `publish-new`)
- Token rotated every 90 days
- Access limited to repository maintainers
- Token belongs to verified crates.io account with 2FA enabled

**Package Ownership**:
- Primary owner: Verified maintainer account
- Secondary owner: Project organization account (if applicable)
- Never add untrusted collaborators as owners

## Release Checklist

### Pre-Release (1-2 days before)

- [ ] **Security Audit**
  - Run `cargo audit` and resolve all vulnerabilities
  - Review dependency updates for security patches
  - Check for known CVEs in dependencies
  - Review recent commits for security implications

- [ ] **Code Quality**
  - All CI checks passing on `main` branch
  - `cargo clippy -- -D warnings` passes
  - `cargo test --all-features` passes
  - `cargo fmt --check` passes
  - No outstanding critical bugs

- [ ] **Documentation**
  - README.md is up to date
  - CHANGELOG.md has complete release notes
  - API documentation is current (`cargo doc --no-deps`)
  - Installation instructions verified

- [ ] **Version Preparation**
  - Update version in `Cargo.toml`
  - Update version references in documentation
  - Commit with message: `chore: bump version to X.Y.Z`
  - Create PR for version bump
  - Get review and approval from another maintainer

### Release Execution

- [ ] **Create Release Tag**
  ```bash
  # Ensure you're on latest main
  git checkout main
  git pull origin main

  # Verify commit is signed
  git log --show-signature -1

  # Create annotated, signed tag
  git tag -s vX.Y.Z -m "Release vX.Y.Z"

  # Verify tag signature
  git tag -v vX.Y.Z

  # Push tag (triggers automated workflows)
  git push origin vX.Y.Z
  ```

- [ ] **Monitor Automated Workflows**
  - Watch `.github/workflows/publish.yml` execution
  - Verify all tests pass
  - Verify clippy and security checks pass
  - Confirm successful publish to crates.io
  - Monitor `.github/workflows/release.yml` for binary builds

- [ ] **Verify crates.io Publication**
  ```bash
  # Wait ~2 minutes for crates.io index update
  # Test installation from crates.io
  cargo install caro --force

  # Verify version
  caro --version
  # Should output: caro X.Y.Z

  # Basic functionality test
  caro "list files" --dry-run
  ```

- [ ] **Create GitHub Release**
  - Release workflow creates draft automatically
  - Review release notes
  - Attach checksums for binaries
  - Mark as "Latest release"
  - Publish release (not draft)

### Post-Release

- [ ] **Announcement**
  - Update project README.md with latest version
  - Post announcement to relevant channels
  - Update documentation sites

- [ ] **Verification**
  - Test installation on fresh systems (Linux, macOS, Windows)
  - Verify binaries work correctly
  - Monitor issue tracker for installation problems

- [ ] **Security Monitoring**
  - Monitor crates.io download statistics
  - Watch for reported security issues
  - Enable GitHub security advisories notifications

## Emergency Procedures

### Yanking a Release

If a security vulnerability is discovered in a published release:

1. **Immediate Action**
   ```bash
   # Yank the vulnerable version from crates.io
   cargo yank --version X.Y.Z caro
   ```

2. **Communication**
   - Create security advisory on GitHub
   - Post warning in README.md
   - Notify users via release notes
   - Coordinate disclosure timeline

3. **Fix and Release**
   - Create hotfix branch from vulnerable tag
   - Apply minimal fix for vulnerability
   - Follow expedited release process
   - Publish security patch release (X.Y.Z+1)

### Compromised Token

If `CARGO_REGISTRY_TOKEN` is compromised:

1. **Revoke immediately** on crates.io
2. **Rotate secret** in GitHub repository settings
3. **Review recent publishes** for unauthorized releases
4. **Yank suspicious versions** if found
5. **Audit access logs** for compromise extent
6. **Publish incident report** with timeline

## Version Numbering

We follow [Semantic Versioning 2.0.0](https://semver.org/):

- **MAJOR** (X.0.0): Breaking changes, major rewrites, incompatible API changes
- **MINOR** (x.Y.0): New features, backend additions, backward-compatible changes
- **PATCH** (x.y.Z): Bug fixes, security patches, dependency updates

**Security patches** may warrant PATCH or MINOR bumps depending on severity.

## Testing Requirements

### Before Any Release

**Automated Tests** (CI enforced):
```bash
# Unit tests
cargo test --all-features

# Integration tests
cargo test --test '*'

# Property tests
cargo test --release

# Clippy linting
cargo clippy --all-features -- -D warnings

# Format check
cargo fmt --check

# Security audit
cargo audit
```

**Manual Testing** (maintainer verification):
- Test on macOS (Apple Silicon and Intel)
- Test on Linux (Ubuntu/Debian)
- Test basic command generation workflow
- Test safety validation with dangerous commands
- Verify `--help` and `--version` output
- Test dry-run mode
- Test execution mode with confirmation

### Platform Coverage

**Tier 1** (must work):
- macOS ARM64 (Apple Silicon)
- macOS x86_64 (Intel)
- Linux x86_64 (GNU)

**Tier 2** (best effort):
- Linux ARM64
- Windows x86_64

## Rollback Procedure

If a release needs to be rolled back:

1. **Yank the version** from crates.io
2. **Remove GitHub release** (mark as draft or delete)
3. **Delete the git tag** locally and remotely
   ```bash
   git tag -d vX.Y.Z
   git push origin :refs/tags/vX.Y.Z
   ```
4. **Publish corrected version** (X.Y.Z+1)
5. **Document the incident** in CHANGELOG.md

## Security Best Practices

### For Maintainers

1. **Enable 2FA** on all accounts (GitHub, crates.io)
2. **Use GPG-signed commits** for all release-related work
3. **Never share tokens** or credentials
4. **Rotate tokens regularly** (every 90 days)
5. **Review dependencies** for supply chain attacks
6. **Audit CI/CD workflows** for injection vulnerabilities
7. **Use dedicated release environment** (not personal dev machine)

### For Contributors

1. **Never commit secrets** or tokens
2. **Sign your commits** (encouraged but not required for non-release)
3. **Report security issues privately** (see SECURITY.md)
4. **Review dependency changes** in your PRs
5. **Follow code review process** for all changes

## Release Cadence

- **Security patches**: As needed (expedited process)
- **Bug fixes**: Monthly or as needed
- **Feature releases**: Quarterly or when ready
- **Major versions**: Yearly or for breaking changes

**No fixed schedule** - we prioritize quality and security over velocity.

## Changelog Maintenance

Every release must have corresponding CHANGELOG.md entry with:

- **Added**: New features
- **Changed**: Changes in existing functionality
- **Deprecated**: Soon-to-be removed features
- **Removed**: Removed features
- **Fixed**: Bug fixes
- **Security**: Security improvements and vulnerability fixes

See [Keep a Changelog](https://keepachangelog.com/) for format.

## Questions or Issues

For questions about the release process:
- Open a discussion on GitHub
- Contact maintainers via security@[project-domain] for security concerns

**Remember**: We are responsible for the security of every user who runs caro. Take your time, verify each step, and never rush a release.
