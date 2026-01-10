# Beta Testing Instructions - v1.1.0-beta.2

**Welcome to Beta.2!** Thank you for helping us test the critical P0 fixes for caro v1.1.0-beta.2.

This document provides complete instructions for:
1. Installing beta.2
2. Verifying P0 fixes from beta.1
3. Testing enhanced features
4. Providing feedback

---

## 🎯 Beta.2 Testing Goals

**Duration**: 3 days (focused regression testing)
**Your Mission**:
- **PRIMARY**: Verify all 5 P0 fixes from beta.1 are resolved
- Test command generation quality improvements (40% → 100%)
- Verify telemetry UX improvements
- Validate JSON output correctness
- General stability and workflow testing

**What's New in Beta.2**:
- ✅ **Issue #402 FIXED**: Telemetry prompt no longer spams every command
- ✅ **Issue #403 FIXED**: Telemetry can be disabled and stays disabled
- ✅ **Issue #404 FIXED**: JSON output is now spec-compliant
- ✅ **Issue #405 FIXED**: Documentation accuracy restored
- ✅ **Issue #406 FIXED**: Command quality 100% for File Management (was 40%)

---

## 📦 Installation

### Prerequisites

**Supported Platforms**:
- macOS Apple Silicon (M1/M2/M3) - Binary available
- macOS Intel - Build from source
- Linux x86_64 - Build from source
- Linux ARM64 - Build from source

**Requirements**:
- Terminal access
- Admin/sudo privileges for installation
- For building from source: Rust 1.83+

### Installation Steps

#### Option 1: Binary Download (macOS Apple Silicon Only)

```bash
# Download beta.2 binary
curl -L https://github.com/wildcard/caro/releases/download/v1.1.0-beta.2/caro-1.1.0-beta.2-macos-aarch64 -o caro

# Verify checksum (optional but recommended)
curl -L https://github.com/wildcard/caro/releases/download/v1.1.0-beta.2/caro-1.1.0-beta.2-macos-aarch64.sha256 -o caro.sha256
shasum -c caro.sha256

# Make executable
chmod +x caro

# Install to PATH
sudo mv caro /usr/local/bin/caro

# Verify installation
caro --version
# Expected output: caro 1.1.0-beta.2 (617a5ba 2026-01-09)
```

#### Option 2: Build from Source (All Platforms)

```bash
# Clone repository
git clone https://github.com/wildcard/caro
cd caro

# Checkout beta.2 tag
git checkout v1.1.0-beta.2

# Build release binary
cargo build --release

# The binary is at: ./target/release/caro
./target/release/caro --version

# Install to PATH (optional)
sudo cp target/release/caro /usr/local/bin/caro
```

### Upgrading from Beta.1

If you already have beta.1 installed:

```bash
# Backup your config (optional)
cp ~/.config/caro/config.toml ~/.config/caro/config.toml.backup

# Install beta.2 using instructions above

# Verify upgrade
caro --version  # Should show: caro 1.1.0-beta.2
```

**Note**: Your telemetry preference will be preserved from beta.1.

---

## 🔥 Priority 1: Verify P0 Fixes

### Test #1: Issue #402 - Telemetry Prompt Spam (CRITICAL)

**What was broken**: Telemetry consent prompt appeared on EVERY command invocation (28 lines, 2 second overhead).

**How to test**:
1. **Fresh install** (or delete `~/.config/caro/config.toml`):
   ```bash
   rm -f ~/.config/caro/config.toml
   ```

2. **First run** - You should see the telemetry consent prompt ONCE:
   ```bash
   caro "list files"
   # Expected: Telemetry consent prompt appears
   ```

3. **Second run** - Prompt should NOT appear:
   ```bash
   caro "show disk usage"
   # Expected: No telemetry prompt, just command output
   ```

4. **Third run** - Still no prompt:
   ```bash
   caro "find large files"
   # Expected: No telemetry prompt
   ```

**Success criteria**:
- [ ] Telemetry prompt appears ONCE on first run
- [ ] Prompt NEVER appears on subsequent runs
- [ ] No 2-second overhead after first run

### Test #2: Issue #403 - Telemetry Cannot Be Disabled (CRITICAL)

**What was broken**: Setting `telemetry.enabled = false` didn't persist; telemetry stayed enabled.

**How to test**:
1. **Disable telemetry**:
   ```bash
   caro config set telemetry.enabled false
   ```

2. **Verify disabled** (check config file):
   ```bash
   cat ~/.config/caro/config.toml
   # Expected: [telemetry] section shows enabled = false
   ```

3. **Run multiple commands** and verify telemetry stays disabled:
   ```bash
   caro "list files"
   caro "show disk usage"
   caro "find large files"
   ```

4. **Check config again** after multiple runs:
   ```bash
   cat ~/.config/caro/config.toml
   # Expected: Still shows enabled = false
   ```

**Success criteria**:
- [ ] Config file shows `telemetry.enabled = false` after disable command
- [ ] Setting persists across multiple command invocations
- [ ] Re-enabling with `caro config set telemetry.enabled true` also persists

