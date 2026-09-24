# ADR-065: `caro.unattended.v1` — Classify Commands by What Happens If You Stop Them

- **Status**: Proposed (implementation ADR — meant to become code, and deliberately buildable on
  the tree that exists today, not on the paper stack)
- **Date**: 2026-09-04
- **Produced by**: `caro-research--scoping-process` scheduled task (autonomous run, no user present)
- **Feature researched**: **Claude Code background Bash tasks** — the Bash tool's
  `run_in_background` parameter, `Ctrl+B` backgrounding, the background task ID + output-file
  retrieval model, `/bashes`, `Ctrl+X Ctrl+K`, the documented auto-cleanup rules (session exit;
  5 GB output cap), and the `CLAUDE_CODE_DISABLE_BACKGROUND_TASKS=1` kill switch. Read
  2026-09-04 at [code.claude.com/docs/en/interactive-mode](https://code.claude.com/docs/en/interactive-mode),
  cross-read against a field guide that documents the same four rules
  ([backgroundclaude.com/blog/background-commands](https://backgroundclaude.com/blog/background-commands), 2026-04-14)
- **Failure-mode corpus** (six upstream issues, read 2026-09-04):
  [#25188](https://github.com/anthropics/claude-code/issues/25188) (session end / **compaction** /
  context exhaustion SIGTERMs every tracked task, daemons included — closed as duplicate, no fix
  referenced), [#84981](https://github.com/anthropics/claude-code/issues/84981) (undocumented
  ~1800 s SIGTERM, exit 144, **open**), [#58662](https://github.com/anthropics/claude-code/issues/58662)
  (tasks from closed sessions persist indefinitely and cannot be stopped),
  [#16198](https://github.com/anthropics/claude-code/issues/16198) (dev servers survive session
  transitions), [#42388](https://github.com/anthropics/claude-code/issues/42388) (output files in
  `/private/tmp` grow unbounded, no cleanup),
  [#67895](https://github.com/anthropics/claude-code/issues/67895) (completed tasks still render
  as Running)
- **Depends on**: nothing unlanded. `SafetyValidator`, `DangerPattern`, `RiskLevel`,
  `SuggestedRouting`, `SafetyDecision`, `clap`, `serde`, `schemars` and `assert_cmd` all ship in
  1.4.0. No daemon, no network, no new crate, no new module
- **Relates to**: ADR-050 (execution budget ledger — that ADR counts *cumulative* spend across
  invocations and had to relax the no-state constraint; this one is stateless and per-command, and
  the two do not overlap), ADR-031 (session circuit breaker — halts a *session*; this decides
  whether a single command may leave the session's supervision at all), ADR-039 (sandbox-aware
  verdict tier — *may this run inside a sandbox*; this is orthogonal: *may this run without a
  watcher*), ADR-033 (`caro undo` — the recovery path for the damage this ADR tries to prevent),
  ADR-007 (shell AST — deliberately **not** depended on; see D6)
- **Does not depend on**: ADR-024 (headless envelope / exit-code enum), ADR-058/059
  (`caro.assessment.v1`), ADR-060 (`caro.eval.v1`), ADR-063 (`caro.bench.v1`), ADR-064
  (`caro.egress.v1`). All five are paper as of this date; `src/safety/` still contains only
  `mod.rs`, `patterns.rs` and `cve_patterns.rs`
- **Full scope document**: `caro-scope-unattended-execution-2026-09-04.md`
- **Numbering note**: highest existing is ADR-064. Per `.claude/rules/adr-numbering.md`, renumber
  on merge if another 065 lands first

---

> **Provenance note (autonomous run).** The task template left `[FEATURE NAME]` unbound and ran
> with no user present, so target selection was mine. Rationale: the 2026-09-03 Hermes scan (§2C)
> records buyers articulating *action-level* tiering — "watching doesn't scale — I watch the first
> ten minutes, and the thing it does at minute forty is the one that mattered." Backgrounding is
> the mechanism by which minute forty arrives unwatched, it is shipped and widely used in the host
> Caro most wants to sit in front of, and its lifecycle has six open or duplicate-closed issues.
> No existing ADR covers it: grep across `docs/adr/` finds "background" only as prose in the
> headless/streaming ADRs, and no ADR reasons about interruption at all.
>
> **Moratorium note.** ADR-059 declared itself "the last ADR in this space until
> `src/safety/assessment.rs` merges." Re-verified 2026-09-04: it has not merged
> (`src/assessment/` is the *hardware* recommender — `cpu.rs`, `gpu.rs`, `memory.rs`,
> `recommender.rs` — not the safety payload). This ADR mints no policy vocabulary, no approval
> exchange, no lifecycle event, and no new tier. It does mint one report struct, and §D5 commits
> that struct to nesting inside `caro.assessment.v1` as a single optional field when that lands,
> rather than competing with it. Treat that as the reviewable seam.
>
> **Validation-discipline note.** This is an architecture scope, not a feature spec. It makes no
> PMF claim and asserts no user demand beyond the six cited issues. Gate 3 ("what breaks at 100
> real users") is answered in scope §6. Gates 1, 2, 4 and 5 attach to the implementation PR.

---

## Context

### The problem, in one sentence

Caro answers *is this command dangerous to run*; nothing in Caro or in the hosts it integrates with
answers *is this command dangerous to **stop***, and backgrounding is precisely the act of handing a
command to a supervisor that will stop it on a schedule it knows nothing about.

### Phase 1 — how the reference implementation works

Claude Code's Bash tool takes `run_in_background: true` (also reachable by pressing `Ctrl+B` on an
already-running invocation, or `!cmd` then `Ctrl+B`). The call returns immediately with a unique
background task ID. `stdout` and `stderr` are redirected to a file, which the agent reads on demand
via the output-retrieval tool — so a chatty process never floods the model's context. `/bashes`
lists and manages tasks interactively. `Ctrl+X Ctrl+K` twice within three seconds kills every task
in the session. `CLAUDE_CODE_DISABLE_BACKGROUND_TASKS=1` disables the whole feature, which the field
guide recommends for CI.

The lifecycle is governed by three documented rules:

1. **Output cap.** A task emitting more than 5 GB is auto-terminated with a note in stderr.
2. **Session exit.** All of a session's background tasks are cleaned up when the session ends.
3. **Manual kill.** The double-press shortcut stops everything at once.

The architecture is clean and the separation of concerns is right: the host owns the process table,
the model owns the decision to read output, the filesystem owns the buffer. The gap is not in the
mechanism. It is that **rule 2 is keyed on session membership and nothing else.**

### Why it is limited — the documented failure modes

The six issues fall into two symmetric halves, which is what makes this an architectural gap rather
than a bug list.

**It kills what must live.** #25188 reports that session end, *context compaction*, and context
exhaustion all SIGTERM every tracked task — including a Telegram bot, a supervisor daemon and three
workers started via `nohup … &`. The reporter's words: "session cleanup kills all three — taking down
my entire system silently." There is no `persistent:` or `detach:` flag; the issue proposes four and
was closed as a duplicate. #84981 (open) reports SIGTERM at almost exactly 1800 s on macOS with exit
code 144 (`128 + 16`), a lifetime that appears in no documentation.

**It keeps alive what must die.** #58662 reports tasks from closed sessions persisting indefinitely
and refusing to stop. #16198 reports dev servers surviving session transitions. #42388 reports the
output files themselves accumulating in `/private/tmp` with no cleanup.

**And the state it reports is wrong in between.** #67895 reports completed tasks rendering as
Running, with dismissals that do not persist.

### The structural cause

`sleep 3600`, `rsync -a --delete src/ dst/`, and `node dist/daemon/supervisor.js` are the same object
to the cleanup path. It has a task ID, a session, and a process handle. It has no idea which of the
three it is holding. So it applies one policy — *stop it when the session boundary arrives* — and that
policy is correct for the first, catastrophic for the third, and silently corrupting for the second:
an `rsync --delete` SIGTERM'd at 1800 s has performed some of its deletions and none of its copies,
which is a state neither the before nor the after.

The contract makes this unrecoverable rather than merely wrong. The harness classifies a terminated
task as failed and surfaces exit 144. **Nothing in the payload distinguishes "the command finished
badly" from "we stopped it."** A consumer cannot tell a failed build from a half-completed
destructive sync, and therefore cannot know whether a retry is safe.

### Session/context lifecycle

Background tasks are session-scoped and do not survive a restart; a fresh process starts clean, so
there is no redundant-initialization problem to solve. The lifecycle defect is the inverse: a
*context* event (compaction) is treated as a *process* boundary. Context management and process
management are coupled through a shared session identity that neither of them chose.

### What Caro already has, and what it does not

Present in 1.4.0: `SafetyValidator` and the 52+ `DANGEROUS_PATTERNS`, `RiskLevel`
(`Safe|Moderate|High|Critical`), `SuggestedRouting` (`AutoApprove|AsyncLog|HumanGate|Block`) with a
canonical `from_risk_and_safety` mapping, `SafetyDecision` (serialized, agent-facing),
`CommandExecutor::with_timeout` (a wall-clock ceiling on *foreground* execution), `schemars` for
schema emission, `assert_cmd` for CLI integration tests.

Absent: any notion of duration, of interruption, of a process outliving the decision that authorized
it. `SuggestedRouting::AutoApprove` currently means "safe to run," and every consumer reads it as
"safe to run *now, while someone is looking*." Backgrounding silently drops the second clause.

One finding worth recording for whoever implements: **`DangerPattern` has no stable identifier** —
it is `{ pattern, risk_level, description, shell_specific }`, and `ValidationResult.matched_patterns`
carries description strings. Any feature that wants to key behaviour off a specific pattern cannot,
today. This ADR is designed not to need one (D4).

---

## Decision

Add a stateless subcommand, **`caro unattended`**, that takes a command string and answers a question
Caro does not currently ask: *given that this command is going to run without a watcher, what happens
if the host stops it, and where should it run?* It emits one serialized report and one exit code, and
it never spawns, supervises, tracks or persists anything.

### D1 — Interruption is a separate axis from danger

Introduce `InterruptClass`, orthogonal to `RiskLevel`:

| Class | Meaning | Example |
|---|---|---|
| `Idempotent` | Re-running from scratch is equivalent; interruption costs time only | `cargo build`, `npm test` |
| `Resumable` | Partial state is a valid resume point | `rsync -a` (no `--delete`), `wget -c`, `curl -C -` |
| `PartialDestructive` | Interruption leaves the target in a state that is neither the before nor the after | `rsync --delete`, `dd`, `mkfs`, `tar -x` over a live tree, `find -delete` |
| `Custodial` | The process *is* the service; stopping it removes a capability something else depends on | daemons, supervisors, port and lock holders, `node …/supervisor.js` |
| `Indeterminate` | Not decidable from the command text | anything composed, or an unrecognised head |

The axes are genuinely independent, which is the argument for a second one. `sleep 3600` is
`RiskLevel::Safe` and `InterruptClass::Idempotent`. `node supervisor.js` is `Safe` and `Custodial`.
`rsync --delete` is `Moderate` and `PartialDestructive`. A single scalar cannot express that the
third is the one you must not background under a 30-minute timer.

### D2 — Caro emits the supervision contract; Caro never supervises

`caro unattended` is one process in, one JSON document out, one exit code, then exit. It does not
fork, does not write a ledger, does not open a socket, does not remember the previous invocation. The
report *describes* the supervision the command requires — teardown signal, grace period, output
volume class, duration class, whether it must survive session cleanup — and the host performs it.

This is the constraint the task imposes, and it is also the better design: tmux, `systemd`, `launchd`
and Claude Code's own task table are all better supervisors than anything Caro could ship, and each
of them already exists on the machine.

### D3 — The fix for #25188 is placement, decided before spawn

Caro cannot add a `persistent: true` flag to somebody else's Bash tool. It can do something strictly
better: refuse to let the class of command that cannot survive cleanup enter the cleanup-eligible set
in the first place. The report carries a `Placement` recommendation:

- `Foreground` — do not background this; the interruption risk is only acceptable while watched.
- `HostBackground` — the host's own background task table is appropriate.
- `ExternalSupervisor` — this must be started outside the session, under `tmux`/`systemd`/`launchd`,
  because session cleanup would destroy something the session does not own.

`Custodial` maps to `ExternalSupervisor`. `PartialDestructive` maps to `Foreground` under strict and
moderate safety levels. This is the aident.ai remediation — "move jobs that must outlive the session
outside the session" — converted from a post-mortem instruction into a pre-spawn decision, which is
the difference the task asked for: solve the failure mode by design, not by workaround.

### D4 — `Indeterminate` is a value, never a coercion

An unrecognised command head, or any command containing a control operator, yields
`InterruptClass::Indeterminate` carrying a machine-readable `reason`. It is never silently promoted
to `Idempotent` (which would be fail-open) and never silently demoted to `PartialDestructive` (which
would make the tool useless on the long tail). The *routing* is where safety level applies:
`Indeterminate` routes to `HumanGate` under `Strict`, `AsyncLog` under `Moderate`, `AutoApprove`
under `Permissive` — the same shape as the existing `from_risk_and_safety` table, so no new policy
vocabulary is minted.

This also means the classifier does not need stable pattern IDs from `patterns.rs`. It carries its
own small head-keyed table and answers `Indeterminate` for everything else, which is exactly what the
absence of IDs forces and what honesty requires anyway.

### D5 — Thresholds are classes, not numbers

The report emits `OutputVolume::{Bounded, Growing, Unbounded}` and
`DurationClass::{Seconds, Minutes, Unbounded}`. It does **not** emit "5 GB" or "1800 s". One of those
is a documented host constant that may change; the other is an open bug. Hard-coding either would
make Caro's output wrong the day the host changes, and would encode a defect as a specification. The
host compares Caro's class against its own limits and decides.

`UnattendedReport` is shaped to nest: when `caro.assessment.v1` (ADR-058/059) lands, it becomes one
optional field on `Assessment`, not a competing top-level payload.

### D6 — No shell AST in v1

Any command containing `|`, `&&`, `||`, `;`, `&`, command substitution, or an `sh -c` wrapper returns
`Indeterminate { reason: Composed }`. Implementing composition analysis means either depending on
ADR-007 (unlanded) or hand-rolling a parser inside a safety-critical path, and a parser that is
subtly wrong about `&&` is worse than an honest refusal to answer. The refusal is measurable — see
scope §6 — so the v2 case can be made from data.

### Exit-code contract

Three codes, none of them invented here:

| Code | Meaning |
|---|---|
| `0` | Advisory produced; the requested placement is acceptable |
| `2` | Advisory produced; the requested placement is refused |
| `1` | Caro failed (bad usage, unreadable input) — **no verdict was reached** |

Exit `2` is inherited, not minted: Claude Code's own `PreToolUse` hook contract already reads exit 2
as *block*, so a hook wrapping `caro unattended` needs no translation layer. Exit `1` is Caro's
existing error convention and must never be read as approval. `EXIT_CODE_EDIT = 201` is untouched.

---

## Consequences

### Positive

- **The silent-kill class of failure becomes preventable rather than diagnosable.** #25188's daemons
  never enter the tracked set, so compaction cannot reach them.
- **`AutoApprove` stops quietly meaning two different things.** A consumer can now distinguish
  "safe to run" from "safe to run unwatched," which is the distinction backgrounding erases.
- **Buildable today.** No unlanded dependency, no new crate, no new module, no daemon. This is the
  second consecutive ADR (with 064) that can be implemented against the shipped tree, which is the
  direct answer to the memo's standing risk: 24 governance ADRs, zero shipped modules.
- **Positioning that no one in the 2026-09-03 cohort occupies.** Traccia counts tool calls; Skydive
  allowlists domains; Maritime isolates VMs at a dollar a month. None of them can say whether the
  process inside the box is safe to stop. Command semantics remain the empty row.
- **Offline and universal.** The classifier is a static table over command heads. No model, no
  network, no host API — it works for `tmux`, `systemd`, CI runners and any agent that can spawn a
  subprocess, not just for Claude Code.

### Negative

- **A second axis is a second thing to get wrong.** Two classifications can disagree in ways users
  find confusing (`Safe` + `ExternalSupervisor` reads oddly until explained). Mitigation: the report
  carries a one-line human `rationale`, and the docs lead with the `sleep`/`supervisor` contrast.
- **Head-keyed classification has a long tail.** Wrappers (`make`, `just`, `npm run`, `docker
  compose up`) hide the real command. v1 answers `Indeterminate { reason: OpaqueWrapper }` for the
  known wrapper set, which is correct but unhelpful, and the tail is where real usage lives.
- **`Custodial` detection is heuristic.** Nothing in `node server.js` says "daemon." v1 keys on a
  small explicit list plus long-lived flags (`--watch`, `serve`, `-d`, `daemon`), and will miss
  bespoke supervisors. It fails toward `Indeterminate`, not toward `Idempotent`.
- **One more payload before the assessment payload merges.** Mitigated by D5's nesting commitment,
  not eliminated.

### Neutral

- No change to any existing verdict. `caro <prompt>` output is byte-identical; this is additive.
- No new dependency, no MSRV movement, no feature flag. Ships in the default binary.
- Hosts that ignore the advisory are exactly as safe as they are today.

---

## Alternatives considered

**A1 — Add `--background` to the existing validation path instead of a new verb.**
Rejected. It would overload one verdict with two questions whose answers can point opposite ways
(`Safe` + "do not background"), and every existing consumer of `SafetyDecision` would silently
inherit a field it does not understand. A separate verb keeps the existing contract frozen.

**A2 — Wait for `caro.assessment.v1` and ship this as a field on it.**
Rejected on timing, accepted on shape. Five ADRs are already blocked on a type that has not merged
in three months; adding a sixth converts a scoping exercise into a queue. D5's nesting commitment
gets the architectural benefit without the dependency.

**A3 — Have Caro supervise the process itself (`caro run --supervise`).**
Rejected twice over. It violates the no-daemon constraint outright, and it would reimplement, worse,
the four supervisors already installed on every machine this runs on. Caro's leverage is that it can
answer a question about a command; it has no leverage on holding a process handle.

**A4 — Put interrupt classification into `patterns.rs` as a field on `DangerPattern`.**
Rejected. `patterns.rs` is a danger table: 52+ entries, of which perhaps six have anything to say
about interruption. Adding a field that is `None` for 90% of rows makes the safety-critical table
harder to audit, and `DangerPattern` has no stable ID to key the other direction either. A separate,
smaller, differently-keyed table is honest about being a different thing.

**A5 — Emit the host's actual limits (5 GB, 1800 s) in the report.**
Rejected. 1800 s is an open bug, not a specification; encoding it would make Caro's output a
monument to someone else's defect, and wrong the day it is fixed. Classes survive the fix.

---

## References

- Claude Code docs — [interactive mode](https://code.claude.com/docs/en/interactive-mode) (background
  Bash commands, `Ctrl+B`, `/bashes`, `Ctrl+X Ctrl+K`, cleanup rules), read 2026-09-04
- Field guide — [backgroundclaude.com/blog/background-commands](https://backgroundclaude.com/blog/background-commands),
  2026-04-14 (independent confirmation of the four rules and the `CLAUDE_CODE_DISABLE_BACKGROUND_TASKS`
  switch)
- Upstream issues [#25188](https://github.com/anthropics/claude-code/issues/25188),
  [#84981](https://github.com/anthropics/claude-code/issues/84981),
  [#58662](https://github.com/anthropics/claude-code/issues/58662),
  [#16198](https://github.com/anthropics/claude-code/issues/16198),
  [#42388](https://github.com/anthropics/claude-code/issues/42388),
  [#67895](https://github.com/anthropics/claude-code/issues/67895)
- Post-mortem and remediation pattern — [aident.ai, *Claude Code Background Task Dies at 30 Minutes?*](https://aident.ai/blog/fix-claude-code-background-task-sigterm-30-minutes),
  2026-08-10
- `.hermes/digests/2026-09-03-agent-market-scan.md` §2C, §2D, §2G
- `.claude/rules/` — `constitution.md`, `validation-discipline.md`, `adr-numbering.md`,
  `good-boy-scout.md`, `git-workflow.md`
- Caro tree at 1.4.0, branch `integrator/20260711-postmerge` (line numbers verified 2026-09-04 and
  expected to drift): `src/models/mod.rs:152,189,198` · `src/safety/mod.rs:175,188` ·
  `src/safety/patterns.rs:12` · `src/execution/executor.rs:38,51` · `src/main.rs:381,938` ·
  `Cargo.toml:36,42,46,148`
- Full scope: `caro-scope-unattended-execution-2026-09-04.md`
