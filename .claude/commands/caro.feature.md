---
description: Unified guided workflow for feature development using Spec-Kitty. Detects current state and suggests next steps.
---

**Path reference rule:** When you mention directories or files, provide either the absolute path or a path relative to the project root (for example, `.worktrees/<feature>/`). Never refer to a folder by name alone.

## User Input

```text
$ARGUMENTS
```

You **MUST** consider the user input before proceeding (if not empty).

---

## Quick Reference

| Command | Action |
|---------|--------|
| `/caro.feature` | Auto-detect state, suggest next step |
| `/caro.feature status` | Show detailed feature status |
| `/caro.feature "Add caching"` | Start new feature |
| `/caro.feature plan` | Jump to planning phase |
| `/caro.feature tasks` | Jump to task generation |
| `/caro.feature implement` | Start/continue implementation |
| `/caro.feature implement WP03` | Implement specific work package |
| `/caro.feature review` | Run code review |
| `/caro.feature accept` | Run acceptance validation |
| `/caro.feature merge` | Merge and cleanup |

---

## What This Command Does

`/caro.feature` is an intelligent orchestrator that guides you through the complete Spec-Kitty feature workflow. It automatically detects where you are in the process and suggests the appropriate next step.

**Core Workflow**:
```
specify → clarify → plan → research → tasks → implement → review → accept → merge
```

**Smart Features**:
- Detects whether you're in main repo or a feature worktree
- Shows current phase and work package status
- Presents numbered options based on context
- Delegates to underlying `/spec-kitty.*` commands

---

## Outline

### 1. Parse Input and Determine Mode

Parse the `$ARGUMENTS` to determine operation mode:

```
ARGUMENTS check:
- Empty/whitespace only → AUTO_MODE (detect state, suggest next)
- "status" (case-insensitive) → STATUS_MODE (show status only)
- Quoted string (e.g., "Add caching") → NEW_FEATURE_MODE (start new feature)
- Phase keyword (plan|tasks|implement|review|accept|merge) → PHASE_JUMP_MODE
- "implement WP03" pattern → SPECIFIC_WP_MODE (implement specific WP)
```

### 2. Detect Current Context

Determine whether you're in a feature worktree or the main repository.

Run context detection:
```bash
pwd
git branch --show-current 2>/dev/null || echo "unknown"
```

**Context Logic**:
- If `git branch --show-current` returns a branch matching pattern `^\d{3}-` → **IN_WORKTREE=true**
  - Extract FEATURE_SLUG from branch name
  - Extract FEATURE_NUM from branch name
- Otherwise → **IN_WORKTREE=false**

**When NOT in worktree** (IN_WORKTREE=false):

Check for existing worktrees:
```bash
ls .worktrees/ 2>/dev/null | grep -E '^\d{3}-'
```

If worktrees exist AND mode is not NEW_FEATURE_MODE:
```
Found existing features:
  [1] 001-add-caching
  [2] 002-fix-authentication

Options:
  [1-2] Resume feature (enter number)
  [n] Start new feature
  [q] Quit

Your choice:
```

Wait for user input:
- If number (1-2): Navigate to that worktree using `cd .worktrees/<feature>/` and re-run detection
- If 'n': Continue to NEW_FEATURE_MODE
- If 'q': EXIT

If no worktrees exist OR mode is NEW_FEATURE_MODE:
- If no description provided in $ARGUMENTS:
  - Ask: "What feature would you like to build?"
  - Wait for description
- Invoke `/spec-kitty.specify` with the description
- EXIT (command completes after specify)

**When IN worktree** (IN_WORKTREE=true):
- Continue to Step 3 (Gather Feature State)

### 3. Gather Feature State (when IN_WORKTREE=true)

Run the tasks CLI status command:
```bash
python3 .kittify/scripts/tasks/tasks_cli.py status --feature "$FEATURE_SLUG" --json --lenient 2>/dev/null
```

**If JSON available**, parse the response to extract:
- `lanes.planned`: Array of WP IDs
- `lanes.doing`: Array of WP IDs
- `lanes.for_review`: Array of WP IDs
- `lanes.done`: Array of WP IDs
- `missing_artifacts`: Array of missing artifact names
- `ok`: Boolean indicating readiness

**Fallback detection** (if tasks_cli.py fails or is unavailable):

Check file existence:
```bash
test -f kitty-specs/$FEATURE_SLUG/spec.md && echo "spec_exists"
test -f kitty-specs/$FEATURE_SLUG/plan.md && echo "plan_exists"
test -f kitty-specs/$FEATURE_SLUG/tasks.md && echo "tasks_exists"
ls kitty-specs/$FEATURE_SLUG/tasks/planned/ 2>/dev/null | wc -l
ls kitty-specs/$FEATURE_SLUG/tasks/doing/ 2>/dev/null | wc -l
ls kitty-specs/$FEATURE_SLUG/tasks/for_review/ 2>/dev/null | wc -l
ls kitty-specs/$FEATURE_SLUG/tasks/done/ 2>/dev/null | wc -l
```

Check spec.md completeness:
```bash
grep -c "\[NEEDS CLARIFICATION" kitty-specs/$FEATURE_SLUG/spec.md 2>/dev/null || echo "0"
```

