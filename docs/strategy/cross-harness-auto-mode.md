# Caro as a Cross-Harness Auto-Mode Engine

**Date**: 2026-10-11 (revised after the devil's-advocate review, §9)
**Status**: Positioning hypothesis, not a feature spec. It clears none of the
gates in [`validation-discipline.md`](../../.claude/rules/validation-discipline.md)
(see §7). Treat every "Caro would" below as a claim to test.
**Evidence**: [`2026-10-11-competitive-landscape-refresh.md`](../research/2026-10-11-competitive-landscape-refresh.md),
[`auto-modes.md`](../research/2026-10-11-competitive-refresh/auto-modes.md),
[`guardrail-tools.md`](../research/2026-10-11-competitive-refresh/guardrail-tools.md)
**Builds on**: PR #1513 (`caro guard`, spec 011, ADR-019, not merged yet)
and [`GUARDIAN_AGENT.md`](../GUARDIAN_AGENT.md)

## 1. The thesis

Every frontier agent now ships an "auto mode": something that decides
whether the agent's next shell command may run without asking you.

- **Claude Code**: an auto mode classifier.
- **Codex**: Auto-review, also called "guardian".
- **Cursor**: Auto-review.
- **VS Code**: Assisted permissions.
- **Grok Build** and **Qwen Code**: each has an Auto mode.

**The owner's thesis:** Caro becomes the alternative engine. It has these
properties:

- **Cross-agent, cross-harness, cross-model.**
- **Under the user's control.** The user can configure it and inspect its
  decisions.
- **Maintained over time by a community** of security researchers and
  terminal power users.

What the vendor modes do and do not give users today:

- **Configurable in prose.** Claude Code has `autoMode` settings and
  `claude auto-mode critique`. Codex's reviewer policy is open source and
  replaceable (`[auto_review].policy`).
- **Not portable.** A policy written for one harness does not run in
  another.
- **Not deterministic.** Each judge is an LLM on the vendor's model, and
  the vendor can change it server-side.
- **Tied to the vendor's model.** Claude Code auto mode needs a supported
  Claude model. Codex Auto-review breaks on custom providers. Cursor's
  classifier runs on Cursor's backend.
- **Sends data off the machine.** Each judge sends parts of the transcript
  to the vendor.

Codex also runs an OS sandbox and deterministic `execpolicy` rules by
default. The honest comparison is Caro against a whole stack, not against
the classifier alone.

## 2. What the vendors ship

| | Claude Code auto mode | Codex Auto-review | Cursor Auto-review | Caro (proposed) |
|---|---|---|---|---|
| Decides with | Rules, then an LLM classifier | Sandbox + execpolicy rules, then a reviewer agent | Allowlist, sandbox, then a classifier | Deterministic floor, then an optional local judge |
| Models | Claude only | Breaks on custom providers | Cursor backend | Any, or none |
| Open source | No | Yes (`core/src/guardian/policy.md`) | No | Yes (AGPL-3.0) |
| User-editable policy | Prose (`autoMode`) | Prose (`[auto_review].policy`) + Starlark rules | Prose (`permissions.json`) | Rules files |
| Same policy in other harnesses | No | No | No | Through hooks, if the hooks allow it (§4) |
| Sandbox | Optional, off by default | On by default | Yes | None (ADR-010 is only proposed) |

## 3. Where Caro would sit

```
 agent proposes a shell command
            │
            ▼
 harness PreToolUse hook ──► caro guard
            │                 ├─ 1. deterministic floor: critical → deny (no LLM, cannot be overridden)
            │                 ├─ 2. curated rules: high → ask
            │                 ├─ 3. routine rules: known-safe → no opinion today, "allow" only if §4 spike passes
            │                 └─ 4. optional local judge for the rest (opt-in, escalates to ask, never relaxes 1)
            ▼
 harness's own rules / auto mode (still runs)
            ▼
 OS sandbox (srt, Codex sandbox, Copilot MXC, nono…)  ← the real boundary
            ▼
 shell
```

This keeps the `GUARDIAN_AGENT.md` principle: the floor is not an LLM. A
judge may only escalate to "ask" (ADR-017). Caro adds to the vendor's mode
and the sandbox; it replaces neither. Every vendor says its classifier is
not a security boundary. Caro's regex floor is not one either.

## 4. Two roles, and what each needs

**Role 1: the guard (instrumentation first, not a market position).**

- It denies critical commands and asks on high-risk ones. It never says
  "allow". PR #1513 builds it, in shadow mode by default.
- As a product, it is a third deterministic regex guard, entering behind
  dcg and cc-safety-net. Its value today is **data**: the shadow log shows
  what Caro would deny or ask on real agent traffic.
- Copilot CLI, Cursor and VS Code read `.claude/settings.json` hooks, so
  the Claude adapter may also reach them. Test this before you claim it.

**Role 2: the approver for harnesses without a vendor judge.**

- Users who bring their own model get no Claude, Codex or Cursor judge.
  Other approaches exist:
  - goose Smart Approve uses the user's own provider.
  - Cline lets the model mark its own commands.
  - Dippy and safe-chains auto-approve routine commands with real
    parsers.
  - Sage, HOL Guard and Numbat gate across harnesses.
- The approver must emit "allow". Many hooks cannot do that:

| Harness (BYO-model capable) | Hook can "allow"? | Evidence |
|---|---|---|
| Codex on a custom provider | Yes | Verified; order versus Auto-review undocumented |
| Copilot CLI (BYOK) | Yes | Verified |
| Factory Droid | Yes (Claude shape) | Verified; BYOK unverified |
| OpenCode | Not useful: it allows all by default | Verified |
| Cline, Gemini CLI, Kiro, Windsurf | No (veto only) | Verified |
| Aider | Git hooks only | Verified |
| Crush, Mistral Vibe | Unknown | Not researched |
| Zed | Through ACP `session/request_permission`, not hooks | Secondary source |

- Role 2 also needs work that does not exist yet:
  - A real shell parser. Regex on raw text gets bypassed: GuardFall, Roo
    Code CVE-2026-82536/82537, and the 6 gaps that #1513 measured (#1054).
  - Flag-level knowledge of routine commands, as in safe-chains and Dippy.
  - A calibrated local judge that cannot relax the floor (ADR-017/018).

## 5. The community idea (aspiration, no evidence yet)

Caro has **no outside rule contributors today**. The CVE pipeline is
automated. The research found almost no community rule packs for agent
commands: 1 Codex execpolicy repo, with 1 star. The ideas below are
options, not plans:

1. **Tags that security people use**: MITRE ATT&CK IDs and the OWASP
   Agentic Top 10 (ASI02, ASI05).
2. **A benchmark built on corpora Caro did not write**:
   - GuardFall examples, Atomic Red Team commands, AgentDojo.
   - Publish the corpus and the harness, so others can reproduce the
     results.
   - Caro cannot cheaply run vendor judges on a stateless corpus.
     Single-command tests also favor stateless rules over intent-aware
     judges. Say both.
3. **Shadow logs stay local.** Users can share a redacted log by hand. No
   upload path, so "never leaves your machine" stays true.
4. **Export to native formats** (Codex `prefix_rule`, Gemini TOML, Claude
   `permissions`). The export loses detail, and it removes a reason to
   install Caro. Treat it as reach, not lock-in.
5. **Standards**: watch the AARTS draft (Gen Digital). Sage, its reference
   engine, is the closest rival with a large company behind it.

## 6. The case against

1. **The vendor judge is free, on by default, and sees intent.** Prompt
   injection and exfiltration need context. A stateless check of one
   command cannot see it.
2. **The sandbox is the boundary.**
   - In late Aug 2026, Johann Rehberger reported a prompt-injection chain
     that beat Claude auto mode in 60–80% of a small number of runs.
   - The source is secondary coverage, for example
     [BankInfoSecurity](https://www.bankinfosecurity.net/hidden-attack-slips-past-claude-code-auto-mode-a-32693).
     We did not read his post.
   - Anthropic called auto mode a best-effort convenience, not a security
     guarantee.
   - The fix everyone recommends is a sandbox and egress control.
3. **The deterministic niche is taken.** These tools are free and further
   ahead:
   - dcg: ~6.1k stars, 17+ harnesses.
   - cc-safety-net: MIT, ~1.6k stars, 14+ harnesses.
   - Sage + AARTS (Gen Digital), HOL Guard (16 harnesses), and Numbat
     (Perplexity).
4. **AGPL may slow adoption** as a hook in companies.
5. **Most hooks cannot fully replace native auto mode.** Many accept only
   vetoes. Claude Code applies its own deny/ask rules after a hook
   "allow", and nobody has tested whether "allow" skips its classifier.
6. **Yolo users chose yolo to avoid prompts.** "Ask on high" brings the
   prompts back and may get the guard uninstalled.

## 7. Validation status and what breaks at 100 users

None of the five gates is cleared:

- **Gate 1: zero transcripts** (`docs/discovery/transcripts/` is empty).
  The two roles have two cohorts:
  - **Role 1**: people who already run vendor auto modes, or pre-approved
    shell access, across at least two harnesses.
  - **Role 2**: people who bring their own or local models, in harnesses
    with no vendor judge. This cohort is undefined and unsized.
- **Selection bias.** Shadow mode attracts safety-minded users. It measures
  what Caro would block, not what it misses or whether anyone wants it.

What breaks at 100 real users:

- **Latency and memory:** a local judge per tool call, on machines that
  already run a local model.
- **Fail direction:** Grok Build and Defender fail open on timeout. An
  approver that fails open is worse than none.
- **Friction:** too many "ask" verdicts and users uninstall.
- **Rule churn:** CVE rules are compiled into the binary, so each rule
  update needs a release.
- **Hook drift:** each harness changes its hook semantics on its own
  schedule.

## 8. Cheap falsifiers (run these before building anything new)

1. **Allow-capability spike (1 day).** In each harness in §4, check that a
   hook "allow" is honored and removes a prompt. If it fails, Role 2's
   target list collapses.
2. **Delta test.** Replay a week of #1513 shadow traffic through dcg and
   cc-safety-net. If Caro's deny/ask set is not larger, Role 1 has no
   differentiation.
3. **Bypass test.** Run the GuardFall corpus and the Roo CVE class
   against today's validator.
4. **Claude test (1 hour).** Check whether a `PreToolUse` "allow" skips
   the auto-mode classifier.
5. **Eight interviews** with OpenCode, Cline or Crush users who bring
   their own model. If half already run dcg or a sandbox, Role 2 is dead.

## 9. Decisions for the owner

| # | Decision | Default |
|---|---|---|
| 1 | Warp messaging after Warp open-sourced its client | Lead with "fully open, fully local". This holds: the code is AGPL-3.0, and the default model (Qwen2.5-Coder 1.5B) is Apache-2.0. Keep "keep your terminal" as the supporting line. |
| 2 | Compete with dcg and cc-safety-net, or interoperate | **Undecided until the delta test (§8.2).** cc-safety-net (MIT), Dippy (MIT) and safe-chains (Apache-2.0) can be reused legally. dcg's license rider blocks reuse. |
| 3 | License rule packs separately (e.g. CC BY 4.0) | **Defer.** There are no contributors yet, and GPL-3.0 sources (GTFOBins) would conflict. This needs a human: it is a licensing decision. |
| 4 | What to build first | **Role 1 as instrumentation only** (#1513, shadow mode). Gate parser and judge work on the falsifiers in §8. |
| 5 | Recurring competitive refresh | Monthly re-check of `/compare` facts, quarterly discovery sweep. |

The devil's-advocate review that shaped this revision is posted on PR #1583.

## Open items

- `GUARDIAN_AGENT.md` says "62+" patterns; `CLAUDE.md` and the website
  say "52+". Count the patterns once and align both.
