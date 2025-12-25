---
description: Verify published caro release on crates.io
---

**Path reference rule:** When you mention directories or files, provide either the absolute path or a path relative to the project root (for example, `Cargo.toml`). Never refer to a folder by name alone.

## User Input

```text
$ARGUMENTS
```

You **MUST** consider the user input before proceeding (if not empty).

**If user provides version** (e.g., "v1.0.0" or "1.0.0"):
- Use that specific version for verification
- Otherwise, detect latest version from git tags

---

## Branch Pre-flight Check (OPTIONAL)

**This command does NOT enforce branch requirements.**

You can run this command from any branch to verify a published release.

---

## Workflow Context

**Before this**: `/caro.release.publish` tagged and published the release

**This command**: Verifies the release is installable and functional

**After this**: Release workflow is complete

---

## Outline

### 1. Determine Version to Verify

**If user provided version via $ARGUMENTS**:
- Parse version (strip 'v' prefix if present)
- Use that version

**Otherwise, get latest tag**:
```bash
LATEST_TAG=$(git describe --tags --abbrev=0 2>/dev/null)
VERSION=${LATEST_TAG#v}
echo "Verifying latest release: v$VERSION"
```

**If no version found**:
- Display error: "ERROR: No version specified and no git tags found"
- Exit with instructions: "Usage: /caro.release.verify [version]"

### 2. Wait for crates.io Index Update

**Display waiting message**:
```
Waiting for crates.io index to update...
This usually takes 1-2 minutes after tag push.
```

**Poll crates.io for the version**:
```bash
echo "Checking crates.io availability..."
for i in {1..20}; do
  PUBLISHED_VERSION=$(curl -s https://crates.io/api/v1/crates/caro | jq -r .crate.newest_version)

  if [ "$PUBLISHED_VERSION" = "$VERSION" ]; then
    echo "✓ Version $VERSION found on crates.io"
    break
  fi

  echo "Attempt $i/20: crates.io shows $PUBLISHED_VERSION, waiting for $VERSION..."
  sleep 15
done

if [ "$PUBLISHED_VERSION" != "$VERSION" ]; then
  echo "WARNING: Version $VERSION not yet available on crates.io"
  echo "Current version: $PUBLISHED_VERSION"
  echo "This may indicate a publish failure. Check workflow logs."
  exit 1
fi
```

### 3. Backup Current Installation (if exists)

**Check if caro is already installed**:
```bash
if which caro > /dev/null 2>&1; then
  CURRENT_VERSION=$(caro --version 2>/dev/null || echo "unknown")
  echo "Current installation: $CURRENT_VERSION"
  echo "This will be replaced with v$VERSION from crates.io"
fi
```

### 4. Install from crates.io

**Install the specific version**:
```bash
echo "Installing caro v$VERSION from crates.io..."
cargo install caro --version "$VERSION" --force
```

**If installation fails**:
- Display error output
- Check common issues:
  - Network connectivity
  - Cargo registry access
  - Compilation errors
- Provide troubleshooting steps
- Exit with error

**Display installation path**:
```bash
which caro
```

### 5. Verify Version

**Check installed version matches**:
```bash
INSTALLED_VERSION=$(caro --version | grep -oE '[0-9]+\.[0-9]+\.[0-9]+')
echo "Installed version: $INSTALLED_VERSION"
echo "Expected version: $VERSION"
```

**If versions don't match**:
- Display error: "ERROR: Version mismatch!"
- Display: "Installed: $INSTALLED_VERSION"
- Display: "Expected: $VERSION"
- Possible causes:
  - crates.io index not updated
  - Cargo cache issue
  - Wrong version installed
- Exit with error

**If versions match**:
```
✓ Version verification passed: v$VERSION
```

### 6. Test Basic Functionality

Run basic functionality tests:

**Test 1: Help command**
```bash
echo "Test 1: Help command"
caro --help > /dev/null 2>&1
if [ $? -eq 0 ]; then
  echo "  ✓ Help command works"
else
  echo "  ✗ Help command failed"
  exit 1
fi
```

**Test 2: Version command**
```bash
echo "Test 2: Version command"
caro --version
if [ $? -eq 0 ]; then
  echo "  ✓ Version command works"
else
  echo "  ✗ Version command failed"
  exit 1
fi
```

**Test 3: Show config**
```bash
echo "Test 3: Show config"
caro --show-config > /dev/null 2>&1
if [ $? -eq 0 ]; then
  echo "  ✓ Config command works"
else
  echo "  ✗ Config command failed"
  exit 1
fi
```

**Test 4: Dry run with mock prompt**
```bash
echo "Test 4: Dry run command generation"
caro "list files in current directory" --dry-run --output plain > /dev/null 2>&1
if [ $? -eq 0 ]; then
  echo "  ✓ Dry run command works"
else
  echo "  ✗ Dry run command failed"
  exit 1
fi
```

