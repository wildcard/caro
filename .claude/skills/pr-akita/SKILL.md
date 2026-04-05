---
name: pr-akita
description: AI-powered PR triage and fix-merge workflow based on vibe-maintainer philosophy
---

# PR Akita Skill

**Purpose**: Triage open pull requests using the Vibe Maintainer philosophy. Default to "yes," fix issues rather than bouncing them back, and ensure every contributor feels valued.

**When to Use**: When reviewing incoming PRs, managing the PR queue, or when a specific PR needs disposition.

**Named After**: The Akita dog breed -- loyal, dedicated, protective of the community.

**Reference**: [ADR-015](../../docs/adr/ADR-015-vibe-maintainer-workflow.md) | [Steve Yegge: Vibe Maintainer](https://steve-yegge.medium.com/vibe-maintainer-a2273a841040)

---

## Prerequisites

Before using this skill:
- [ ] GitHub MCP tools are available
- [ ] You have push access to the repository
- [ ] You understand the caro project architecture (see CLAUDE.md)

---

## The 4-Phase Triage Workflow

```
Phase 1: Triage Scan (5 min)
   ↓
Phase 2: Fix-Merge Execution (10-30 min per PR)
   ↓
Phase 3: Attribution (2 min per PR)
   ↓
Phase 4: Report (5 min)
```

---

## Phase 1: Triage Scan

Fetch all open PRs and classify each into a triage bucket.

### Triage Buckets

| Bucket | Criteria | Disposition |
|--------|----------|-------------|
| **Easy Win** | Docs/config changes, <20 lines, CI passing, single concern | Merge |
| **Fix-Merge** | Sound intent but fixable issues (CI, lint, conflicts, minor bugs) | Merge-fix |
| **Needs-Review** | Substantial code, touches safety/inference/core, architectural implications | Flag for human |
| **Hygiene Issue** | Multi-concern, lingering draft, cross-project pollution | Guide kindly |
| **Retire** | Stale >14 days with no response, fundamentally misaligned | Close kindly |

### Classification Signals

**Easy Win signals**:
- Files changed are only in `docs/`, `*.md`, `.github/`, config files
- Less than 20 additions and 10 deletions
- CI is passing
- Single logical change

**Fix-Merge signals**:
- CI failing with known fixable patterns (clippy, fmt, missing imports)
- Merge conflicts that are straightforward to resolve
- Minor test failures from rebasing
- Lint issues or formatting problems
- The core idea/intent of the PR is sound

**Needs-Review signals**:
- Touches `src/safety/`, `src/inference/`, or `src/main.rs`
- Adds new dependencies
- Changes public API surface
- More than 200 lines changed
- Architectural implications

**Hygiene Issue signals**:
- PR title suggests multiple unrelated changes
- Draft PR open for more than 2 weeks
- References external projects or unrelated crates

**Retire signals**:
- No activity for 14+ days after comments/feedback
- Superseded by another PR
- Addresses a problem that no longer exists

---

## Phase 2: Fix-Merge Execution

For each PR classified as Fix-Merge:

### Step 1: Checkout
```bash
git fetch origin pull/<NUMBER>/head:pr-<NUMBER>
git checkout pr-<NUMBER>
```

### Step 2: Identify Issues
- Run `cargo build` to check compilation
- Run `cargo clippy` for lint issues
- Run `cargo fmt --check` for formatting
- Run `cargo test` for test failures
- Check for merge conflicts with main

### Step 3: Fix Issues
- Apply fixes for compilation errors, clippy warnings, formatting
- Resolve merge conflicts (rebase on main)
- Fix minor test failures

**Scope boundaries** -- escalate to Needs-Review if:
- Fix requires changing the PR's core logic
- Fix requires understanding domain-specific intent
- More than ~30 minutes of fix work needed

### Step 4: Preserve Attribution
```bash
# Add co-author trailer to fix commits
git commit --trailer "Co-authored-by: Original Author <email>"

# Or use --author for the main commit
git commit --author="Original Author <email@users.noreply.github.com>"
```

### Step 5: Push and Comment
```bash
git push origin pr-<NUMBER> --force-with-lease
```

Leave a comment:
> Thanks for this contribution! I fixed [specific issues] and merged.
> You get full credit as co-author. Welcome to the project!

### Step 6: Merge
Squash merge to keep history clean. Ensure the merge commit includes `Co-authored-by`.

---

## Phase 3: Attribution

For EVERY PR disposition, ensure contributor credit:

| Disposition | Attribution Method |
|---|---|
| Merge | Standard merge, contributor is commit author |
| Merge-fix | `Co-authored-by` trailer on fix commits |
| Cherry-pick | Reference original PR number in commit message |
| Split-merge | `Co-authored-by` on each split PR |
| Reimplement | "Based on #NNN by @contributor" in commit message |
| Retire | Thank the contributor in the closing comment |
| Reject | Thank the contributor, explain why, suggest alternatives |

---

## Phase 4: Report

Generate a summary report:

```
PR Akita Triage Report
══════════════════════
Date: YYYY-MM-DD
PRs scanned: N

Dispositions:
  Easy Win (Merged):     N
  Fix-Merge (Merged):    N
  Needs-Review (Flagged): N
  Hygiene (Guided):      N
  Retired:               N
  Rejected:              N

Merge Rate: NN%
Actions Taken: N

PRs Requiring Human Review:
  #NNN - [title] - [reason for escalation]
```

---

## Integration

- **PR Management Loop**: This skill can be invoked from `/pr-management-loop` during the vibe triage step
- **GitHub Actions**: The `pr-akita.yml` workflow auto-labels PRs with triage buckets
- **Good Boy Scout Rule**: Aligns with "just fix it" philosophy
- **Vibe Maintainer Rule**: See `.claude/rules/vibe-maintainer.md`

---

## Examples

See the `examples/` directory:
- [Easy Win walkthrough](examples/example-easy-win.md)
- [Fix-Merge walkthrough](examples/example-fix-merge.md)
