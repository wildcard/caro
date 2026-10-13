# Caro and Agent Auto Modes: Positioning, Tests and Decisions

**Date**: 2026-10-11, revised 2026-10-13 after the PR #1583 review and the
owner's sourced corrections.
**Status**: Positioning hypothesis and test plan. Not a feature spec. It
clears none of the gates in
[`validation-discipline.md`](../../.claude/rules/validation-discipline.md).
Nothing in this note is scheduled. Section 7 lists the owner's directions and the gates still open.
**Evidence**: [`2026-10-11-competitive-landscape-refresh.md`](../research/2026-10-11-competitive-landscape-refresh.md)
and its [raw notes](../research/2026-10-11-competitive-refresh/README.md)
**Builds on**: `caro guard` (#1513, merged 2026-10-11 as `9516868`;
[ADR-020](../adr/ADR-020-guardian-hook-harness-adapters.md);
[spec 011](../../specs/011-harness-guardian/spec.md)) and
[`GUARDIAN_AGENT.md`](../GUARDIAN_AGENT.md)
**Approver**: `@wildcard` (`.github/STAKEHOLDERS.yml`: every area)

## 1. Recommendation

1. **Position Caro as an integrated, local-by-default shell experience with
   bundled deterministic checks.** Do not position it as a cross-harness
   auto-mode engine yet. Avoid "only Caro", universal protection, or any
   promise that all commands or harness events are covered.
2. **Scope every claim to what ships by default.**
   - Openness: the code is AGPL-3.0. The default embedded model
     (Qwen2.5-Coder 1.5B) is Apache-2.0.
   - Offline: only after the first model download, with a cached model and
     the default embedded backend. Optional remote backends send prompts off
     the machine.
   - Safety: at the default `moderate` safety level, Critical is blocked and
     High asks.
3. **Keep stable defaults lean.** Experimental backends, guard enforcement,
   harness adapters, any judge and any rule export stay opt-in. `caro guard`
   stays shadow-only until the spec 011 pilot passes. Shadow mode is
   instrumentation, not proof of safety or product-market fit.
4. **Fix verification before stronger safety claims.**
   - #1597: the website claims suite never calls the validator.
   - #1537: 26 of 49 test targets run in no workflow (census at `4a84d09`).
5. **Interoperate provisionally with cc-safety-net** (and dcg, docs only).
   Measure **incremental true catches at matched friction**, not a larger
   deny or ask set. Existing sandbox or guard use can complement Caro; it
   does not by itself falsify demand.
6. **Defer rule-pack licensing.** There are no outside contributors yet.

## 2. What the vendors ship (corrected)

