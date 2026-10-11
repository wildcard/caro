# Caro as a Cross-Harness Auto-Mode Engine

**Date**: 2026-10-11
**Status**: Positioning proposal. Not a feature spec. It clears none of the
gates in [`validation-discipline.md`](../../.claude/rules/validation-discipline.md)
(see §7).
**Evidence**: [`2026-10-11-competitive-landscape-refresh.md`](../research/2026-10-11-competitive-landscape-refresh.md)
and [`auto-modes.md`](../research/2026-10-11-competitive-refresh/auto-modes.md)
**Builds on**: PR #1513 (`caro guard`, spec 011, ADR-019) and
[`GUARDIAN_AGENT.md`](../GUARDIAN_AGENT.md)

## 1. The thesis

Every frontier agent now ships an "auto mode": something that decides
whether the agent's next shell command may run without asking you.

- **Claude Code**: an auto mode classifier.
- **Codex**: Auto-review, also called "guardian".
- **Cursor**: Auto-review.
- **VS Code**: Assisted permissions.
- **Grok Build** and **Qwen Code**: each has an Auto mode.

Each one is built into one harness and runs on one vendor's models. The
user cannot read it, test it or carry it to another tool.

Caro can be the alternative. One engine has these properties:

- **Cross-agent, cross-harness, cross-model**: the same policy in Claude
  Code, Codex, Copilot CLI, Cursor, Grok Build, OpenCode or a script, with
  any model or none.
- **Yours**: it runs locally. You can read it, configure it, and see every
  decision in a local log.
- **Community-run**: security researchers maintain the threat rules.
  Terminal power users maintain the "this is routine, don't ask me" rules.

## 2. What the vendors ship

| | Claude Code auto mode | Codex Auto-review | Cursor Auto-review | Caro (proposed) |
|---|---|---|---|---|
| Decides with | Rules, then an LLM classifier | Sandbox, then a reviewer agent | Allowlist, sandbox, then a classifier | Deterministic floor, then an optional local judge |
| Models | Claude only | Breaks on custom providers | Cursor backend | Any, or none |
| Open source | No | Yes (policy in `core/src/guardian/policy.md`) | No | Yes (AGPL-3.0) |
| Same policy in other harnesses | No | No | No | Yes, through hooks |
| Deterministic, testable | No | No | No | The floor is; the judge is optional |
| Sends the transcript off the machine | Yes | Yes | Yes | No |

## 3. Where Caro sits

```
 agent proposes a shell command
            │
            ▼
 harness PreToolUse hook ──► caro guard
            │                 ├─ 1. deterministic floor: critical → deny (cannot be overridden)
            │                 ├─ 2. curated + community rules: high → ask
            │                 ├─ 3. routine rules: known-safe → no opinion today, allow later
            │                 └─ 4. optional local judge for the rest (opt-in, escalates to ask)
            ▼
 harness's own rules / auto mode (still runs)
            ▼
 OS sandbox (srt, Codex sandbox, Copilot MXC, nono…)  ← the real boundary
            ▼
 shell
```

Caro adds to the vendor's auto mode and the sandbox. It does not replace
either. Every vendor says its classifier is not a security boundary, and
Caro's regex floor is not one either.

## 4. Two roles, and what each needs

**Role 1: the guard.** It sits under any vendor's auto mode.

- It denies critical commands and asks on high-risk ones. It never says
  "allow".
- PR #1513 builds this role: shadow mode by default, adapters for Claude
  Code, Codex, Grok Build and OpenCode.
- Copilot CLI, Cursor and VS Code read `.claude/settings.json` hooks, so the
  same adapter may reach them. Test this before you claim it.

**Role 2: the approver.** It is the auto mode for harnesses that have none.

- Users who bring their own model get no vendor judge. This covers
  OpenCode, Crush, Aider, Cline, Zed, Mistral Vibe and Codex on custom
  providers. This is Caro's clearest opening.
