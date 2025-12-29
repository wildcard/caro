# Caro Skills Guide

Complete reference for all Claude Code skills available in the Caro project.

## Overview

Caro uses **Claude Code skills** (local slash commands) to automate complex development workflows. Skills are markdown files in `.claude/commands/` that provide structured, repeatable workflows for common tasks.

**Total Skills**: 20 commands across 3 namespaces

## Skill Namespaces

### 1. `caro.release.*` - Release Management (6 skills)
Complete workflow for versioned releases with security controls.

### 2. `caro.deps.*` - Dependency Management (1 skill)
Automated Dependabot PR review and merge workflow.

### 3. `spec-kitty.*` - Feature Development (13 skills)
Rapid feature development with spec-driven methodology.

---

## Quick Start

### Common Workflows

**Dependency Update + Release:**
```bash
/caro.deps.review           # Review and merge Dependabot PRs
/caro.release.prepare       # Create release branch
/caro.release.version       # Bump version, update changelog
/caro.release.publish       # Merge, tag, publish to crates.io
/caro.release.verify        # Verify installation works
```

**New Feature Development:**
```bash
bin/sk-new-feature "Add Redis caching"  # Create worktree
cd kitty-specs/001-add-redis-caching/
/spec-kitty.specify                     # Generate spec.md
/spec-kitty.plan                        # Create implementation plan
/spec-kitty.tasks                       # Break down into work packages
/spec-kitty.implement                   # Execute all tasks
/spec-kitty.accept                      # Run acceptance checks
/spec-kitty.merge                       # Merge and cleanup
```

**Emergency Hotfix:**
```bash
/caro.release.hotfix        # Create hotfix branch from tag
# Apply minimal fix
/caro.release.publish       # Fast-track merge and publish
/caro.release.verify        # Verify hotfix works
```

---

## Release Management Skills

### `/caro.release.prepare`

**Purpose**: Create release branch and run pre-flight checks

**Pre-flight**: Must be on `main` branch with clean working directory

**Workflow**:
1. Pull latest from main
2. Prompt for version number
3. Create release branch: `release/vX.Y.Z`
4. Check for release blockers via GitHub Issues
5. Verify CI is green on main
6. List pending changes since last release

**When to use**: Starting a new release cycle

**Example**:
```bash
/caro.release.prepare
# Enter version: 1.1.0
```

---

### `/caro.release.security`

**Purpose**: Run security audit and fix vulnerabilities

**Pre-flight**: Must be on `release/v*` branch

**Workflow**:
1. Install/verify `cargo-audit`
2. Run `cargo audit` and parse results
3. Categorize vulnerabilities:
   - Critical/Unsound: Must fix before release
   - Unmaintained (direct): Update dependency
   - Unmaintained (indirect): Document as known issue
4. Apply fixes with guidance
5. Run tests to verify fixes
6. Commit security fixes

**When to use**: After creating release branch, before version bump

**Example**:
```bash
/caro.release.security
```

---

### `/caro.release.version`

**Purpose**: Bump version and update documentation

**Pre-flight**: Must be on `release/v*` branch

**Workflow**:
1. Extract target version from branch name
2. Update `Cargo.toml` version
3. Update `CHANGELOG.md`:
   - Move [Unreleased] to [X.Y.Z] - YYYY-MM-DD
   - Add release summary
   - Document breaking changes
4. Update `docs/RELEASE_PROCESS.md` if needed
5. Run `cargo check` to verify
6. Commit version bump

**When to use**: After security audit, before creating PR

**Example**:
```bash
/caro.release.version
```

---

### `/caro.release.publish`

**Purpose**: Create PR, merge, tag, and monitor release

**Pre-flight**: Must be on `release/v*` branch with all changes committed

**Workflow**:
1. Push release branch
2. Create GitHub PR with release notes
3. Wait for CI checks to pass
4. Merge PR (squash merge)
5. Switch to main and pull
6. Create annotated git tag
7. Push tag (triggers automated workflows)
8. Monitor publish.yml and release.yml
9. Verify crates.io publication

**When to use**: After version bump is committed

**Example**:
```bash
/caro.release.publish
```

---

### `/caro.release.verify`

**Purpose**: Post-release verification testing

**Pre-flight**: Must have pushed a release tag

**Workflow**:
1. Wait for crates.io index update (~2 minutes)
2. Install from crates.io: `cargo install caro --force`
3. Verify version matches release
4. Run basic functionality tests:
   - Help command
   - Version command
   - Config command
   - Dry run
   - JSON output
5. Check GitHub release exists
6. Verify documentation links
7. Output comprehensive summary

**When to use**: After publish workflow completes

