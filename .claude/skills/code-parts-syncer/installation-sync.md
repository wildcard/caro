# Installation Sync Module

Synchronize installation instructions across documentation:
1. **Website** (Download.astro + version.ts) - **SOURCE OF TRUTH**
2. **README files** - Main project documentation
3. **Package READMEs** - npm, nuget, crates.io
4. **Skill documentation** - Claude Code skill docs
5. **Scripts** - setup.sh, install.sh headers

---

## Process

### Step 1: Fetch Ground Truth (Website)

**Extract current version**:
```bash
VERSION=$(grep "CARO_VERSION" website/src/config/version.ts | sed "s/.*'\\([^']*\\)'.*/\\1/")
```

**Extract canonical install command**:
```bash
INSTALL_CMD=$(grep -A 1 "data-command=" website/src/components/Download.astro | grep "data-command=" | sed 's/.*data-command="\\([^"]*\\)".*/\\1/')
```

Store results:
```
VERSION: 1.0.3
SCRIPT_URL: https://setup.caro.sh
CURL_OPTIONS: --proto '=https' --tlsv1.2 -sSfL
FULL_COMMAND: bash <(curl --proto '=https' --tlsv1.2 -sSfL https://setup.caro.sh)
CARGO_INSTALL: cargo install caro
CARGO_MLX: cargo install caro --features embedded-mlx
BINARY_FORMAT: caro-{VERSION}-{PLATFORM}
```

---

### Step 2: Read Current State from Target Files

**High Priority Files**:

```bash
# README.md - Extract version
grep -n "caro-[0-9]" README.md | head -5

# README.md - Find curl commands
grep -n "curl.*https://.*\.sh" README.md

# README.crates.io.md - Find curl commands
grep -n "curl.*https://.*\.sh" README.crates.io.md

# nuget/README.md - Extract version from paths
grep -n "caro-[0-9]" nuget/README.md
```

**Medium Priority Files**:

```bash
# docs/MACOS_SETUP.md - Check for one-liner
grep -n "curl.*setup.caro.sh" docs/MACOS_SETUP.md

# docs-site/src/content/docs/guides/macos-setup.md
grep -n "curl.*setup.caro.sh" docs-site/src/content/docs/guides/macos-setup.md

# npm/README.md - Check for setup.caro.sh
grep -n "setup.caro.sh" npm/README.md
```

**Low Priority Files**:

```bash
# Claude skill docs
grep -rn "curl.*https://.*\.sh" .claude/skills/caro-shell-helper/

# Release template
grep -n "install.sh\|setup.caro.sh" .github/RELEASE_TEMPLATE.md
```

---

### Step 3: Detect Drift

**Compare each file against ground truth**:

```
For each target file:
  IF version != GROUND_TRUTH_VERSION:
    → VERSION_DRIFT

  IF script_url contains "install.sh" (instead of "setup.caro.sh"):
    → SCRIPT_URL_DRIFT

  IF curl_options != "--proto '=https' --tlsv1.2 -sSfL":
    → CURL_OPTIONS_DRIFT

  IF binary format != "caro-{VERSION}-{PLATFORM}":
    → BINARY_FORMAT_DRIFT
```

**Report drift**:

