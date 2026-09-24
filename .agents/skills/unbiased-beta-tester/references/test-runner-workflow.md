# Test Runner Workflow Reference

This document describes the state machine that drives the beta testing workflow.

## State Machine Overview

```
                                    +-------------+
                                    |    INTRO    |
                                    +------+------+
                                           |
                                           v
                                    +-------------+
                             +----->|  DISCOVER   |
                             |      +------+------+
                             |             |
                    doc issue|             | found docs
                             |             v
                    +--------+      +-------------+
                    | REPORT |<-----|   INSTALL   |
                    +--------+      +------+------+
                         ^                 |
                         |                 | install ok
            install fail |                 v
                         |          +-------------+
                         +----------|  SMOKE_TEST |
                         |          +------+------+
                         |                 |
            smoke fail   |                 | smoke ok
                         |                 v
                         |          +-------------+
                         +----------|  WORKFLOWS  |
                         |          +------+------+
                         |                 |
            workflow fail|                 | all workflows ok
                         |                 v
                         |          +-------------+
                         +--------->|   REPORT    |
                                    +------+------+
                                           |
                                           v
                                    +-------------+
                                    |     END     |
                                    +-------------+
```

## State Definitions

### INTRO State

**Purpose**: Establish the tester identity and context.

**Actions**:
1. State the tester's name and role from profile
2. Articulate what they know and don't know
3. State their goal for this testing session
4. Set expectations based on profile

**Example Output**:
```markdown
# Beta Test Session: Terminal Novice

## Who I Am
I'm Alex, a marketing analyst who mostly works with spreadsheets
and presentation software. I occasionally need to run terminal
commands from tutorials but I find the command line intimidating.

## What I Know
- I heard about a tool called "caro" that converts English to
  shell commands
- My colleague said it's useful for people who don't know terminal syntax
- I'm on a MacBook Pro but I've never installed command-line tools before

## What I Don't Know
- How to check if I have the required software installed
- What "cargo" or "Rust" means
- Where to find installation instructions

## My Goal Today
I want to install caro and use it to search for large files on my computer.
If I can do that, I'll consider this a success.

## Proceeding to DISCOVER state...
```

**Transition**: Always proceeds to DISCOVER.

---

### DISCOVER State

**Purpose**: Find and read product documentation from the tester's perspective.

**Actions**:
1. Search for the product (as a real user would)
2. Visit the official website/docs
3. Summarize what you learned in tester's voice
4. Note any confusion or unclear points

**Knowledge Sources** (in order of priority):
1. Official website (caro.sh)
2. GitHub README
3. Documentation pages
4. Getting started / quickstart guides

**Example Output**:
```markdown
## Discovery Phase

### Finding the Product
Let me search for "caro command line tool"...

[WebSearch: "caro shell command generator"]

Found the website at caro.sh. Let me read what this tool does...

[WebFetch: https://caro.sh]

### What I Learned
- caro converts natural language to shell commands
- It uses local AI (runs on my computer, not cloud)
- It has "safety features" to prevent dangerous commands
- There are three installation options mentioned

### Installation Options Found
1. "Quick Install Script" - curl | bash (one command)
2. "Pre-built Binaries" - download directly
3. "Using Cargo" - something about Rust?

### Confusion Points
- What's "cargo"? The site assumes I know what this is
- "Apple Silicon" vs "Intel" - not sure which I have
- What does "MLX optimization" mean?

### Next Step
I'll try the "Quick Install Script" since it says "Recommended"
and appears to be the simplest option.

## Proceeding to INSTALL state...
```

**Transitions**:
- If docs are clear enough to attempt installation → INSTALL
- If docs are too confusing to proceed → REPORT (doc issue)

---

### INSTALL State

**Purpose**: Attempt to install the product following documentation.

**Actions**:
1. Run the documented installation command(s)
2. Capture all output
3. Wait for completion
4. Check for success indicators

**Rules**:
- Run ONLY what the documentation says
- Do NOT improvise or use knowledge outside the docs
- If something fails, try documented troubleshooting first
- After 2-3 attempts, stop and report