### Test #3: Issue #404 - Invalid JSON Output (CRITICAL)

**What was broken**: `--output json` produced invalid JSON because telemetry prompt polluted stdout.

**How to test**:
1. **Fresh install** (trigger first-run state):
   ```bash
   rm -f ~/.config/caro/config.toml
   ```

2. **Run with JSON output on first run**:
   ```bash
   caro --output json "list files" | jq '.'
   ```
   **Expected**: Valid JSON output, no errors from jq

3. **Verify JSON structure**:
   ```bash
   caro --output json "show disk usage" | jq '.command'
   ```
   **Expected**: Should print the command string

4. **Test YAML output too**:
   ```bash
   caro --output yaml "find large files"
   ```
   **Expected**: Valid YAML output, no telemetry prompt

**Success criteria**:
- [ ] `--output json` produces valid JSON (jq parses successfully)
- [ ] No telemetry prompt appears in JSON/YAML output modes
- [ ] JSON structure is correct (has .command, .explanation, etc.)

### Test #4: Issue #405 - Documentation Mismatch (LOW PRIORITY)

**What was broken**: Beta testing instructions incorrectly stated `caro assess` and `caro telemetry` commands don't exist.

**How to test**:
1. **Verify assess command works**:
   ```bash
   caro assess
   # Expected: Shows system resources (CPU, GPU, RAM, recommendations)
   ```

2. **Verify telemetry command works**:
   ```bash
   caro telemetry status
   # Expected: Shows current telemetry configuration
   ```

**Success criteria**:
- [ ] `caro assess` command works and shows system info
- [ ] `caro telemetry status` command works
- [ ] No confusion about command availability

### Test #5: Issue #406 - Command Quality Below Target (HIGH PRIORITY)

**What was broken**: File Management category had 40% pass rate (2/5 tests passing).

**How to test** - Run these EXACT queries:

1. **"show disk space by directory"**:
   ```bash
   caro "show disk space by directory"
   ```
   **Expected**: `du -h -d 1` (macOS) or `du -h --max-depth=1` (Linux)
   **NOT**: `ls -lh` or other incorrect commands

2. **"find python files from last week"**:
   ```bash
   caro "find python files from last week"
   ```
   **Expected**: Must include both `*.py` AND `-mtime -7`
   Example: `find . -name "*.py" -type f -mtime -7`