**Example**:
```bash
/caro.release.verify
# Or verify specific version:
/caro.release.verify v1.0.1
```

---

### `/caro.release.hotfix`

**Purpose**: Emergency security patch workflow

**Pre-flight**: Can start from any branch (emergency mode)

**Workflow**:
1. Fetch latest tags
2. Prompt for version to hotfix
3. Create hotfix branch from tag
4. Guide minimal fix application
5. Bump patch version
6. Create fast-track PR with security label
7. After merge, follow publish workflow

**When to use**: Critical security vulnerability requires immediate patch

**Example**:
```bash
/caro.release.hotfix
# Enter version to patch: v1.0.1
```

---

### `/caro.deps.review`

**Purpose**: Review and merge Dependabot PRs with breaking change analysis

**Pre-flight**: No branch requirement (can run from any branch)

**Requirements**: GitHub CLI (`gh`) must be authenticated

**Workflow**:
1. Fetch open Dependabot PRs
2. Categorize by update type:
   - Major (breaking): Requires investigation
   - Minor/Patch: Safe to merge
   - GitHub Actions: Safe to merge
3. For major updates:
   - Search codebase for affected APIs
   - Verify compatibility
   - Provide merge recommendation
4. Execute batch merges with CI monitoring
5. Handle CI failures (rerun flaky tests, fix issues)
6. Optionally update CHANGELOG with summary
7. Verify build after merges

**When to use**: When Dependabot PRs are pending, before starting a release

**Example**:
```bash
/caro.deps.review
# Or merge specific PR:
/caro.deps.review merge #123
```

**Output**:
```
Dependabot PR Review Summary:
✓ Merged: 8 PRs
⚠ Requires review: 1 PR (#109 - API changes)
⏭ Skipped: 0 PRs

Next steps: Review #109 manually
```

---

## Feature Development Skills (Spec-Kitty)

### Overview

Spec-Kitty provides rapid feature development using git worktrees and spec-driven methodology. Each feature gets an isolated worktree for parallel development.

**Key benefits**:
- Work on multiple features simultaneously
- No branch switching overhead
- Real-time dashboard tracking
- Automated task management

### `/spec-kitty.specify`

**Purpose**: Create or update feature specification

**Pre-flight**: Must be in a spec-kitty worktree

**Workflow**:
1. Analyze feature description
2. Generate comprehensive spec.md:
   - Background and motivation
   - Requirements (functional and non-functional)
   - Architecture overview
   - Success criteria
3. Save to spec.md in worktree
4. Display next steps

**When to use**: First step after creating new feature worktree

**Example**:
```bash
cd kitty-specs/001-add-caching/
/spec-kitty.specify
```

---

### `/spec-kitty.plan`

**Purpose**: Create implementation plan from specification

**Pre-flight**: Must be in spec-kitty worktree with spec.md

**Workflow**:
1. Read and analyze spec.md
2. Design implementation approach
3. Create plan.md with:
   - High-level architecture
   - Phase breakdown
   - Critical files to modify
   - Trade-offs and decisions
4. Display plan for review

**When to use**: After specification is approved

**Example**:
```bash
/spec-kitty.plan
```

---

### `/spec-kitty.tasks`

**Purpose**: Generate grouped work packages from plan

**Pre-flight**: Must have plan.md in worktree

**Workflow**:
1. Read plan.md and spec.md
2. Break down into actionable tasks
3. Group tasks into work packages
4. Create tasks.md with:
   - Task descriptions
   - Dependencies
   - Acceptance criteria
5. Generate prompt files for each task
6. Update dashboard metadata

**When to use**: After plan is reviewed and approved

**Example**:
```bash
/spec-kitty.tasks
```

---

### `/spec-kitty.implement`

**Purpose**: Execute implementation plan by processing all tasks

**Pre-flight**: Must have tasks.md and prompt files

**Workflow**:
1. Read tasks.md
2. For each task:
   - Read task prompt file
   - Execute implementation
   - Run tests
   - Mark task complete
3. Update dashboard with progress
4. Report completion summary

**When to use**: After tasks are generated and ready to execute

**Example**:
```bash
/spec-kitty.implement
```

---

### `/spec-kitty.review`

**Purpose**: Perform structured code review and kanban transitions

**Pre-flight**: Must be in spec-kitty worktree after implementation

**Workflow**:
1. Review completed task files
2. Check code quality:
   - Tests passing
   - Documentation complete
   - No TODO comments
3. Transition tasks through kanban stages
4. Generate review report
5. Identify issues requiring fixes

**When to use**: After implementation, before acceptance

**Example**:
```bash
/spec-kitty.review
```

---

### `/spec-kitty.accept`

**Purpose**: Validate feature readiness and guide final acceptance

**Pre-flight**: Must have completed implementation

