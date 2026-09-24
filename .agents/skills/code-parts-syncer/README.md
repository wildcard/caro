# Code Parts Syncer

A pluggable Claude skill for keeping related content synchronized across different parts of the codebase.

## Overview

Modern codebases often have the same information represented in multiple locations:
- Roadmap data in ROADMAP.md, website, and GitHub milestones
- Documentation in README, website, CLAUDE.md, and inline comments
- Instructions in CLAUDE.md, README, CONTRIBUTING, and website

The **Code Parts Syncer** detects when these sources drift apart and helps you sync them back together.

## Philosophy

**Ground Truth Pattern**: Each sync domain has an authoritative source (e.g., GitHub API for roadmap, CLAUDE.md for developer instructions). All other sources align to this truth.

**Drift Detection First**: Before making any changes, the skill shows you exactly what's out of sync and why.

**Safe Updates**: You approve changes before they're applied, with full git diff visibility.

**Pluggable Architecture**: Each sync domain is an independent module - easy to add new ones.

## Architecture

```
.claude/
├── commands/
│   └── caro.sync.md              # Main command dispatcher
└── skills/
    └── code-parts-syncer/
        ├── README.md              # This file
        ├── roadmap-sync.md        # [✓] Roadmap module (functional)
        ├── docs-sync.md           # [⋯] Docs module (placeholder)
        └── instructions-sync.md   # [⋯] Instructions module (placeholder)
```

### How It Works

1. **Command Dispatcher** (`caro.sync.md`):
   - Parses arguments to determine mode
   - Routes to appropriate sync module
   - Handles status display and error cases

2. **Sync Modules** (`*-sync.md`):
   - Self-contained sync logic for specific domain
   - Follows 5-step process:
     1. Fetch ground truth
     2. Read current values from all sources
     3. Detect drift
     4. Report discrepancies
     5. Apply updates (with confirmation)

3. **Extensibility**:
   - Add new module → drop file in this directory
   - Update dispatcher → add new mode case
   - No changes needed to existing modules

## Available Modules

### ✓ Roadmap Sync (Functional)

**File**: `roadmap-sync.md`

**Command**: `/caro.sync roadmap`

**Sources**:
- GitHub API (milestones, issues) - **Ground Truth**
- `ROADMAP.md` - Markdown documentation
- `website/src/pages/roadmap.astro` - Public roadmap page

**What It Syncs**:
- Milestone item counts (total, open, closed)
- Progress percentages
- Days remaining until due dates
- Issue references and completed items
- Last updated timestamps

**Example Output**:
```
================================================================================
Roadmap Drift Detection
================================================================================

Source of Truth: GitHub API (gh api repos/wildcard/caro/milestones)

v1.1.0 - Core Improvements:
  GitHub (TRUTH):    20 open, 1 closed, 21 total (5% complete, 46 days)
  ROADMAP.md:        15 open, 1 closed, 16 total (7% complete, 48 days) ✗ DRIFT
  website/roadmap:   20 open, 1 closed, 21 total (5% complete, 46 days) ✓ SYNCED

Summary:
  - ROADMAP.md: 1 milestone needs updates
  - website/roadmap: All synced ✓

Recommended action:
  /caro.sync roadmap → Apply updates now
================================================================================
```

### ⋯ Documentation Sync (Planned)

**File**: `docs-sync.md` (placeholder)

**Command**: `/caro.sync docs`

**Planned Sources**:
- `README.md` - User-facing documentation (ground truth)
- `CLAUDE.md` - Developer guide
- `website/src/content/docs/**` - Website docs
- `src/**/*.rs` - Inline doc comments

**Planned Sync Points**:
- Installation instructions
- Feature descriptions
- CLI usage examples
- API documentation

**Status**: Not yet implemented - see `docs-sync.md` for planned structure

### ⋯ Instructions Sync (Planned)

**File**: `instructions-sync.md` (placeholder)