Check for acceptance/merge status in meta.json:
```bash
grep -q '"accepted_at"' kitty-specs/$FEATURE_SLUG/meta.json 2>/dev/null && echo "accepted"
grep -q '"merged_at"' kitty-specs/$FEATURE_SLUG/meta.json 2>/dev/null && echo "merged"
```

**Compute CURRENT_PHASE** based on detection results:

```
if spec.md missing → PHASE_NONE
elif spec.md has NEEDS CLARIFICATION markers → PHASE_SPECIFYING
elif plan.md missing → PHASE_SPECIFIED
elif tasks.md missing → PHASE_PLANNED
elif lanes.doing is non-empty → PHASE_IMPLEMENTING
elif lanes.for_review is non-empty → PHASE_REVIEWING
elif lanes.planned is non-empty → PHASE_TASKED
elif lanes.done has all WPs and not accepted → PHASE_DONE
elif accepted but not merged → PHASE_ACCEPTED
elif merged → PHASE_MERGED
else → PHASE_COMPLETE
```

### 4. Execute Based on Mode

#### 4.1 STATUS_MODE

Display comprehensive status:

```
================================================================================
Feature: {FEATURE_NUM}-{FEATURE_SLUG}
Branch: {CURRENT_BRANCH}
Phase: {CURRENT_PHASE}
Location: {WORKTREE_PATH}
================================================================================

Artifacts:
  [X] spec.md (complete)
  [X] plan.md (complete)
  [X] tasks.md (8 work packages)
  [ ] research.md (optional)
  [ ] data-model.md (optional)

Work Packages:
  planned:    WP04-validation, WP05-testing
  doing:      WP03-integration (assigned: claude, pid: 12345)
  for_review: WP02-cache-layer
  done:       WP01-setup

Current Phase: IMPLEMENTING
================================================================================
```

EXIT (do not suggest actions in status mode)

#### 4.2 NEW_FEATURE_MODE

If description is in $ARGUMENTS, use it directly.
Otherwise, ask: "What feature would you like to build?"

Invoke `/spec-kitty.specify` with the description:
```
Invoking /spec-kitty.specify with feature description...
```

EXIT after specify completes.

#### 4.3 PHASE_JUMP_MODE

Validate prerequisites for the requested phase:

| Requested Phase | Prerequisites |
|----------------|---------------|
| plan | spec.md must exist |
| tasks | plan.md must exist |
| implement | tasks.md must exist with WPs |
| review | At least one WP in for_review |
| accept | All WPs in done lane |
| merge | Feature must be accepted (meta.json has accepted_at) |

If prerequisites not met:
```
ERROR: Cannot proceed to {PHASE}. Missing prerequisites:
  - {prerequisite_1}
  - {prerequisite_2}

Suggested next step: {correct_command}
```
EXIT with error

If prerequisites met:
```
Jumping to {PHASE} phase...
```

Invoke the corresponding `/spec-kitty.{phase}` command:
- `plan` → `/spec-kitty.plan`
- `tasks` → `/spec-kitty.tasks`
- `implement` → `/spec-kitty.implement`
- `review` → `/spec-kitty.review`
- `accept` → `/spec-kitty.accept`
- `merge` → `/spec-kitty.merge`

EXIT after command completes.

#### 4.4 SPECIFIC_WP_MODE

Extract WP identifier from arguments (e.g., "WP03" from "implement WP03").

Verify WP exists:
```bash
ls kitty-specs/$FEATURE_SLUG/tasks/*/WP* 2>/dev/null | grep -i "$WP_ID"
```

If WP not found:
```
ERROR: Work package {WP_ID} not found.

Available work packages:
{list from tasks.md}
```
EXIT with error

If WP found:
```
Implementing work package {WP_ID}...
```

Invoke `/spec-kitty.implement` (which will prompt for WP selection or you can specify):
```
/spec-kitty.implement
```

In the implement prompt, mention which WP to focus on.

EXIT after implement completes.

#### 4.5 AUTO_MODE (Default)

Display brief status summary first:
```
Feature: {FEATURE_NUM}-{FEATURE_SLUG} ({CURRENT_PHASE})
Artifacts: {brief_list}
WPs: planned={count} doing={count} for_review={count} done={count}
```

Based on CURRENT_PHASE, present context-appropriate numbered options:

**PHASE_NONE:**
```
No feature found. Let's create one!

What feature would you like to build?
```
Wait for description, then invoke `/spec-kitty.specify`.

**PHASE_SPECIFYING:**
```
Your spec has {N} clarification markers remaining.

Options:
  [1] Continue editing spec.md
  [2] Mark spec as complete and proceed to planning
  [q] Quit

Your choice:
```

**PHASE_SPECIFIED:**
```
Spec is complete. Ready for planning.

Options:
  [1] Run clarification questions (optional)
  [2] Skip to planning
  [q] Quit

Your choice:
```

**PHASE_PLANNED:**
```
Plan is complete. Ready to generate work packages.

Options:
  [1] Generate work packages
  [2] Review plan.md first
  [q] Quit

Your choice:
```

