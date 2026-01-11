# Caro Development Workflow Improvement Plan

> **Status**: Draft
> **Date**: 2026-01-11
> **Purpose**: Map current development workflows and plan improvements for autonomous, high-quality development

---

## Table of Contents

1. [Current State Overview](#current-state-overview)
2. [Gap Analysis](#gap-analysis)
3. [Target State Vision](#target-state-vision)
4. [Improvement Phases](#improvement-phases)
5. [Technical Implementation Details](#technical-implementation-details)

---

## Current State Overview

### 1. Development Entry Points

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        DEVELOPMENT ENTRY POINTS                              │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌──────────────┐   ┌──────────────┐   ┌──────────────┐   ┌──────────────┐ │
│  │ Claude Code  │   │ Claude Code  │   │   Terminal   │   │   GitHub     │ │
│  │    Mobile    │   │     Web      │   │   Sessions   │   │   Actions    │ │
│  └──────┬───────┘   └──────┬───────┘   └──────┬───────┘   └──────┬───────┘ │
│         │                  │                  │                  │          │
│         ▼                  ▼                  ▼                  ▼          │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │                     Creates Branches + PRs                            │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 2. Current Session Types (Disconnected)

| Session Type | Purpose | Skills Used | Trigger |
|--------------|---------|-------------|---------|
| **Feature Development** | New features via spec-kitty | `/caro.feature`, `/spec-kitty.*` | Manual |
| **Documentation** | Website/docs content | `/caro.sync` | Manual |
| **QA/Testing** | Unbiased beta testing | `/unbiased-beta-tester`, `/qa-bundle-validation` | Manual |
| **Management** | PR review, roadmap | `/caro.roadmap`, PR agents | Manual |
| **Release** | Version bump, publish | `/caro.release.*` | Manual |
| **Bug Fixing** | Fix from beta feedback | `/beta-feedback-fixer`, `/caro.qa` | Manual |

### 3. Existing Automation Infrastructure

#### 3.1 Skills Inventory (16+ skills)

**Session/Continuity:**
- `create_handoff` - Cross-session context transfer
- `resume_handoff` - Resume from handoff with analysis
- `continuity_ledger` - State preservation across `/clear`
- `onboard` - Brownfield codebase analysis

**Feature Development (Spec-Kitty):**
- `caro.feature` - Orchestrator for spec-kitty workflow
- `spec-kitty.specify` → `clarify` → `plan` → `tasks` → `implement` → `review` → `accept` → `merge`
- `spec-kitty.research`, `analyze`, `checklist`, `dashboard`

**Release Management:**
- `caro.release.prepare` → `version` → `security` → `publish` → `verify`
- `caro.release.hotfix` - Emergency patches

**Project Management:**
- `caro.roadmap` - Work selection, milestone tracking
- `caro.sync` - Content synchronization (roadmap, installation, docs)
- `caro.qa` - Bug investigation and documentation

**Testing/QA:**
- `beta-test-cycles` - Iterative pattern testing
- `beta-feedback-fixer` - Fix issues from beta feedback
- `quality-engineer-manager` - Release validation orchestration
- `qa-bundle-validation` - Multi-profile testing with sign-off
- `unbiased-beta-tester` - Simulate unbiased testers

**Safety/Validation:**
- `validate-constitution` - Enforce consolidated knowledge rules
- `safety-pattern-auditor` - Audit safety patterns
- `safety-pattern-developer` - TDD for new safety patterns

#### 3.2 Hooks System

```
SessionStart     → Load continuity ledger
PreCompact       → Preserve state before compaction
SubagentStop     → Track subagent outcomes
SessionEnd       → Cleanup
PostToolUse      → Index artifacts (handoff-index.sh)
Post-git-push    → validate-constitution.sh
```

#### 3.3 GitHub Actions Workflows

| Workflow | Trigger | Purpose |
|----------|---------|---------|
| `ci.yml` | Push/PR | Tiered testing (lint → smoke → full) |
| `nightly` | Cron 2am | Comprehensive tests |
| `safety-validation.yml` | Push/PR | Safety pattern validation |
| `website-claims.yml` | Push/PR | Verify marketing claims |
| `release.yml` | After publish | GitHub release creation |
| `deploy-website.yml` | Push to website/** | Deploy to GitHub Pages |
| `stale.yml` | Scheduled | Mark stale issues |
| `issue-triage.yml` | Issue created | Auto-triage |

#### 3.4 Sync Infrastructure

| Module | Source of Truth | Targets | Status |
|--------|-----------------|---------|--------|
| Roadmap | GitHub API | ROADMAP.md, website/roadmap.astro | ✅ Functional |
| Installation | website/Download.astro | README, package docs, skills | ✅ Functional |
| Documentation | /docs/*.md | docs-site/external/ | ✅ Functional (script) |
| Docs-sync | TBD | TBD | 📋 Placeholder |
| Instructions-sync | TBD | TBD | 📋 Placeholder |

### 4. Current Workflow Diagram

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        CURRENT WORKFLOW (DISCONNECTED)                       │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   IDEATION                    DEVELOPMENT                  QA                │
│   ─────────                   ───────────                 ────               │
│                                                                              │
│   ┌─────────┐                 ┌─────────────┐           ┌─────────────┐     │
│   │ Mobile  │ ───(manual)───▶ │ Claude Code │           │ Unbiased    │     │
│   │ Ideas   │                 │ Web Session │           │ Tester      │     │
│   └─────────┘                 └──────┬──────┘           └──────┬──────┘     │
│                                      │                         │ (manual)   │
│   ┌─────────┐                        ▼                         ▼            │
│   │ ChatGPT │ ───(manual)───▶ ┌─────────────┐           ┌─────────────┐     │
│   │Research │                 │  Branch +   │◀──────────│  Feedback   │     │
│   └─────────┘                 │  PR Created │  (copy/   │  Report     │     │
│                               └──────┬──────┘  paste)   └─────────────┘     │
│                                      │                                       │
│   MANAGEMENT                         ▼                                       │
│   ──────────                  ┌─────────────┐                               │
│                               │   GitHub    │                               │
│   ┌─────────┐                 │   PR/Issue  │◀────┐                         │
│   │Roadmap  │ ───(manual)───▶ └──────┬──────┘     │                         │
│   │ Driven  │                        │            │                         │
│   └─────────┘                        ▼            │                         │
│                               ┌─────────────┐     │                         │
│   ┌─────────┐                 │  CI/CD      │     │                         │
│   │PR Review│ ───(manual)───▶ │  Workflows  │     │                         │
│   │ Agent   │                 └──────┬──────┘     │                         │
│   └─────────┘                        │            │                         │
│        │                             ▼            │                         │
│        │                      ┌─────────────┐     │                         │
│        └─────────────────────▶│  Merge to   │─────┘                         │
│                               │    Main     │                               │
│                               └─────────────┘                               │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Gap Analysis

### 1. Disconnected Flows

| Gap | Current State | Impact |
|-----|---------------|--------|
| QA → Dev feedback | Manual copy/paste | Context loss, delays |
| Ideation → Development | Manual handoff | Ideas get lost |
| PR Review → Action | Manual intervention | Stale PRs accumulate |
| Web sessions → PRs | Often forgotten | Incomplete work |
| Beta feedback → Issues | Manual creation | Slow response |

### 2. Missing Automation

| Missing | Description | Priority |
|---------|-------------|----------|
| **Visual Regression Testing** | Screenshot comparison for website | High |
| **Cadence Triggers** | Scheduled automation for local/remote | High |
| **Chrome Extension Integration** | Browser automation with Claude Chrome | Medium |
| **Cross-Session Orchestration** | Automated handoffs between sessions | Medium |
| **Stale PR/Issue Management** | Automated revival and attention | Medium |

### 3. Infrastructure Gaps

| Component | Status | Gap |
|-----------|--------|-----|
| Playwright | Configured | No visual tests implemented |
| E2E Tests | Directory ready | Empty, no test cases |
| Chrome Extension | Not present | No browser automation |
| Visual Comparison | Not present | Need screenshot diffing |
| Remote Execution | Not present | Only local terminals |

---

## Target State Vision

### 1. Integrated Workflow Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                      TARGET STATE: INTEGRATED FLOWS                          │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│                        ┌─────────────────────┐                              │
│                        │   ORCHESTRATOR      │                              │
│                        │   (Cadence-Based)   │                              │
│                        └─────────┬───────────┘                              │
│                                  │                                          │
│           ┌──────────────────────┼──────────────────────┐                   │
│           ▼                      ▼                      ▼                   │
│   ┌───────────────┐     ┌───────────────┐     ┌───────────────┐            │
│   │   QA LOOP     │     │   DEV LOOP    │     │  MGMT LOOP    │            │
│   │               │     │               │     │               │            │
│   │ ┌───────────┐ │     │ ┌───────────┐ │     │ ┌───────────┐ │            │
│   │ │ Unbiased  │ │     │ │ Feature   │ │     │ │ PR Review │ │            │
│   │ │ Testers   │ │     │ │ Dev Agent │ │     │ │ Agent     │ │            │
│   │ └─────┬─────┘ │     │ └─────┬─────┘ │     │ └─────┬─────┘ │            │
│   │       │       │     │       │       │     │       │       │            │
│   │       ▼       │     │       │       │     │       ▼       │            │
│   │ ┌───────────┐ │     │       │       │     │ ┌───────────┐ │            │
│   │ │ Visual QA │ │     │       │       │     │ │ Roadmap   │ │            │
│   │ │ (Chrome)  │ │     │       │       │     │ │ Sync      │ │            │
│   │ └─────┬─────┘ │     │       │       │     │ └─────┬─────┘ │            │
│   │       │       │     │       │       │     │       │       │            │
│   │       ▼       │     │       ▼       │     │       ▼       │            │
│   │ ┌───────────┐ │     │ ┌───────────┐ │     │ ┌───────────┐ │            │
│   │ │ Issue     │─┼─────┼▶│ Bug Fix   │ │     │ │ Stale     │ │            │
│   │ │ Creator   │ │     │ │ Queue     │ │     │ │ Revival   │ │            │
│   │ └───────────┘ │     │ └───────────┘ │     │ └───────────┘ │            │
│   └───────────────┘     └───────────────┘     └───────────────┘            │
│           │                      │                      │                   │
│           └──────────────────────┴──────────────────────┘                   │
│                                  │                                          │
│                                  ▼                                          │
│                        ┌─────────────────────┐                              │
│                        │   SHARED STATE      │                              │
│                        │ - Handoffs          │                              │
│                        │ - Ledgers           │                              │
│                        │ - GitHub Issues     │                              │
│                        │ - Roadmap           │                              │
│                        └─────────────────────┘                              │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 2. Key Integrated Loops

#### QA Loop (Automated Cadence: Daily)
```
Trigger (daily/on-demand)
    │
    ▼
/qa-bundle-validation
    │
    ├──▶ Spawn unbiased-beta-tester profiles
    │
    ├──▶ Visual regression testing (Chrome)
    │       │
    │       ├── Spin up local dev server
    │       ├── Navigate with Chrome extension
    │       ├── Capture screenshots
    │       └── Compare with baseline
    │
    └──▶ Consolidate feedback
            │
            ▼
        Auto-create GitHub issues
            │
            ▼
        Notify dev loop (queue work)
```

#### Dev Loop (Event-Driven + Cadence)
```
Trigger (issue created, PR feedback, manual)
    │
    ▼
/caro.roadmap next (select work)
    │
    ▼
/caro.feature (spec-kitty workflow)
    │
    ├── /spec-kitty.specify → clarify → plan → tasks
    │
    ├── /spec-kitty.implement → review → accept
    │
    └── /spec-kitty.merge → PR created
            │
            ▼
        CI/CD runs
            │
            ▼
        External agents review (Kubic, Copilot)
            │
            ▼
        /create_handoff (preserve context)
```

#### Management Loop (Cadence: Every 4 hours)
```
Trigger (scheduled)
    │
    ▼
Scan open PRs
    │
    ├── Check for stale PRs (> 3 days no activity)
    │       │
    │       └── Rebase, request reviews, or close
    │
    ├── Check external agent feedback
    │       │
    │       └── Respond to Kubic/Copilot comments
    │
    ├── Sync roadmap
    │       │
    │       └── /caro.sync roadmap
    │
    └── Check Claude Code Web sessions
            │
            └── Create PRs for abandoned branches
```

### 3. Visual Regression Testing Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    VISUAL REGRESSION TESTING FLOW                            │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   ┌────────────────┐                                                        │
│   │ Test Trigger   │  (PR, nightly, manual)                                 │
│   └───────┬────────┘                                                        │
│           │                                                                  │
│           ▼                                                                  │
│   ┌────────────────┐                                                        │
│   │ Start Dev      │  npm run dev (website)                                 │
│   │ Server         │                                                        │
│   └───────┬────────┘                                                        │
│           │                                                                  │
│           ▼                                                                  │
│   ┌────────────────┐     ┌────────────────┐                                 │
│   │ Playwright     │────▶│ Navigate Pages │                                 │
│   │ + Chrome       │     └───────┬────────┘                                 │
│   └────────────────┘             │                                          │
│                                  ▼                                          │
│                         ┌────────────────┐                                  │
│                         │ Capture        │                                  │
│                         │ Screenshots    │                                  │
│                         └───────┬────────┘                                  │
│                                 │                                           │
│              ┌──────────────────┼──────────────────┐                        │
│              ▼                  ▼                  ▼                        │
│   ┌────────────────┐ ┌────────────────┐ ┌────────────────┐                  │
│   │  Homepage      │ │  Roadmap       │ │  Docs          │                  │
│   │  (light/dark)  │ │  (light/dark)  │ │  (light/dark)  │                  │
│   └───────┬────────┘ └───────┬────────┘ └───────┬────────┘                  │
│           │                  │                  │                           │
│           └──────────────────┼──────────────────┘                           │
│                              ▼                                              │
│                   ┌────────────────────┐                                    │
│                   │ Compare with       │                                    │
│                   │ Baseline Images    │                                    │
│                   └─────────┬──────────┘                                    │
│                             │                                               │
│              ┌──────────────┴──────────────┐                                │
│              ▼                             ▼                                │
│   ┌────────────────┐            ┌────────────────┐                          │
│   │ No Diff        │            │ Diff Found     │                          │
│   │ ✓ Pass         │            │ ⚠ Generate     │                          │
│   └────────────────┘            │   Report       │                          │
│                                 └───────┬────────┘                          │
│                                         │                                   │
│                                         ▼                                   │
│                              ┌────────────────────┐                         │
│                              │ - HTML Report      │                         │
│                              │ - Side-by-side     │                         │
│                              │ - Diff overlay     │                         │
│                              │ - Approve/Reject   │                         │
│                              └────────────────────┘                         │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Improvement Phases

### Phase 1: Visual Regression Testing (Priority: High)

**Objective**: Catch website regressions before they reach production

**Components**:
1. Playwright visual tests for key pages
2. Screenshot comparison with `pixelmatch` or similar
3. Baseline image storage in git LFS
4. CI integration (fail PR if visual diff > threshold)
5. Manual review workflow for intentional changes

**Implementation**:
```bash
website/
├── playwright.config.ts          # Visual test config
├── tests/
│   └── visual/
│       ├── homepage.spec.ts      # Homepage visual tests
│       ├── roadmap.spec.ts       # Roadmap visual tests
│       └── docs.spec.ts          # Docs visual tests
├── screenshots/
│   └── baseline/                 # Git LFS baseline images
└── visual-report/                # Generated diff reports
```

**Skill**: `/visual-regression-test`
- Capture current screenshots
- Compare with baseline
- Generate diff report
- Update baseline (with approval)

---

### Phase 2: Cadence-Based Automation (Priority: High)

**Objective**: Run automated loops on schedule (local or remote)

**Components**:
1. Cron-style scheduler (local machine)
2. GitHub Actions scheduled workflows (remote)
3. Orchestrator skill that triggers appropriate flows
4. State tracking across runs

**Local Automation Script**:
```bash
#!/bin/bash
# caro-automation.sh - Run on cron or launchd

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
CARO_DIR="$(dirname "$SCRIPT_DIR")"

# Daily QA Loop (run at 9 AM)
if [[ "$(date +%H)" == "09" ]]; then
    cd "$CARO_DIR" && claude -p "Run /qa-bundle-validation with all profiles"
fi

# Management Loop (every 4 hours)
if (( $(date +%H) % 4 == 0 )); then
    cd "$CARO_DIR" && claude -p "Run /caro.roadmap status and check stale PRs"
fi

# Nightly visual regression (run at 2 AM)
if [[ "$(date +%H)" == "02" ]]; then
    cd "$CARO_DIR/website" && npm run test:visual
fi
```

**Remote Automation (GitHub Actions)**:
```yaml
# .github/workflows/cadence.yml
name: Cadence Automation

on:
  schedule:
    - cron: '0 9 * * *'   # Daily QA at 9 AM
    - cron: '0 */4 * * *' # Management every 4 hours
    - cron: '0 2 * * *'   # Nightly visual tests

jobs:
  qa-loop:
    if: github.event.schedule == '0 9 * * *'
    # ...

  management-loop:
    if: github.event.schedule == '0 */4 * * *'
    # ...

  visual-regression:
    if: github.event.schedule == '0 2 * * *'
    # ...
```

---

### Phase 3: Chrome Extension Integration (Priority: Medium)

**Objective**: Automate browser interactions for QA and testing

**Options**:
1. **Claude Chrome Extension** - Use for interactive QA sessions
2. **Playwright** - Programmatic browser automation
3. **Combined** - Playwright for automation, Chrome extension for exploratory

**Implementation Strategy**:
- Start with Playwright (already configured)
- Add Chrome extension integration for human-in-the-loop QA
- Create skill for browser-based exploratory testing

**Skill**: `/visual-qa-session`
```markdown
## Visual QA Session

1. Start dev server (website, docs-site)
2. Open Chrome with Claude extension
3. Navigate through key user flows
4. Capture observations and screenshots
5. Generate QA report with findings
```

---

### Phase 4: Flow Integration (Priority: Medium)

**Objective**: Connect QA → Dev → Management loops

**Components**:
1. **Feedback Pipeline**: QA findings → GitHub Issues → Dev queue
2. **Handoff Automation**: Auto-create handoffs at session boundaries
3. **PR Lifecycle Management**: Track PRs through review → merge → release
4. **Stale Work Revival**: Identify and resurface neglected work

**Implementation**:

**QA → Issue Pipeline**:
```
/qa-bundle-validation output
    │
    ▼
Parse findings (bugs, regressions, UX issues)
    │
    ▼
For each finding:
    ├── Check if similar issue exists
    ├── If not, create GitHub issue with:
    │   - Labels: qa, regression, etc.
    │   - Priority based on severity
    │   - Link to test output
    │   - Reproduction steps
    └── Add to /caro.roadmap backlog
```

**PR Lifecycle Tracker**:
```
Every 4 hours:
    │
    ▼
Fetch open PRs
    │
    For each PR:
    ├── Check last activity
    ├── Check CI status
    ├── Check review status
    │
    ├── If stale (> 3 days, no activity):
    │   ├── Rebase with main
    │   ├── Request reviews
    │   └── Comment on PR
    │
    ├── If CI failing:
    │   └── Analyze and create fix PR or notify
    │
    └── If approved, not merged:
        └── Auto-merge if conditions met
```

---

### Phase 5: Documentation Flow Improvement (Priority: Medium)

**Objective**: Unified documentation sync with single source of truth

**Current Challenges**:
1. Markdown in repo vs docs site vs README
2. Manual sync between locations
3. Drift detection is reactive

**Solution**:
```
┌─────────────────────────────────────────────────────────────────────────────┐
│                      DOCUMENTATION SYNC ARCHITECTURE                         │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   SOURCE FILES                    DERIVED                                    │
│   ────────────                    ───────                                    │
│                                                                              │
│   ┌─────────────┐                                                           │
│   │ /docs/*.md  │◀────────────── Ground Truth                               │
│   └──────┬──────┘                                                           │
│          │                                                                   │
│          ├──────────────────────────────────────────┐                        │
│          │                                          │                        │
│          ▼                                          ▼                        │
│   ┌─────────────────┐                      ┌─────────────────┐              │
│   │ docs-site/      │                      │ README.md       │              │
│   │ external/*.md   │                      │ (excerpts)      │              │
│   └─────────────────┘                      └─────────────────┘              │
│                                                                              │
│   ┌─────────────┐                                                           │
│   │ /website    │◀────────────── Ground Truth (website content)             │
│   │ /src/**     │                                                           │
│   └──────┬──────┘                                                           │
│          │                                                                   │
│          ▼                                                                   │
│   ┌─────────────────┐                                                       │
│   │ caro.sh         │                                                       │
│   │ (deployed)      │                                                       │
│   └─────────────────┘                                                       │
│                                                                              │
│   SYNC COMMANDS                                                              │
│   ─────────────                                                              │
│                                                                              │
│   /caro.sync docs        - Sync /docs → docs-site                           │
│   /caro.sync installation - Sync installation everywhere                    │
│   /caro.sync roadmap     - Sync GitHub → ROADMAP.md → website               │
│   /caro.sync all         - Run all sync modules                             │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Technical Implementation Details

### 1. Visual Regression Testing Setup

**Dependencies**:
```json
{
  "devDependencies": {
    "@playwright/test": "^1.45.0",
    "pixelmatch": "^5.3.0",
    "pngjs": "^7.0.0"
  }
}
```

**Playwright Config**:
```typescript
// website/playwright.config.ts
import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: './tests/visual',
  snapshotDir: './screenshots/baseline',
  updateSnapshots: process.env.UPDATE_SNAPSHOTS ? 'all' : 'none',
  expect: {
    toHaveScreenshot: {
      maxDiffPixels: 100,
      threshold: 0.1,
    },
  },
  projects: [
    {
      name: 'chromium-light',
      use: { colorScheme: 'light' },
    },
    {
      name: 'chromium-dark',
      use: { colorScheme: 'dark' },
    },
  ],
  webServer: {
    command: 'npm run dev',
    port: 4321,
    reuseExistingServer: !process.env.CI,
  },
});
```

### 2. Cadence Automation Skill

**New Skill**: `/automation-orchestrator`
```markdown
## Automation Orchestrator

Runs scheduled automation loops based on time of day and configuration.

### Loops:
1. **QA Loop** (daily 9 AM) - Run /qa-bundle-validation
2. **Management Loop** (every 4 hours) - Check PRs, sync roadmap
3. **Visual Regression** (nightly 2 AM) - Run visual tests
4. **Sync Loop** (daily 6 PM) - Run /caro.sync all

### Usage:
- `/automation-orchestrator run qa` - Run QA loop now
- `/automation-orchestrator run mgmt` - Run management loop now
- `/automation-orchestrator status` - Show last run times
- `/automation-orchestrator schedule` - Show schedule
```

### 3. Issue Creator Integration

**QA Finding → Issue Template**:
```yaml
# .github/ISSUE_TEMPLATE/qa-finding.yml
name: QA Finding
description: Automatically created from QA testing
labels: ["qa", "triage"]
body:
  - type: markdown
    attributes:
      value: |
        This issue was automatically created from QA testing.
  - type: input
    id: severity
    attributes:
      label: Severity
      options: ["critical", "high", "medium", "low"]
  - type: textarea
    id: reproduction
    attributes:
      label: Reproduction Steps
  - type: textarea
    id: expected
    attributes:
      label: Expected Behavior
  - type: textarea
    id: actual
    attributes:
      label: Actual Behavior
  - type: input
    id: qa-source
    attributes:
      label: QA Source
      description: Link to QA report or test run
```

---

## Summary: Improvement Roadmap

| Phase | Objective | Key Deliverables | Dependencies |
|-------|-----------|------------------|--------------|
| 1 | Visual Regression | Playwright tests, screenshot comparison, CI integration | None |
| 2 | Cadence Automation | Scheduler, orchestrator skill, GitHub Actions | Phase 1 |
| 3 | Chrome Integration | Browser QA skill, exploratory testing | Phase 1 |
| 4 | Flow Integration | QA→Issue pipeline, PR lifecycle, stale revival | Phase 2 |
| 5 | Docs Flow | Complete sync modules, drift prevention | Phase 4 |

---

## Next Steps

1. **Immediate**: Implement Phase 1 visual regression testing
2. **Short-term**: Create automation orchestrator skill (Phase 2)
3. **Medium-term**: Integrate flows and create issue pipeline (Phase 4)
4. **Ongoing**: Refine and expand based on usage patterns

---

*Document created as part of workflow improvement initiative*