3. **"list hidden files"**:
   ```bash
   caro "list hidden files"
   ```
   **Expected**: `ls -d .*`
   **NOT**: `ls -la` (that's all files, not just hidden)

4. **"find python files modified last week"** (backward compatibility):
   ```bash
   caro "find python files modified last week"
   ```
   **Expected**: Same as #2, must include `-mtime -7`

5. **"show disk space by directory sorted"** (specific variant):
   ```bash
   caro "show disk space by directory sorted"
   ```
   **Expected**: Must include both `du` AND `sort`
   Example: `du -h -d 1 | sort -hr`

**Success criteria**:
- [ ] All 5 queries generate correct commands (5/5 = 100%)
- [ ] No incorrect commands (ls -lh for disk space, find without -mtime, etc.)
- [ ] Commands are platform-appropriate (BSD vs GNU)

---

## 🧪 Regression Testing Checklist

After verifying P0 fixes, run general regression tests:

### Day 1: Installation & P0 Verification

**Installation**:
- [ ] Installation completed successfully
- [ ] `caro --version` shows `1.1.0-beta.2`
- [ ] No errors during installation

**P0 Fix Verification** (see detailed tests above):
- [ ] Test #1: Telemetry prompt spam fixed
- [ ] Test #2: Telemetry disable persists
- [ ] Test #3: JSON output is valid
- [ ] Test #4: Commands work as documented
- [ ] Test #5: Command quality 100% (5/5)

**Basic Functionality**:
- [ ] `caro "list files"` generates correct command
- [ ] `caro assess` shows system info
- [ ] `caro doctor` runs diagnostics
- [ ] `caro --help` displays help

### Day 2: Feature Stability

**Command Generation** (spot check various categories):
- [ ] `caro "find files modified today"`
- [ ] `caro "show top CPU processes"`
- [ ] `caro "git recent commits"`
- [ ] `caro "search for TODO in code"`

**Safety Validation** (should still work):
- [ ] `caro "delete everything"` → Blocked or warned
- [ ] `caro "chmod 777 everything"` → Blocked or warned
- [ ] `caro "list files"` → NOT blocked (no false positive)

**Telemetry Management**:
- [ ] Can enable telemetry: `caro config set telemetry.enabled true`
- [ ] Can disable telemetry: `caro config set telemetry.enabled false`
- [ ] Settings persist across restarts

### Day 3: Edge Cases & Final Validation

**Edge Cases**:
- [ ] Very long queries (50+ words)
- [ ] Queries with special characters
- [ ] Empty query: `caro ""`
- [ ] Complex multi-step commands

**Performance**:
- [ ] Commands generate in <1 second
- [ ] No memory leaks (run 20+ commands)
- [ ] No crashes or hangs

**Final P0 Re-verification**:
- [ ] Re-run all 5 P0 tests to confirm stability

---

## 📊 Feedback Collection

### What to Report

**High Priority** - Issues with P0 fixes:
- If ANY of the 5 P0 tests fail, report immediately
- Label as `beta.2-regression` on GitHub

**Medium Priority** - New bugs:
- Any new issues not present in beta.1
- Label as `beta.2` on GitHub

**Low Priority** - General feedback:
- Improvements and suggestions
- Post in GitHub Discussions

### Bug Report Template

```markdown
## Bug Description
[Clear description]

## Beta Version
v1.1.0-beta.2

## Is this a P0 Regression?
[ ] Yes - Related to Issues #402-406
[ ] No - New issue

## Steps to Reproduce
1.
2.
3.

## Expected vs Actual
- Expected: [what should happen]
- Actual: [what happened]

## Environment
- OS: [macOS 14.0, Ubuntu 22.04, etc.]
- Shell: [zsh, bash, fish]
- Installation method: [binary/source]
```

### Daily Check-In Format

**Day X/3 Check-In**:

```
✅ P0 Fixes Status:
- Issue #402 (Telemetry spam): [PASS/FAIL]
- Issue #403 (Disable persist): [PASS/FAIL]
- Issue #404 (JSON output): [PASS/FAIL]
- Issue #405 (Documentation): [PASS/FAIL]
- Issue #406 (Command quality): [X/5 PASS]

🐛 New Issues Found:
- [List any new bugs]

💭 Observations:
- [What's working well]
- [Any concerns]
```

---

## 🆘 Troubleshooting

### "Version still shows beta.1"

```bash
# Check which caro is running
which caro

# Check version
caro --version

# If wrong, remove old version
sudo rm $(which caro)

# Reinstall beta.2
```

### "Telemetry prompt still appears every time"

This would indicate Issue #402 is NOT fixed. Please:
1. Delete config: `rm ~/.config/caro/config.toml`
2. Run `caro "list files"` twice
3. If prompt appears both times, **REPORT IMMEDIATELY**
4. Include config file contents: `cat ~/.config/caro/config.toml`

### "JSON output still broken"

This would indicate Issue #404 is NOT fixed. Please:
1. Delete config: `rm ~/.config/caro/config.toml`
2. Run: `caro --output json "list files" | jq '.'`
3. If jq reports an error, **REPORT IMMEDIATELY**
4. Include full output (redirect to file if needed)

### Getting Help

**Channels**:
- GitHub Issues: https://github.com/wildcard/caro/issues
  - Use label: `beta.2`
  - Use label: `beta.2-regression` for P0 fix failures
- GitHub Discussions: https://github.com/wildcard/caro/discussions
- Email: beta@caro.sh
- Emergency: privacy@caro.sh

---

## 📋 Quick Reference

### Essential Commands

```bash
# Basic usage
caro "your natural language query"

# Version check
caro --version  # Should show: 1.1.0-beta.2

# System diagnostics
caro doctor
caro assess

# Telemetry management
caro config set telemetry.enabled false
caro config set telemetry.enabled true
caro telemetry status

# JSON/YAML output
caro --output json "list files"
caro --output yaml "show disk usage"
```

### P0 Test Quick Run

```bash
# Test #1: Telemetry spam fix
rm -f ~/.config/caro/config.toml
caro "list files"  # Should see prompt
caro "list files"  # Should NOT see prompt

# Test #2: Telemetry disable
caro config set telemetry.enabled false
cat ~/.config/caro/config.toml  # Should show enabled = false

# Test #3: JSON output
rm -f ~/.config/caro/config.toml
caro --output json "list files" | jq '.'  # Should be valid JSON

# Test #5: Command quality
caro "show disk space by directory"  # Should be: du -h -d 1
caro "find python files from last week"  # Must include: -mtime -7
caro "list hidden files"  # Should be: ls -d .*
```

### Success Criteria for Beta.2

**For Beta.2 to be Successful**:
- [ ] All 5 P0 tests pass (Issues #402-406 resolved)
- [ ] No P0 regressions (fixes don't break)
- [ ] <2 new P1 bugs found
- [ ] Command quality maintains 100% for File Management
- [ ] Average satisfaction ≥4.0/5.0

---

## 🙏 Thank You!

Your focused testing of these critical fixes is essential for GA readiness. Every P0 verification you complete brings us closer to a stable 1.1.0 release.

**Beta.2 Testing Perks**:
- ✅ First to validate critical fixes
- ✅ Direct impact on GA release quality
- ✅ Acknowledgment in release notes
- ✅ Priority access to GA release

**Questions?**
- Email: beta@caro.sh
- GitHub: https://github.com/wildcard/caro/discussions

Let's ship a rock-solid GA! 🚀

---

**Document Version**: 2.0
**Beta Version**: v1.1.0-beta.2
**Last Updated**: 2026-01-09
**Focus**: P0 Fix Verification + Regression Testing
