# caro-research--scoping-process — Run Report

**Date**: 2026-09-18
**Agent**: Automated scheduled task (`caro-research--scoping-process`), autonomous, no user present
**Status**: ✅ Completed — ADR-074 + scope document produced
**Headline**: the run followed the project's own strategic signal instead of picking freely — and
that same signal says **stop writing ADRs and ship one**. Recommendation at the bottom: the next
run of this task should be a code run.

---

## What happened this run

The task's `SKILL.md` still carries the unfilled `[FEATURE NAME]` placeholder (outstanding since
2026-06-02).

1. **Duplication sweep** over `docs/adr/ADR-001…073` (73 files; the numbered `001-008` set as
   well). Confirmed ADR-074 is the next free number per
   [`.claude/rules/adr-numbering.md`](../../.claude/rules/adr-numbering.md).
2. **Selection.** Rather than choosing freely, read `.hermes/digests/2026-09-18-agent-market-scan.md`
   (written 02:12 today). Its §4 **"Build or test next — one thing"** names *"a durable
   pending-decision record (3.3), scoped as a data contract, not a feature"*, and §3.3's next step
   is literally *"scope an ADR for a durable pending-decision record — `assessment` payload +
   origin + expiry + resolution — written so that Pizza Bot's Action queue and a Claude Code
   `PreToolUse` hook are both valid front ends. Do **not** build a UI."* That is this task's exact
   deliverable, so it was taken as the target.
3. **Gap check against existing ADRs.** ADR-046 (AEP v1) defines the approval *payload* and its own
   out-of-scope list ends with *"persistence or replay of pending requests"*. ADR-065 classifies
   interruption of a *running* command. ADR-020 routes to `HumanGate` and stops. Nothing in the tree
   says what `HumanGate` means when the human is asleep. Clean gap, named by a prior ADR itself.
