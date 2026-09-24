# ADR-050: Session-Scoped Execution Budgets for Destructive-Operation Classes

- **Status**: Proposed (scope only — no implementation in this document)
- **Date**: 2026-08-13
- **Produced by**: `caro-research--scoping-process` scheduled task
- **Feature researched**: Cloudflare **Wallets / cloudflare.pay** per-agent
  delegated spending caps (announced 2026-08-04; funding and programmable
  spend still future-tense — pre-GA), read against AWS **AgentCore 14-day
  session runtimes** and the Claude Code hooks system's documented absence
  of cross-invocation session state.
- **Depends on**: ADR-036 (`caro guard` — this ADR extends guard mode and is
  **blocked by ADR-036's implementation**, which has not landed as of this
  date), ADR-020 (`SuggestedRouting` tier vocabulary), ADR-040 (policy file —
  hosts the `[budgets]` section)
- **Relates to**: ADR-031 (session circuit breaker — the *caro-session*
  sibling of this ADR; see Alternatives §1), ADR-041 (lifecycle events —
  budget-exceeded event), ADR-047 (trusted targets — exemption interplay,
  deferred), ADR-048 (anthropic-hook host — inherits budgets for free once
  both land), ADR-049 (evidence packets — ledger export, deferred)
- **Numbering note**: highest existing is ADR-049; per
  `.claude/rules/adr-numbering.md`, renumber on merge if another 050 lands
  first.

