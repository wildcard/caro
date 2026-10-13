# Caro and Agent Auto Modes: Positioning, Tests and Decisions

**Date**: 2026-10-11, revised 2026-10-13 after the PR #1583 review and the
owner's sourced corrections.
**Status**: Positioning hypothesis and test plan. Not a feature spec. It
clears none of the gates in
[`validation-discipline.md`](../../.claude/rules/validation-discipline.md).
Nothing in this note is scheduled, and no decision below is approved.
**Evidence**: [`2026-10-11-competitive-landscape-refresh.md`](../research/2026-10-11-competitive-landscape-refresh.md)
and its [raw notes](../research/2026-10-11-competitive-refresh/README.md)
**Builds on**: `caro guard` (#1513, merged 2026-10-11 as `9516868`;
[ADR-020](../adr/ADR-020-guardian-hook-harness-adapters.md);
[spec 011](../../specs/011-harness-guardian/spec.md)) and
[`GUARDIAN_AGENT.md`](../GUARDIAN_AGENT.md)
**Approver**: `@wildcard` (`.github/STAKEHOLDERS.yml`: every area)

## 1. Recommendation

1. **Position Caro as an integrated, local-by-default shell specialist with
   bundled deterministic checks.** Do not position it as a cross-harness
   auto-mode engine yet.
2. **Scope "fully open, fully local"** to the stable default embedded backend
   with a cached model. The code is AGPL-3.0. The default model (Qwen2.5-Coder
   1.5B) is Apache-2.0. Optional remote backends send prompts off the machine.
3. **Keep `caro guard` shadow-only** until the spec 011 pilot passes. Any
   new guard behavior ships opt-in and labeled experimental.
4. **Fix verification before stronger safety claims.**
   - #1597: the website claims suite never calls the validator.
   - #1537: 26 of 49 test targets run in no workflow (census at `4a84d09`).
5. **Coexist with cc-safety-net and dcg for now.** Claim differentiation only
   after test 1 shows better detection at a matched nuisance rate.
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

### Test 1: matched-friction delta

- **Question:** at the same nuisance rate, does Caro catch more destructive
  commands than cc-safety-net?
- **Method:** run every corpus string through Caro (the `SafetyValidator`
  library, or `caro guard --harness generic --mode enforce --no-log` on
  stdin) and through cc-safety-net's documented checker. Nothing executes.
- **Metrics:** misses (false negatives) on destructive cases **and** false
  positives on benign cases. Report both, per class. A larger deny set alone
  proves nothing.
- **Controls:**
  - Labels are fixed before any tool runs.
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

### Test 2: semantic-equivalence attacks and benign lookalikes

- **Question:** does a command that means the same thing get the same
  verdict, and does a harmless lookalike stay quiet?
- **Corpus:**
  - The bypass classes in #1639 and the GuardFall classes, each paired with
    its plain canonical form.
  - Benign lookalikes: `echo`, commit messages, heredocs, quoted mentions.
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

### Test 3: hook authority and coverage, version-pinned, with a non-executing stub

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

### Test 4: clean-cache local setup, offline egress and resource cost

- **Question:** does the scoped "fully local" claim hold on a clean machine?
- **Method:**
  1. Start from an empty model cache and time the first command.
  2. Record the download size.
  3. Block the network and run N `--dry-run` commands with the default
     embedded backend.
  4. Count outbound connection attempts.
  5. Record RSS, p50/p95 latency and disk use.
- **Control:** the same run with the network on.
- **Threshold:** zero outbound attempts after the cached download. Any
  attempt falsifies the claim, and the website copy changes.
- **Owner:** `@wildcard`; agents `llm-integration-expert` and
  `macos-unix-systems-expert` (`src/backends/embedded_backend.rs`), and
  `rust-cli-expert` (`src/cache/**`).
- **Depends on:** nothing. This is the cheapest test.
- **DoD:** a script, raw numbers per OS, and the result recorded in the
  research note.

### Test 5: shadow user value and retention

- **Use the existing spec 011 pilot. Do not invent new metrics.**
  - 10 opted-in users, 2 weeks.
  - Cohort: users who pre-approve shell commands.
- **Reconsider the guard if any of these holds (spec 011):**
  - fewer than 1 would-be deny/ask per 500 logged commands;
  - 20% or more of would-be asks are routine (user-labeled);
  - p95 latency of 500 ms or more.