**Command**: `/caro.sync instructions`

**Planned Sources**:
- `CLAUDE.md` - Developer instructions (ground truth)
- `README.md` - User-facing guide
- `CONTRIBUTING.md` - Contributor workflow
- `website/src/pages/about.astro` - Marketing page

**Planned Sync Points**:
- Project description and mission
- Development commands (build, test, run)
- Testing procedures
- Code quality standards

**Status**: Not yet implemented - see `instructions-sync.md` for planned structure

## Usage

### Show Available Modules

```bash
/caro.sync
```

Displays:
- List of all sync modules
- Status (functional vs planned)
- Brief description of each
- Usage examples

### Sync Roadmap

```bash
# Full sync with updates
/caro.sync roadmap

# Drift detection only (dry run)
/caro.sync --check roadmap
```

Process:
1. Fetches GitHub milestones and issues (ground truth)
2. Reads ROADMAP.md and website/roadmap.astro
3. Compares values and detects drift
4. Shows detailed drift report
5. Asks for confirmation to apply updates
6. Updates files and shows git diff

### Sync Documentation (Future)

```bash
# Full sync with updates
/caro.sync docs

# Drift detection only
/caro.sync --check docs
```

Will ensure installation instructions, feature lists, and examples are consistent across README, website, and CLAUDE.md.

### Sync Instructions (Future)

```bash
# Full sync with updates
/caro.sync instructions

# Drift detection only
/caro.sync --check instructions
```

Will align project descriptions, development commands, and quality standards across CLAUDE.md, README, CONTRIBUTING, and website.

## Design Patterns

### 5-Step Sync Process

All modules follow this pattern:

1. **Fetch Ground Truth**: Get authoritative data from source of truth
2. **Read Current State**: Extract values from all sync targets
3. **Detect Drift**: Compare ground truth with current state
4. **Report Drift**: Show detailed discrepancies to user
5. **Apply Updates**: Make changes with user confirmation

### Ground Truth Hierarchy

Each module defines a source of truth:

| Module | Ground Truth | Why |
|--------|-------------|-----|
| Roadmap | GitHub API | Developers update issues/milestones directly |
| Docs | README.md | User-facing docs are most frequently reviewed |
| Instructions | CLAUDE.md | Most comprehensive developer guidance |

### Drift Detection Report Format

Standard format across all modules:

```
================================================================================
[Module Name] Drift Detection
================================================================================

Source of Truth: [Ground truth source]

[Entity Name]:
  [Ground Truth]:  [values]  [status]
  [Target 1]:      [values]  ✓ SYNCED or ✗ DRIFT
  [Target 2]:      [values]  ✓ SYNCED or ✗ DRIFT

Summary:
  - [Summary of what needs updates]

Recommended action:
  /caro.sync [module] → Apply updates now
  /caro.sync --check [module] → View this report again
================================================================================
```

## Adding New Modules

### Step 1: Create Module File

Create `.claude/skills/code-parts-syncer/new-module-sync.md`:

```markdown
# New Module Sync

Synchronize [description] across:
1. **Source 1** - [path/location] - **Ground Truth**
2. **Target 1** - [path/location]
3. **Target 2** - [path/location]

## Process

### Step 1: Fetch Ground Truth
[Commands to fetch authoritative data]

### Step 2: Read Current Values
[Commands to extract from targets]

### Step 3: Detect Drift
[Comparison logic]

### Step 4: Report Drift
[Report format]

### Step 5: Apply Updates
[Update commands with Edit tool]
```

### Step 2: Update Command Dispatcher

Edit `.claude/commands/caro.sync.md`:

1. Add to Quick Reference table
2. Add case in mode detection logic
3. Add execution mode section

### Step 3: Test

```bash
# Check status shows new module
/caro.sync

# Test drift detection
/caro.sync --check new-module

# Test full sync
/caro.sync new-module
```

## Integration with Project Workflow