**Workflow**:
1. Run full test suite
2. Verify acceptance criteria from spec.md
3. Check documentation completeness
4. Run linters and security checks
5. Generate acceptance report
6. If all pass, prepare for merge

**When to use**: Final step before merging feature

**Example**:
```bash
/spec-kitty.accept
```

---

### `/spec-kitty.merge`

**Purpose**: Merge completed feature and clean up worktree

**Pre-flight**: Must pass acceptance checks

**Workflow**:
1. Verify acceptance status
2. Create feature PR
3. Wait for CI and approval
4. Merge to main
5. Delete worktree
6. Update dashboard
7. Archive feature artifacts

**When to use**: After feature is accepted

**Example**:
```bash
/spec-kitty.merge
```

---

### `/spec-kitty.clarify`

**Purpose**: Identify underspecified areas and ask clarification questions

**Pre-flight**: Must have spec.md in worktree

**Workflow**:
1. Analyze spec.md for ambiguities
2. Generate up to 5 targeted questions
3. Collect user answers
4. Update spec.md with clarifications
5. Display updated sections

**When to use**: When spec needs refinement before planning

**Example**:
```bash
/spec-kitty.clarify
```

---

### `/spec-kitty.research`

**Purpose**: Run Phase 0 research workflow before planning

**Pre-flight**: Must be in spec-kitty worktree

**Workflow**:
1. Identify research areas from spec
2. Search codebase for existing patterns
3. Investigate dependencies and APIs
4. Document findings in research/
5. Update spec.md with research insights

**When to use**: For complex features requiring codebase investigation

**Example**:
```bash
/spec-kitty.research
```

---

### `/spec-kitty.analyze`

**Purpose**: Cross-artifact consistency analysis

**Pre-flight**: Must have spec.md, plan.md, tasks.md

**Workflow**:
1. Read all three artifacts
2. Check for inconsistencies:
   - Requirements in spec missing from tasks
   - Plan decisions not reflected in tasks
   - Tasks not aligned with spec
3. Generate analysis report
4. Suggest corrections

**When to use**: After task generation, before implementation

**Example**:
```bash
/spec-kitty.analyze
```

---

### `/spec-kitty.constitution`

**Purpose**: Create or update project constitution

**Pre-flight**: No requirements

**Workflow**:
1. Collect project principles via prompts
2. Generate constitution.md
3. Update dependent templates
4. Ensure all specs align with principles

**When to use**: Setting up project governance or updating principles

**Example**:
```bash
/spec-kitty.constitution
```

---

### `/spec-kitty.checklist`

**Purpose**: Generate custom checklist for current feature

**Pre-flight**: Must have spec.md

**Workflow**:
1. Read spec.md requirements
2. Generate task checklist
3. Include testing requirements
4. Add documentation tasks
5. Save to checklist.md

**When to use**: Need manual checklist for tracking progress

**Example**:
```bash
/spec-kitty.checklist
```

---

### `/spec-kitty.dashboard`

**Purpose**: Open Spec-Kitty dashboard in browser

**Pre-flight**: None

**Workflow**:
1. Start dashboard server (if not running)
2. Open http://127.0.0.1:9237 in browser
3. Display all features with status

**When to use**: Want visual overview of all features

**Example**:
```bash
/spec-kitty.dashboard
```

---

## Skill Structure

### Anatomy of a Skill

Every skill follows this structure:

```markdown
---
description: Brief description of what the skill does
---

**Path reference rule:** ...

## User Input

```text
$ARGUMENTS
```

## Pre-flight Check

[Branch requirements and validations]

## Workflow Context

[When to use this skill and how it fits in the workflow]

## Outline

### Step 1: [First step]
[Detailed instructions]

### Step 2: [Second step]
[Detailed instructions]

...

## Troubleshooting

[Common issues and solutions]

## References

[Related documentation]
```

### Skill Conventions

**YAML Frontmatter**:
- Always include `description` field
- Keep description concise (< 100 chars)

**User Input**:
- Always have `$ARGUMENTS` section
- Handle both empty and provided arguments

**Pre-flight Checks**:
- Clearly state branch requirements
- Validate tool dependencies (gh, cargo, etc.)
- Fail fast with helpful error messages

**Workflow Steps**:
- Use numbered outline structure
- Include bash code examples
- Show expected output
- Handle both success and failure cases

**Error Handling**:
- Include Troubleshooting section
- Document common failures
- Provide resolution steps

---

## Creating New Skills

### Step 1: Define the Workflow

Ask yourself:
- What manual process does this automate?
- What are the pre-conditions?
- What are the expected outcomes?
- What can go wrong?

### Step 2: Choose a Namespace

