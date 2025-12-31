---
description: Sync code across different parts of the codebase (roadmap, docs, instructions)
---

**Path reference rule:** When you mention directories or files, provide either the absolute path or a path relative to the project root (for example, `ROADMAP.md`). Never refer to a folder by name alone.

## User Input

```text
$ARGUMENTS
```

You **MUST** consider the user input before proceeding (if not empty).

---

## Quick Reference

| Command | Action |
|---------|--------|
| `/caro.sync` | Show available sync modules and status |
| `/caro.sync roadmap` | Sync roadmap across ROADMAP.md, website, and GitHub |
| `/caro.sync docs` | Sync documentation (future) |
| `/caro.sync instructions` | Sync user instructions (future) |
| `/caro.sync --check <module>` | Check for drift without making changes |

---

## What This Command Does

`/caro.sync` keeps related content synchronized across different parts of the codebase:
- **Roadmap sync**: ROADMAP.md ↔ website/roadmap.astro ↔ GitHub milestones/issues
- **Docs sync** (future): Website docs ↔ README.md ↔ CLAUDE.md ↔ inline docs
- **Instructions sync** (future): CLAUDE.md ↔ README.md ↔ CONTRIBUTING.md ↔ website

**Core Philosophy**:
- **GitHub as source of truth** for roadmap data
- **Drift detection** before making changes
- **Safe updates** with user confirmation
- **Pluggable modules** for different sync domains

---

## Pre-flight Checks

Run these checks before proceeding:

```bash
# Verify GitHub CLI (required for roadmap sync)
which gh || echo "WARNING: gh CLI not installed - roadmap sync unavailable"

# Verify authentication
gh auth status 2>&1 | grep -q "Logged in" || echo "WARNING: Run 'gh auth login' for roadmap sync"
```

**Note**: GitHub CLI is only required for roadmap sync. Other modules may not need it.

---

## Outline

### 1. Parse Arguments and Determine Mode

Parse `$ARGUMENTS` to determine the operation mode:

```
ARGUMENTS patterns:
- Empty/whitespace only → STATUS_MODE (show available modules)
- "roadmap" (case-insensitive) → ROADMAP_SYNC (sync roadmap)
- "docs" → DOCS_SYNC (future - show placeholder)
- "instructions" → INSTRUCTIONS_SYNC (future - show placeholder)
- "--check roadmap" → CHECK_MODE (drift detection only)
- "--check docs" → CHECK_MODE (future)
- "--check instructions" → CHECK_MODE (future)
```

### 2. Execute Mode-Specific Logic

#### 2.1 STATUS_MODE (Default)

**Display available sync modules**:

```
================================================================================
Code Parts Syncer - Available Modules
================================================================================

Sync Modules:

  [✓] roadmap         - Sync ROADMAP.md ↔ website ↔ GitHub
                        Sources: ROADMAP.md, website/src/pages/roadmap.astro,
                                GitHub milestones, GitHub issues
                        Status: Fully functional

  [⋯] docs            - Sync documentation across sources (PLANNED)
                        Sources: website/docs, README.md, CLAUDE.md, inline docs
                        Status: Placeholder - not implemented yet

  [⋯] instructions    - Sync user instructions (PLANNED)
                        Sources: CLAUDE.md, README.md, CONTRIBUTING.md, website
                        Status: Placeholder - not implemented yet

================================================================================

Usage:
  /caro.sync roadmap        → Sync roadmap now
  /caro.sync --check roadmap → Check for drift (dry run)
  /caro.sync docs            → View docs sync (future)

================================================================================
```

EXIT (do not proceed to other modes)

#### 2.2 ROADMAP_SYNC Mode

**Load roadmap sync module**:

Read and execute the instructions in:
```
.claude/skills/code-parts-syncer/roadmap-sync.md
```

This module contains the full logic for:
1. Fetching GitHub milestones and issues as ground truth
2. Reading ROADMAP.md current values
3. Reading website/src/pages/roadmap.astro current values
4. Detecting drift between sources
5. Applying updates to align files with GitHub truth

The module is self-contained and can be executed independently.

EXIT

#### 2.3 DOCS_SYNC Mode (Placeholder)