```
================================================================================
Installation Instructions Drift Detection
================================================================================

Source of Truth: website/src/components/Download.astro + version.ts
  Version: 1.0.3
  Script URL: https://setup.caro.sh
  Curl Options: --proto '=https' --tlsv1.2 -sSfL
  Binary Format: caro-{version}-{platform}

Drift Status:

README.md (HIGH PRIORITY):
  Version: 1.0.2 ✗ DRIFT (expected: 1.0.3)
  Script URL: raw.githubusercontent.com/wildcard/caro/main/install.sh ✗ DRIFT
  Curl Options: -fsSL ✗ DRIFT (missing --proto '=https' --tlsv1.2)
  Binary URLs: caro-1.0.2-{platform} ✗ DRIFT
  Lines affected: 80-143

README.crates.io.md (HIGH PRIORITY):
  Version: N/A ✓ SYNCED
  Script URL: https://setup.caro.sh ✓ SYNCED
  Curl Options: --proto '=https' --tlsv1.2 -sSfL ✓ SYNCED
  Lines affected: None

nuget/README.md (HIGH PRIORITY):
  Version: 1.0.2 ✗ DRIFT (expected: 1.0.3)
  Lines affected: ~20

docs/MACOS_SETUP.md (MEDIUM PRIORITY):
  One-liner command: MISSING ⚠ (should add setup.caro.sh reference)

docs-site/src/content/docs/guides/macos-setup.md (MEDIUM PRIORITY):
  One-liner command: MISSING ⚠ (should add setup.caro.sh reference)

npm/README.md (MEDIUM PRIORITY):
  setup.caro.sh reference: MISSING ⚠

.claude/skills/caro-shell-helper/QUICK_START.md (LOW PRIORITY):
  Curl Options: -sSfL ✗ DRIFT (missing --proto '=https' --tlsv1.2)

.claude/skills/caro-shell-helper/SKILL.md (LOW PRIORITY):
  Curl Options: -sSfL ✗ DRIFT (missing --proto '=https' --tlsv1.2)

.claude/skills/caro-shell-helper/README.md (LOW PRIORITY):
  Curl Options: -sSfL ✗ DRIFT (missing --proto '=https' --tlsv1.2)

.github/RELEASE_TEMPLATE.md (LOW PRIORITY):
  Script URL: raw.githubusercontent.com/wildcard/caro/main/install.sh ✗ DRIFT

================================================================================

Summary:
  - 7 files need updates (version, script URL, or curl options)
  - 3 files missing one-liner installation reference
  - 1 file fully synced ✓

Recommended action:
  /caro.sync installation → Apply updates now
  /caro.sync --check installation → View this report again

================================================================================
```

---

### Step 4: Apply Updates

**Only execute this step if user confirms**

**Ask for confirmation**:
```
Apply updates to sync all installation instructions with website ground truth?

This will update:
  HIGH PRIORITY:
    - README.md: version 1.0.2 → 1.0.3, script URL, curl options, binary URLs
    - nuget/README.md: version 1.0.2 → 1.0.3

  MEDIUM PRIORITY:
    - docs/MACOS_SETUP.md: Add one-liner installation command
    - docs-site/src/content/docs/guides/macos-setup.md: Add one-liner
    - npm/README.md: Add setup.caro.sh reference

  LOW PRIORITY:
    - .claude/skills/caro-shell-helper/*: Update curl security options
    - .github/RELEASE_TEMPLATE.md: Update script URL

Continue? (yes/no):
```

If yes:

**Update README.md**:

Use `Edit` tool to:
1. Replace all occurrences of `1.0.2` with `1.0.3` in binary download URLs
2. Replace `curl -fsSL https://raw.githubusercontent.com/wildcard/caro/main/install.sh | bash` with:
   ```bash
   bash <(curl --proto '=https' --tlsv1.2 -sSfL https://setup.caro.sh)
   ```
3. Update wget alternative to:
   ```bash
   bash <(wget -qO- https://setup.caro.sh)
   ```

**Update nuget/README.md**:

Use `Edit` tool to replace version `1.0.2` with `1.0.3`

**Update docs/MACOS_SETUP.md**:

Use `Edit` tool to add installation section referencing setup.caro.sh

**Update docs-site/src/content/docs/guides/macos-setup.md**:

Use `Edit` tool to add installation section referencing setup.caro.sh

**Update npm/README.md**:

Use `Edit` tool to add setup.caro.sh one-liner in installation section

**Update .claude/skills/caro-shell-helper files**:

Use `Edit` tool to replace `-sSfL` with `--proto '=https' --tlsv1.2 -sSfL` in:
- QUICK_START.md
- SKILL.md
- README.md

**Update .github/RELEASE_TEMPLATE.md**:

Use `Edit` tool to replace install.sh URL with setup.caro.sh

