---
description: Project manager skill for roadmap alignment, work selection, and workflow routing
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
| `/caro.roadmap` | Show roadmap status (default) |
| `/caro.roadmap next` | Get next recommended work item |
| `/caro.roadmap select` | Interactive work selection |
| `/caro.roadmap select --area CLI` | Filter by area |
| `/caro.roadmap start #123` | Start work on issue |
| `/caro.roadmap complete #123` | Mark issue as done |
| `/caro.roadmap blocked` | List all blockers |
| `/caro.roadmap profile` | Show current agent profile |
| `/caro.roadmap profile rust` | Switch to rust expert profile |

---

## What This Command Does

`/caro.roadmap` is your project manager assistant that helps you:
- **Find the right work**: Auto-select tasks based on priority, milestone urgency, and your expertise
- **Align with roadmap**: Ensure work matches project milestones and strategic goals
- **Route workflows**: Automatically choose spec-kitty (quick iterations) or spec-kit (large scope)
- **Track progress**: Update GitHub projects, milestones, and roadmap status

**Core Workflow**:
```
STATUS → NEXT → START → [implement] → COMPLETE → STATUS
```

---

## Pre-flight Checks

Run these checks before proceeding:

```bash
# Verify GitHub CLI
which gh || echo "ERROR: gh CLI not installed (https://cli.github.com)"

# Verify authentication
gh auth status 2>&1 | grep -q "Logged in" || echo "ERROR: Run 'gh auth login'"
```

**If checks fail**: Stop and ask user to install/configure GitHub CLI.

---

## Outline

### 1. Parse Arguments and Determine Mode

Parse `$ARGUMENTS` to determine the operation mode:

```
ARGUMENTS patterns:
- Empty/whitespace only → STATUS_MODE (show roadmap overview)
- "next" (case-insensitive) → NEXT_MODE (auto-select next work)
- "select" → SELECT_MODE (interactive selection)
- "select --area <area>" → SELECT_MODE with area filter
- "select --milestone <milestone>" → SELECT_MODE with milestone filter
- "start #123" OR "start 123" → START_MODE (begin work on issue)
- "complete #123" OR "complete 123" → COMPLETE_MODE (mark as done)
- "blocked" → BLOCKED_MODE (list blockers)
- "profile" → PROFILE_MODE (show current profile)
- "profile <name>" → PROFILE_MODE (switch profile)
```

### 2. Load Context

**Load ROADMAP.md**:
```bash
cat ROADMAP.md
```

Extract:
- Milestone due dates (v1.1.0: Feb 15, v1.2.0: Mar 31, v2.0.0: Jun 30)
- Open/closed counts per milestone
- Strategic themes and priorities

**Load Agent Profile** (if exists):
```bash
test -f .claude/agent-profiles.yaml && cat .claude/agent-profiles.yaml
```

If file exists, parse YAML to get:
- Current profile name
- Areas of expertise
- Strengths

If file doesn't exist, use default profile:
```yaml
current_profile: default
profiles:
  default:
    name: "General Agent"
    areas: []
    strengths: []
```

### 3. Execute Mode-Specific Logic

#### 3.1 STATUS_MODE (Default)

**Query milestone progress**:
```bash
gh api repos/wildcard/caro/milestones --jq '.[] | {title, due_on, open_issues, closed_issues}'
```

**Calculate progress for each milestone**:
```
Progress = (closed_issues / (open_issues + closed_issues)) * 100
```

**Check for release blockers**:
```bash
gh issue list --label release-blocker --state open --json number,title,milestone --jq 'length'
```

**Display roadmap status**:
```
================================================================================
Caro Development Roadmap - Status Overview
Last Updated: [current date from ROADMAP.md]
================================================================================

Milestones:
  [🎯] v1.1.0 - Core Improvements
      Due: Feb 15, 2025 (XX days remaining)
      Status: [##########----------] 14 open, 1 closed (7%)
      Focus: Production-ready core functionality

  [🌐] v1.2.0 - Website & Documentation Launch
      Due: Mar 31, 2025 (XX days remaining)
      Status: [--------------------] 24 open, 0 closed (0%)
      Focus: Public launch, marketing, and documentation

  [🚀] v2.0.0 - Advanced Features
      Due: Jun 30, 2025 (XX days remaining)
      Status: [#-------------------] 19 open, 2 closed (9%)
      Focus: Innovation and advanced capabilities

================================================================================
Release Blockers: [count]
[If blockers > 0, list them with issue numbers]

Current Agent Profile: [profile name]
  Areas: [areas list]
  Strengths: [strengths list]

Suggested Next Work:
  [Run NEXT selection algorithm, show top candidate]

Commands:
  /caro.roadmap next      → Start suggested work
  /caro.roadmap select    → Browse all work items
  /caro.roadmap blocked   → Focus on blockers
  /caro.roadmap profile   → Change expertise profile
================================================================================
```

