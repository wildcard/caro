# Roadmap Sync Module

Synchronize roadmap data across three sources:
1. **GitHub** (milestones + issues) - **SOURCE OF TRUTH**
2. **ROADMAP.md** - Markdown documentation
3. **website/src/pages/roadmap.astro** - Public roadmap page

---

## Process

### Step 1: Fetch GitHub Data (Ground Truth)

**Fetch milestones**:
```bash
gh api repos/wildcard/caro/milestones --jq '.[] | {title, due_on, open_issues, closed_issues}'
```

Store results:
```
v1.1.0:
  due_date: 2026-02-15
  open: 20
  closed: 1
  total: 21

v1.2.0:
  due_date: 2026-03-31
  open: 29
  closed: 0
  total: 29

v2.0.0:
  due_date: 2026-06-30
  open: 13
  closed: 8
  total: 21
```

**Calculate progress**:
```
Progress = (closed / total) * 100
```

**Calculate days remaining**:
```bash
# For each milestone
DUE_DATE="2026-02-15"
TODAY=$(date +%Y-%m-%d)
DAYS=$(( ( $(date -d "$DUE_DATE" +%s) - $(date -d "$TODAY" +%s) ) / 86400 ))
```

---

### Step 2: Read ROADMAP.md Current Values

**Extract data using grep/awk**:

```bash
# Get "Last Updated" date
grep "Last Updated:" ROADMAP.md | sed 's/.*: //'

# Get milestone status lines
grep -A 1 "v1.1.0 - Core Improvements" ROADMAP.md | grep "Status:"
```

Parse to extract:
- Last updated date
- Each milestone's:
  - Open count
  - Closed count
  - Progress percentage
  - Days remaining

---

### Step 3: Read website/roadmap.astro Current Values

**Extract JavaScript object values**:

```bash
# For v1.1.0
sed -n '/version: "v1.1.0"/,/}/p' website/src/pages/roadmap.astro
```

Parse to extract:
- `totalItems`
- `completedItems`
- `progress`
- `daysRemaining`

---

### Step 4: Detect Drift

**Compare GitHub truth with ROADMAP.md**:

```
For each milestone (v1.1.0, v1.2.0, v2.0.0):
  IF GitHub open_issues != ROADMAP.md open_count:
    → DRIFT DETECTED

  IF GitHub closed_issues != ROADMAP.md closed_count:
    → DRIFT DETECTED

  IF Calculated progress != ROADMAP.md progress:
    → DRIFT DETECTED

  IF Calculated days_remaining != ROADMAP.md days_remaining:
    → DRIFT DETECTED
```

**Compare GitHub truth with website/roadmap.astro**:

```
For each milestone:
  IF GitHub total != website totalItems:
    → DRIFT DETECTED

  IF GitHub closed_issues != website completedItems:
    → DRIFT DETECTED

  IF Calculated progress != website progress:
    → DRIFT DETECTED

  IF Calculated days_remaining != website daysRemaining:
    → DRIFT DETECTED
```

**Report drift**:

```
================================================================================
Roadmap Drift Detection
================================================================================

Source of Truth: GitHub API (gh api repos/wildcard/caro/milestones)

v1.1.0 - Core Improvements:
  GitHub (TRUTH):    20 open, 1 closed, 21 total (5% complete, 46 days)
  ROADMAP.md:        15 open, 1 closed, 16 total (7% complete, 48 days) ✗ DRIFT
  website/roadmap:   20 open, 1 closed, 21 total (5% complete, 46 days) ✓ SYNCED

v1.2.0 - Website & Documentation:
  GitHub (TRUTH):    29 open, 0 closed, 29 total (0% complete, 91 days)
  ROADMAP.md:        24 open, 0 closed, 24 total (0% complete, 93 days) ✗ DRIFT
  website/roadmap:   29 open, 0 closed, 29 total (0% complete, 91 days) ✓ SYNCED

v2.0.0 - Advanced Features:
  GitHub (TRUTH):    13 open, 8 closed, 21 total (38% complete, 182 days)
  ROADMAP.md:        13 open, 8 closed, 21 total (38% complete, 184 days) ✗ DRIFT
  website/roadmap:   13 open, 8 closed, 21 total (38% complete, 182 days) ✓ SYNCED

================================================================================

Summary:
  - ROADMAP.md: 3 milestones need updates (dates, counts, percentages)
  - website/roadmap: All synced ✓

Recommended action:
  /caro.sync roadmap → Apply updates now
  /caro.sync --check roadmap → View this report again

================================================================================
```