**Show planned functionality**:

```
================================================================================
Documentation Sync (Planned)
================================================================================

This module will sync documentation across:

  1. Website docs: website/src/content/docs/**
  2. Project README: README.md
  3. Developer guide: CLAUDE.md
  4. Inline documentation: src/**/*.rs (doc comments)

Sync points:
  - Installation instructions
  - Feature descriptions
  - CLI usage examples
  - API documentation

================================================================================

Status: Not yet implemented

This is a placeholder for future development. The docs sync module will:
  - Detect inconsistencies in installation commands
  - Ensure feature lists match across all docs
  - Sync CLI examples between README and website
  - Validate inline docs match external documentation

Would you like to help implement this module?
  - See: .claude/skills/code-parts-syncer/docs-sync.md for structure
  - Reference: roadmap-sync.md for implementation pattern

================================================================================
```

EXIT

#### 2.4 INSTRUCTIONS_SYNC Mode (Placeholder)

**Show planned functionality**:

```
================================================================================
Instructions Sync (Planned)
================================================================================

This module will sync user instructions across:

  1. Developer guide: CLAUDE.md
  2. Project README: README.md
  3. Contributing guide: CONTRIBUTING.md
  4. Website about page: website/src/pages/about.astro

Sync points:
  - Project description
  - Development commands (cargo build, cargo test, etc.)
  - Testing instructions
  - Build commands
  - Contribution workflow

================================================================================

Status: Not yet implemented

This is a placeholder for future development. The instructions sync module will:
  - Ensure consistent project descriptions
  - Sync development commands across all docs
  - Validate testing instructions match
  - Keep build commands up to date everywhere

Would you like to help implement this module?
  - See: .claude/skills/code-parts-syncer/instructions-sync.md for structure
  - Reference: roadmap-sync.md for implementation pattern

================================================================================
```

EXIT

#### 2.5 CHECK_MODE (Drift Detection Only)

**Extract module name** from arguments (e.g., "--check roadmap" → roadmap)

If module is "roadmap":
- Load `.claude/skills/code-parts-syncer/roadmap-sync.md`
- Execute drift detection only (steps 1-4)
- DO NOT apply changes (skip step 5)
- Show drift report to user

If module is "docs" or "instructions":
- Show placeholder message: "Drift detection not available for [module] (not implemented yet)"

EXIT

---

## Integration with Other Skills

### With /caro.roadmap
The roadmap sync complements `/caro.roadmap`:
- `/caro.roadmap` - Manages GitHub issues, work selection, workflow routing
- `/caro.sync roadmap` - Ensures ROADMAP.md and website stay in sync with GitHub

### With /caro.release.*
Before releases, use `/caro.sync --check roadmap` to verify docs are up to date.

---

## Error Handling

### Module Not Found
```
ERROR: Sync module '[module]' not found.

Available modules:
  - roadmap (functional)
  - docs (placeholder)
  - instructions (placeholder)

Usage: /caro.sync <module>
```

### Module File Missing
```
ERROR: Module file not found at:
  .claude/skills/code-parts-syncer/[module]-sync.md

This may indicate a skill installation issue.
Try reinstalling the code-parts-syncer skill.
```

### GitHub CLI Not Available (for roadmap sync)
```
WARNING: GitHub CLI (gh) is not installed or not authenticated.

Roadmap sync requires GitHub CLI to fetch milestone and issue data.

Install: https://cli.github.com/
Auth: gh auth login

Continuing with limited functionality (no GitHub data)...
```

---

## Notes

- **Modular design**: Each sync module is a separate file
- **Extensible**: Easy to add new sync modules
- **Safe**: Always detect drift before applying changes
- **GitHub-first**: For roadmap, GitHub is the authoritative source
- **Future-ready**: Placeholders for docs and instructions sync

**Module locations**:
- Roadmap: `.claude/skills/code-parts-syncer/roadmap-sync.md`
- Docs: `.claude/skills/code-parts-syncer/docs-sync.md` (placeholder)
- Instructions: `.claude/skills/code-parts-syncer/instructions-sync.md` (placeholder)
- Overview: `.claude/skills/code-parts-syncer/README.md`
