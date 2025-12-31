# Documentation Sync Module (Placeholder)

**Status**: Not yet implemented - This is a placeholder for future development

Synchronize documentation across multiple sources:
1. **Website docs** - `website/src/content/docs/**`
2. **Project README** - `README.md`
3. **Developer guide** - `CLAUDE.md`
4. **Inline documentation** - `src/**/*.rs` (doc comments)

---

## Purpose

Keep documentation consistent across all sources by detecting and fixing drift in:
- Installation instructions
- Feature descriptions
- CLI usage examples
- API documentation
- Development commands
- Testing procedures

---

## Planned Process

### Step 1: Extract Documentation Sections

**From README.md**:
```bash
# Extract installation section
sed -n '/## Installation/,/^##/p' README.md

# Extract usage examples
sed -n '/## Usage/,/^##/p' README.md
```

**From CLAUDE.md**:
```bash
# Extract development commands
sed -n '/## Development Commands/,/^##/p' CLAUDE.md

# Extract testing strategy
sed -n '/## Testing Strategy/,/^##/p' CLAUDE.md
```

**From website docs**:
```bash
# Find all documentation pages
find website/src/content/docs -name '*.md' -o -name '*.mdx'

# Extract frontmatter and content
grep -A 10 '^---' website/src/content/docs/getting-started.md
```

**From inline docs**:
```bash
# Extract doc comments from Rust files
grep -r '///' src/ | head -n 20
```

---

### Step 2: Identify Sync Points

**Installation Instructions**:
- README.md `## Installation`
- website/docs/getting-started.md
- CLAUDE.md `## Development Environment`

**CLI Usage Examples**:
- README.md `## Usage`
- website/docs/cli-reference.md
- CLAUDE.md `## Development Commands`

**Feature Descriptions**:
- README.md `## Features`
- website/docs/features/
- CLAUDE.md `## Architecture Overview`

**API Documentation**:
- Inline `///` doc comments in `src/`
- website/docs/api/

---

### Step 3: Detect Drift

**Compare installation commands**:
```
For each source (README, website, CLAUDE.md):
  Extract: cargo install commands
  Compare: versions, flags, prerequisites
  IF differences found:
    → DRIFT DETECTED
```

**Compare feature lists**:
```
For each source:
  Extract: feature bullets/descriptions
  Compare: feature names, capabilities
  IF missing features or inconsistent descriptions:
    → DRIFT DETECTED
```

**Compare code examples**:
```
For each source:
  Extract: CLI command examples
  Compare: syntax, flags, expected output
  IF examples differ or outdated:
    → DRIFT DETECTED
```

**Report drift**:
```
================================================================================
Documentation Drift Detection
================================================================================

Installation Instructions:
  README.md:     cargo install caro --version 0.1.0
  website/docs:  cargo install caro  ✗ DRIFT (missing version)
  CLAUDE.md:     cargo install caro --features mlx  ✗ DRIFT (different flags)

Feature: Safety Validation
  README.md:     ✓ Command safety validation with risk assessment
  website/docs:  ✓ Safety validation  ✗ DRIFT (incomplete description)
  CLAUDE.md:     ✓ Matches README

CLI Example: Basic Usage
  README.md:     caro "list all files"
  website/docs:  caro "show me all files"  ✗ DRIFT (different phrasing)
  CLAUDE.md:     ✓ Matches README

================================================================================

Summary:
  - README.md: Source of truth for user-facing docs
  - website/docs: 2 sync points need updates
  - CLAUDE.md: 1 sync point needs update
  - Inline docs: Not checked (requires parsing)

Recommended action:
  /caro.sync docs → Apply updates now
  /caro.sync --check docs → View this report again

================================================================================
```

---

### Step 4: Apply Updates

**Only execute this step if user confirms**

**Ask for confirmation**:
```
Apply updates to sync documentation across all sources?

This will:
  - Update website/docs to match README.md
  - Update CLAUDE.md to match README.md
  - Update inline doc comments where applicable
  - Maintain consistent examples and commands

Continue? (yes/no):
```

If yes:

**Update website docs**:
Use `Edit` tool to:
1. Align installation commands with README.md
2. Sync feature descriptions
3. Update code examples to match
4. Add missing features

**Update CLAUDE.md**:
Use `Edit` tool to:
1. Sync development commands
2. Update testing procedures
3. Align architecture descriptions
4. Ensure build commands match

**Update inline docs**:
Use `Edit` tool to:
1. Add missing `///` doc comments
2. Update outdated examples in doc comments
3. Sync API descriptions with external docs

**Verify updates**:
```bash
# Show git diff to user
git diff README.md
git diff website/src/content/docs/
git diff CLAUDE.md
git diff src/
```

**Report completion**:
```
✓ README.md (no changes - source of truth)
✓ website/docs updated (3 files)
✓ CLAUDE.md updated
✓ Inline docs updated (5 files)

All documentation now aligned with README.md standards.

Next steps:
  1. Review changes: git diff
  2. Test website: cd website && npm run dev
  3. Commit changes: git add -A && git commit -m "docs: Sync documentation across sources"

Documentation is now consistent! 📚
```

---

## Implementation Notes

**Design Decisions**:
- README.md is the source of truth for user-facing documentation
- CLAUDE.md is the source of truth for developer instructions
- Website docs should match README.md for consistency
- Inline docs should match API descriptions in website

**Source Priority**:
1. README.md (user-facing) → website/docs
2. CLAUDE.md (developer) → inline comments, dev sections
3. Website as presentation layer aligns with both

**Extensibility**:
- Can add CONTRIBUTING.md as additional source
- Can sync changelog entries
- Can validate code examples actually work (run them)

---

## Future Enhancements

**Automated Testing**:
```bash
# Extract and run code examples
grep -A 5 '```bash' README.md | bash

# Verify installation commands work
cargo install caro --dry-run
```

**Smart Merging**:
- Detect which source has most recent update
- Suggest merging newer content to older docs
- Track "last updated" timestamps

**Content Validation**:
- Check links in documentation
- Verify command syntax
- Validate API references exist

---

## Example Execution

```
User: /caro.sync docs
Claude:
  ERROR: Documentation sync module is not yet implemented.

  This is a placeholder for future development. The docs sync module will:
    - Sync installation instructions across README, website, CLAUDE.md
    - Ensure feature descriptions are consistent
    - Align CLI examples and code snippets
    - Validate inline documentation matches external docs

  Would you like to help implement this module?
    - See: .claude/skills/code-parts-syncer/roadmap-sync.md for implementation pattern
    - Reference: This file for planned structure
    - Follow: Same drift detection → apply updates workflow

  For now, you can manually check documentation consistency.
```

---

## Reference Implementation

This module follows the same pattern as `roadmap-sync.md`:
1. Fetch/extract data from multiple sources
2. Compare for drift
3. Report discrepancies
4. Apply updates with user confirmation
5. Verify and commit changes

See `.claude/skills/code-parts-syncer/roadmap-sync.md` for a fully functional example of this pattern.
