# GitHub Project Setup Guide

> **Organizing cmdai development using GitHub Issues, Milestones, Labels, and Projects**

This guide shows maintainers and contributors how to set up and use GitHub's project management tools to track the roadmap from MVP → V1.0 → Enterprise.

---

## Table of Contents

1. [GitHub Milestones](#github-milestones)
2. [Issue Labels](#issue-labels)
3. [GitHub Projects](#github-projects)
4. [Issue Templates](#issue-templates)
5. [Creating Issues from Roadmap](#creating-issues-from-roadmap)

---

## GitHub Milestones

Milestones represent major product releases aligned with [ROADMAP.md](ROADMAP.md).

### How to Create Milestones

1. Go to https://github.com/wildcard/cmdai/milestones
2. Click **"New milestone"**
3. Fill in details using the templates below

### Milestone Definitions

#### Milestone 1: v1.0 - Production CLI

**Title**: `v1.0 - Production CLI`
**Due Date**: December 31, 2025
**Description**:
```markdown
Production-ready CLI tool ready for Hacker News/Product Hunt launch.

## Goals
- All MVP features stable and tested
- Binary distribution (Homebrew, crates.io)
- Complete documentation
- Performance targets met (<100ms startup, <2s inference)
- Security audit complete

## Success Criteria
- [ ] All tests passing
- [ ] Binary size <50MB
- [ ] Homebrew formula published
- [ ] Documentation complete
- [ ] Ready for public launch

See [MVP_TO_V1.md](MVP_TO_V1.md) for detailed checklist.
```

---

#### Milestone 2: Q1 2025 - Cloud Launch

**Title**: `Q1 2025 - Cloud Launch`
**Due Date**: March 31, 2025
**Description**:
```markdown
Launch cloud backend and team collaboration features.

## Goals
- Cloud API backend (Rust + Axum)
- Authentication (JWT + API keys)
- Team collaboration (shared patterns, approval workflows)
- Analytics and learning pipeline
- 1,000 cloud signups, 100 paid users

## Revenue Target
$2K MRR by end of Q1

See [ROADMAP.md#phase-1](ROADMAP.md#phase-1-cloud-foundation-q1-2025) for details.
```

---

#### Milestone 3: Q2 2025 - Enterprise Features

**Title**: `Q2 2025 - Enterprise Features`
**Due Date**: June 30, 2025
**Description**:
```markdown
Enterprise-ready features for regulated industries.

## Goals
- Audit logs (immutable, SOC 2 ready)
- Access control (RBAC, SSO)
- Self-hosted deployment (Docker, Kubernetes)
- 5 enterprise deals closed ($25K-50K each)

## Revenue Target
$150K ARR by end of Q2

See [ROADMAP.md#phase-2](ROADMAP.md#phase-2-enterprise-features-q2-2025) for details.
```

---

#### Milestone 4: Q3 2025 - Platform

**Title**: `Q3 2025 - Platform`
**Due Date**: September 30, 2025
**Description**:
```markdown
Transform cmdai into an ops automation platform.

## Goals
- Workflow engine (DAG-based)
- Integration marketplace (50+ integrations)
- Proprietary model fine-tuning
- 500+ community workflows

## Revenue Target
$500K ARR by end of Q3

See [ROADMAP.md#phase-3](ROADMAP.md#phase-3-platform-play-q3-2025) for details.
```

---

#### Milestone 5: Q4 2025 - Scale & Fundraise

**Title**: `Q4 2025 - Scale & Fundraise`
**Due Date**: December 31, 2025
**Description**:
```markdown
Hit Series A metrics and close funding round.

## Goals
- 20,000 cloud users
- 100 enterprise customers
- $100K MRR (path to $2M ARR in 2026)
- Close Series A ($5-10M)
- Team of 8 people

See [ROADMAP.md#phase-4](ROADMAP.md#phase-4-scale--fundraise-q4-2025) for details.
```

---

## Issue Labels

Labels help categorize and prioritize issues. Here's the complete label system:

### Create These Labels

Go to https://github.com/wildcard/cmdai/labels and create:

#### Type Labels

| Label | Color | Description |
|-------|-------|-------------|
| `type: feature` | `#0075ca` | New feature or enhancement |
| `type: bug` | `#d73a4a` | Something isn't working |
| `type: docs` | `#0075ca` | Documentation improvements |
| `type: refactor` | `#fbca04` | Code refactoring, no new features |
| `type: test` | `#1d76db` | Testing improvements |
| `type: infra` | `#5319e7` | Infrastructure, CI/CD, deployment |

#### Priority Labels

| Label | Color | Description |
|-------|-------|-------------|
| `priority: P0` | `#d73a4a` | Critical, blocking |
| `priority: P1` | `#fbca04` | High priority |
| `priority: P2` | `#0e8a16` | Medium priority |
| `priority: P3` | `#c2e0c6` | Low priority |

#### Component Labels

| Label | Color | Description |
|-------|-------|-------------|
| `component: cli` | `#006b75` | CLI interface and argument parsing |
| `component: backend` | `#1d76db` | LLM backends (MLX, vLLM, Ollama) |
| `component: safety` | `#d93f0b` | Safety validation and risk assessment |
| `component: cloud` | `#0052cc` | Cloud API and SaaS features |
| `component: enterprise` | `#5319e7` | Enterprise features (RBAC, audit, SSO) |
| `component: workflows` | `#0e8a16` | Workflow engine |
| `component: integrations` | `#fbca04` | Third-party integrations |

#### Phase Labels (Aligned with Roadmap)

| Label | Color | Description |
|-------|-------|-------------|
| `phase: mvp` | `#c2e0c6` | Current MVP work |
| `phase: v1.0` | `#0075ca` | Production-ready CLI |
| `phase: q1-2025` | `#1d76db` | Q1 2025 - Cloud Launch |
| `phase: q2-2025` | `#0052cc` | Q2 2025 - Enterprise |
| `phase: q3-2025` | `#5319e7` | Q3 2025 - Platform |
| `phase: q4-2025` | `#d93f0b` | Q4 2025 - Scale |

#### Other Labels

| Label | Color | Description |
|-------|-------|-------------|
| `good-first-issue` | `#7057ff` | Good for newcomers |
| `help-wanted` | `#008672` | Extra attention needed |
| `blocked` | `#d73a4a` | Blocked by another issue |
| `needs-design` | `#fbca04` | Needs design/architecture decisions |
| `needs-review` | `#0075ca` | Needs code review |
| `breaking-change` | `#d93f0b` | Breaking API change |

---

## GitHub Projects

Use GitHub Projects (beta) for kanban-style tracking.

### Create Project

1. Go to https://github.com/wildcard/cmdai/projects
2. Click **"New project"**
3. Choose **"Board"** template
4. Name: **"cmdai Roadmap 2025"**

### Board Columns

Create these columns:

1. **Backlog** - Not yet prioritized
2. **Ready** - Prioritized, ready to start
3. **In Progress** - Currently being worked on
4. **In Review** - PR submitted, needs review
5. **Done** - Completed and merged

### Custom Fields

Add these custom fields to issues:

1. **Quarter**: Single select (Q1 2025, Q2 2025, Q3 2025, Q4 2025)
2. **Effort**: Number (story points: 1, 2, 3, 5, 8, 13)
3. **Revenue Impact**: Single select (None, Low, Medium, High, Critical)

### Views

Create multiple views for different audiences:

1. **Roadmap View** (Table)
   - Group by: Quarter
   - Sort by: Priority
   - Show: Title, Assignee, Labels, Effort

2. **Team View** (Board)
   - Group by: Status
   - Filter: Current quarter only
   - Show: Title, Assignee, Labels

3. **Enterprise View** (Table)
   - Filter: `component: enterprise` OR `component: cloud`
   - Sort by: Revenue Impact (descending)
   - Show: Title, Milestone, Revenue Impact

---

## Issue Templates

Create issue templates for consistency.

### Feature Request Template

**File**: `.github/ISSUE_TEMPLATE/feature_request.yml`

```yaml
name: Feature Request
description: Suggest a new feature for cmdai
title: "[Feature]: "
labels: ["type: feature"]
body:
  - type: markdown
    attributes:
      value: |
        Thanks for suggesting a feature! Please provide details below.

  - type: dropdown
    id: phase
    attributes:
      label: Roadmap Phase
      description: Which phase does this belong to?
      options:
        - v1.0 - Production CLI
        - Q1 2025 - Cloud Launch
        - Q2 2025 - Enterprise
        - Q3 2025 - Platform
        - Q4 2025 - Scale
        - Future (post-2025)
    validations:
      required: true

  - type: dropdown
    id: component
    attributes:
      label: Component
      options:
        - CLI
        - Backend
        - Safety
        - Cloud
        - Enterprise
        - Workflows
        - Integrations
    validations:
      required: true

  - type: textarea
    id: problem
    attributes:
      label: Problem Statement
      description: What problem does this feature solve?
      placeholder: "As a [user type], I want [feature] so that [benefit]"
    validations:
      required: true

  - type: textarea
    id: solution
    attributes:
      label: Proposed Solution
      description: How should this feature work?
    validations:
      required: true

  - type: textarea
    id: alternatives
    attributes:
      label: Alternatives Considered
      description: What other approaches did you consider?

  - type: dropdown
    id: priority
    attributes:
      label: Priority
      options:
        - P0 - Critical
        - P1 - High
        - P2 - Medium
        - P3 - Low
    validations:
      required: true
```

---

### Bug Report Template

**File**: `.github/ISSUE_TEMPLATE/bug_report.yml`

```yaml
name: Bug Report
description: Report a bug in cmdai
title: "[Bug]: "
labels: ["type: bug"]
body:
  - type: markdown
    attributes:
      value: |
        Thanks for reporting a bug! Please provide details below.

  - type: dropdown
    id: component
    attributes:
      label: Component
      options:
        - CLI
        - Backend (MLX)
        - Backend (CPU)
        - Backend (Ollama)
        - Backend (vLLM)
        - Safety Validator
        - Configuration
        - Other
    validations:
      required: true

  - type: textarea
    id: description
    attributes:
      label: Bug Description
      description: What happened?
    validations:
      required: true

  - type: textarea
    id: reproduction
    attributes:
      label: Steps to Reproduce
      description: How can we reproduce this?
      placeholder: |
        1. Run `cmdai "..."`
        2. See error
    validations:
      required: true

  - type: textarea
    id: expected
    attributes:
      label: Expected Behavior
      description: What should have happened?
    validations:
      required: true

  - type: textarea
    id: actual
    attributes:
      label: Actual Behavior
      description: What actually happened?
    validations:
      required: true

  - type: textarea
    id: environment
    attributes:
      label: Environment
      description: |
        Please provide:
        - OS and version
        - cmdai version (`cmdai --version`)
        - Rust version (`rustc --version`)
      value: |
        - OS:
        - cmdai version:
        - Rust version:
    validations:
      required: true

  - type: textarea
    id: logs
    attributes:
      label: Logs
      description: Paste relevant logs (run with `RUST_LOG=debug`)
      render: shell
```

---

## Creating Issues from Roadmap

Here are example issues for each phase:

### V1.0 Issues

#### Issue #13: Performance Optimization for <100ms Startup

**Title**: `[v1.0] Performance Optimization for <100ms Startup`
**Labels**: `type: feature`, `priority: P0`, `component: cli`, `phase: v1.0`
**Milestone**: `v1.0 - Production CLI`
**Assignee**: (TBD)

**Description**:
```markdown
## Goal
Reduce CLI startup time to <100ms (cold start) for release builds.

## Current Status
- Current startup time: ~200ms (estimated, needs benchmarking)

## Tasks
- [ ] Benchmark current startup performance
- [ ] Profile with `cargo flamegraph`
- [ ] Identify hot paths in initialization
- [ ] Lazy-load dependencies (defer until first use)
- [ ] Optimize config file parsing
- [ ] Cache compiled regexes
- [ ] Re-benchmark and verify <100ms target

## Acceptance Criteria
- [ ] Release build starts in <100ms
- [ ] Debug build starts in <200ms (acceptable for dev)
- [ ] All tests still passing
- [ ] No functionality regressions

## Related
- See [MVP_TO_V1.md](MVP_TO_V1.md#1-performance-optimization-1-2-weeks)
- Blocked by: None
- Blocks: v1.0 launch
```

---

#### Issue #14: Homebrew Formula for Easy Installation

**Title**: `[v1.0] Create Homebrew Formula for macOS Installation`
**Labels**: `type: infra`, `priority: P0`, `component: cli`, `phase: v1.0`
**Milestone**: `v1.0 - Production CLI`

**Description**:
```markdown
## Goal
Enable one-command installation on macOS via Homebrew.

## Command
```bash
brew tap wildcard/cmdai
brew install cmdai
```

## Tasks
- [ ] Create Homebrew tap repository (`homebrew-cmdai`)
- [ ] Write Formula (`cmdai.rb`)
- [ ] Test installation on fresh macOS
- [ ] Update README.md with installation instructions
- [ ] Submit to Homebrew core (optional, after validation)

## Formula Template
```ruby
class Cmdai < Formula
  desc "Convert natural language to shell commands using local LLMs"
  homepage "https://github.com/wildcard/cmdai"
  url "https://github.com/wildcard/cmdai/releases/download/v1.0.0/cmdai-1.0.0.tar.gz"
  sha256 "..."
  license "AGPL-3.0"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    system "#{bin}/cmdai", "--version"
  end
end
```

## Acceptance Criteria
- [ ] `brew install cmdai` works
- [ ] Binary in PATH after installation
- [ ] All dependencies bundled
- [ ] Uninstall works (`brew uninstall cmdai`)

## Related
- See [MVP_TO_V1.md](MVP_TO_V1.md#3-package-distribution-1-week)
```

---

### Q1 2025 Issues

#### Issue #15: Cloud API Backend (Axum + Postgres)

**Title**: `[Q1] Build Cloud API Backend with Axum and PostgreSQL`
**Labels**: `type: feature`, `priority: P0`, `component: cloud`, `phase: q1-2025`
**Milestone**: `Q1 2025 - Cloud Launch`

**Description**:
```markdown
## Goal
Build the cloud backend API for cmdai SaaS.

## Architecture
- **Framework**: Axum (Rust async web framework)
- **Database**: PostgreSQL (via Supabase or RDS)
- **Cache**: Redis (rate limiting, sessions)
- **Hosting**: Fly.io or Railway

## API Endpoints (MVP)
- `POST /api/v1/auth/login` - User login
- `POST /api/v1/auth/signup` - User registration
- `POST /api/v1/auth/api-keys` - Generate API key
- `POST /api/v1/commands/generate` - Generate command
- `GET /api/v1/commands/history` - Command history

## Tasks
- [ ] Set up Axum project structure
- [ ] Design database schema (users, api_keys, commands)
- [ ] Implement authentication (JWT)
- [ ] Implement command generation endpoint
- [ ] Add rate limiting (Redis)
- [ ] Deploy to Fly.io
- [ ] Set up monitoring (DataDog or Grafana)

## Acceptance Criteria
- [ ] All endpoints working and tested
- [ ] 99.9% uptime
- [ ] <1s P95 latency
- [ ] API documentation (OpenAPI spec)

## Related
- See [ROADMAP.md#11-cloud-backend-weeks-1-4](ROADMAP.md#11-cloud-backend-weeks-1-4)
- See [ARCHITECTURE.md#cloud-architecture-v20](ARCHITECTURE.md#cloud-architecture-v20)
```

---

## Automation with GitHub Actions

### Auto-Label PRs

**File**: `.github/workflows/auto-label.yml`

```yaml
name: Auto Label PRs

on:
  pull_request:
    types: [opened]

jobs:
  label:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/labeler@v4
        with:
          repo-token: "${{ secrets.GITHUB_TOKEN }}"
```

**File**: `.github/labeler.yml`

```yaml
'component: cli':
  - src/cli/**/*

'component: backend':
  - src/backends/**/*

'component: safety':
  - src/safety/**/*

'component: cloud':
  - src/cloud/**/*
  - src/server/**/*

'type: docs':
  - '**/*.md'
  - docs/**/*

'type: test':
  - tests/**/*
```

---

## Best Practices

### For Maintainers

1. **Triage new issues within 48 hours**
   - Add appropriate labels
   - Assign to milestone
   - Ask clarifying questions

2. **Keep milestones up to date**
   - Move completed issues promptly
   - Adjust due dates if needed
   - Close milestone when all issues done

3. **Review project board weekly**
   - Move issues through columns
   - Unblock stuck issues
   - Celebrate completions

### For Contributors

1. **Before starting work**:
   - Comment on issue: "I'd like to work on this"
   - Wait for assignment (prevents duplicate work)
   - Ask questions if unclear

2. **Link PR to issue**:
   - Use "Closes #123" in PR description
   - GitHub auto-closes issue when PR merges

3. **Keep issue updated**:
   - Comment with progress updates
   - Ask for help if blocked
   - Let maintainers know if you need to step away

---

## Document Maintenance

**Owner**: Maintainers
**Update cadence**: As needed when process changes
**Last updated**: 2025-11-19

---

**Questions?**
- GitHub setup issues: [Discussions](https://github.com/wildcard/cmdai/discussions)

---

*Let's organize this project for success!*