- **`caro.*`** - Project-specific workflows (release, deps, etc.)
- **`spec-kitty.*`** - Feature development workflows
- **`<custom>.*`** - Your own namespace for experimental skills

### Step 3: Write the Skill File

Create `.claude/commands/namespace.skillname.md`:

```markdown
---
description: What this skill does in one line
---

[Follow standard structure from Anatomy section]
```

### Step 4: Test the Skill

1. Save the file
2. Run the skill: `/namespace.skillname`
3. Verify it works as expected
4. Handle edge cases

### Step 5: Document the Skill

Add entry to this guide (SKILLS_GUIDE.md) and update CLAUDE.md.

---

## Best Practices

### Do's

✅ **Make skills idempotent** - Running twice should be safe
✅ **Validate inputs** - Check preconditions before executing
✅ **Provide clear output** - User should understand what happened
✅ **Handle failures gracefully** - Don't leave system in broken state
✅ **Include examples** - Show expected command usage
✅ **Reference related skills** - Help users discover workflow connections

### Don'ts

❌ **Don't make direct commits to main** - Always use PR workflow
❌ **Don't skip error handling** - Always check command exit codes
❌ **Don't assume tool availability** - Check and install dependencies
❌ **Don't hard-code values** - Use variables and prompts
❌ **Don't ignore user input** - Always handle `$ARGUMENTS`
❌ **Don't mutate state silently** - Display what changed

---

## Workflow Diagrams

### Complete Release Workflow

```
┌─────────────────────┐
│ Dependency Updates  │
└──────────┬──────────┘
           │
           │ /caro.deps.review
           ▼
┌─────────────────────┐
│ Merge Dependabot    │
│ Update CHANGELOG    │
└──────────┬──────────┘
           │
           │ /caro.release.prepare
           ▼
┌─────────────────────┐
│ Create Release      │
│ Branch              │
└──────────┬──────────┘
           │
           │ /caro.release.security
           ▼
┌─────────────────────┐
│ Security Audit      │
│ Fix Vulnerabilities │
└──────────┬──────────┘
           │
           │ /caro.release.version
           ▼
┌─────────────────────┐
│ Version Bump        │
│ Update Docs         │
└──────────┬──────────┘
           │
           │ /caro.release.publish
           ▼
┌─────────────────────┐
│ Create PR           │
│ Merge & Tag         │
│ Publish to crates   │
└──────────┬──────────┘
           │
           │ /caro.release.verify
           ▼
┌─────────────────────┐
│ Test Installation   │
│ Verify Functionality│
└─────────────────────┘
```

### Spec-Kitty Feature Development

```
┌─────────────────────┐
│ bin/sk-new-feature  │
│ "description"       │
└──────────┬──────────┘
           │
           │ Creates worktree
           ▼
┌─────────────────────┐
│ /spec-kitty.specify │
└──────────┬──────────┘
           │
           │ Optional: /spec-kitty.clarify
           │ Optional: /spec-kitty.research
           ▼
┌─────────────────────┐
│ /spec-kitty.plan    │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│ /spec-kitty.tasks   │
└──────────┬──────────┘
           │
           │ Optional: /spec-kitty.analyze
           ▼
┌─────────────────────┐
│ /spec-kitty.        │
│ implement           │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│ /spec-kitty.review  │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│ /spec-kitty.accept  │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│ /spec-kitty.merge   │
└─────────────────────┘
```

---

## Troubleshooting

### "Command not found: /skill.name"

**Cause**: Skill file doesn't exist or has wrong name

**Solution**:
1. Check file exists: `ls .claude/commands/`
2. Verify filename matches: `namespace.skillname.md`
3. Ensure proper YAML frontmatter

### "ERROR: Must be on release branch"

**Cause**: Skill has branch enforcement and you're on wrong branch

**Solution**:
1. For `/caro.release.*` skills: Run `/caro.release.prepare` first
2. Or manually create branch: `git checkout -b release/v1.2.3`

### "ERROR: GitHub CLI not authenticated"

**Cause**: `gh` CLI not authenticated

**Solution**:
```bash
gh auth login
# Follow prompts to authenticate
```

### Skill hangs or times out

**Cause**: Waiting for CI or external service

**Solution**:
1. Check workflow status: `gh run list`
2. Cancel with Ctrl+C
3. Monitor manually: `gh run watch`

---

## References

- **Spec-Kitty Guide**: `docs/SPEC_KITTY_GUIDE.md`
- **Release Process**: `docs/RELEASE_PROCESS.md`
- **Project Constitution**: `.specify/memory/constitution.md`
- **CLAUDE.md**: Project-level AI agent instructions
- **Claude Code Docs**: https://docs.anthropic.com/claude/claude-code