- The approver must say "allow" for routine commands, so it needs more than
  the guard:
  - A real shell parser. Regex on raw text gets bypassed: GuardFall, Roo
    Code CVE-2026-82536/82537, and the 6 gaps that #1513 measured
    (#1054).
  - Flag-level knowledge of safe commands (`git log` yes, `git push` no),
    as in safe-chains and Dippy.
  - A calibrated local judge with a hard floor it cannot relax. ADR-017
    and ADR-018 already define the decision types and the calibration
    experiment.
- Hook limits apply:
  - In Claude Code, a hook "allow" skips the user's own prompt. An
    approver there takes on real responsibility.
  - Gemini CLI, Kiro, Windsurf and Cline accept only vetoes from a hook.
  - ACP `session/request_permission` is the one protocol-level choke point
    for many agents.

## 5. The community model

1. **Rules as data, separate from the engine.**
   - Version the rule packs (threat rules, routine rules, CVE rules) as
     data files with their own license, for example CC BY 4.0, the way
     Sigma separates rules from SIEMs.
   - Other guards, vendors and harness-native formats can then use them.
   - The engine stays AGPL-3.0.
   - Choosing a license is a human decision (see §8).
2. **Tags that security people already use**: MITRE ATT&CK technique IDs,
   OWASP Agentic Top 10 (ASI02 tool misuse, ASI05 unexpected code
   execution), and GTFOBins / LOOBins / LOLBAS references.
3. **A public benchmark as the shared asset.**
   - Measure false-positive and false-negative rates on an open corpus:
     GuardFall-style bypasses, Atomic Red Team commands, and real agent
     traffic from opt-in shadow logs.
   - Compare against dcg, cc-safety-net and the vendor judges.
   - A neutral, reproducible scoreboard is something no vendor can
     publish about its own judge.
4. **Export, not only hooks.** Compile the same policy to Codex
   `prefix_rule`, Gemini CLI TOML, Claude Code `permissions`, Cursor
   `permissions.json` and Kiro `permissions.yaml`. Each harness's native
   feature then becomes a distribution channel.
5. **Standards.** Implement the AARTS draft's security-engine interface if
   it gets adopters. Emit decisions as OpenTelemetry events for audit.

## 6. The case against (from the research)

1. **The vendor judge is free, on by default and sees intent.** Prompt
   injection and exfiltration need context that a stateless check of one
   command cannot see.
2. **The sandbox is the boundary.**
   - In late Aug 2026, Johann Rehberger reported a prompt-injection chain
     that beat Claude auto mode in 60–80% of a small number of runs.
   - The source is secondary coverage, for example
     [BankInfoSecurity](https://www.bankinfosecurity.net/hidden-attack-slips-past-claude-code-auto-mode-a-32693).
     We did not read his post.
   - Anthropic called auto mode a best-effort convenience, not a security
     guarantee.
   - The fix everyone recommends is a sandbox and egress control, not a
     better list.
3. **The deterministic niche is taken.** dcg (~6.1k stars, 17+ harnesses)
   and cc-safety-net (~1.6k stars, MIT, 14+ harnesses) are free and further
   ahead. Caro would be the third entrant.
4. **AGPL may slow adoption.** Some companies will not run AGPL code even
   as a local hook. Rules under a permissive license (§5.1) reduce this.
5. **Hooks cannot fully replace native auto mode.** Most harnesses accept
   only vetoes from a hook, and Claude Code still applies its own deny and
   ask rules after a hook "allow".

What survives these objections:

- Role 2 for bring-your-own-model users.
- One portable, auditable policy.
- A neutral benchmark.

"A better regex list" does not survive them.

## 7. Validation status

This is a new product line under `validation-discipline.md`. None of the
five gates is cleared:

- **Gate 1: no transcripts yet.** The cohort to interview: people who run
  coding agents with pre-approved shell access, or with
  auto/yolo/bypass modes, across at least two harnesses.
- **Gate 3: "what breaks at 100 real users".** Hook timeouts fail open.
  False positives get the guard uninstalled. Rule churn breaks pinned
  versions.
- **Gate 4: a devil's-advocate review of this note.** It goes in the PR
  that carries this note.
- **Shadow mode in #1513 is the evidence path.** Its decision log measures
  how often Caro would deny or ask on real agent traffic. Its kill criteria
  are in spec 011.

## 8. Decisions for the owner

| # | Decision | Default |
|---|---|---|
| 1 | Warp messaging after Warp open-sourced its client | Lead with "fully open, fully local"; keep "keep your terminal" as the supporting line |
| 2 | Compete with dcg and cc-safety-net, or interoperate with them | Interoperate: shared benchmark, rule export, no rule import from dcg (license rider) |
| 3 | License the rule packs separately (e.g. CC BY 4.0) | Yes, after legal review. This needs a human: it is a licensing decision. |
| 4 | Which role to build first | Role 1 (guard) via #1513, then Role 2 for bring-your-own-model harnesses |
| 5 | Recurring competitive refresh | Monthly re-check of `/compare` facts, quarterly discovery sweep |