4. **Phase 1 research** (all read live, 2026-09-18):
   - **AWS Pizza Bot** — [blog](https://aws.amazon.com/blogs/opensource/introducing-pizza-bot-an-open-source-inbox-for-ai-agents-that-work-in-the-background/)
     (2026-09-10, updated 09-14), Apache-2.0. Action queue, `interruptOn` / `allowedDecisions`,
     durable pause via LangGraph checkpoints on SQLite, `edit` as a first-class human decision.
   - **Claude Code `defer`** — [#41791](https://github.com/anthropics/claude-code/issues/41791):
     `permissionDecision: "defer"` shipped in v2.1.89 and is documented in one changelog line.
   - **The failure mode** — [#89561](https://github.com/anthropics/claude-code/issues/89561):
     `"ask"` has *no blocking effect* in `permission_mode: "auto"`; it *"silently resolves to an
     implicit allow"*. Plus [#39344](https://github.com/anthropics/claude-code/issues/39344):
     `"ask"` overriding `permissions.deny`.
5. **Phase 2 / 3 output** — [`docs/adr/ADR-074-pending-decision-record.md`](../adr/ADR-074-pending-decision-record.md)
   and [`caro-scope-pending-decision-2026-09-18.md`](../../caro-scope-pending-decision-2026-09-18.md):
   `caro.pending.v1` — 2 verbs (4 subcommands), 12 new serializable types, **1 new file in `src/`**,
   6 files touched, no new dependency, no new config key, one new exit code (4), 22 integration
   tests, 7 alternatives considered, 10 out-of-scope items.

**Design answers to the researched defects**, in one line each: deferral is a constructor that only
accepts `HumanGate` (#39344); minting exits 4 and executes nothing, so missing input is never input
(#89561); `Edited` is a distinct outcome that re-assesses instead of inheriting a verdict (Pizza
Bot's `edit`); re-assessment at resolution is mandatory, not a flag (#41791's inferred semantics,
made explicit).

---

## The constraint this scope bends, stated openly

The template requires *"a pure subprocess call (no daemon, no state)."* A durable record is, by
definition, state. The reconciliation (ADR-074 D4): the **process** keeps nothing; the **store** is
a directory the caller names, with no default path and no fallback, holding plain append-only JSON.
Caro never reads or writes a path it was not handed, and never accumulates anything on its own.

A reviewer who thinks that is cheating should reject the ADR on that ground specifically. It is
flagged as open question #1 in the scope document because an autonomous run should not settle it.

---

## Deferred candidate, with its evidence preserved

Before the digest was read, this run had researched and nearly scoped a different target:
**per-command environment-exposure analysis**. It is a genuine uncovered axis (nothing in
ADR-001…073 models which environment variables a command hands to which child), and the research is
already done, so it is recorded here rather than thrown away.

**The gap.** Both hosts decide what counts as a secret *ahead of time, by name*:

- **Codex CLI** `[shell_environment_policy]` — `inherit = none|core|all`, `set`, `exclude`,
  `include_only`, `ignore_default_excludes`; the default excludes filter any variable whose name
  contains **KEY, SECRET or TOKEN** (case-insensitive), and *includes do not restore a variable
  already excluded* ([config-advanced](https://developers.openai.com/codex/config-advanced)).
- **Claude Code** `sandbox.credentials.{files,envVars}` with `deny` / `mask` modes, plus
  `CLAUDE_CODE_SUBPROCESS_ENV_SCRUB` for all subprocesses. The docs state outright:
  *"There is no built-in credential deny list, so only the files and variables you list are
  restricted"* and *"Sandboxed Bash commands inherit the parent process environment by default,
  including any credentials set there"*
  ([sandboxing](https://code.claude.com/docs/en/sandboxing)).

**Why the name-based approach fails, with field evidence.** `PGPASSWORD`, `DATABASE_URL` and
`COPILOT_JOB_NONCE` contain none of KEY/SECRET/TOKEN. The CSA research note of 2026-06-05
([Lab Space](https://labs.cloudsecurityalliance.org/research/csa-research-note-claude-code-github-action-prompt-injection/))
documents `COPILOT_JOB_NONCE` among the credentials actually stolen, and two mechanisms no
name-filter sees: reading `/proc/self/environ` with an allowed `cat`, and a Copilot filtering
bypass that scrubbed *child* bash subprocesses while *"leaving the parent Node.js process readable
via `ps auxeww`"*. The remediation list for CVE-2025-66032 includes *"scrubbed environment
variables from child processes spawned by Claude Code"* — the same one-sided fix.

**The shape of the Caro feature**, if a future run picks it up: `caro.envscope.v1`, a deterministic
offline classifier over *two* finding classes — (a) **aperture** findings decidable from the command
text with no inventory at all (`env`, `printenv`, `cat /proc/*/environ`, `ps auxeww`, `sudo -E`,
`docker run --env-file`, `ssh -o SendEnv`, `make -e`), and (b) **named** findings judged against a
caller-supplied inventory of variable *names only, never values*, with `Unknown` sensitivity as a
value rather than a silence. `sudo` without `-E` is an env *reducer* and is modelled as one, with
the `env_reset` assumption declared in the payload rather than assumed. Complement, not duplicate,
of `src/caroml/validators/secrets.rs`, which finds credential *literals in* the command; this finds
credentials the command *moves without naming*.

Boundary against existing ADRs: **ADR-064** (`caro.egress.v1`) requires both halves of a
source→sink pair to be textually populated, which structurally cannot represent an implicitly
inherited variable; **ADR-068** classifies output content; **ADR-069** owns the
`ANTHROPIC_BASE_URL`-style "env as control plane" case.

---

## Boy-scout findings (not fixed this run — no branch was created)

1. **`CLAUDE.md` states MSRV 1.83; `Cargo.toml:5` says `rust-version = "1.85"`**, and the
   `criterion` pin comment at `Cargo.toml:~155` also says "above MSRV 1.85". `CLAUDE.md` is stale.
2. **ADR index staleness persists** — `docs/adr/README.md` is current through ADR-015 and carries a
   note saying so. 58 rows behind as of today. Not backfilled here, consistent with prior runs'
   stated reasoning.
3. **ADR-072's citations remain unverified** — ADR-073 recorded on 2026-09-17 that it could locate
   neither the Black Hat item nor the vendor "Novee" for CVE-2026-12537 / CVE-2026-54316. Still
   outstanding; ADR-072 should not leave Proposed until resolved.
4. **The repository root now holds 13 `caro-scope-*.md` files.** Each scoping run adds one. They
   would read better under `docs/scopes/`; moving them is a one-line `git mv` per file plus the
   back-links in each ADR's "Full scope document" line.

---

## Recommendation to the next run

**Do not scope anything.** Today's digest §3.4 and §4 both say the tree is accumulating paper
faster than validation: ADR-069 and ADR-073 are Proposed with zero lines of code behind them, and
ADR-074 now makes three. The digest's own instruction is to *"pick ADR-069 (it states its only
prerequisite is a two-line derive fix) and cut the smallest slice that reads one ambient source and
restricts — never widens — the decision."*

ADR-074 was written to be the smallest possible increment precisely so that it can be implemented in
one sitting if it is chosen instead — but it should be chosen *instead of*, not *in addition to*.

Concretely, for the next run:

1. Pick **one** of ADR-069 / ADR-073 / ADR-074.
2. Create a feature branch per [`.claude/rules/git-workflow.md`](../../.claude/rules/git-workflow.md)
   — this run created none and committed nothing.
3. Land the two-line `JsonSchema` derive fix first, as its own commit; every one of the three ADRs
   names it as a precondition.
4. Address **Gate 1** (20 transcripts) or cut the slice as a refactor of existing validation output,
   per [`.claude/rules/validation-discipline.md`](../../.claude/rules/validation-discipline.md).

---

## Artifacts produced

| Path | What |
|---|---|
| `docs/adr/ADR-074-pending-decision-record.md` | the ADR — context, 8 decisions, consequences, 7 alternatives, exit-code contract, 14 headline tests, gate status |
| `caro-scope-pending-decision-2026-09-18.md` | full type definitions, method contracts, file-change table, CLI surface, worked example, 22 tests, Gate-3 section, 4 open questions |
| `docs/research/caro-research-scoping-2026-09-18.md` | this report |

Nothing was branched, committed, pushed, or implemented. No external service was written to.

## Sources

- [AWS — *Introducing Pizza Bot, an open source inbox for AI agents that work in the background*](https://aws.amazon.com/blogs/opensource/introducing-pizza-bot-an-open-source-inbox-for-ai-agents-that-work-in-the-background/)
- [anthropics/claude-code #41791 — docs omit `defer` `PreToolUse` decision for headless `--resume`](https://github.com/anthropics/claude-code/issues/41791)
- [anthropics/claude-code #89561 — `"ask"` has no blocking effect in `permission_mode: "auto"`](https://github.com/anthropics/claude-code/issues/89561)
- [anthropics/claude-code #39344 — `"ask"` silently overrides `permissions.deny`](https://github.com/anthropics/claude-code/issues/39344)
- [Claude Code — Configure the sandboxed Bash tool](https://code.claude.com/docs/en/sandboxing)
- [OpenAI Codex — Advanced configuration (`shell_environment_policy`)](https://developers.openai.com/codex/config-advanced)
- [openai/codex #22023 — env var allowlist for `inherit = "core" | "none"`](https://github.com/openai/codex/issues/22023)
- [CSA Lab Space — *AI Agent Prompt Injection: The New CI/CD Supply Chain Threat*](https://labs.cloudsecurityalliance.org/research/csa-research-note-claude-code-github-action-prompt-injection/)