EXIT (do not proceed to other modes)

#### 3.2 NEXT_MODE

**Apply full selection algorithm**:

1. **Query all open issues**:
```bash
gh issue list --state open --json number,title,labels,milestone,assignees --limit 100
```

2. **Apply selection algorithm** (priority order):

   a. **BLOCKER_CHECK**:
      - Filter issues with label "release-blocker"
      - If any exist, these take absolute priority
      - Score: 1000

   b. **MILESTONE_PRIORITY**:
      - Assign scores based on milestone due date:
        * v1.1.0 (Feb 15): +300
        * v1.2.0 (Mar 31): +200
        * v2.0.0 (Jun 30): +100
        * No milestone: +0

   c. **PRIORITY_SORT** (from labels):
      - Extract priority from labels matching `priority/*`:
        * priority/critical: +50
        * priority/high: +30
        * priority/medium: +20
        * priority/low: +10
        * (no priority label): +15 (default medium)

   d. **STATUS_FILTER**:
      - SKIP issues with labels: "blocked", "on-hold", "waiting"
      - SKIP issues with assignees (already being worked on)

   e. **AREA_MATCH** (if agent profile has areas):
      - Check if issue has matching area label
      - Area labels: `area/core-cli`, `area/safety`, `area/backends`, etc.
      - If match: +25
      - If no match but related: +10
      - If no area data: +0

3. **Sort by total score** (descending)

4. **Return top candidate**:
```
================================================================================
Next Recommended Work
================================================================================

Issue #[number]: [title]
Milestone: [milestone] (Due: [date])
Priority: [priority]
Area: [area]
Labels: [relevant labels]

Score Breakdown:
  Blocker: +[score]
  Milestone: +[score] ([milestone name])
  Priority: +[score] ([priority level])
  Area Match: +[score] ([matched area])
  TOTAL: [total score]

Why this issue?
  [1-2 sentence explanation based on scoring]

Next Steps:
  /caro.roadmap start #[number]    → Begin implementation
  /caro.roadmap select             → See other options
  /caro.roadmap blocked            → Check blockers first

Context from ROADMAP.md:
  [Brief excerpt about this issue from roadmap if found]
================================================================================
```

EXIT

#### 3.3 SELECT_MODE

**Parse filters from arguments**:
```
--area <area>        → Filter by area (e.g., --area "Core CLI")
--milestone <milestone> → Filter by milestone (e.g., --milestone "v1.1.0")
--priority <priority> → Filter by priority (e.g., --priority "high")
--type <type>        → Filter by type (e.g., --type "bug")
```

**Query issues with filters**:
```bash
# Base query
QUERY="gh issue list --state open --json number,title,labels,milestone"

# Add filters
[if --milestone provided]: QUERY="$QUERY --milestone '$MILESTONE_VALUE'"
[if --area provided]: filter results by area label
[if --priority provided]: filter results by priority label
[if --type provided]: filter results by type label
```

**Present filtered list** (max 10 items):
```
================================================================================
Available Work Items
Filters: [show active filters]
================================================================================

  [1] #123 - Implement Hugging Face model download
      Milestone: v1.1.0 | Priority: High | Area: Backends

  [2] #132 - Performance analysis and optimization
      Milestone: v1.1.0 | Priority: High | Area: Core CLI

  [3] #135 - Build LLM evaluation harness
      Milestone: v1.1.0 | Priority: High | Area: Backends

  ... (up to 10 total)

Options:
  [1-10] Select issue number
  [n] Show next 10 items
  [f] Change filters
  [q] Cancel

Your choice:
```

**Wait for user input**:
- If number (1-10): Display full issue details, suggest `/caro.roadmap start #XXX`
- If 'n': Show next page of results
- If 'f': Prompt for new filters
- If 'q': EXIT

#### 3.4 START_MODE

Extract issue number from arguments (e.g., "start #123" → 123).

**Fetch issue details**:
```bash
gh issue view $ISSUE_NUMBER --json number,title,labels,milestone,body
```

**Determine workflow routing**:

```
ROUTING_LOGIC:

1. Check labels:
   IF has label "architecture" OR "research" OR "major-refactor":
     → ROUTE = spec-kit

   ELSE IF has label "quick-fix" OR "bug" OR "enhancement":
     → ROUTE = spec-kitty

2. Check milestone + estimated scope:
   IF milestone is v2.0.0 AND no quick-fix label:
     → ROUTE = spec-kit

   ELSE IF milestone is v1.1.0 OR v1.2.0:
     → ROUTE = spec-kitty (default for near-term work)

3. Fallback - Ask user:
   "How would you classify this work?
     [1] Quick iteration (1-2 weeks) → spec-kitty
     [2] Large scope (> 2 weeks) → spec-kit"
```

**Route to appropriate workflow**:

**If ROUTE = spec-kitty**:
```
Routing to Spec-Kitty workflow (rapid iteration)...

This issue will be implemented using spec-kitty's worktree-based workflow.

Next steps:
  1. Run: bin/sk-new-feature "Issue #[number]: [title]"
  2. This will:
     - Create isolated worktree at .worktrees/[NNN]-[slug]/
     - Generate spec.md from issue description
     - Enter guided feature development workflow

  3. Or use the unified command:
     /caro.feature "Issue #[number]: [title]"

Creating worktree now...
```

Execute:
```bash
cd /Users/kobik-private/workspace/caro
bin/sk-new-feature "Issue #$ISSUE_NUMBER: $ISSUE_TITLE"
```

**If ROUTE = spec-kit**:
```
Routing to Spec-Kit workflow (large scope)...

This issue requires the full spec-kit methodology for complex features.

Next steps:
  1. Create spec directory: specs/[NNN]-[slug]/
  2. Use templates from .specify/templates/
  3. Create these artifacts:
     - spec.md (comprehensive specification)
     - plan.md (architecture and design)
     - tasks.md (phased implementation)
     - research.md (if needed for technical decisions)

  4. Follow .specify/memory/constitution.md principles

The spec-kit workflow is manual and designed for features requiring:
  - Extensive research and prototyping
  - Architectural decisions
  - Multiple competing approaches
  - Long-running development (> 2 weeks)

Next step directory path:
  specs/[NNN]-[slug]/

Would you like me to create the directory structure now? (yes/no)
```

If yes:
```bash
# Find next spec number
NEXT_NUM=$(ls -1 specs/ | grep -E '^\d{3}-' | tail -1 | cut -d'-' -f1)
NEXT_NUM=$((NEXT_NUM + 1))
NEXT_NUM_PADDED=$(printf "%03d" $NEXT_NUM)

# Create directory
mkdir -p specs/$NEXT_NUM_PADDED-$SLUG

# Copy templates
cp .specify/templates/spec.md specs/$NEXT_NUM_PADDED-$SLUG/
cp .specify/templates/plan.md specs/$NEXT_NUM_PADDED-$SLUG/
cp .specify/templates/tasks.md specs/$NEXT_NUM_PADDED-$SLUG/

echo "Created: specs/$NEXT_NUM_PADDED-$SLUG/"
echo "Next: Edit spec.md to define the feature"
```

**Update issue status**:
```bash
# Add in-progress label
gh issue edit $ISSUE_NUMBER --add-label "in-progress"

# Add assignee (if possible to detect current agent/user)
gh issue edit $ISSUE_NUMBER --add-assignee @me
```

EXIT

#### 3.5 COMPLETE_MODE

Extract issue number from arguments (e.g., "complete #123" → 123).

**Verify issue was assigned to current agent**:
```bash
gh issue view $ISSUE_NUMBER --json assignees --jq '.assignees[].login'
```

**Ask for confirmation**:
```
Mark issue #$ISSUE_NUMBER as complete?

This will:
  - Close the issue on GitHub
  - Remove "in-progress" label
  - Update project board status to "Done"
  - Update ROADMAP.md progress (if milestone item)

Continue? (yes/no):
```

If yes:

```bash
# Close issue
gh issue close $ISSUE_NUMBER --comment "Completed via /caro.roadmap"

# Remove in-progress label
gh issue edit $ISSUE_NUMBER --remove-label "in-progress"
```

**Update ROADMAP.md if milestone item**:
- Read ROADMAP.md
- Find reference to issue #$ISSUE_NUMBER
- Update progress counts
- Recalculate percentages

**Suggest next work**:
```
✓ Issue #$ISSUE_NUMBER marked as complete!

Updated status:
  Milestone: [milestone] now at XX% complete

Suggested next work:
  [Run NEXT selection algorithm]

Commands:
  /caro.roadmap next     → Start suggested work
  /caro.roadmap          → View full status
```

EXIT

#### 3.6 BLOCKED_MODE

**Query all release blockers**:
```bash
gh issue list --label release-blocker --state open --json number,title,milestone,labels --jq '.'
```