### Before Releases

```bash
# Ensure all docs are synced before release
/caro.sync --check roadmap
/caro.sync --check docs
/caro.sync --check instructions
```

### After Major Changes

```bash
# Roadmap updated on GitHub
/caro.sync roadmap

# Documentation rewritten
/caro.sync docs

# New development commands added
/caro.sync instructions
```

### Regular Maintenance

```bash
# Weekly/bi-weekly sync check
/caro.sync  # See status of all modules
```

## Error Handling

Each module handles common errors:

### GitHub CLI Not Available
```
ERROR: GitHub CLI is required for roadmap sync.

Install: https://cli.github.com/
Auth: gh auth login
```

### File Not Found
```
ERROR: Could not read ROADMAP.md

Expected location: ROADMAP.md (project root)
Current directory: [pwd output]
```

### Parse Error
```
WARNING: Could not parse milestone data from ROADMAP.md

The file may have been manually edited in an unexpected format.
Proceeding with partial data...
```

## Best Practices

### When to Sync

**Roadmap Module**:
- After creating/closing GitHub issues
- Before updating website
- When milestone progress changes
- Before major releases

**Docs Module** (Future):
- After updating README
- When adding new features
- Before website deployment
- After API changes

**Instructions Module** (Future):
- After changing development workflow
- When adding new commands
- Before updating CONTRIBUTING
- After architecture changes

### Drift Prevention

Instead of syncing after drift occurs, prevent it:

1. **Single Source of Truth**: Update the ground truth first
2. **Regular Checks**: Run `/caro.sync --check` frequently
3. **Pre-commit Hooks**: Consider automating drift detection
4. **Documentation**: Document which file is authoritative

### Git Workflow

Sync operations create commits:

```bash
# Typical workflow
/caro.sync roadmap        # Apply updates
git diff                  # Review changes
git add -A
git commit -m "sync: Update roadmap data from GitHub"
```

Consider creating a dedicated sync commit message convention (e.g., `sync: [module] - [description]`).

## Technical Details

### Dependencies

- **GitHub CLI** (`gh`): Required for roadmap sync
- **Edit tool**: For file modifications
- **Bash**: For data extraction and calculations

### Performance

- Roadmap sync: ~2-5 seconds (depends on GitHub API)
- Docs sync: ~1-3 seconds (local file reads)
- Instructions sync: ~1-2 seconds (local file reads)

### Safety

- All changes shown via git diff before commit
- User confirmation required before applying updates
- No destructive operations (only file edits)
- Rollback with `git reset HEAD~1` if needed

## Future Enhancements

### Automated Drift Detection

Set up hooks to detect drift automatically:

```bash
# Pre-push hook
if /caro.sync --check roadmap | grep -q "DRIFT"; then
  echo "WARNING: Roadmap is out of sync with GitHub"
  echo "Run '/caro.sync roadmap' before pushing"
fi
```

### CI/CD Integration

Add drift checks to CI pipeline:

```yaml
# .github/workflows/drift-check.yml
- name: Check for documentation drift
  run: |
    /caro.sync --check roadmap
    /caro.sync --check docs
    /caro.sync --check instructions
```

### Dashboard

Create a drift status dashboard:

```bash
/caro.sync status --all
```

Output:
```
Sync Status Dashboard
=====================
Roadmap:      ✗ DRIFT (2 targets need updates)
Docs:         ✓ SYNCED
Instructions: ⚠ PARTIAL (1 target outdated)
```

## Contributing

To improve this skill:

1. **Enhance existing modules**: Add new sync points, improve detection
2. **Implement planned modules**: See `docs-sync.md`, `instructions-sync.md`
3. **Add new modules**: Follow "Adding New Modules" guide above
4. **Improve patterns**: Enhance drift detection or update workflows

See `roadmap-sync.md` for a fully functional reference implementation.

## License

Part of the Caro project - see main repository for license details.
