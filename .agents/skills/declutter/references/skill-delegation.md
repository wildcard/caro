# Skill Delegation Guide

Declutter identifies reorganization needs but delegates execution to specialized skills. This reference explains when and how to delegate.

---

## Core Principle

**Declutter plans, skills execute.**

Declutter excels at:
- Surveying project structure
- Classifying files
- Proposing reorganization
- Coordinating the process

Declutter should NOT:
- Refactor code directly
- Update import paths
- Modify documentation content
- Change build configurations

---

## Delegation Matrix

### File Moves Requiring Code Changes

| Situation | Delegate To | What They Do |
|-----------|-------------|--------------|
| Moving source files | `refactor` skill | Updates imports, references |
| Moving modules | `refactor` skill | Adjusts module declarations |
| Renaming packages | `refactor` skill | Updates all references |

### Documentation Reorganization

| Situation | Delegate To | What They Do |
|-----------|-------------|--------------|
| Restructuring /docs | `technical-writer` agent | Creates hierarchy, updates links |
| Moving READMEs | `technical-writer` agent | Ensures links remain valid |
| Creating doc structure | `technical-writer` agent | Sets up proper categories |

### Build & DevOps

| Situation | Delegate To | What They Do |
|-----------|-------------|--------------|
| Moving CI configs | `devops` skill | Updates workflow paths |
| Reorganizing scripts | `devops` skill | Updates script references |
| Docker reorganization | `devops` skill | Updates Dockerfiles |

### Research & Best Practices

| Situation | Delegate To | What They Do |
|-----------|-------------|--------------|
| Framework conventions | `Explore` agent | Researches standard patterns |
| OSS best practices | `Explore` agent | Finds exemplary projects |
| Technology-specific | `rust-cli-architect`, etc. | Provides stack guidance |

---

## Delegation Workflow

### Step 1: Identify Need

During classification, note when a move will break references:

```markdown
## Moves Requiring Delegation

| File | Destination | Breaks | Delegate To |
|------|-------------|--------|-------------|
| src/utils.rs | src/lib/utils.rs | Imports | refactor |
| DESIGN.md | docs/architecture/ | Links | technical-writer |
```

### Step 2: Check Skill Availability

```bash
# List available skills
ls .claude/skills/*/SKILL.md

# Check for specific skill
cat .claude/skills/refactor/SKILL.md 2>/dev/null || echo "Not installed"
```

### Step 3: Handle Missing Skills

If skill is not installed:

```markdown
## Missing Skill Alert

The `refactor` skill is not installed but is needed for:
- Updating import paths in 12 files after moving src/utils.rs

**Options:**
1. Install the refactor skill: [instructions]
2. Proceed manually (I'll list what needs updating)
3. Skip this move for now
```

### Step 4: Prepare Delegation Prompt

When delegating, provide full context:

```
Task: refactor
Prompt: "I'm decluttering this project and need to move source files.

Files to move:
- src/utils.rs → src/lib/utils.rs
- src/helpers.rs → src/lib/helpers.rs

Please:
1. Identify all files that import from the old paths
2. Update the imports to use new paths
3. Update any mod.rs or lib.rs declarations
4. Report what was changed

The moves have already been done via git mv."
```

### Step 5: Verify Results

After delegation completes:

```bash
# Check for broken references
cargo check
npm run build
python -m py_compile src/**/*.py
```

---

## Agent Descriptions Reference

### Exploration & Research

**`Explore` agent**
```
Use for: Finding OSS conventions, codebase patterns, structure research
Example: "What's the standard directory structure for Next.js projects?"
```

**`general-purpose` agent**
```
Use for: Complex multi-step research, cross-codebase analysis
Example: "Analyze our project structure vs similar OSS projects"
```

### Code & Architecture

**`rust-cli-architect` agent**
```
Use for: Rust project structure, CLI organization
Example: "How should I organize this Rust workspace?"
```

**`rust-cli-expert` agent**
```
Use for: Rust implementation details, module organization
Example: "Best way to split this large module?"
```

### Documentation

**`technical-writer` agent**
```
Use for: Documentation structure, README improvement
Example: "Create a /docs hierarchy for this project"
```

### Quality & Safety

**`safety-pattern-auditor` skill**
```
Use for: Checking if files are safe to delete/move
Example: "Is it safe to remove these backup files?"
```

**`validate-constitution` skill**
```
Use for: Checking project rules before moves
Example: "Does moving this file violate project conventions?"
```

---

## When NOT to Delegate

### Simple Moves

If moving doesn't break anything, just do it:

```bash
# Safe to execute directly
mkdir -p scripts/
git mv setup.sh scripts/
git mv build.sh scripts/
```

### Empty Directories

Creating structure is declutter's job:

```bash
# Declutter handles this
mkdir -p docs/{architecture,guides,api}
mkdir -p scripts/{build,dev,release}
```

### Git Operations

File moves with git are declutter's domain:

```bash
# Declutter handles this
git mv old-location new-location
```

### Cleanup

Removing orphaned files (with user approval):

```bash
# Declutter handles this (after user confirms)
rm backup.old
rm temp_file.txt
```

---

## Skill Installation Prompts

When a skill is missing, suggest installation:

### For refactor skill
```markdown
The `refactor` skill would help update code references after file moves.

To install, create:
- .claude/skills/refactor/SKILL.md

Or proceed manually by updating these files:
[list of files with broken imports]
```

### For devops skill
```markdown
The `devops` skill would help reorganize CI/CD and build scripts.

To install, create:
- .claude/skills/devops/SKILL.md

Or manually update these references:
[list of CI files referencing moved scripts]
```

---

## Parallel Delegation

When multiple skills are needed, coordinate:

```markdown
## Delegation Plan

### Phase 1: Code Moves (refactor skill)
- Move source files
- Update imports

### Phase 2: Docs Reorganization (technical-writer agent)
- Create docs hierarchy
- Move documentation files
- Update internal links

### Phase 3: Scripts (devops skill or manual)
- Organize scripts folder
- Update CI references

### Verification
After all phases:
- cargo check
- npm run build
- Verify links
```

---

## Error Handling

### Skill Fails

If delegated skill reports failure:

1. Document what failed
2. Attempt manual fix
3. Report to user with context

### Partial Success

If skill only partially completes:

1. Note completed items
2. Create follow-up for remaining
3. Don't block overall declutter

### Conflict with Existing Patterns

If skill suggests something different from project patterns:

1. Trust existing project patterns
2. Ask user to resolve conflict
3. Document decision

---

*Delegation enables declutter to focus on organization while specialized skills handle the details. When in doubt, delegate to the expert.*