**Group by milestone**:
```
================================================================================
Release Blockers
================================================================================

v1.1.0 (Feb 15, 2025):
  [1] #150 - Fix error blocking release [CLOSED ✓]

v1.2.0 (Mar 31, 2025):
  (no blockers)

v2.0.0 (Jun 30, 2025):
  (no blockers)

No Milestone:
  (no blockers)

================================================================================
Total: 0 active blockers

✓ All release blockers resolved! Safe to proceed with releases.

Next milestone: v1.1.0 (XX days remaining)
  /caro.roadmap next → See next recommended work
================================================================================
```

**If blockers exist**:
```
Total: [count] active blockers

⚠️ CRITICAL: These issues are blocking releases!

Suggested resolution order:
  1. Earliest milestone first (v1.1.0 > v1.2.0 > v2.0.0)
  2. Within milestone: Critical > High > Medium

Next action:
  /caro.roadmap start #[first_blocker] → Fix highest priority blocker
```

EXIT

#### 3.7 PROFILE_MODE

**If no profile name provided** (just "/caro.roadmap profile"):

Load and display current profile:
```bash
cat .claude/agent-profiles.yaml 2>/dev/null || echo "current_profile: default"
```

Display:
```
================================================================================
Agent Profile
================================================================================

Current Profile: [profile_name]

Name: [name]
Areas of Expertise:
  - [area 1]
  - [area 2]
  - [area 3]

Strengths:
  - [strength 1]
  - [strength 2]

Available Profiles:
  - default: General Agent
  - rust: Rust CLI Expert (areas: Core CLI, Backends, Safety)
  - docs: Documentation Writer (areas: DX, Website)
  - devops: DevOps Engineer (areas: DevOps, Infrastructure)

Commands:
  /caro.roadmap profile <name>  → Switch profile
  /caro.roadmap next            → Get work matching your expertise
================================================================================
```

EXIT

**If profile name provided** (e.g., "/caro.roadmap profile rust"):

Validate profile exists in `.claude/agent-profiles.yaml`:
```yaml
profiles:
  rust:
    name: "Rust CLI Expert"
    areas: ["Core CLI", "Backends", "Safety"]
    strengths: ["TDD", "error handling", "performance"]
```

If not found:
```
ERROR: Profile '[name]' not found.

Available profiles:
  - default
  - rust
  - docs
  - devops

To create a new profile, edit .claude/agent-profiles.yaml
```

If found, update current_profile:
```yaml
current_profile: rust
```

Display:
```
✓ Profile switched to: Rust CLI Expert

Areas: Core CLI, Backends, Safety
Strengths: TDD, error handling, performance

Work selection will now prioritize issues matching these areas.

Next:
  /caro.roadmap next    → Get work matching your expertise
  /caro.roadmap         → View roadmap status
```

EXIT

---

## Selection Algorithm Details (For Implementers)

The selection algorithm is a weighted scoring system that considers multiple factors:

### Scoring Formula

```
TOTAL_SCORE = BLOCKER_SCORE + MILESTONE_SCORE + PRIORITY_SCORE + AREA_SCORE

Where:
  BLOCKER_SCORE:
    - Has "release-blocker" label: 1000
    - Otherwise: 0

  MILESTONE_SCORE:
    - v1.1.0 (Feb 15, 2025): 300
    - v1.2.0 (Mar 31, 2025): 200
    - v2.0.0 (Jun 30, 2025): 100
    - No milestone: 0

  PRIORITY_SCORE:
    - priority/critical: 50
    - priority/high: 30
    - priority/medium: 20
    - priority/low: 10
    - (no label): 15 (assume medium)

  AREA_SCORE (if agent profile has areas):
    - Perfect match (issue area in agent areas): 25
    - Related (similar area): 10
    - No match: 0
```

### Example Scoring

Issue: #10 "Implement Hugging Face model download"
- Labels: priority/high, area/backends
- Milestone: v1.1.0
- No blocker label
- Agent profile: rust (areas: Core CLI, Backends, Safety)

Calculation:
```
BLOCKER_SCORE: 0 (no blocker label)
MILESTONE_SCORE: 300 (v1.1.0)
PRIORITY_SCORE: 30 (priority/high)
AREA_SCORE: 25 (perfect match: backends)

TOTAL: 355
```

---

## Error Handling

### GitHub CLI Not Installed
```
ERROR: GitHub CLI (gh) is not installed.

Install instructions:
  macOS: brew install gh
  Linux: https://github.com/cli/cli/blob/trunk/docs/install_linux.md
  Windows: https://github.com/cli/cli/releases

Then run: gh auth login
```

### GitHub Not Authenticated
```
ERROR: GitHub CLI is not authenticated.

Run: gh auth login

This will open a browser to authorize the CLI.
```

