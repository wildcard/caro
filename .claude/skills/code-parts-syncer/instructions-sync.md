# Instructions Sync Module (Placeholder)

**Status**: Not yet implemented - This is a placeholder for future development

Synchronize user instructions and development guidelines across:
1. **Developer guide** - `CLAUDE.md`
2. **Project README** - `README.md`
3. **Contributing guide** - `CONTRIBUTING.md`
4. **Website about page** - `website/src/pages/about.astro`

---

## Purpose

Keep instructions coherent across all documentation by detecting and fixing drift in:
- Project description and mission
- Development commands (build, test, run)
- Testing instructions and procedures
- Build commands and workflows
- Contribution guidelines
- Code quality standards

---

## Planned Process

### Step 1: Extract Instruction Sections

**From CLAUDE.md**:
```bash
# Extract project overview
sed -n '/## Project Overview/,/^##/p' CLAUDE.md

# Extract development commands
sed -n '/## Development Commands/,/^##/p' CLAUDE.md

# Extract testing strategy
sed -n '/## Testing Strategy/,/^##/p' CLAUDE.md

# Extract quality standards
sed -n '/## Quality Standards/,/^##/p' CLAUDE.md
```

**From README.md**:
```bash
# Extract project description
sed -n '/^# /,/^##/p' README.md | head -n 10

# Extract installation/usage sections
sed -n '/## Installation/,/^##/p' README.md
sed -n '/## Usage/,/^##/p' README.md

# Extract contributing section
sed -n '/## Contributing/,/^##/p' README.md
```

**From CONTRIBUTING.md** (if exists):
```bash
# Extract contribution workflow
sed -n '/## Getting Started/,/^##/p' CONTRIBUTING.md

# Extract code standards
sed -n '/## Code Standards/,/^##/p' CONTRIBUTING.md

# Extract testing requirements
sed -n '/## Testing/,/^##/p' CONTRIBUTING.md
```

**From website**:
```bash
# Extract about page content
grep -A 50 '<main>' website/src/pages/about.astro

# Look for project mission/values
grep -i 'mission\|values\|philosophy' website/src/pages/about.astro
```

---

### Step 2: Identify Sync Points

**Project Description**:
- CLAUDE.md `## Project Overview` → Single source of truth (most detailed)
- README.md opening paragraph → Condensed version
- website/about.astro → Marketing-friendly version
- CONTRIBUTING.md header → Brief mention

**Development Commands**:
- CLAUDE.md `## Development Commands` → Complete reference
- README.md `## Development` → User-friendly subset
- CONTRIBUTING.md `## Getting Started` → Contributor-focused

**Testing Instructions**:
- CLAUDE.md `## Testing Strategy` → Comprehensive guide
- README.md `## Testing` → Quick reference
- CONTRIBUTING.md `## Testing` → PR requirements

**Build Commands**:
- CLAUDE.md `### Building & Testing` → All build options
- README.md `## Installation` → Standard build
- CONTRIBUTING.md → Development build with checks

**Code Quality Standards**:
- CLAUDE.md `## Quality Standards` → Definitive list
- CONTRIBUTING.md → Contributor expectations
- README.md → Not usually included (optional)

---

### Step 3: Detect Drift

**Compare project descriptions**:
```
For each source (CLAUDE.md, README.md, website, CONTRIBUTING.md):
  Extract: project description, goals, features
  Compare: core messaging, feature highlights
  IF descriptions conflict or outdated:
    → DRIFT DETECTED
```

**Compare development commands**:
```
For each command type (build, test, run, lint):
  Extract: exact command syntax from all sources
  Compare: flags, arguments, expected behavior
  IF commands differ without explanation:
    → DRIFT DETECTED
```

**Compare testing instructions**:
```
For each source:
  Extract: test commands, coverage requirements
  Compare: testing workflows, CI expectations
  IF testing procedures inconsistent:
    → DRIFT DETECTED
```

**Report drift**:
```
================================================================================
Instructions Drift Detection
================================================================================

Project Description:
  CLAUDE.md:       "single-binary Rust CLI tool... Apple Silicon optimization"
  README.md:       "CLI tool that converts natural language to shell commands"
                   ✗ DRIFT (missing key details about safety, MLX)
  website/about:   "AI-powered command generator"
                   ✗ DRIFT (too vague, missing safety focus)
  CONTRIBUTING.md: No description  ✗ DRIFT (should have brief intro)

Build Command:
  CLAUDE.md:       cargo build --release
  README.md:       cargo build  ✗ DRIFT (missing --release flag)
  CONTRIBUTING.md: cargo build && cargo clippy  ✓ MORE COMPLETE (dev workflow)

Test Command:
  CLAUDE.md:       cargo test
  README.md:       cargo test  ✓ SYNCED
  CONTRIBUTING.md: cargo test && cargo clippy  ✓ ADDITIONAL (pre-PR check)

Code Quality Standards:
  CLAUDE.md:       Comprehensive list (7 standards)
  CONTRIBUTING.md: Missing  ✗ DRIFT (should reference CLAUDE.md)
  README.md:       Not applicable  ✓ OK

================================================================================

Summary:
  - CLAUDE.md: Source of truth for developer instructions
  - README.md: 2 instruction points need enhancement
  - website/about: Project description needs detail
  - CONTRIBUTING.md: Missing quality standards reference

Recommended action:
  /caro.sync instructions → Apply updates now
  /caro.sync --check instructions → View this report again

================================================================================
```

