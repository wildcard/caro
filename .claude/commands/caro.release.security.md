---
description: Run security audit and fix vulnerabilities for caro release
---

**Path reference rule:** When you mention directories or files, provide either the absolute path or a path relative to the project root (for example, `Cargo.toml`). Never refer to a folder by name alone.

## User Input

```text
$ARGUMENTS
```

You **MUST** consider the user input before proceeding (if not empty).

---

## Branch Pre-flight Check (MANDATORY)

**This command enforces the release branch workflow per docs/RELEASE_PROCESS.md.**

Run these checks:
```bash
CURRENT_BRANCH=$(git branch --show-current)
```

**If NOT on a release branch** (`release/v*` or `hotfix/v*`):
- REFUSE to proceed with error:
  "ERROR: Must be on a release branch. Current: $CURRENT_BRANCH
  Run /caro.release.prepare first."

---

## Workflow Context

**Before this**: `/caro.release.prepare` created the release branch

**This command**: Runs security audit and fixes vulnerabilities

**After this**: Run `/caro.release.version` to bump version and update changelog

---

## Outline

### 1. Install/Verify cargo-audit

Check if cargo-audit is installed:
```bash
which cargo-audit || cargo install cargo-audit
```

Wait for installation to complete if needed.

### 2. Run Security Audit

Execute cargo audit and capture output:
```bash
cargo audit
```

### 3. Parse and Categorize Vulnerabilities

Analyze the output and categorize each warning:

**Critical/Unsound** (MUST FIX):
- RUSTSEC advisories with "unsound" or vulnerability severity high/critical
- Direct dependencies with known exploits
- Example: atty unaligned read vulnerability (RUSTSEC-2021-0145)

**Unmaintained (Direct Dependency)**:
- Direct dependencies marked as unmaintained
- Should update to maintained alternative if available
- Example: yaml-rust → yaml-rust2

**Unmaintained (Indirect/Transitive)**:
- Indirect dependencies via upstream crates
- Document as known issue, monitor upstream
- Example: number_prefix via hf-hub → indicatif

### 4. Fix Critical Vulnerabilities

For each critical/unsound vulnerability:

1. **Research the fix**:
   - Check if there's a replacement crate (e.g., atty → std::io::IsTerminal)
   - Check if updating the version fixes it
   - Check advisory URL for recommendations

2. **Apply the fix**:
   - Update `Cargo.toml` if version update needed
   - Modify source code if API changed
   - Remove dependency if replacement available

3. **Verify the fix**:
   ```bash
   cargo check
   ```

### 5. Update Direct Dependencies

For unmaintained direct dependencies:

1. Check for maintained alternatives:
   ```bash
   cargo search <crate-name>
   ```

2. Update `Cargo.toml`:
   - Change crate name if switching to alternative
   - Update version to latest

3. Verify compilation:
   ```bash
   cargo check
   ```

### 6. Document Remaining Warnings

For unmaintained indirect dependencies:

1. List them with context:
   - Crate name and version
   - Which direct dependency pulls it in
   - Why it's low priority (e.g., "proc macro, no runtime impact")

2. Add to commit message for transparency

### 7. Run Tests

Verify all fixes pass tests:
```bash
cargo test --lib
cargo test --test '*'
```

**If tests fail**:
- Review and fix test failures
- Re-run until all pass

### 8. Commit Security Fixes

Create a well-documented commit:
```bash
git add Cargo.toml Cargo.lock src/
git commit -m "$(cat <<'EOF'
security: Fix Dependabot vulnerabilities and update dependencies

Fixed N security warnings identified by cargo audit.

**Critical Fixes:**

1. <crate-name> X.Y.Z → <fix description>
   - RUSTSEC-XXXX-YYYY: <vulnerability description>
   - Fix: <what was changed>

**Dependency Updates:**

2. <crate> X.Y → X.Z
   - Fixed: RUSTSEC advisory
   - Impact: <description>

**Remaining Warnings (N):**

All remaining are unmaintained warnings from indirect dependencies:

1. <crate> - via <upstream> (low priority: <reason>)

**Testing:**

- ✅ All N unit tests pass
- ✅ All N integration tests pass

**Impact:**

- No breaking changes to public API
- Binary size and performance unchanged
- Follows security-first development practices per docs/RELEASE_PROCESS.md

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
EOF
)"
```

### 9. Output Summary

Display what was fixed:
```
✓ Security audit completed
✓ N critical vulnerabilities fixed
✓ N dependencies updated
✓ N warnings remaining (low priority)
✓ All tests passing
✓ Changes committed

Next step:
Run /caro.release.version to bump version and update changelog
```

---

## References

- **Release Process**: `docs/RELEASE_PROCESS.md` (Security Audit section)
- **cargo-audit**: https://docs.rs/cargo-audit/latest/cargo_audit/
- **RustSec Database**: https://rustsec.org/advisories/