- **Quality controls:**
  - User labels need `caro guard report --label` (#1512).
  - Logs stay local; users share redacted logs by hand.
  - Report how many users kept the hook installed at day 14, as a
    descriptive number only.
- **Interviews (proposal):** 8 exploratory interviews with
  bring-your-own-model users. They are not a kill criterion. They count
  toward Gate 1's 20 transcripts.
- **#1510 data gates** govern any judge or classifier work that follows.
- **Owner:** `@wildcard`. Transcripts go to `docs/discovery/transcripts/` via
  the `caro.discovery` skill. A human runs the interviews.
- **DoD:** the pilot evaluation is posted on #1512, and transcripts are
  saved under the anonymization rules.

## 5. Prioritized work packet

| P | Item | Evidence | Benefit | Falsifier | Maintenance cost | Disposition | Stakeholders | DoD |
|---|---|---|---|---|---|---|---|---|
| P0 | #1583: factual repair and merge conflicts | 26 review threads; 5 conflicting pages | Public claims match sources | A reviewer finds an unsourced claim | Re-check per refresh | Local fix ready on `review/pr1583-fixes`; push needs owner go | `@wildcard`; technical-writer, dx-product-manager | Threads answered; CI green; no conflicts |
| P0 | #1597: claims suite calls the real validator | `--validate-only` does not exist; 8 of 12 landing scenarios disagree | Safety claims become testable | Suite stays green while copy is false | One test file | Existing issue; needs copy or pattern decisions | `@wildcard` (safety); qa-testing-expert | Suite fails on a false claim |
| P0 | #1537: test-target census | 26 of 49 targets run in no workflow (`4a84d09`) | Regression guards run | A new target is silently skipped | CI time | Existing issue; CI-gate decision | `@wildcard` (`.github/workflows/**`) | Every target runs, or is listed with a reason |
| P1 | #1054, #1614, #1639, #1302: safety core | Measured bypasses; #1614 open | Fewer bypasses | Test 2 | Parser upkeep | One human safety review track | `@wildcard`; safety-pattern-auditor, safety-pattern-developer | Each class fixed or recorded as accepted |
| P1 | #1623: Copilot CLI live contract | Output shape unverified; silent fail-open risk | Guard works in Copilot | Test 3 | One adapter | Existing issue | `@wildcard`; caro-integrator | Live result documented |
| P1 | #1512 / spec 011: guard delta and pilot | Kill criteria exist | Evidence before positioning | Tests 1 and 5 | Log and labels | Existing tracking issue | `@wildcard` | Pilot evaluated |
| P2 | Discovery before any engine work | Gate 1 has 0 transcripts | Real demand signal | Interviews find no pain | Human time | Proposal; respects #1510 gates | `@wildcard`; `caro.discovery` | Transcripts saved; ledger updated |

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

## 7. Decisions for the owner (choices, none approved)

| # | Decision | Options | Default |
|---|---|---|---|
| D1 | Lead message | (a) scoped "fully open, fully local"; (b) "keep your terminal"; (c) hold | (a), with (b) as the supporting line |
| D2 | Positioning | (a) local-by-default shell specialist with bundled checks; (b) cross-harness engine now | (a) |
| D3 | `caro guard` mode | (a) shadow-only until the spec 011 pilot passes; (b) recommend enforce now | (a); new features opt-in and experimental |
| D4 | cc-safety-net and dcg | (a) coexist until test 1; (b) compete; (c) interoperate now | (a) |
| D5 | Rule-pack license | (a) defer; (b) decide now | (a) |
| D6 | dcg | (a) docs-only comparison; (b) seek a human license-eligibility decision to test it | (a) |
| D7 | Refresh cadence | (a) route through Hermes, the integrator, coder-loop and QA as in the watchlist; (b) a new routine | (a); wiring it into agent instructions is a separate change |
| D8 | Paid vendor calls for test 3 (Claude Code) | (a) docs only; (b) approve a capped live run | (a) |
| D9 | Test corpus and interviews | (a) adopt the 400-case corpus and 8 interviews as proposals; (b) other | (a), not as graduation thresholds |
| D10 | Push the local fixes to PR #1583 | (a) after review of the local patch; (b) hold | (a) |

## Open items

- `GUARDIAN_AGENT.md` and `CLAUDE.md` still say "CRITICAL, HIGH, MEDIUM,
  LOW". The code uses Safe/Moderate/High/Critical. `GUARDIAN_AGENT.md` also
  says "62+" patterns, while other files say "52+". Align them in a separate
  change.
- `src/guard/**` has no entry in `.github/STAKEHOLDERS.yml`.