---

### Step 4: Apply Updates

**Only execute this step if user confirms**

**Ask for confirmation**:
```
Apply updates to sync instructions across all sources?

This will:
  - Update README.md project description (add safety/MLX mentions)
  - Enhance website/about.astro description
  - Add CONTRIBUTING.md quality standards section
  - Align build/test commands where appropriate
  - Maintain source-of-truth hierarchy (CLAUDE.md → others)

Continue? (yes/no):
```

If yes:

**Update README.md**:
Use `Edit` tool to:
1. Enhance project description with key differentiators (safety, MLX, single-binary)
2. Update build command to include `--release` flag
3. Add link to CLAUDE.md for detailed developer instructions
4. Ensure contributing section points to CONTRIBUTING.md

**Update website/about.astro**:
Use `Edit` tool to:
1. Expand project description with concrete details
2. Highlight safety-first approach
3. Mention Apple Silicon optimization
4. Link to GitHub and documentation

**Update CONTRIBUTING.md**:
Use `Edit` tool to:
1. Add project description header
2. Reference CLAUDE.md quality standards
3. Include pre-PR checklist (build, test, clippy, fmt)
4. Add link to CLAUDE.md for architecture details

**Verify updates**:
```bash
# Show git diff to user
git diff README.md
git diff website/src/pages/about.astro
git diff CONTRIBUTING.md
```

**Report completion**:
```
✓ CLAUDE.md (no changes - source of truth)
✓ README.md updated (project description, build command)
✓ website/about.astro updated (enhanced description)
✓ CONTRIBUTING.md updated (added quality standards reference)

All instructions now coherent across sources.

Next steps:
  1. Review changes: git diff
  2. Test website: cd website && npm run dev
  3. Verify build works: cargo build --release
  4. Commit changes: git add -A && git commit -m "docs: Sync instructions across all sources"

Instructions are now consistent! 📖
```

---

## Implementation Notes

**Design Decisions**:
- CLAUDE.md is the authoritative source for developer instructions
- README.md serves users and quick-start developers
- CONTRIBUTING.md focuses on PR workflow and standards
- Website is marketing-friendly but technically accurate

**Source Hierarchy**:
```
CLAUDE.md (Developer Source of Truth)
    ↓
    ├─→ README.md (User-facing, condensed)
    ├─→ CONTRIBUTING.md (Contributor workflow)
    └─→ website/about.astro (Marketing + technical)
```

**Sync Strategy**:
- Project description: CLAUDE.md → summarize for README → marketing for website
- Development commands: CLAUDE.md complete → README subset → CONTRIBUTING workflow
- Quality standards: CLAUDE.md definitive → CONTRIBUTING references → README omits

**Extensibility**:
- Can add GitHub issue templates
- Can sync with GitHub Wiki if used
- Can validate commands actually work (execute them)

---

## Future Enhancements

**Command Validation**:
```bash
# Extract and test commands from docs
grep '```bash' CLAUDE.md | grep 'cargo' | bash -n

# Verify all cargo commands are valid
cargo build --dry-run
cargo test --dry-run
```

**Consistency Checking**:
- Detect version mismatches in installation commands
- Ensure flag consistency across examples
- Validate links between documents

**Smart Updates**:
- Track "last updated" per section
- Suggest which source to update from (newest wins)
- Auto-generate CONTRIBUTING.md from CLAUDE.md templates

---

## Example Execution

```
User: /caro.sync instructions
Claude:
  ERROR: Instructions sync module is not yet implemented.

  This is a placeholder for future development. The instructions sync module will:
    - Sync project descriptions across CLAUDE.md, README, website
    - Align development commands (build, test, lint)
    - Ensure testing procedures are consistent
    - Keep quality standards coherent
    - Maintain CLAUDE.md as source of truth

  Would you like to help implement this module?
    - See: .claude/skills/code-parts-syncer/roadmap-sync.md for implementation pattern
    - Reference: This file for planned structure
    - Follow: Same drift detection → apply updates workflow

  For now, you can manually check instruction consistency by comparing:
    - CLAUDE.md (developer source of truth)
    - README.md (user-facing)
    - CONTRIBUTING.md (contributor workflow)
    - website/about.astro (marketing)
```

---

## Reference Implementation

This module follows the same pattern as `roadmap-sync.md`:
1. Extract data from multiple sources (CLAUDE.md, README, CONTRIBUTING, website)
2. Compare for drift (project description, commands, standards)
3. Report discrepancies with source hierarchy
4. Apply updates respecting CLAUDE.md as source of truth
5. Verify and commit changes

See `.claude/skills/code-parts-syncer/roadmap-sync.md` for a fully functional example of this pattern.