| | Claude Code auto mode | Codex Auto-review ("guardian") | Cursor Auto-review | Caro today |
|---|---|---|---|---|
| On by default | Yes, from v2.1.283 | No (`approvals_reviewer = "auto_review"`) | Yes, from 3.6 | `caro guard` is opt-in, shadow by default |
| Decides with | Rules, built-in critical-path protections, then an LLM classifier | OS sandbox and execpolicy rules, then a reviewer agent | Allowlist, sandbox, then a classifier | 52+ patterns with risk levels; Critical blocked at the default safety level |
| Extension points | `PreToolUse` hooks, and mods (`tool.check`) that can approve before the classifier. Hooks and mods have different authority. | `PreToolUse` (allow/deny), editable policy | `beforeShellExecution`, `preToolUse` | `[safety]` custom patterns and allowlist |
| Models | Claude only | Any provider by design; a custom-provider bug has workarounds ([openai/codex#31255](https://github.com/openai/codex/issues/31255)) | Cursor backend | Embedded by default; remote optional |
| Open source | No | Yes (Apache-2.0) | No | Yes (AGPL-3.0) |

Other facts that limit the claims we can make:

- goose Smart Approve uses the user's own provider, so it is provider-agnostic.
- cc-safety-net (MIT) covers 16 harnesses and parses commands. It protects
  secrets and has cloud/IaC rulebooks. Its documented limits are incomplete
  event coverage and fail-open hosts.
- dcg is source-available. Its rider excludes Anthropic and parties acting
  for it, and counts benchmarking, testing and analyzing as use.
- Atuin AI does Caro's core job, with local inference when self-hosted.
- Vendors say neither a classifier nor a pattern list is a security boundary.

Sources and dates: research note §1 and §4.

## 3. Where `caro guard` sits today

```
 agent proposes a shell command
            │
            ▼
 harness hook (PreToolUse) ──► caro guard (shadow by default: logs only)
            │                    enforce mode (opt-in): Critical → deny, High → ask,
            │                    otherwise no opinion; never emits "allow"
            ▼
 harness's own rules, protections and auto mode (unchanged)
            ▼
 OS sandbox, if the user runs one  ← the real boundary
            ▼
 shell
```

The guard adds a deterministic floor. It replaces neither the vendor's
mode nor a sandbox. An "approver" role that emits "allow" is out of scope
until tests 1–3 pass and a human safety review signs off.

## 4. The five tests

Every test runs on strings or harmless stub commands (`true`, `printf ok`).
No test executes a destructive command, calls a paid vendor model, or runs
dcg. Pin versions when the test starts and record them in the result.

Reference versions seen on 2026-10-11 (re-pin at run time):

| Component | Version |
|---|---|
| Caro | `origin/main` at `c3169411` (includes #1513); re-test on #1614 if it merges |
| Claude Code | 2.1.296 |
| Codex CLI | 0.162.1 |
| GitHub Copilot CLI | 1.0.95 |
| OpenCode | 1.19.0 |
| cc-safety-net | pin the release tag at run time (not recorded) |
| Grok Build | pin at run time (not recorded) |

### Test 1 (a): frozen, independently labeled detection delta at matched nuisance

- **Question:** at the same nuisance rate, does Caro catch more destructive
  commands than cc-safety-net?
- **Method:** run every corpus string through Caro (the `SafetyValidator`
  library, or `caro guard --harness generic --mode enforce --no-log` on
  stdin) and through cc-safety-net's documented checker. Nothing executes.
- **Metrics:** incremental true catches (destructive cases one tool flags
  and the other misses, both ways); misses; benign false positives; verdicts
  on ambiguous cases; per-call latency; and which harness events each tool
  can see. Report each per class. A larger deny or ask set alone proves
  nothing.
- **Oracle rule (from #1597):** a usage error, parse error, crash or
  non-zero exit without a parsed verdict counts as a **failed run**, never
  as a block.
- **Controls:**
  - Labels are fixed, and the holdout frozen, before any tool runs. Two
    labelers work independently; disagreements are resolved and recorded.
  - A no-op baseline is scored alongside.
  - Caro is scored at both `moderate` and `strict` safety levels.
  - The corpus includes a 25% holdout that nobody tunes against.
- **Corpus (proposal, not an existing threshold):** 400 cases (200 benign,
  120 destructive, 80 ambiguous), each labeled by two people.
- **Threshold to claim differentiation:** Caro's recall exceeds
  cc-safety-net's at a matched false-positive rate on the holdout, with a
  bootstrap 95% interval that excludes zero. Otherwise: coexist, and claim
  nothing.
- **Excluded:** dcg (license rider). Including it needs a human license
  decision.
- **Owner:** `@wildcard`; agents `safety-pattern-auditor` and
  `qa-testing-expert` (STAKEHOLDERS: `src/safety/**`, `tests/safety_*`).
- **Depends on:** #1597 (a real-validator harness), the cc-safety-net license
  check (MIT; done), and the corpus labels.
- **DoD:** the corpus, labels, versions and a per-class table are committed
  under `tests/`, and the result is recorded in this note.

### Test 2 (b): semantic-equivalence variants and benign lookalikes, validator only

- **Question:** does a command that means the same thing get the same
  verdict, and does a harmless lookalike stay quiet?
- **Corpus:**
  - The bypass classes in #1639 and the GuardFall classes, each paired with
    its plain canonical form.
  - Benign lookalikes: `echo`, commit messages, heredocs, quoted mentions.
- **Method:** strings go into the validator only (library call or
  `caro guard --harness generic --mode enforce --no-log`). Nothing executes.
  The test-1 oracle rule applies.
- **Controls:** each canonical form must be flagged. If one is not, that
  pair is void.
- **Metric:** the share of equivalence pairs with the same verdict, and the
  false-positive count on lookalikes.
- **Threshold:** any Critical-class miss becomes a #1639 entry. Fixes go
  through `safety-pattern-developer` TDD. This test never changes patterns
  itself.
- **Owner:** `@wildcard` as human safety owner; agents
  `safety-pattern-auditor` and `safety-pattern-developer`.
- **Depends on:** #1614 (segment-aware validation, open), with #1302 and
  #1054 for parser work.
- **DoD:** the pair list and results are committed, and each miss links to
  an issue.

### Test 3 (c): hook authority and event coverage, version-pinned, with a non-executing stub

- **Question:** what does each harness really do with a hook decision?
- **Method:** install a stub hook that logs its payload and returns a fixed
  decision. The agent's command is always harmless (`true`).
- **What to probe, per harness:**
  - Which of deny, ask and allow is honored.
  - Behavior on timeout (the stub sleeps past the limit) and on failure
    (non-zero exit, malformed JSON).
  - The `cwd` field.
  - Interactive vs headless runs.
  - Subagent tool calls.
  - Custom and MCP tools.
- **Model calls:** use local-model modes where they exist (Codex `--oss`,
  OpenCode with a local provider, Copilot BYOK with `COPILOT_OFFLINE=true`).
  Claude Code needs Anthropic models; a live run there needs an owner
  decision on paid calls (D8). Without one, record its documented behavior
  only.
- **Claude Code specifics:**
  - Does a hook "allow" skip the classifier?
  - How do hooks and `tool.check` mods rank against each other?
- **Copilot CLI:** routed to #1623 (live contract; nested vs top-level
  `permissionDecision`).
- **Owner:** `@wildcard`; executing agent caro-integrator (per #1623).
  `src/guard/**` has no area in `.github/STAKEHOLDERS.yml`, which is a gap
  to fill.
- **Depends on:** #1623; #1512 (unknown tool names, timeout detection).
- **DoD:** a version-pinned matrix (harness × probe) is committed to
  `docs/coding-agents/`, and ADR-020's adapter table is updated.

### Test 4 (d): clean vs cached setup, offline egress, latency and RSS on named hardware

- **Question:** does the scoped "fully local" claim hold on a clean machine?
- **Method:**
  1. Start from an empty model cache and time the first command.
  2. Record the download size.
  3. Block the network and run N `--dry-run` commands with the default
     embedded backend.
  4. Count outbound connection attempts.
  5. Record RSS, p50/p95 latency and disk use.
- **Hardware:** name it in the result: one Apple-silicon Mac and one
  x86_64 Linux machine, with CPU model, RAM and OS version.
- **Control:** the same run with the network on.
- **Threshold:** zero outbound attempts after the cached download. Any
  attempt falsifies the claim, and the website copy changes.
- **Owner:** `@wildcard`; agents `llm-integration-expert` and
  `macos-unix-systems-expert` (`src/backends/embedded_backend.rs`), and
  `rust-cli-expert` (`src/cache/**`).
- **Depends on:** nothing. This is the cheapest test.
- **DoD:** a script, raw numbers per OS, and the result recorded in the
  research note.

### Test 5 (e): opted-in shadow value and retention

- **Use the existing spec 011 pilot. Do not invent new metrics.**
  - 10 opted-in users, 2 weeks.
  - Cohort: users who pre-approve shell commands.
- **Reconsider the guard if any of these holds (spec 011):**
  - fewer than 1 would-be deny/ask per 500 logged commands;
  - 20% or more of would-be asks are routine (user-labeled);
  - p95 latency of 500 ms or more.
- **Quality controls:**
  - User labels need `caro guard report --label` (#1512).
  - Each week, sample a fixed number of **unflagged** commands per user for
    labeling, so the pilot estimates misses as well as nuisance.
  - Logs stay local; users share redacted logs by hand.
  - Report how many users kept the hook installed at day 14, as a
    descriptive number only.
- **Interviews (proposal):** 8 exploratory interviews with
  bring-your-own-model users. They are not a kill criterion: a user who
  already runs a sandbox or another guard may still want Caro alongside it.
  They count toward Gate 1's 20 transcripts. Outreach needs separate
  authorization.
- **#1510 data gates** govern any judge or classifier work that follows.
- **Owner:** `@wildcard`. Transcripts go to `docs/discovery/transcripts/` via
  the `caro.discovery` skill. A human runs the interviews.
- **DoD:** the pilot evaluation is posted on #1512, and transcripts are
  saved under the anonymization rules.

## 5. Dispatch entries (prepared, not filed or sent)

These route work to existing issues and owners. Nothing here creates an
issue, comment or routine. The human owner for every area is `@wildcard`
(`.github/STAKEHOLDERS.yml`). Disposition: **build** (do the work),
**interoperate** (use or align with another tool), **watch** (track, no
work now) or **stop**.

| P | Unit | Owner (STAKEHOLDERS.yml) | Depends on | Expected benefit | Falsifier | Maintenance cost | Disposition | DoD |
|---|---|---|---|---|---|---|---|---|
| P0 | #1583 factual and conflict repair | `@wildcard`; technical-writer, dx-product-manager, docs-release-manager (`website/**`, `docs/**`) | Owner review of the local patch | Public claims match dated primary sources | A reviewer finds an unsourced or overbroad claim | One refresh per month | build | 26 threads dispositioned; no conflicts with `main`; build and spell checks pass; rendered pages checked |
| P0 | #1597 real-validator oracle for website claims | `@wildcard` (safety and copy); qa-testing-expert | Copy-or-pattern decisions in #1597 | Safety claims become testable | The suite stays green while a landing claim is false | One test file | build | The suite calls `SafetyValidator`; usage or parse errors fail the test instead of counting as blocks |
| P0 | #1537 CI census | `@wildcard` (`.github/workflows/**`) | CI-gate decision in #1537 | Regression guards actually run | A target in `tests/` is skipped again | CI minutes | build | Latest census (26 of 49 omitted at `4a84d09`) is closed: each target runs or is listed with a reason |
| P0 | #1603 with #1536: QA reads and records routine state | `@wildcard`; QA loops in `.claude/automation/config/schedule.yaml` | #1536 (strict schedule contract) | QA runs stop re-scanning months of history | A QA run still starts from the May 2026 bootstrap | None new; uses `automation/routine-status` | build | QA reads and writes its state with `bin/routine-status state get/put`; no new storage |
| P1 | #1054, #1614 (open), #1639, #1302: safety core | `@wildcard` as human safety owner; safety-pattern-auditor, safety-pattern-developer (`src/safety/**`) | Human safety review; TDD | Fewer measured bypasses | Test 2 (b) | Parser upkeep | build, one review track | Each bypass class fixed with a RED test first, or recorded as accepted risk |
| P1 | #1623 live Copilot CLI hook contract | `@wildcard`; caro-integrator (per #1623). `src/guard/**` has no STAKEHOLDERS area. | A live Copilot CLI with a local model (no paid calls) | The guard is not silently ignored (fail-open) | Test 3 (c) shows the nested output is ignored | One adapter | build | Input and output shape documented; silent fail-open closed or documented |
| P1 | #1512 / spec 011 guard delta and pilot | `@wildcard` | #1597, #1623, #1512 `report --label` | Evidence before any positioning | Tests 1 (a) and 5 (e) | Log, labels | interoperate (cc-safety-net), then decide | Matched-friction delta and pilot results posted on #1512 |
| P2 | Discovery before any new engine | `@wildcard`; `caro.discovery` skill (`docs/**`) | Separate authorization for outreach; #1510 gates for any judge | Real demand signal | Interviews show no unmet need | Human time | watch | Transcripts saved under the anonymization rules; ledger updated |
| — | dcg | `@wildcard` | A separately eligible human license decision | — | — | — | watch (docs only) | No execution, testing, benchmarking or import by this agent |

**Run rules for any routine that picks up these entries**
(`.claude/automation/CONTRACT.md`):

- Run preflight first. If it fails, record `Blocked`.
- Record exactly one status per run, with evidence (issue, PR or run link)
  and an honest `--max-age-hours`.
- Never record `Succeeded` on a failure path. A platform session that ends
  cleanly is not task success.
- Search for the fingerprint `fp:<routine>/<slug>` in open and closed issues
  before filing.
- Keep working state on `automation/routine-status`.

## 6. The case against (kept for honesty)

1. In Claude Code and Cursor, the vendor judge is free, on by default and
   sees intent. Claude Code also has built-in critical-path protections.
2. Vendors say the sandbox is the boundary. Prompt-injection chains have
   beaten LLM auto modes (late Aug 2026, secondary coverage).
3. cc-safety-net (MIT, 16 harnesses) and dcg are further ahead on reach.
4. Some companies avoid AGPL code, even as a local hook.
5. Many harnesses accept only vetoes from a hook.
6. Users chose auto modes to avoid prompts; extra asks may get the guard
   uninstalled.

## 7. Owner decisions (as given on 2026-10-13, and the gates left)

The owner's steering on 2026-10-13 set these directions for the proposal:

1. Integrated, local-by-default shell experience with bundled deterministic
   checks; claims scoped to the code, the default embedded model, cached-model
   offline use and the default safety level.
2. Interoperate provisionally; measure incremental true catches at matched
   friction.
3. Defer separate rule-pack licensing; no license change; no dcg execution or
   import by this agent.
4. Verification, safety and guard contracts come before another engine;
   shadow mode is instrumentation.
5. Monthly primary-source checks and quarterly discovery through existing
   Hermes, integrator, coder-loop and QA workflows, as a proposal only.

**Gates that still need a separate human approval** (none is granted by this
note):

| # | Gate | Default if nobody decides |
|---|---|---|
| G1 | Push the local fixes to PR #1583 | Hold |
| G2 | Install dependencies to build and render locally (network, ~900 MB) | Hold; CI renders on push |
| G3 | Wire the refresh into Hermes, integrator or QA instructions | Hold |
| G4 | Paid vendor calls for test 3 (Claude Code live run) | Docs only |
| G5 | Participant outreach for interviews or the pilot | Hold |
| G6 | Any dcg execution (needs an eligible human and a license decision) | No |
| G7 | Adopt the 400-case corpus as test-1 design (proposal, not a graduation threshold) | Adopt as a proposal |

## Open items

- `GUARDIAN_AGENT.md` and `CLAUDE.md` still say "CRITICAL, HIGH, MEDIUM,
  LOW". The code uses Safe/Moderate/High/Critical. `GUARDIAN_AGENT.md` also
  says "62+" patterns, while other files say "52+". Align them in a separate
  change.
- `src/guard/**` has no entry in `.github/STAKEHOLDERS.yml`.
