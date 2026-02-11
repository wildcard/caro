---
name: agent-teams
description: Create coordinated Agent Teams for parallel development tasks - code review, implementation, debugging, releases, and research
---

# Agent Teams Skill

**Purpose**: Provide ready-to-use team recipes for caro development workflows where parallel coordination adds value.

**When to Use**: When a task benefits from multiple independent Claude Code sessions that need to communicate, share findings, or work on parallel deliverables.

**Reference**: [ADR-015](docs/adr/ADR-015-agent-teams-adoption.md)

---

## Prerequisites

- `CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS` must be `1` in settings (already configured)
- Task must benefit from parallelism (see "When NOT to Use" below)

## When NOT to Use

- Sequential tasks or same-file edits → use single session
- Quick focused tasks → use Task tool (subagents)
- Tasks with tight interdependencies → use single session
- Simple one-concern work → overkill, use normal flow

---

## Recipe 1: Multi-Perspective Code Review

**Best for**: PRs touching safety, backends, or cross-cutting concerns.

### Prompt Template

```
Create an agent team to review PR #{PR_NUMBER}. Spawn three reviewers:

- Security reviewer: Focus on command injection, input validation,
  safety pattern coverage, and OWASP risks. Check src/safety/ thoroughly.
- Performance reviewer: Check for unnecessary allocations, regex
  compilation in hot paths, and async efficiency.
- API contract reviewer: Validate trait implementations match contracts,
  public API stability, and backward compatibility.

Use Sonnet for each teammate. Have them challenge each other's
findings before the lead synthesizes a final report.

Wait for all teammates to finish before proceeding.
```

### Expected Output
- Each reviewer produces findings with severity ratings
- Reviewers cross-check each other's findings
- Lead produces synthesized review comment for the PR

---

## Recipe 2: Parallel Feature Implementation

**Best for**: Spec-kitty features with 3+ independent work packages.

### Prompt Template

```
Create an agent team to implement the feature in kitty-specs/{NNN}-{name}/.
Read the work packages in tasks/planned/ and assign each to a teammate:

- WP01 teammate: {title} - owns files: {file list}
- WP02 teammate: {title} - owns files: {file list}
- WP03 teammate: {title} - owns files: {file list}

Rules:
- Require plan approval before any teammate writes code
- Only approve plans that include test coverage
- Each teammate runs `make check` before marking tasks done
- If a teammate needs something from another WP, message directly
- Use delegate mode - the lead must NOT implement anything

After all WPs complete, run `make check` for integration validation.
```

### Pre-flight Checks
1. Verify WPs have no overlapping files
2. Ensure each WP has clear inputs/outputs
3. Confirm the feature branch exists

---

## Recipe 3: Competing Hypothesis Debugging

**Best for**: Bugs with unclear root cause, intermittent failures, platform issues.

### Prompt Template

```
Bug: {describe the bug with symptoms and repro steps}

Spawn 4 teammates to investigate competing hypotheses:
- Hypothesis A: {theory 1}
- Hypothesis B: {theory 2}
- Hypothesis C: {theory 3}
- Devil's advocate: Challenge all theories, propose alternatives

Have teammates talk to each other to disprove theories. The theory
that survives adversarial challenge is most likely correct.

Each teammate should:
1. State their hypothesis clearly
2. Gather evidence (read code, run tests, check logs)
3. Attempt to disprove other theories
4. Report confidence level (0-100%)

Lead synthesizes the winning hypothesis and proposes a fix.
```

---

## Recipe 4: Release Preparation

**Best for**: Running `/caro.release.prepare` checks in parallel.

### Prompt Template

```
Create an agent team for release v{VERSION} preparation:

- Security auditor: Run `cargo audit`, review dependency licenses,
  check OWASP patterns in src/safety/. Report PASS/FAIL.
- Test runner: Execute `make check`, run contract tests, integration
  tests, and beta regression. Report PASS/FAIL with details.
- Documentation reviewer: Verify CHANGELOG.md has release entry,
  README.md is current, version in Cargo.toml matches tag,
  API docs compile. Report PASS/FAIL.
- Build validator: Build release binaries for linux-amd64 and
  macos-silicon, verify binary size < 50MB, check feature flags.
  Report PASS/FAIL.

ALL teammates must report PASS for release to proceed.
Use Sonnet for all teammates.
```

---

## Recipe 5: Research Sprint

**Best for**: Evaluating new technology, backends, or approaches.

### Prompt Template

```
Create an agent team to research {TOPIC}:

- Technical researcher: Evaluate implementation feasibility, API
  surface, performance characteristics, and integration complexity
  with caro's architecture.
- Ecosystem researcher: Check community adoption, maintenance activity,
  license compatibility (must be AGPL-3.0 compatible), quality of
  documentation, and viable alternatives.
- Integration researcher: Prototype integration points, identify
  required trait changes, estimate scope of work, and flag risks.

Require plan approval so research stays focused.
Have researchers share findings and challenge assumptions.
Produce a recommendation document at docs/research/{topic}.md.
```

---

## Team Management

### Starting a team
Tell Claude to create a team with the recipe prompt above.

### Monitoring
- **In-process mode**: Shift+Up/Down to select teammates
- Press Enter to view a teammate's session
- Press Ctrl+T to toggle the shared task list

### Steering
- Message teammates directly to redirect their approach
- Lead can reassign tasks if someone gets stuck

### Cleanup
```
Clean up the team
```
Always shut down teammates first, then clean up via the lead.

---

## Model Selection for Teammates

| Recipe | Recommended Model | Rationale |
|--------|------------------|-----------|
| Code Review | Sonnet | Analysis tasks, cost-effective |
| Implementation | Sonnet | Code generation, good balance |
| Debugging | Sonnet | Investigation, sufficient reasoning |
| Release | Sonnet | Checklist execution, low complexity |
| Research | Sonnet (technical), Opus (architecture) | Deep analysis for architecture |
| Safety audit | Opus | Critical reasoning required |

---

## Integration Points

### With Spec-Kitty
Use Recipe 2 at the `implement` phase when work packages are independent.
The lead reads task files from `tasks/planned/` and distributes.

### With Existing Hooks
All teammates inherit project hooks automatically:
- `block-main-commits.sh` enforces feature branches
- `worktree-protection.sh` protects worktrees
- `session-start-continuity.sh` loads project context

### With Agent Profiles
Map `agent-profiles.yaml` specializations to teammate roles:
- `security` profile → security reviewer/auditor teammates
- `rust` profile → implementation teammates
- `docs` profile → documentation reviewer teammates
- `devops` profile → build/release teammates