**PHASE_TASKED:**
```
{N} work packages ready. None in progress.

Work packages:
  {list WPs from planned lane}

Options:
  [1] Start implementing WP01
  [2-N] Start implementing WP{N}
  [a] Analyze quality (optional)
  [q] Quit

Your choice:
```

**PHASE_IMPLEMENTING:**
```
{N} work package(s) in progress:
  - {WP_ID} (assigned: {agent}, pid: {pid})

Options:
  [1] Continue {WP_ID}
  [2] Move {WP_ID} to review
  [3] Start next work package
  [s] Show detailed status
  [q] Quit

Your choice:
```

**PHASE_REVIEWING:**
```
{N} work package(s) ready for review:
  {list WPs in for_review}

Options:
  [1-N] Review WP{N}
  [a] Review all pending
  [q] Quit

Your choice:
```

**PHASE_DONE:**
```
All work packages complete!

Options:
  [1] Run acceptance checks
  [2] Review acceptance criteria
  [q] Quit

Your choice:
```

**PHASE_ACCEPTED:**
```
Feature accepted and ready to merge.

Acceptance details:
  - Accepted at: {timestamp}
  - Accepted by: {actor}

Options:
  [1] Merge to main
  [2] Review acceptance details
  [q] Quit

Your choice:
```

**PHASE_MERGED:**
```
Feature complete and merged!

What's next?
  [1] Start new feature
  [2] Clean up worktree
  [q] Quit

Your choice:
```

**Wait for user input and execute selected action:**

Parse the user's choice:
- If number: Execute the corresponding action (invoke appropriate `/spec-kitty.*` command)
- If 'q': EXIT
- If 's' (in IMPLEMENTING phase): Show detailed status using STATUS_MODE logic, then re-display options
- Otherwise: Show error and re-display options

### 5. Post-Action Guidance

After any delegated `/spec-kitty.*` command completes:

Re-run state detection (steps 2-3) to get updated phase.

Display brief update:
```
✓ {PHASE} complete.

Updated status:
  Phase: {NEW_PHASE}
  {brief status}
```

Ask:
```
Continue with next step? [Y/n]:
```

If 'Y' or Enter: Return to AUTO_MODE (step 4.5)
If 'n': EXIT with message "Run /caro.feature when ready to continue."

---

## Phase Transition Table

| Current Phase | Next Command | Description |
|--------------|--------------|-------------|
| NONE | /spec-kitty.specify | Create feature specification |
| SPECIFYING | Continue editing or /spec-kitty.clarify | Resolve ambiguities |
| SPECIFIED | /spec-kitty.plan | Create implementation plan |
| PLANNED | /spec-kitty.tasks | Generate work packages |
| TASKED | /spec-kitty.implement | Start implementation |
| IMPLEMENTING | Continue or /spec-kitty.review | Review completed work |
| REVIEWING | /spec-kitty.review | Approve or request changes |
| DONE | /spec-kitty.accept | Validate merge readiness |
| ACCEPTED | /spec-kitty.merge | Merge and cleanup |
| MERGED | /caro.feature "New feature" | Start next feature |

---

## Error Handling

### Location Errors

If PHASE_JUMP_MODE or SPECIFIC_WP_MODE requested but NOT in worktree:
```
ERROR: This operation requires being in a feature worktree.

Available worktrees:
  {list from ls .worktrees/}

To resume a feature:
  cd .worktrees/{feature-slug}
  /caro.feature

To start a new feature:
  /caro.feature "Feature description"
```

### Missing Prerequisites

```
ERROR: Cannot proceed to {PHASE}. Missing:
  - {artifact_1} (run /spec-kitty.{command} to create)
  - {artifact_2}

Current phase: {CURRENT_PHASE}
Suggested: {correct_next_command}
```

### Script Failures

If `tasks_cli.py` fails or is unavailable:
```
WARNING: Could not run tasks status command. Using fallback detection.

(Continue with file-based detection)
```

If `.kittify` scripts are completely missing:
```
ERROR: Spec-Kitty infrastructure not found.

This command requires the Spec-Kitty framework to be set up.
Check that .kittify/ directory exists in the project root.

See docs/SPEC_KITTY_GUIDE.md for setup instructions.
```

---

## Examples

### Starting from scratch
```
User: /caro.feature
Claude: What feature would you like to build?
User: Add Redis caching with TTL support
Claude: [runs /spec-kitty.specify]
```

### Resuming existing feature
```
User: /caro.feature
Claude: Found existing features:
  [1] 001-add-caching
  [2] 002-fix-auth

  Options:
  [1-2] Resume feature
  [n] Start new
  [q] Quit

User: 1
Claude: [loads 001-add-caching status, shows current phase]
```

### Phase jump
```
User: /caro.feature implement
Claude: Jumping to IMPLEMENT phase...
        [runs /spec-kitty.implement]
```

### Specific work package
```
User: /caro.feature implement WP03
Claude: Implementing work package WP03...
        [runs /spec-kitty.implement focused on WP03]
```

### Status check
```
User: /caro.feature status
Claude: [displays comprehensive status, then exits]
```