**Test 5: JSON output format**
```bash
echo "Test 5: JSON output format"
OUTPUT=$(caro "echo hello" --dry-run --output json 2>/dev/null)
if echo "$OUTPUT" | jq . > /dev/null 2>&1; then
  echo "  ✓ JSON output format works"
else
  echo "  ✗ JSON output format failed"
  exit 1
fi
```

### 7. Verify GitHub Release

**Check GitHub release exists**:
```bash
echo "Verifying GitHub release..."
gh release view "v$VERSION" > /dev/null 2>&1
if [ $? -eq 0 ]; then
  echo "✓ GitHub release v$VERSION exists"
  gh release view "v$VERSION" --json tagName,name,publishedAt,url
else
  echo "✗ GitHub release v$VERSION not found"
  echo "This may indicate release workflow failure"
fi
```

### 8. Check Binary Size (if available in release)

**Optional**: If release includes binaries, check sizes:
```bash
echo "Checking release assets..."
gh release view "v$VERSION" --json assets -q '.assets[] | "\(.name): \(.size) bytes"'
```

**Verify binary size target** (< 50MB per project goals):
- If binaries present, check sizes
- Warn if any binary exceeds 50MB

### 9. Verify Checksums (if available)

**If release includes checksums**:
```bash
# Download and verify checksum files
gh release download "v$VERSION" --pattern "*.sha256" --dir /tmp/caro-verify

# Verify each checksum
for shafile in /tmp/caro-verify/*.sha256; do
  echo "Verifying $(basename $shafile)"
  # Verification logic
done

rm -rf /tmp/caro-verify
```

### 10. Test Documentation Links

**Verify key URLs are accessible**:
```bash
echo "Verifying documentation links..."

# crates.io page
curl -s -o /dev/null -w "%{http_code}" "https://crates.io/crates/caro" | grep -q "200" && echo "  ✓ crates.io page accessible" || echo "  ✗ crates.io page not accessible"

# GitHub release
curl -s -o /dev/null -w "%{http_code}" "https://github.com/wildcard/caro/releases/tag/v$VERSION" | grep -q "200" && echo "  ✓ GitHub release page accessible" || echo "  ✗ GitHub release page not accessible"

# Repository
curl -s -o /dev/null -w "%{http_code}" "https://github.com/wildcard/caro" | grep -q "200" && echo "  ✓ Repository accessible" || echo "  ✗ Repository not accessible"
```

### 11. Output Verification Summary

Display comprehensive summary:
```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Release Verification Summary: v$VERSION
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

✓ PUBLICATION
  • crates.io: Published and indexed
  • GitHub Release: Created successfully
  • Binary assets: Available (if applicable)

✓ INSTALLATION
  • cargo install: Successful
  • Version match: $VERSION
  • Installation path: [path]

✓ FUNCTIONALITY
  • Help command: Working
  • Version command: Working
  • Config command: Working
  • Dry run: Working
  • JSON output: Working

✓ DOCUMENTATION
  • crates.io page: Accessible
  • GitHub release: Accessible
  • Repository: Accessible

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Release v$VERSION verified successfully! 🎉
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Release URLs:
  • crates.io: https://crates.io/crates/caro/$VERSION
  • GitHub: https://github.com/wildcard/caro/releases/tag/v$VERSION
  • Documentation: https://github.com/wildcard/caro#readme

You can now announce the release! 📢
```

### 12. Optional: Cleanup Old Versions

**Ask user**:
"Do you want to uninstall previous versions? (y/n)"

If yes:
```bash
# List all installed caro binaries
# Offer to remove old ones
# Keep only the latest
```

---

## Troubleshooting

### Version Not Found on crates.io

**Possible causes**:
1. Publish workflow failed - Check `.github/workflows/publish.yml` logs
2. crates.io index not updated yet - Wait 5-10 minutes
3. Version already exists - Check for version conflicts
4. Invalid Cargo.toml - Verify package metadata

**Resolution**:
1. Check workflow logs: `gh run list --workflow=publish.yml --limit 1`
2. Manual publish if needed: `cargo publish` (requires maintainer credentials)
3. Verify version in Cargo.toml matches tag

### Installation Fails

**Possible causes**:
1. Compilation errors - Check dependencies
2. Platform incompatibility - Check target platform support
3. Network issues - Check cargo registry connectivity

**Resolution**:
1. Check error output for specific compilation errors
2. Try with verbose: `cargo install caro --version "$VERSION" --force --verbose`
3. Check platform support in README.md

### Functionality Tests Fail

**Possible causes**:
1. Binary incompatible with system
2. Missing dependencies (MLX framework, etc.)
3. Configuration issues

**Resolution**:
1. Check system requirements
2. Verify platform-specific dependencies
3. Review error messages for specific failures

---

## References

- **Release Process**: `docs/RELEASE_PROCESS.md` (Post-Release Verification section)
- **crates.io**: https://crates.io/crates/caro
- **GitHub Releases**: https://github.com/wildcard/caro/releases