### Invalid Issue Number
```
ERROR: Issue #[number] not found in wildcard/caro.

Verify the issue exists:
  https://github.com/wildcard/caro/issues/[number]
```

### Missing Agent Profile File
```
WARNING: Agent profile file not found.

Creating default profile at: .claude/agent-profiles.yaml

You can customize this file to match your expertise.
```

Then create the file with default content.

### Invalid Profile Name
```
ERROR: Profile '[name]' not found in .claude/agent-profiles.yaml

Available profiles:
  - default
  - rust
  - docs
  - devops

Edit .claude/agent-profiles.yaml to add new profiles.
```

---

## Examples

### Example 1: Check roadmap status
```
User: /caro.roadmap
Claude: [Shows STATUS mode output with milestones, blockers, and suggested next work]
```

### Example 2: Get next work item
```
User: /caro.roadmap next
Claude:
  Next Recommended Work: Issue #10
  Title: Implement Hugging Face model download
  Milestone: v1.1.0 (Feb 15)
  Priority: High
  Area: Backends

  Score: 355 (Milestone +300, Priority +30, Area Match +25)

  Next: /caro.roadmap start #10
```

### Example 3: Interactive selection
```
User: /caro.roadmap select --area "Core CLI"
Claude:
  Available Work Items (filtered by area: Core CLI)

  [1] #9 - Add benchmark suite (v1.1.0, Medium)
  [2] #147 - Add machine resource assessment (v1.1.0, Medium)
  [3] #146 - Plan new interface architecture (v1.1.0, Medium)

  Your choice: 1

  Issue #9: Add benchmark suite
  [Shows full details]

  Next: /caro.roadmap start #9
```

### Example 4: Start work with spec-kitty routing
```
User: /caro.roadmap start #9
Claude:
  Routing to Spec-Kitty workflow...

  Creating worktree for: Issue #9 - Add benchmark suite

  [Executes: bin/sk-new-feature "Issue #9: Add benchmark suite"]

  ✓ Worktree created at: .worktrees/001-add-benchmark-suite/
  ✓ Issue #9 marked as "in-progress"

  Next: cd .worktrees/001-add-benchmark-suite && /caro.feature
```

### Example 5: Complete work
```
User: /caro.roadmap complete #150
Claude:
  Mark issue #150 as complete?

  This will close the issue and update the roadmap.
  Continue? (yes/no)

User: yes
Claude:
  ✓ Issue #150 closed
  ✓ v1.1.0 milestone progress: 7% → 13% (1 → 2 completed)

  Suggested next work: Issue #10 (Hugging Face model download)

  /caro.roadmap next → View details
```

### Example 6: Check blockers
```
User: /caro.roadmap blocked
Claude:
  Release Blockers: 0 active

  ✓ All blockers resolved!

  Safe to proceed with v1.1.0 release preparation.
```

### Example 7: Switch agent profile
```
User: /caro.roadmap profile rust
Claude:
  ✓ Profile switched to: Rust CLI Expert

  Areas: Core CLI, Backends, Safety
  Strengths: TDD, error handling, performance

  Work selection will prioritize these areas.

  /caro.roadmap next → Get matching work
```

---

## Integration with Other Skills

### With /caro.feature (Spec-Kitty)
When routing to spec-kitty workflow:
```
/caro.roadmap start #123 → Routes to spec-kitty
  → bin/sk-new-feature creates worktree
  → Automatically suggests: /caro.feature
```

### With /caro.release.* (Release Workflow)
Before starting a release:
```
/caro.roadmap blocked → Check for release blockers
/caro.roadmap → Verify milestone progress
```

### With Spec-Kit Workflow
When routing to spec-kit:
```
/caro.roadmap start #456 → Routes to spec-kit
  → Creates specs/NNN-feature/ directory
  → User manually follows constitution-based development
```

---

## Notes for Maintainers

- **Scoring weights** can be adjusted in the Selection Algorithm section
- **Workflow routing rules** can be modified in START_MODE section
- **Agent profiles** are stored in `.claude/agent-profiles.yaml` (YAML format)
- **Integration points** with spec-kitty rely on `bin/sk-new-feature` script
- **GitHub API calls** use `gh` CLI for authentication and rate limiting

**Performance considerations**:
- Issue queries are limited to 100 items to avoid rate limits
- ROADMAP.md is read from disk (no API call)
- Profile config is cached per session (read once)

**Future enhancements**:
- Add `--watch` mode for continuous monitoring
- Integrate with project boards (Projects v2 API)
- Add custom scoring formulas per project
- Support multiple repositories