**Example Output**:
```markdown
## Installation Attempt

### Following: Quick Install Script

The README says to run:
```bash
curl -fsSL https://raw.githubusercontent.com/wildcard/caro/main/install.sh | bash
```

Let me try this...

$ curl -fsSL https://raw.githubusercontent.com/wildcard/caro/main/install.sh | bash
  % Total    % Received % Xferd  Average Speed   Time    Time     Time  Current
                                 Dload  Upload   Total   Spent    Left  Speed
100  4523  100  4523    0     0  12345      0 --:--:-- --:--:-- --:--:-- 12345

Detected: macOS (Apple Silicon)
Downloading caro vX.Y.Z for macos-silicon...
Installing to /usr/local/bin...
Installation complete!

Run 'caro --version' to verify.

### Verification

$ caro --version
caro vX.Y.Z  # example version

### Result
Installation succeeded! The command is now available.

## Proceeding to SMOKE_TEST state...
```

**Transitions**:
- If installation succeeds → SMOKE_TEST
- If installation fails → REPORT (installation failure)

---

### SMOKE_TEST State

**Purpose**: Verify basic functionality works.

**Actions**:
1. Run `--version` to confirm binary works
2. Run `--help` to see available options
3. Try the simplest documented command
4. Check output matches expectations

**Standard Smoke Tests**:
```bash
# Version check
caro --version

# Help output
caro --help

# Simplest example from docs
caro "list files"
```

**Example Output**:
```markdown
## Smoke Test

### Test 1: Version
$ caro --version
caro vX.Y.Z  # example version; actual output may differ

Result: PASS - Version displayed correctly

### Test 2: Help
$ caro --help
caro - Convert natural language to safe shell commands

USAGE:
    caro [OPTIONS] <PROMPT>

OPTIONS:
    -h, --help       Print help information
    -v, --verbose    Enable verbose output
    ...

Result: PASS - Help is readable and explains basic usage

### Test 3: First Command
Following the README example...

$ caro "list files"

Generated command:
  ls -la

Safety Assessment: Safe (Green)
Execute this command? (y/N) n

Result: PASS - Generated a sensible command

## All smoke tests passed. Proceeding to WORKFLOWS state...
```

**Transitions**:
- If all smoke tests pass → WORKFLOWS
- If any smoke test fails → REPORT (first-run failure)

---

### WORKFLOWS State

**Purpose**: Execute profile-specific use cases.

**Actions**:
1. For each workflow in the profile:
   - State the goal
   - Attempt the workflow
   - Check success criteria
   - Note friction points
2. Continue until all workflows complete or patience exhausted

**Example Output**:
```markdown
## Workflow Tests

### Workflow 1: Find Large Files
**Goal**: Use caro to find files larger than 100MB

**Attempt 1**:
$ caro "find files larger than 100 megabytes"

Generated command:
  find . -type f -size +100M -ls

Safety Assessment: Safe (Green)
Execute this command? (y/N) y

[Command output showing files found]

**Result**: SUCCESS
- Command was generated correctly
- Worked on first try
- Output was understandable

### Workflow 2: Safe File Deletion
**Goal**: Use caro to delete old log files safely

**Attempt 1**:
$ caro "delete log files older than 30 days"

Generated command:
  find . -name "*.log" -type f -mtime +30 -delete

Safety Assessment: High (Orange)
This command will permanently delete files.
Execute this command? (y/N) n

**Observation**: The safety warning is appropriate.
Let me try a preview first...

$ caro "preview which log files are older than 30 days"

Generated command:
  find . -name "*.log" -type f -mtime +30 -ls

Safety Assessment: Safe (Green)

**Result**: SUCCESS
- Correctly identified the safety risk
- Allowed me to preview first
- Good user experience

### Workflow 3: Complex Pipe Command
**Goal**: Find and count unique file extensions

**Attempt 1**:
$ caro "list all unique file extensions in this directory"

Generated command:
  find . -type f | sed 's/.*\.//' | sort -u

Safety Assessment: Safe (Green)

**Result**: SUCCESS
- Complex command generated correctly
- Worked as expected

## All workflows completed successfully.
## Proceeding to REPORT state (positive feedback)...
```