> **Provenance note (autonomous run).** Produced with no user present; the
> task template's `[FEATURE NAME]` was unbound. Target selection: the
> 2026-08-13 Hermes scan's opportunities map A→ADR-036/048, B→ADR-037/041/049,
> C→content (outside charter), E→parked behind validation-discipline Gate 1.
> Opportunity D — "Execution Budgets for Destructive Operation Classes",
> explicit next step *"design note only"* — is the only engineering item with
> no ADR behind it (grep: no ADR covers budgets/quotas per operation class;
> ADR-031's counters are risk-level-keyed and caro-session-bound). The memo
> marks D "Later · revisit after A ships"; A's *scope* shipped as ADR-048 on
> 08-11, so producing D's design note now is the reviewable interpretation.
> One task constraint is deliberately relaxed: "no state" is read as *no
> daemon, no resident state* (the ADR-031 precedent) — a budget is
> definitionally cumulative, so durable file state is the honest minimum.
> Treat that relaxation and every de-scoping decision below as reviewable.

---

## Context

### The market problem

Binary allow/block is coarse for long-running agent sessions. AWS AgentCore
now runs agent sessions up to **14 days** (vs. 8 hours); over that horizon a
per-command gate answers "is this command dangerous?" but not "is this the
*ninth* file deletion this session?" Buyers are being trained by the
payments layer to expect delegated, capped autonomy: Cloudflare's Wallets
give each agent an allowance, an allowlist, and a max transaction size
delegated from a human-held account. The Hermes 2026-08-13 scan calls this
shift directly: *"Expect buyers to ask for 'budgets' on destructive
operations, not just yes/no gates."*

Caro today expresses exactly one quantity dimension: ADR-031's proposed
circuit breaker counts High/Critical/Blocked verdicts within a **caro-owned
`AiSession`** and halts the whole session on threshold. Nothing expresses
"allow up to N operations of class X, then escalate" — and nothing counts
anything at all in **guard mode** (ADR-036), where caro is a guest inside
*someone else's* session (a Claude Code `session_id` caro does not own).

### Phase 1 — the analogs, and why they are limited

**Cloudflare Wallets / cloudflare.pay (announced 2026-08-04, pre-GA).**
Architecture: a human-held **Account Wallet** is the root balance; it issues
per-agent **Virtual Wallets**, each configured at creation with (a) a
spending cap/allowance, (b) an approved-merchant allowlist, (c) a maximum
single-transaction size the agent cannot exceed on its own. Delegation is
one-directional — a Virtual Wallet can never widen its own limits. Why it is
not yet GA: handle reservation opened at announcement, but funding, spending,
and merchant support are explicitly future-tense in Cloudflare's own copy;
the enforcement path exists only for money, inside Cloudflare's control
plane. Structural limits for our problem: **platform-bound** (Cloudflare
account required), **currency-denominated only** (a dollar cap, not "zero
disk-level operations"), and **centralized** (the ledger lives in their
control plane — useless offline or air-gapped).

**Claude Code hooks (the enforcement point caro already occupies via
ADR-036).** The hooks system has **no session state between invocations**:
each hook is a cold subprocess; official docs offer no initialization or
state pooling mechanism. An integrator who wants "halt after N deletions"
must hand-roll a counter in external state — and the hooks runtime makes
that hazardous by design: matching hooks run **in parallel** (concurrent
tool batches ⇒ concurrent counter writers, last-write-wins races), timeouts
and non-zero/non-two exit codes are **fail-open** ("exit 1 … proceeds with
the action"), and mid-session config edits don't apply until restart. So the
one place budgets could be enforced client-side today is precisely where the
host gives you no safe primitive to count with.

**The specific failure mode to solve by design** (per task charter): a
budget implemented as integrator-side hook state is (1) racy under the
host's parallel-hook execution, (2) fail-open on any error path, and
(3) invisible — no structured record of what was spent. The design below
makes the counter a locked, append-only, local ledger owned by the guard
binary itself, with tighten-only semantics and fail-closed defaults.

### Phase 2 — what caro already has (nothing here is duplicated)

| Capability | Where | Status |
|---|---|---|
| Per-command verdict + routing | `RiskLevel` (`src/models/mod.rs:152`), `SuggestedRouting::from_risk_and_safety` (`src/models/mod.rs:198`) | Shipped |
| Pattern registry (67 entries¹) | `DANGEROUS_PATTERNS` (`src/safety/patterns.rs:12`), `DangerPattern` struct (serde + JsonSchema) | Shipped |
| Guard subprocess, host adapters, fail-closed emission, `AssessmentEnvelope` | ADR-036 | Scoped, **not implemented** |
| Declarative policy file | ADR-040 `policy.toml`, layered resolution | Scoped |
| Risk-level session counters + halt | ADR-031 `BreakerCounts` / `BreakerPolicy` (caro `AiSession`-bound) | Scoped |
| Lifecycle event emitter (JSONL) | ADR-041 | Scoped |
| JSONL discipline, append-only stores | `src/caroml/history.rs` journal; telemetry rusqlite store | Shipped |

¹ `patterns.rs` module docs and several ADRs say "52+"; the literal count on
disk today is **67** `DangerPattern` entries. Marketing copy is fine
("52+"), but this ADR uses the real number.

**Unique positioning** vs. the analogs: operation-class-denominated (not
currency), host-agnostic (any agent CLI that can call a hook), offline and
air-gapped-safe (local ledger, no control plane), deterministic (same ledger
+ same command ⇒ same verdict), and auditable (the ledger is human-readable
NDJSON, exportable later via ADR-049).

---

## Decision

Add **execution budgets** to guard mode: a `[budgets]` policy section
(ADR-040 file), a per-`(host, session)` append-only **budget ledger** on
local disk, and a post-validation **budget evaluation step** in
`caro guard` that can only *tighten* the verdict. No daemon. No new
subcommand. No new exit codes.

### 1. Operation classes (new enum + pattern classification)

```rust
/// src/models/mod.rs — next to RiskLevel
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum OperationClass {
    FileDeletion,     // rm, find -delete, shred …
    DiskWrite,        // dd to block devices, mkfs, fdisk, GEOM/disklabel …
    PermissionChange, // chmod/chown -R on system paths …
    ProcessControl,   // kill -9 patterns, fork bombs …
    NetworkExec,      // curl|sh pipe-to-shell family …
    PackageMutation,  // forced package removal/install patterns …
}
```

`DangerPattern` gains one backward-compatible field:

```rust
#[serde(default, skip_serializing_if = "Option::is_none")]
pub operation_class: Option<OperationClass>,
```

The destructive subset of the 67 built-in patterns is classified in
`patterns.rs` (data-only diff). `None` = unclassified: risk tiers apply as
today and **no budget is charged**. User custom patterns may set the field;
`validate_user_pattern` accepts it unchanged.

### 2. Policy surface (`[budgets]` in the ADR-040 policy file)

```toml
[budgets]
enabled = true          # default false until ADR-040 lands; flag-gated
ttl_days = 14           # ledger lifetime; matches the AgentCore horizon

[budgets.class.file-deletion]
max = 5                 # allowance per session
on_exhausted = "ask"    # ask | deny

[budgets.class.disk-write]
max = 0                 # zero-budget class: every occurrence escalates
on_exhausted = "deny"
```

Semantics are **tighten-only**, mirroring Cloudflare's one-directional
delegation and ADR-035's "hooks can tighten, never loosen":

| Base routing (ADR-020) | Budget remaining | Budget exhausted |
|---|---|---|
| `Block` | `Block` (budgets never rescue) | `Block` |
| `HumanGate` | `HumanGate` | `on_exhausted` (≥ `HumanGate`) |
| `AsyncLog` / `AutoApprove` | unchanged, **charged** | escalated to `on_exhausted` |

`Critical` risk is always `Block` regardless of any budget — a budget is an
allowance for *tolerated* risk, never an override channel.

### 3. The ledger (state without a daemon)

- **Location**: `<state_dir>/budgets/<key>.jsonl` where `state_dir` defaults
  to `dirs::state_dir()/caro`, falling back to `dirs::data_dir()/caro/state`
  (`state_dir()` is `None` on macOS — verified against the in-tree `dirs = "6"`),
  and is overridable via `--state-dir` (required
  for test isolation — recon note: `CARO_CONFIG_DIR` in existing tests is
  not actually wired; guard takes explicit flags instead of repeating that
  bug).
- **Key**: `blake3(host ‖ session_id)` hex-truncated. `session_id` comes
  from the host payload (`correlation` map in ADR-036's `GuardInput`).
  **No session id ⇒ budgets silently disabled** for that invocation and the
  envelope reports `budget.status: "unkeyed"` — a documented, honest
  degradation instead of a global counter that conflates sessions.
- **Entry** (append-only NDJSON, one line per charge):

```rust
/// src/safety/mod.rs — next to AssessmentEnvelope
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BudgetEntry {
    pub schema_version: u32,           // 1
    pub at: chrono::DateTime<chrono::Utc>,
    pub class: OperationClass,
    pub command_hash: String,          // "sha256:<hex>", joins ADR-036 envelope
    pub charged_routing: SuggestedRouting,
}
```

- **Concurrency** (the Phase-1 race, solved): the read-count-decide-append
  sequence holds an exclusive advisory lock (`flock(LOCK_EX)`) on the ledger
  file for its entire (<1ms) duration. Parallel guard invocations from
  concurrent host tool batches serialize on the lock; two racers can never
  both consume the last unit. Appends are single `write(2)` calls of one
  line, so a crashed writer leaves at worst a truncated final line, which
  the replay parser skips with a stderr warning (fail-closed: an unparseable
  ledger counts as exhausted, see §5).
- **Charging rule**: a charge is appended when the *emitted* verdict for a
  classified command is allow-tier (`AutoApprove`/`AsyncLog`). Guard cannot
  observe actual execution (PostToolUse auditing is ADR-036 out-of-scope);
  charging at approval time is the conservative direction — over-counting
  escalates early, never late. Denied/asked commands are not charged.
- **TTL**: ledger files with mtime older than `ttl_days` are unlinked
  opportunistically at guard start (one `readdir`, no daemon).

### 4. Envelope + host output (add-only within `schema_version: 1`)

`AssessmentEnvelope` (ADR-036) gains one optional field — legal under the
ADR-036 add-only stability promise:

```rust
#[serde(skip_serializing_if = "Option::is_none")]
pub budget: Option<BudgetStatus>,

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BudgetStatus {
    pub status: BudgetState,           // Charged | Exhausted | Unkeyed | Error (kebab-case)
    pub class: Option<OperationClass>,
    pub spent: u32,                    // including this invocation if charged
    pub max: u32,
}
```

Host-dialect outputs need **no new vocabulary**: an escalation emits the
host's existing `ask`/`deny` shape with reason prefixed
`budget-exhausted(<class> <spent>/<max>): `. Exit codes are exactly the
ADR-036 registry (0/2/3/64; 5 stays ADR-048's). A budget event
(`budget_exceeded`) is emitted through the ADR-041 sink when both land.

### 5. Fail-closed (extends ADR-036 §3)

Ledger I/O failure (unreadable file, lock timeout >250ms, poisoned line,
full disk on append) never fail-opens. Default: treat the class as
**exhausted** for this invocation (escalate per `on_exhausted`), set
`budget.status: "error"`, one-line stderr diagnostic. `--on-ledger-error
allow|ask|deny` overrides for observe-only rollouts, mirroring ADR-036's
`--on-error`. Note the deliberate asymmetry: a ledger error degrades to
*escalation of classified commands only* — it never denies safe,
unclassified commands, so a corrupt ledger cannot brick a session.

### 6. Session/context lifecycle

Per the pure-subprocess constraint: no daemon, no resident memory, no
warm cache. Cross-invocation continuity lives entirely in the ledger file;
every invocation cold-reads it (bounded: one session's charges, lock-held
<1ms). Idempotence caveat (unlike ADR-036/048): guard with budgets enabled
is deterministic *given a ledger state*, and each allow-tier classified
verdict advances that state by exactly one appended line — asserted as
such in tests rather than pretending byte-identical reruns.

---

## Consequences

**Positive:** caro answers the "budgets on destructive operations" buying
question with an offline, host-agnostic, auditable primitive neither
Cloudflare (money-only, platform-bound) nor any agent CLI (stateless hooks)
offers; the ledger is a natural ADR-049 evidence input; ADR-048's host
inherits the same budgets with zero additional surface.

**Negative / risks:** first durable state in the guard path — mitigated by
tighten-only semantics, TTL, `--state-dir`, and the §5 no-brick rule;
classification of 67 patterns is judgment-laden — mitigated by shipping only
unambiguous classifications in v1 and leaving the rest `None`; charging at
approval-time over-counts commands the host user manually cancelled
(accepted: conservative direction); blocked by ADR-036 implementation,
which has not landed — this ADR cannot be implemented first.

## Scope — Files Changed (minimal set)

1. `src/models/mod.rs` — `OperationClass`, `BudgetState` (~50 LOC)
2. `src/safety/patterns.rs` — `operation_class` field on `DangerPattern`
   entries; classify destructive subset (data-only)
3. `src/safety/mod.rs` — `BudgetEntry`, `BudgetStatus`, budget evaluation
   fn, envelope field (~120 LOC)
4. `src/cli/guard.rs` — ledger open/lock/replay/append, `--state-dir`,
   `--on-ledger-error`, TTL prune (~150 LOC; file exists once ADR-036 lands)
5. `src/bin/generate-schema.rs` — re-emit envelope schema (~2 LOC)
6. `tests/guard_budget_contract.rs` + `tests/fixtures/guard/budgets/`

No new top-level modules. No new dependencies (`blake3` arrives with
ADR-048; if 048 lands later, substitute `sha2`, already present).

## Integration Tests (known input → deterministic output + exit code)

All tests pass `--state-dir <tempdir>`; fixtures are claude-code payloads
unless noted.

1. `rm -rf ./build` ×5 with `file-deletion.max = 5` → five silent passes
   (exit 0, empty stdout), ledger has 5 entries; 6th → `permissionDecision:
   "ask"`, reason starts `budget-exhausted(file-deletion 5/5):`
2. Same, `on_exhausted = "deny"` → 6th emits `"deny"`
3. `dd if=/dev/zero of=/dev/sda bs=1M` with generous `disk-write.max = 99`
   → still deny (Critical is never budget-rescued), ledger unchanged
4. Two distinct `session_id`s interleaved → independent counts
5. Payload without `session_id` → silent pass, generic-mode envelope has
   `budget.status: "unkeyed"`
6. Ledger pre-seeded with a truncated final line → classified command
   escalates, `budget.status: "error"`, exit 0; unclassified safe command
   (`ls -la`) still silent-passes
7. `--on-ledger-error allow` with unreadable ledger → silent pass
8. 8 concurrent guard invocations against `max = 4` (spawned in parallel)
   → exactly 4 charges in ledger, 4 escalations; no interleaved/corrupt lines
9. Generic mode, exhausted budget → exit 0, envelope `decision: "ask"`,
   `budget.status: "exhausted"`, `spent == max`
10. Ledger file mtime older than `ttl_days` → pruned at start, count resets
11. Exit-code sweep: all of the above exit ∈ {0, 2, 3, 64} — never 1

## Out of Scope (next version)

- Money/wallet integration of any kind (Cloudflare's lane; caro budgets are
  operation-denominated on principle)
- PostToolUse/AfterTool actual-execution accounting (charges at approval;
  refinement joins ADR-036's PostToolUse out-of-scope item)
- Trusted-target budget exemptions (needs ADR-047 first; interplay one-pager
  before any code)
- Cross-session / global / per-day budgets (session-scoped only in v1)
- Ledger signing & evidence export (ADR-049 owns export; signing is its v2)
- `caro budget` CLI verbs (status/reset/inspect — v1.1 UX pass)
- Budgets in interactive (non-guard) caro sessions (ADR-031's breaker is the
  right primitive there; merging the two counters is a later ADR)

## Alternatives Considered

1. **Extend ADR-031's `BreakerCounts` instead of a new ledger** — rejected:
   the breaker counts *risk levels* inside a **caro-owned `AiSession`** and
   halts the whole session; budgets count *operation classes* inside a
   **host-owned session** caro only observes, and escalate per-command.
   Different key, different store (`SessionStore` vs. ledger file),
   different semantics (halt vs. tighten). Unifying them prematurely couples
   guard mode to the `ai` module. A later ADR may merge the counters.
2. **Host-side counters (SessionStart hook caches state, PreToolUse reads
   it)** — rejected: this is exactly the Phase-1 failure mode — integrator-
   owned, racy under parallel hooks, fail-open on error, per-host bespoke.
3. **SQLite ledger (rusqlite already a dependency)** — rejected for v1:
   better concurrency story, but the NDJSON ledger is human-auditable,
   greppable, aligned with ADR-041/049 discipline, and flock suffices at
   guard's write rates. Revisit if lock contention is ever measured.
4. **Sliding time windows (N per hour) instead of session scope** —
   rejected: session scope matches the threat (a 14-day session), needs no
   clock arithmetic at judgment time, and windows invite "wait out the
   window" gaming; `ttl_days` already bounds ledger life.
5. **Daemon with in-memory budgets** — rejected: violates the pure-
   subprocess constraint; also re-created the availability problem the
   ledger avoids (daemon down ⇒ fail-open or fail-stuck).

## References

- Cloudflare press release — "Cloudflare gives AI agents an identity and a
  wallet" (2026-08-04): cloudflare.com/press/press-releases/2026/cloudflare-gives-ai-agents-an-identity-and-a-wallet/
- Help Net Security, "Cloudflare gives AI agents wallets with built-in
  spending controls" (2026-08-05)
- Claude Code hooks reference — code.claude.com/docs/en/hooks (re-verified
  2026-08-13: parallel hook execution, per-event cold spawn, no session
  state mechanism, exit-1 fail-open)
- Hermes digest `.hermes/digests/2026-08-13-weekly-agent-market-scan.md`
  (Opportunity D; market shift 4 "scoped capabilities and budgets are the
  new control primitive")
- ADR-020, ADR-031, ADR-035, ADR-036, ADR-040, ADR-041, ADR-047, ADR-048,
  ADR-049