---

### Step 5: Apply Updates

**Only execute this step if user confirms**

**Ask for confirmation**:
```
Apply updates to sync ROADMAP.md and website/roadmap.astro with GitHub?

This will:
  - Update ROADMAP.md milestone counts, progress, and dates
  - Update website/roadmap.astro data values
  - Update "Last Updated" timestamp in ROADMAP.md

Continue? (yes/no):
```

If yes:

**Update ROADMAP.md**:

Use `Edit` tool to:
1. Update "Last Updated" date to today
2. For each milestone section:
   - Update "Due Date" with days remaining
   - Update "Status" with correct counts and progress
   - Update deliverables if new issues found
   - Mark completed issues with ✅
3. Update "Current Status Summary" table

**Update website/roadmap.astro**:

Use `Edit` tool to:
1. For each milestone object:
   - Update `totalItems`
   - Update `completedItems`
   - Update `progress`
   - Update `daysRemaining`
2. Update deliverables arrays if needed

**Verify updates**:
```bash
# Show git diff to user
git diff ROADMAP.md
git diff website/src/pages/roadmap.astro
```

**Report completion**:
```
✓ ROADMAP.md updated
✓ website/roadmap.astro updated

Synced with GitHub truth:
  - v1.1.0: 20 open, 1 closed (5% complete)
  - v1.2.0: 29 open, 0 closed (0% complete)
  - v2.0.0: 13 open, 8 closed (38% complete)

Next steps:
  1. Review changes: git diff ROADMAP.md website/src/pages/roadmap.astro
  2. Commit changes: git add -A && git commit -m "sync: Update roadmap data"
  3. Verify website: Run local dev server to preview

The roadmap is now in sync with GitHub! 🎯
```

---

## Error Handling

### GitHub CLI Not Available
```
ERROR: GitHub CLI is required for roadmap sync.

The roadmap sync module uses GitHub as the source of truth.
Without GitHub CLI, we cannot fetch milestone and issue data.

Install: https://cli.github.com/
Auth: gh auth login

Exiting roadmap sync.
```

### GitHub API Error
```
ERROR: Failed to fetch data from GitHub API.

Command failed: gh api repos/wildcard/caro/milestones

Possible causes:
  - Network connectivity issue
  - GitHub API rate limit reached
  - Repository access permissions

Try:
  - Check internet connection
  - Wait a few minutes and retry
  - Verify: gh auth status

Exiting roadmap sync.
```

### File Not Found
```
ERROR: Could not read ROADMAP.md

Expected location: ROADMAP.md (project root)

Verify the file exists and you're in the correct directory.
Current directory: [pwd output]

Exiting roadmap sync.
```

### Parse Error
```
WARNING: Could not parse milestone data from ROADMAP.md

Expected format:
  ### 🎯 v1.1.0 - Core Improvements
  **Due Date**: February 15, 2026 (XX days)
  **Status**: X% Complete (Y/Z items)

The file may have been manually edited in an unexpected format.

Proceeding with partial data...
```

---

## Implementation Notes

**Design Decisions**:
- GitHub API is always the source of truth
- Files are updated to match GitHub, never vice versa
- Drift detection runs before any changes
- User confirmation required before applying updates

**Date Calculations**:
- Use `date` command for portable date arithmetic
- Handle edge cases (leap years, timezone differences)
- Display dates in ISO 8601 format for clarity

**Progress Calculation**:
```
Progress = floor((closed / total) * 100)
```
Always round down to avoid overstating completion.

**Extensibility**:
- Can easily add new sources (e.g., specs/ directory tracking)
- Can add new data points (e.g., issue labels, assignees)
- Module is self-contained and can evolve independently

---

## Example Execution

```
User: /caro.sync roadmap
Claude:
  [Fetches GitHub data]
  [Reads ROADMAP.md]
  [Reads website/roadmap.astro]
  [Detects drift]

  Roadmap Drift Detected:
    ROADMAP.md: Out of sync (3 milestones need updates)
    website/roadmap: In sync ✓

  Apply updates? (yes/no)

User: yes
Claude:
  [Updates ROADMAP.md]
  [Updates website/roadmap.astro]

  ✓ Roadmap synced with GitHub!

  Git diff:
    ROADMAP.md: 12 lines changed
    website/roadmap.astro: 8 lines changed

  Ready to commit.
```