**Verify updates**:
```bash
# Show git diff to user
git diff README.md
git diff nuget/README.md
git diff docs/MACOS_SETUP.md
git diff docs-site/src/content/docs/guides/macos-setup.md
git diff npm/README.md
git diff .claude/skills/caro-shell-helper/
git diff .github/RELEASE_TEMPLATE.md
```

**Report completion**:
```
✓ README.md updated (version, script URL, curl options)
✓ nuget/README.md updated (version)
✓ docs/MACOS_SETUP.md updated (added one-liner)
✓ docs-site/src/content/docs/guides/macos-setup.md updated (added one-liner)
✓ npm/README.md updated (added setup.caro.sh)
✓ .claude/skills/caro-shell-helper/* updated (curl security options)
✓ .github/RELEASE_TEMPLATE.md updated (script URL)

Synced with website ground truth:
  - Version: 1.0.3
  - Script: https://setup.caro.sh
  - Curl options: --proto '=https' --tlsv1.2 -sSfL
  - Binary format: caro-{version}-{platform}

Next steps:
  1. Review changes: git diff
  2. Commit changes: git add -A && git commit -m "sync: Update installation instructions"
  3. Verify: Check that all installation docs are consistent

All installation instructions are now in sync! 🎯
```

---

## Error Handling

### File Not Found
```
ERROR: Could not read [filename]

Expected location: [path]

Verify the file exists and you're in the correct directory.
Current directory: [pwd output]

Exiting installation sync.
```

### Version Mismatch
```
WARNING: Ground truth version could not be determined

website/src/config/version.ts not found or malformed.

Expected format:
  export const CARO_VERSION = 'X.Y.Z';

Falling back to Cargo.toml version...
```

### Parse Error
```
WARNING: Could not parse installation command from [file]

Expected format:
  bash <(curl --proto '=https' --tlsv1.2 -sSfL https://setup.caro.sh)

The file may have been manually edited in an unexpected format.
```

### Curl Command Variations
```
INFO: Multiple curl command variations detected

Found variations:
  - curl -fsSL (insecure, missing protocol specification)
  - curl -sSfL (insecure, missing protocol specification)
  - curl --proto '=https' --tlsv1.2 -sSfL (secure, recommended)

Recommendation:
  Always use: --proto '=https' --tlsv1.2 -sSfL
  This enforces HTTPS and TLS 1.2+
```

---

## Implementation Notes

**Design Decisions**:
- Website (Download.astro + version.ts) is always the source of truth
- Files are updated to match website, never vice versa
- Drift detection runs before any changes
- User confirmation required before applying updates

**Version Extraction**:
- Use version.ts as single source of truth
- Version is auto-generated from Cargo.toml during website build
- Avoids manual version updates in multiple places

**Security**:
- Always recommend `--proto '=https' --tlsv1.2 -sSfL` curl options
- This enforces HTTPS and TLS 1.2 minimum
- Flag files using less secure options

**Script Consolidation**:
- `setup.sh` is the canonical installation script
- `install.sh` references should be updated to setup.caro.sh
- Both scripts exist for backward compatibility
- Future: Consolidate or deprecate install.sh

**Extensibility**:
- Can easily add new target files to sync
- Can add new sync points (e.g., Docker commands, package manager instructions)
- Module is self-contained and can evolve independently

---

## Example Execution

```
User: /caro.sync installation
Claude:
  [Fetches ground truth from website]
  [Reads all target files]
  [Detects drift]

  Installation Instructions Drift Detected:
    README.md: Out of sync (version, script URL, curl options)
    nuget/README.md: Out of sync (version)
    docs/MACOS_SETUP.md: Missing one-liner
    ... (7 files total need updates)

  Apply updates? (yes/no)

User: yes
Claude:
  [Updates README.md]
  [Updates nuget/README.md]
  [Updates docs files]
  [Updates skill docs]
  [Updates release template]

  ✓ Installation instructions synced with website!

  Git diff:
    README.md: 23 lines changed
    nuget/README.md: 3 lines changed
    docs/MACOS_SETUP.md: 8 lines added
    ... (7 files modified)

  Ready to commit.
```
