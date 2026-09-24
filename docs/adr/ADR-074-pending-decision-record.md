# ADR-074: `caro.pending.v1` — What "Ask" Means When Nobody Is There

- **Status**: **Proposed (implementation ADR — buildable on the tree that ships in 1.4.0; one
  two-line derive fix is its only precondition).** Gate 1 unmet, Gate 4 pending. See
  *Validation discipline*.
- **Date**: 2026-09-18
- **Produced by**: `caro-research--scoping-process` scheduled task (autonomous run, no user present)
- **Feature researched**: **deferred approval for unattended agent sessions** — two independent
  implementations of the same idea, shipped five days apart. All sources read 2026-09-18:
  - [AWS, *Introducing Pizza Bot*](https://aws.amazon.com/blogs/opensource/introducing-pizza-bot-an-open-source-inbox-for-ai-agents-that-work-in-the-background/)
    (published 2026-09-10, updated 2026-09-14; Apache-2.0, [repo](https://github.com/pizza-bot-app/pizza-bot)) —
    the **Action queue**, `interruptOn` / `allowedDecisions` per-tool approval policy, durable pause
    via LangGraph checkpointing onto SQLite
  - [`anthropics/claude-code` #41791](https://github.com/anthropics/claude-code/issues/41791) —
    `PreToolUse` `permissionDecision: "defer"`, added in changelog **v2.1.89**, documented nowhere
    but the changelog: *"headless sessions can pause at a tool call and resume with `-p --resume`
    to have the hook re-evaluate"*
  - [`anthropics/claude-code` #89561](https://github.com/anthropics/claude-code/issues/89561) —
    `permissionDecision: "ask"` **has no blocking effect** in `permission_mode: "auto"`; the tool
    call proceeds as if the hook had returned `allow`
  - [`anthropics/claude-code` #39344](https://github.com/anthropics/claude-code/issues/39344) —
    a `PreToolUse` `"ask"` decision silently overriding `permissions.deny` rules
  - [`code.claude.com/docs/en/hooks`](https://code.claude.com/docs/en/hooks) and
    [`/hooks-guide`](https://code.claude.com/docs/en/hooks-guide) — the published three-outcome
    `PreToolUse` contract (`allow` / `deny` / `ask`) that #41791 shows is now four
- **Depends on**: one two-line derive addition (D0). No new module directory, no new dependency,
  no new configuration key, no daemon.
- **Relates to**: **ADR-046** (AEP v1 — the request/response payload; its own out-of-scope list
  names *"persistence or replay of pending requests"*, which is exactly this ADR), **ADR-065**
  (`caro.unattended.v1` — classifies what happens if you stop a command; this ADR is what happens
  if nobody starts one), **ADR-020** (tiered approval — `HumanGate` is the only routing this ADR
  can act on), **ADR-045** (strong-auth proof — carried opaquely here, verified there),
  **ADR-032** (receipts — the resolution pair is shaped to hash-chain into one later),
  **ADR-058/059** (`caro.assessment.v1` — the payload this record will embed once that exists in
  code), **ADR-031** (circuit breaker — halts a session; this defers one decision inside it)
- **Full scope document**: [`caro-scope-pending-decision-2026-09-18.md`](../../caro-scope-pending-decision-2026-09-18.md)

> **Provenance note (autonomous run).** The scheduled task's template still leaves `[FEATURE NAME]`
> unbound (outstanding since 2026-06-02) and ran with no user present, so target selection is the
> agent's own. This run did **not** select freely: `.hermes/digests/2026-09-18-agent-market-scan.md`
> §3.3 and §4 name *"a durable pending-decision record, scoped as a data contract, not a feature"*
> as the week's single **build or test next** item, with the explicit next step *"scope an ADR …
> written so that Pizza Bot's Action queue and a Claude Code `PreToolUse` hook are both valid front
> ends. Do **not** build a UI."* This ADR is that scope, and it holds to both constraints.
>
> **Counter-signal recorded, not suppressed.** The same digest's §3.4 and §4 warn that *"the memo
> series is accumulating paper faster than the tree is accumulating validation"* and rank
> *implementing* ADR-069 or ADR-073 above writing anything new. This document is more paper. It is
> written to be the smallest possible increment (one new file in `src/`, ~300 LOC, no new
> dependency) and the run report recommends that the **next** run of this task be a code run, not a
> scoping run. See `docs/research/caro-research-scoping-2026-09-18.md` §"Recommendation to the next
> run".
>
> **Second candidate, deferred with its evidence.** Before the digest was read, this run had
> scoped an alternative target — per-command **environment-exposure** analysis, the gap left by
> Codex CLI's `shell_environment_policy` KEY/SECRET/TOKEN substring filter and Claude Code's
> `sandbox.credentials`, which documents *"There is no built-in credential deny list, so only the
> files and variables you list are restricted."* The evidence gathered for it is preserved in the
> run report rather than discarded, because it is a genuine uncovered axis and the research is
> already done.

---

## Context

### 1. The question two vendors answered in the same week

An agent is given a job that takes hours. Partway through, it proposes something consequential.
The validator says *ask a human*. There is no human.

Every tiered-approval design in this repository — ADR-020's routing table, ADR-040's policy file,
ADR-045's second factor, ADR-046's exchange payload — assumes the `HumanGate` branch terminates at
a person who is present. `SuggestedRouting::HumanGate` (`src/models/mod.rs:189-220`) is a
synchronous instruction: *pause for human approval before execution*. Pause where? Until when? The
type does not say, and neither does anything downstream of it.

Between 2026-09-10 and 2026-09-15 two projects shipped answers:

**Pizza Bot** (AWS, Apache-2.0) makes the pause a first-class inbox state. Finished work lands in
**Unread**; *"anything waiting on your decision lands in **Action**."* The pause is durable because
DeepAgents/LangGraph *"checkpoints a run as it proceeds, so the messages, the tool activity, and a
pause waiting on your approval are on disk rather than in memory"*, in SQLite inside a folder the
user owns. A skill declares `interruptOn` per tool, and `allowedDecisions` sets which buttons the
human gets — including `edit`, which the blog singles out *"because it lets you correct what the
agent proposed rather than rejecting it and starting the run over."*

**Claude Code** makes the pause a fourth hook decision. Changelog v2.1.89 added
`permissionDecision: "defer"`: *"headless sessions can pause at a tool call and resume with
`-p --resume` to have the hook re-evaluate."* The hook process exits; the tool call is preserved;
a later resume re-runs the same hook against whatever the world looks like then.

These are the two front ends Hermes §3.3 asks this contract to serve. They are architecturally
opposite — a stateful server with a queue, versus a stateless process that exits and is re-entered
— and the record defined below has to be readable by both without becoming either.

### 2. The failure mode, stated as the issue tracker states it

The reason to design this rather than adopt it is `claude-code` **#89561**, filed against the
shipped `ask` path:

> *"A `PreToolUse` hook that returns `hookSpecificOutput.permissionDecision: "ask"` has no blocking
> effect when the session's permission mode is `auto` … the tool call proceeds immediately, as if
> the hook had returned `"allow"`. This contradicts the documented behavior."*

and the reporter's own diagnosis of why:

> *"Since there is no interactive surface to route an 'ask' decision to in `auto` mode, it appears
> to silently resolve to an implicit allow rather than a safe default."*

That is the whole problem in one sentence: **the absence of a reviewer is read as consent.** The
issue's requested fix — *"Make `"ask"` fail closed (behave as `"deny"`) when no interactive prompt
surface is available"* — is the correct patch and is still not the design, because it throws the
decision away. The work that needed approval does not get approved later; it gets denied now, and a
human who would have approved it never sees it. `defer` (#41791) is the design answer, and it is
young enough that it is documented only in a changelog line.

Three further defects, all live, mark the edges this contract has to hold:

- **#39344** — an `ask` decision *silently disables* `permissions.deny`. Escalation relieved a
  prohibition. Whatever mints a pending record must be unable to do that.
- **#41791** — `defer`'s re-evaluation semantics exist in a changelog sentence and nowhere else.
  Integrators infer them. An inferred contract is not one.
- **Pizza Bot's `edit`** — the human can change the proposed action inside the approval card. The
  screenshot caption describes *"the exact field values a proposed action would submit above
  approve, edit, and reject buttons."* Whatever comes back from an `edit` is, by construction, not
  the thing that was assessed. Nothing in the described flow re-assesses it.

### 3. What Caro already has, and what is missing

| Already in the tree (1.4.0) | Where |
|---|---|
| `SafetyDecision { risk_level, reason, suggested_routing, matched_patterns, confidence }` | `src/safety/mod.rs:189-200` |
| `SuggestedRouting::{AutoApprove, AsyncLog, HumanGate, Block}` + `from_risk_and_safety` | `src/models/mod.rs:189-220` |
| `SafetyLevel::{Strict, Moderate, Permissive}`, `RiskLevel`, `ShellType` | `src/models/mod.rs:247`, `:152`, `:419` |
| `SafetyValidator` — 52+ patterns, synchronous, offline | `src/safety/mod.rs:155` |
| `sha2`, `uuid` (v4+serde), `chrono` (serde), `thiserror`, `tempfile`/`assert_cmd` (dev) | `Cargo.toml:90, 93, 77, 53, 152, 148` |

Missing: any notion that a decision can outlive the process that made it. `HumanGate` is produced
and then discarded. ADR-046 defines the payload a surface would render but explicitly ends at the
wire: its own out-of-scope list reads *"persistence or replay of pending requests."* ADR-065
classifies what interrupting a running command costs but says nothing about a command that never
started. This ADR is the join.

---

## Decision

Add **`caro.pending.v1`**: a durable, caller-owned, tamper-evident record of one deferred decision,
plus two subprocess verbs that mint and resolve it. No daemon, no database, no index, no UI, no
transport. One new file in `src/`.

```
$ caro defer "rm -rf ./build" --store ./.caro-pending --ttl 24h -o json
{"schema_version":"caro.pending/1","record_id":"…","state":"open", …}
$ echo $?
4
```

```
$ caro pending resolve 018f… --store ./.caro-pending --response ./approval.json
{"verdict":"proceed","command":"rm -rf ./build", …}
$ echo $?
0
```

Eight decisions carry the design. **D1, D2 and D6 are the ones that exist because of #89561,
#39344 and Pizza Bot's `edit` respectively**; the rest are house invariants applied.

**D0 — The only precondition is a derive.** `SuggestedRouting` (derive at `src/models/mod.rs:187`)
and `SafetyDecision` (derive at `src/safety/mod.rs:188`) derive `Serialize, Deserialize` but not `JsonSchema`,
while every payload type in this repository's contract ADRs derives all three. Add `JsonSchema` to
both. `schemars` is already a direct dependency (`Cargo.toml:46`). This is two lines and it is
named here so the implementation PR does not discover it.

**D1 — Deferral is a minting operation, not a permission.** `PendingRecord::mint()` takes a
`SafetyDecision` and returns `Err(MintError::NotDeferrable { routing })` for every routing except
`HumanGate`. `Block` cannot become pending; neither can `AutoApprove` or `AsyncLog`. Escalation can
never relieve a prohibition, because the constructor has no inhabitant in which it does. This is
#39344 designed out rather than patched: there is no precedence rule to get wrong, because there is
no path from `Block` into the pending state at all.

**D2 — Minting exits non-zero and executes nothing.** `caro defer` never runs the command. It
writes the record and exits **`EXIT_CODE_PENDING = 4`**. There is no flag, no configuration, no
`--assume-yes`, and no timeout that turns an unanswered record into an execution. The absence of a
response is not a state the resolver can read as approval, because the resolver reads only response
*files*, and an unanswered record has none. `#89561`'s "silently resolves to an implicit allow"
requires a code path that treats missing input as input; this design has none.

**D3 — State is computed, never stored.** The record file carries `created_at`, `expires_at` and
nothing about its own status. `PendingState` is derived at read time from (a) the clock and (b) the
presence of a sibling resolution file:

```
state = if resolution_file_exists { Resolved } else if now >= expires_at { Expired } else { Open }
```

A stored `status: "open"` field would be a lie the moment the clock passed `expires_at`, and a
tamper target besides. Expiry therefore fails closed by arithmetic, not by a cleanup job — which
matters, because there is no daemon to run one. `--include-expired` is a *listing* flag only; it
has no effect on `resolve`, which refuses an expired record unconditionally.

**D4 — The store is a directory the caller names, and nothing else.** `--store <dir>` is
**required**; there is no default, no `~/.caro/pending`, no environment-variable fallback. Caro
accumulates no hidden state anywhere on the machine, and a store is a portable folder of ordinary
JSON that can be committed, copied, or thrown away. One file per record
(`<record_id>.request.json`), created with `O_EXCL` then `fsync`+`rename`; resolutions are written
as a **sibling** file (`<record_id>.resolution.json`), never as a mutation of the request. The pair
is append-only and hash-linkable, which is what ADR-032 will need. No SQLite, no lockfile, no index
— the contrast with Pizza Bot's server-owned database is the point: *their* durability needs a
process to be alive; *this* durability needs a filesystem.

**D5 — Content binding is checked before anything else.** A resolution whose `command_sha256` does
not equal the record's is `Refuse { reason: ContentMismatch }`. The hash is over the exact
command bytes, computed with `sha2` (already a dependency). This is the ADR-045/046 invariant
carried forward: an approval approves a string, not a request id.

**D6 — `Edited` is a new assessment, never a carried one.** `ResolutionOutcome::Edited {
new_command }` is accepted — the affordance Pizza Bot is right to offer — and it is defined to
**invalidate** the original verdict. `resolve` re-runs `SafetyValidator` on `new_command` and then:
`AutoApprove`/`AsyncLog` → `Proceed` with `reassessed: true`; `HumanGate` → mint a **new** record
carrying `supersedes: <old record_id>`, exit 4; `Block` → `Refuse`. An edited command can therefore
never execute on the strength of an approval granted for different text. Approve-then-edit is the
oldest hole in approval UX and it is closed here by making `Edited` a distinct outcome variant with
its own branch, rather than a field on `Approved`.

**D7 — Resolution re-assesses, and drift can only restrict.** Even for an unedited `Approved`
outcome, `resolve` re-runs the validator against the recorded command at resolution time (offline,
deterministic, sub-millisecond) and compares:

| Routing at mint | Routing now | Result |
|---|---|---|
| `HumanGate` | `HumanGate` | `Proceed` |
| `HumanGate` | `AutoApprove` / `AsyncLog` | `Proceed`, `verdict_drift: relaxed` recorded |
| `HumanGate` | `Block` | **`Refuse { VerdictWorsened }`** — the approval does not apply |

A human's approval from yesterday cannot outrun a pattern added today. This is the same
restrict-only shape as ADR-047's bounded relief and ADR-072/073's untrusted-declaration handling,
and it is what makes the record safe to keep for 24 hours rather than 5 minutes. It is also the
honest reading of Claude Code's `defer` semantics — *"resume … to have the hook **re-evaluate**"* —
made mandatory instead of implied: `reassess` is not a flag.

**D8 — The record is AEP-shaped, so ADR-046 lands on top of it without a schema break.** Field
names, the `command_sha256` binding, the `ApprovalOutcome`/`ActorAuthKind` vocabulary and the
opaque `proof` slot are taken from ADR-046 verbatim where they overlap. `PendingRecord` carries
`assessment: SafetyDecision` today and is documented to become `assessment: Assessment`
(`caro.assessment.v1`) when ADR-058/059 exist in code, via an added field and a bumped
`schema_version`, never by repurposing the existing one. Until then this ADR does not wait on
either: it ships against the types that are in the tree.

### Exit-code contract

There is still no repository-wide exit-code table (ADR-024 remains unimplemented; ADR-073 records
the same absence). This ADR claims **4** and leaves 3 to the verbs that already claim it.

| Verb | 0 | 3 | 4 | 1 | 2 |
|---|---|---|---|---|---|
| `caro defer <cmd>` | routing is `AutoApprove`/`AsyncLog`; **no record written**; caller may proceed | routing is `Block`; no record written; caller must refuse | `HumanGate`; **record written**; caller must stop and not execute | I/O or internal error | usage (clap) |
| `caro pending resolve <id>` | `Proceed` | `Refuse` (denied, expired, content mismatch, verdict worsened) | new record minted after `Edited` | I/O or internal error | usage |
| `caro pending list` | always, including empty store | — | — | I/O error | usage |

`-o json` is honored on all three and is the contract; human-readable output is advisory and
unstable, per the existing `OutputFormat` split (`src/cli/mod.rs:105`).

### Files changed

Six, one of them new. No new module directory, no new dependency, no new config key.

| # | File | Change |
|---|---|---|
| 1 | `src/safety/pending.rs` | **new** — all types, `mint`, `resolve`, store I/O, ~300 LOC + tests |
| 2 | `src/safety/mod.rs` | `pub mod pending;` + `pub use`; add `JsonSchema` to `SafetyDecision` (D0) |
| 3 | `src/models/mod.rs` | add `JsonSchema` to `SuggestedRouting` (D0) — two lines, no field changes |
| 4 | `src/cli/mod.rs` | `pub const EXIT_CODE_PENDING: i32 = 4;` |
| 5 | `src/main.rs` | `Commands::Defer{…}` and `Commands::Pending{…}` variants (enum at `:381`) + two dispatch arms (`match cli.command` at `:2996`) |
| 6 | `tests/pending_contract.rs` | **new**, `*_contract.rs` convention, `assert_cmd` + `tempfile` idiom |

---

## Consequences

**Good.** The `HumanGate` branch acquires a terminus that is not a hang and not a silent allow. An
unattended run can be left going without either fail-open risk or a stalled task, which is the user
value Hermes §3.3 named. The record is a file, so it is backup-able, diff-able, reviewable in a PR,
and readable by a front end Caro knows nothing about — including both of the ones researched here.
Caro emits and persists the request and does not become the inbox, which keeps it inside the
enforcement-point boundary the strategy depends on.

**Costs, stated plainly.**

- *This is state, and the constraint said no state.* The scoping template requires *"a pure
  subprocess call (no daemon, no state)."* The process is stateless — nothing is retained between
  invocations, no cache, no session, no background work — but the feature's whole point is a file
  that outlives it. D4 is the reconciliation: the store is caller-named, has no default, is plain
  JSON, and Caro never reads or writes a path it was not handed. A reviewer who thinks that is
  cheating should reject the ADR on that ground explicitly rather than on any other.
- *Clock dependence.* Expiry is wall-clock arithmetic; a machine whose clock moves backwards
  extends a record's life. Mitigation is bounded: `ttl` is capped (24h default, 7d maximum), and
  D7's re-assessment runs regardless of clock, so a stale approval still cannot execute a
  now-`Block` command. Monotonic time does not survive a reboot and is therefore not an option.
- *Two writers, one store.* `O_EXCL` prevents record-id collision; nothing prevents two
  resolutions racing. The second one loses (`AlreadyResolved`), which is the correct answer and is
  tested (T11).
- *A new exit code.* Four verbs across the Proposed ADRs now claim codes independently. This is the
  cost of ADR-024 not existing, is recorded here, and should be reconciled when it does.

---

## Alternatives considered

1. **Fail closed, as #89561 asks.** Make `HumanGate`-with-no-reviewer equal `Block`. Correct, one
   line, and it is what a cautious integrator should do *today*. Rejected as the design because it
   discards the decision: the work is refused rather than queued, and the human never learns there
   was a question. Adopted as the **fallback**: with no `--store`, `caro defer` is a usage error, and
   any integration that cannot supply one should use `Block`.
2. **A pending *queue* with an index file.** A `pending.json` manifest listing open records makes
   `list` O(1) instead of a directory scan. Rejected: an index is a second source of truth that can
   disagree with the files, needs locking, and turns a folder into a database. `list` scans; a store
   with enough records for that to matter is a store that wanted an inbox, which is out of scope.
3. **SQLite, as Pizza Bot uses.** Rejected: adds a dependency, makes the record unreadable without
   a tool, and buys transactional guarantees this contract does not need because records are
   independent and never updated in place.
4. **Embed the approval transport (HTTP callback, MCP elicitation, Slack).** Rejected on the
   enforcement-point boundary: *"Caro emits and persists the decision request; it does not become
   the inbox"* (digest §3.3). ADR-043 and ADR-046 own the transports.
5. **Store the resolved state inside the request file.** Rejected — D3. A mutable status field is
   both a lie-in-waiting and a tamper target, and it breaks the append-only shape ADR-032 needs.
6. **Let the record carry `allow_always` / session-blanket approval.** Rejected, consistent with
   ADR-046 deliberately omitting it from `ApprovalOptionKind`. A durable record that grants standing
   permission is a policy file with worse provenance; ADR-040 is where standing permission belongs.
7. **Skip re-assessment when the response is prompt (< 60s).** Rejected: a conditional
   re-assessment is a conditional guarantee, and the condition is a clock. D7 costs microseconds.

---

## Out of scope for v1

| Item | Why it waits |
|---|---|
| Any UI, notifier, poller, or transport | Explicitly excluded by the selection signal; ADR-043/046 own transports |
| Signing / attestation of the record pair | ADR-049's evidence packet; v1 is tamper-*evident* (append-only + hashes), not tamper-*proof* |
| Strong-auth proof **verification** | ADR-045 owns it; v1 carries `proof` opaquely and records its presence |
| Multi-approver quorum, delegation chains | ADR-046 defers these too; a quorum needs an identity model Caro does not have |
| Carrying `caro.assessment.v1` instead of `SafetyDecision` | ADR-058/059 unwritten in code — D8 reserves the upgrade path |
| Session-level accumulation ("3 pending in this run") | needs a session identity; ADR-053's chain ledger is the right home |
| Resuming the *agent*, not the decision | Caro is not the runtime; `--resume` belongs to the host |
| Garbage collection of expired records | D3 makes them inert; deleting files is the caller's business |
| `origin` beyond a free-form string | ADR-073's `caro.origin.v1` is unlanded; v1 carries it opaquely |
| Windows path/permission hardening beyond `O_EXCL` semantics | needs a platform matrix the test suite does not yet have; recorded as a known gap |

---

## Integration tests (deterministic: fixed input → fixed JSON + exit code)

Full table in the scope document §5; the invariant-bearing rows:

| # | Setup | Action | `verdict` | Exit |
|---|---|---|---|---|
| T1 | `ls -la`, moderate | `defer` | *(no record)* | 0 |
| T2 | `rm -rf /`, moderate | `defer` | *(no record; Block)* | 3 |
| T3 | `rm -rf ./build`, strict | `defer` | record written, `state: open` | **4** |
| T4 | T3's record, valid `Approved` response | `resolve` | `proceed` | 0 |
| T5 | T3's record, `Denied` response | `resolve` | `refuse{denied}` | 3 |
| T6 | T3's record, TTL elapsed (injected clock) | `resolve` | `refuse{expired}` | 3 |
| T7 | T3's record, response for a different command | `resolve` | `refuse{content_mismatch}` | 3 |
| T8 | T3's record, `Edited` → benign text | `resolve` | `proceed{reassessed}` | 0 |
| T9 | T3's record, `Edited` → `rm -rf /` | `resolve` | `refuse{verdict_worsened}` | 3 |
| T10 | T3's record, `Edited` → another `HumanGate` command | `resolve` | new record, `supersedes` set | **4** |
| T11 | T4 already resolved | `resolve` again | `refuse{already_resolved}` | 3 |
| T12 | store with 0 records | `list` | `[]` | 0 |
| T13 | `defer` without `--store` | — | usage error | 2 |
| T14 | record whose `assessment.suggested_routing` was hand-edited to `block` | `resolve` | `refuse{verdict_worsened}` (D7 re-assesses; the file is not trusted) | 3 |

T14 is the one that proves the record is data, not authority.

---

## Validation discipline

Per [`.claude/rules/validation-discipline.md`](../../.claude/rules/validation-discipline.md), this
is a feature spec for a new user-facing capability, so all five gates apply.

- **Gate 1 (20 transcripts): UNMET.** No first-hand interviews back this. The evidence here is
  market-and-defect evidence: two shipped implementations, three open defect reports, and the
  project's own strategic memo. That is enough to justify *scoping*; the rule is explicit that it is
  not enough to justify *implementation*. The implementation PR must either clear Gate 1 or cut the
  first slice narrowly enough to be a refactor of existing validation output rather than a new
  capability — the same instruction the digest §3.4 gives ADR-073.
- **Gate 2 (no surveys): met** — no survey is cited.
- **Gate 3 (demoware trap): see scope document §7**, *"What breaks at 100 real users"* — the named
  assumption is *one record per file in one flat directory scanned on `list`*, which degrades at
  ~10⁴ records and is instrumented by a `store_scan_ms` field in the `list` payload.
- **Gate 4 (devil's advocate): PENDING.** Not run in this session. The two objections most likely
  to land are pre-stated: *this is state under a no-state constraint* (Consequences) and *this is a
  fourth ADR in a row with no code behind it* (Provenance note).
- **Gate 5 (Sean Ellis): N/A** — no PMF claim is made here.

---

*Produced autonomously on 2026-09-18. Nothing in this ADR has been committed to a branch, merged,
or implemented. It is a proposal for human review.*