**Transitions**:
- If all workflows succeed → REPORT (positive feedback)
- If workflow fails → REPORT (workflow failure)
- If patience exhausted → REPORT (timeout/frustration)

---

### REPORT State

**Purpose**: Compile findings into GitHub issue(s).

**Report Types**:

| Trigger | Report Type | Labels |
|---------|-------------|--------|
| Doc confusion | Documentation Bug | `docs`, `ux` |
| Install failure | Bug Report | `bug`, `installation` |
| Smoke test failure | Bug Report | `bug`, `critical` |
| Workflow failure | Bug Report | `bug`, `workflow` |
| Friction point | Enhancement Request | `enhancement`, `ux` |
| All success | Positive Feedback | `feedback`, `positive` |

**Actions**:
1. Determine report type based on failure point
2. Gather all evidence collected
3. Format according to project template
4. Include reproduction steps
5. Suggest severity/priority

See `references/github-issue-format.md` for template details.

**Transitions**: Always proceeds to END.

---

### END State

**Purpose**: Conclude the testing session.

**Actions**:
1. Summarize session outcomes
2. List all issues filed
3. Provide tester's overall impression
4. Suggest priority order for fixes

**Example Output**:
```markdown
## Session Complete

### Summary
- Tested as: Terminal Novice (Alex)
- Duration: 15 minutes
- Workflows attempted: 3
- Workflows succeeded: 3

### Issues Filed
1. [Enhancement] Documentation should explain what "cargo" is
2. [Positive] Installation script worked flawlessly

### Tester's Impression
"I was pleasantly surprised! I expected this to be complicated
but the install script just worked. The safety warnings are
helpful and the generated commands were correct."

### Recommendations
Priority 1: Improve docs for non-technical users
Priority 2: Add more examples in quickstart

### Session End
```

## Stop Conditions

The test MUST stop and file a report when:

| Condition | Action |
|-----------|--------|
| Documentation too confusing to proceed | Report doc issue |
| Installation command fails | Report installation bug |
| `--version` doesn't work | Report critical bug |
| `--help` output is missing/broken | Report critical bug |
| First example command fails | Report bug |
| Any workflow fails after 2 attempts | Report bug |
| Tester patience exhausted | Report with partial results |
| Security concern discovered | Report immediately |

## Evidence Collection During States

Each state should collect:

| State | Evidence Collected |
|-------|-------------------|
| INTRO | Profile summary, goals, knowledge gaps |
| DISCOVER | URLs visited, documentation sections read |
| INSTALL | Commands run, full output, error messages |
| SMOKE_TEST | Commands run, expected vs actual output |
| WORKFLOWS | Commands run, success/failure for each step |
| REPORT | All above, formatted for template |

## Patience Tracking

Track frustration throughout (illustrative pseudocode, not executable):

```
// Pseudocode - conceptual logic for patience tracking
frustration = 0
patience = profile.patience.max_failed_attempts_per_step

On each failure:
    frustration = frustration + 1
    if frustration >= patience:
        transition_to("REPORT", "patience_exhausted")

On success:
    frustration = max(0, frustration - 1)
```

## Human-Like Behavior Patterns

### Good Beta Tester Behavior

```
Attempt 1: Follow docs exactly
  -> Failed

Thought: "Let me re-read the instructions..."

Attempt 2: Try docs again, check for missed steps
  -> Failed

Thought: "Maybe I misunderstood something. Let me look for FAQ..."

Attempt 3: Check troubleshooting section
  -> Failed

Decision: "I've tried what the docs suggest. Time to report this."
```

### Bad Beta Tester Behavior (Avoid This)

```
Attempt 1: Follow docs
  -> Failed

Action: Dive into source code to find the issue
Action: Patch the binary to work around it
Action: File a PR with the fix
Action: Report "fixed it myself, here's how"

Problem: This doesn't help find UX issues for normal users!
```

The goal is to document friction, not solve everything.
