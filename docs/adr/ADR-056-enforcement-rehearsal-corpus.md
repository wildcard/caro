# ADR-056: Enforcement Rehearsal — Observe Mode and Offline Corpus Replay Without a Daemon

- **Status**: Proposed (scope only — no implementation in this document)
- **Date**: 2026-08-21
- **Produced by**: `caro-research--scoping-process` scheduled task (autonomous run)
- **Feature researched**: **Phinq** (`phinq-co/phinq`, **MIT**, free; Product Hunt
  2026-08-14, #25 of day, ~86 pts) — an open-source runtime governance layer that
  intercepts every agent tool call, classifies it, holds irreversible actions for
  human approval, and hash-chains every decision. Read live 2026-08-21 against the
  vendor's own docs: `/docs/concepts/how-phinq-works`, `/docs/concepts/risk-model`,
  `/docs/concepts/watch-mode-and-enforcement`, `/docs/concepts/approvals`,
  `/docs/concepts/audit-log`, `/docs/reference/cli`, `/docs/reference/http-gate`,
  `/docs/reference/typescript-sdk`, `/docs/security/limitations`. Each of those
  pages carries the vendor's own footer *"Verified against the upstream source
  repository."* Specifically scoped here: Phinq's **watch/shadow mode** and its
  **`npm run replay -- phinq-toolcalls.jsonl`** calibration path — the on-ramp that
  precedes `PHINQ_ENFORCE=1`.
- **Cross-read**: **Execlave** (PH 2026-08-13, #14 of day) tiered autonomy, whose
  lowest tier is also *observe*; **GuardFall** (Adversa AI, *The Hacker News*,
  2026-06-30) which bypassed safety checks in 10 of 11 open-source coding agents by
  exploiting how Bash rewrites commands before execution.
- **Depends on** (all **Proposed, none implemented** — verified against the tree on
  2026-08-21; this ADR is *blocked on them landing first*):
  - **ADR-024** — `HeadlessEnvelope`, the `ExitCode` enum as the single definition of
    process codes, `schema_version`, and the additive-field stability rule.
    `grep -rn "enum ExitCode" src/` and `grep -rn "HeadlessEnvelope" src/` both return
    zero hits; `EXIT_CODE_EDIT = 201` (`src/main.rs:938`) remains the only exit
    constant in the tree.
  - **ADR-032** — `ExecutionReceipt` and the hash-chain mechanism. This ADR reuses
    that chain; it does not define a second one.
- **Relates to**: ADR-053 (portable session chain ledger — supplies the *order-aware*
  half of what Phinq calls "session velocity", also without a daemon; this ADR
  supplies the *rehearsal* half), ADR-049 (`caro audit export|verify` — owns the
  verify verb; this ADR nests under it as `caro audit rehearse`), ADR-044 (fail-closed
  effects resolution), ADR-052 (`Violation`, `RuleId`, D4 fail-closed), ADR-055
  (coverage-completeness invariant, borrowed wholesale in D7), ADR-035/036 (external
  policy hooks and the agent-guard hook adapter — the capture points that feed the
  corpus), **ADR-007** (AST parser shell validation — *Proposed and
  unimplemented*; this ADR does **not** depend on it, and D4 states exactly what
  "unparseable" means without it), Hermes memo 2026-08-20 §3.8 (*observe mode*, promoted to **Now**).
- **Numbering note**: highest existing is ADR-055. Per `.claude/rules/adr-numbering.md`,
  renumber on merge if another 056 lands first.
- **Exit-code note**: this ADR claims exit code **16**. The registry is contended —
  two ADRs already claim 13, ADR-053 claims 14, and ADR-055 claims "a new, distinct
  process exit code" without numbering it (presumed 15). ADR-053's complaint stands:
  **the exit-code registry now needs an owner**, and that owner should be ADR-024's
  `ExitCode` enum, not prose in sibling ADRs.

---

## Context

### The problem and who has it

Nobody turns a blocking safety gate on for a live agent on day one. The person
evaluating Caro is an engineer whose agent already works; enabling something that can
refuse a command is a change with unbounded downside and no visible upside until the
day it saves them. The rational move is to do nothing.

Every serious competitor has now shipped the same answer to this, and shipped it as
the *first* thing a user does:

- **Phinq**: `phinq start` then `phinq watch` runs shadow mode — it inspects and logs
  calls while allowing traffic to continue, explicitly so "an operator [can] review
  classifications and tune rules before enforcement". Enforcement is a separate,
  deliberate step (`PHINQ_ENFORCE=1 phinq start`), and the docs say so in those words:
  *"Enforcement is a deliberate operational change. Test the approval path and keep a
  recovery plan before turning it on for a live agent."* It ships a replay harness —
  `npm run replay -- phinq-toolcalls.jsonl` — to re-run a captured corpus through the
  classifier offline.
- **Execlave**: tiered autonomy whose first tier is observe, escalating through advise
  and act-with-approval to autonomous.

Caro has no equivalent. It has exactly two states: not installed, and blocking. There
is no way to run Caro for a week at zero blast radius, collect what it *would* have
stopped, and flip enforcement with evidence in hand. That is the entire adoption
on-ramp of the category, and Caro is missing it.

There is a second, internal reason. `.claude/rules/validation-discipline.md` requires
20 first-hand transcripts before a feature spec graduates. A rehearsal corpus is the
cheapest source of exactly the artifact those interviews need: real commands, real
verdicts, real disagreements — from the user's own machine, without asking them to
risk anything.

### What Caro already has (do not rebuild)

| Need | Already exists | Location |
| --- | --- | --- |
| Batch validation in one process | `SafetyValidator::validate_batch(&[String], ShellType)` | `src/safety/mod.rs` |
| Per-command verdict, serializable | `ValidationResult { allowed, risk_level, explanation, warnings, matched_patterns, confidence_score }` | `src/safety/mod.rs:175` |
| Decision vocabulary | `SuggestedRouting { AutoApprove, AsyncLog, HumanGate, Block }` + `from_risk_and_safety()` | `src/models/mod.rs:189` |
| Deterministic, reversible redaction | `sanitize` / `restore` pair (hybrid privacy gateway) | `src/backends/hybrid/sanitizer.rs` |
| Risk taxonomy, ordered | `RiskLevel { Safe, Moderate, High, Critical }` (`Serialize`, `Ord`) | `src/models/mod.rs:152` |
| Output formats | `OutputFormat { Json, Yaml, Plain }` | `src/cli/mod.rs` |
| Config + custom patterns + allowlist | `ConfigManager`, `SafetyConfig::from_user_config()`, sibling `patterns.toml` | `src/config/mod.rs`, `src/safety/mod.rs:710` |
| Pattern corpus | `Lazy<Vec<DangerPattern>>` compiled into the binary; CVE patterns from a bincode blob | `src/safety/patterns.rs`, `src/safety/cve_patterns.rs` |

`validate_batch` already existing is the load-bearing fact of this ADR: the expensive
part of a rehearsal run — regex compilation, CVE blob load, config resolution — is
paid once per *process*, not once per *record*. That is the whole reason this can be a
subprocess instead of a daemon.

### The failure modes we must design around

Phinq's own `/docs/security/limitations` page is unusually honest, and four of its five
bullets are design constraints for us. Two more come from its architecture rather than
its disclosures.

**FM1 — Two logs, one chained, and the raw arguments live in the unchained one.**
Phinq's audit chain deliberately excludes arguments and message payloads; a *separate*
`phinq-toolcalls.jsonl` corpus "may contain tool-call arguments and is not
hash-chained". So the artifact you replay for calibration is precisely the artifact
with no integrity guarantee, and precisely the artifact holding the sensitive data. For
Caro the split is not even available: the command string **is** the unit of analysis.

**FM2 — The chain cannot detect a full rewrite or tail truncation.** Stated plainly:
detection of those "requires an externally anchored head hash". Phinq ships
`phinq audit verify` but does not ship the anchoring, so the default posture is a chain
that catches the careless attacker and not the careful one.

**FM3 — Governance only covers traffic routed through the proxy, MCP wrapper, or SDK
gate.** Anything the agent executes out of band is invisible — and, critically,
*silently* invisible. A calibration report built from that corpus overstates coverage
by exactly the amount it cannot see, and gives no signal that it is doing so.

**FM4 — Unknown tools are not denied.** "Do not assume that an unknown tool is blocked:
the documented posture is to flag unknown tools without making them a blanket denial."
A classifier that returns *not-classified* for input it cannot parse is returning
*allowed* for that input, in every consumer that branches on `allowed`.

**FM5 — The localhost gate is open by default.** `POST /phinq/gate`, `/phinq/classify`,
and the hold approve/deny endpoints listen on `127.0.0.1:5100` and are "open by default
unless `PHINQ_GATE_TOKEN` is set". A daemon that adjudicates safety and accepts
unauthenticated approvals from any local process has inverted its own threat model. Any
local process — including the agent under governance — can approve its own holds.

**FM6 — Classification is on tool name plus arguments, not on shell semantics.** The
classifier "classifies the call from its name, arguments, and session velocity". For a
coding agent the tool is `Bash` and the argument is an opaque string; risk lives
entirely inside that string. GuardFall broke 10 of 11 open-source coding agents on
exactly this — by exploiting how Bash rewrites commands before execution — and the sole
survivor won by parsing the command the way bash will before deciding. This is Caro's
differentiator and it is the reason a *replay* of recorded command strings has value
that a replay of recorded tool-call metadata does not.

**FM7 — A hold is stateful, and its safety property is a timer.** Phinq's SDK holds a
pending action for `holdTimeoutMs` (240 s default) with `defaultOnHold: "deny"`; silence
auto-denies. That is the correct semantic, and it costs a live process, a pending-hold
store, and an approval channel. It also means the fail-closed guarantee is only as good
as the daemon's uptime: if the process dies mid-hold, the guarantee dies with it.

---

## Decision

Add **enforcement rehearsal** to Caro as two surfaces over one artifact, with no
daemon, no listener, and no persistent process state.

1. **`--observe`** (global flag, and `CARO_OBSERVE=1`). Caro validates exactly as it
   would under enforcement, emits the full assessment in the payload, appends a record
   to the rehearsal corpus — and **does not change what the process does**. Exit code
   is whatever it would have been with safety disabled. The counterfactual verdict
   ships as data in the envelope (`enforcement: "observe"`, `would_exit: <code>`).

2. **`caro audit rehearse <corpus>`** — a pure batch subprocess. Reads a corpus,
   re-validates every record against the *current* pattern set and config in **one**
   process via `validate_batch`, and emits a `RehearsalReport`: aggregate counts by
   verdict, a per-record delta against the verdict recorded at capture time, and an
   explicit coverage accounting. Deterministic: same corpus plus same pattern set plus
   same config yields byte-identical JSON.

The corpus is a single append-only, hash-chained NDJSON file reusing ADR-032's chain.
There is no second, unchained file.

### D1 — Batch in one process is the answer to init amortization; a daemon is not

Phinq amortizes classifier startup across a long-lived proxy. That buys speed and costs
a listening port, a pending-hold store, a supervision story, and FM5. Caro amortizes
across the *corpus* instead: `caro audit rehearse` compiles patterns once, loads the CVE
blob once, resolves config once, then walks N records. A 10,000-record corpus pays one
cold start, not 10,000. Live `--observe` pays the cold start Caro already pays for the
generation it was going to do anyway — the observe tee adds a serialize and an append,
not an initialization.

This is the direct answer to the "how does it avoid redundant initialization?" question
without accepting anything from FM5 or FM7.

### D2 — One corpus, one chain, redaction at capture (answers FM1)

The command string is the payload and the payload is the point, so it goes **in** the
chain. It is redacted *before* hashing, by a deterministic pass reusing the hybrid
backend's existing sanitizer (`src/backends/hybrid/sanitizer.rs` — already deterministic
and reversible), and the redaction map is **not** persisted. Consequences, stated so
nobody is surprised later:

- There is exactly one artifact. Nothing about the rehearsal path is unchained.
- `redacted: true` and `redaction_profile: "<name>"` are record fields, so a report can
  say how much of its input was obscured.
- Redaction is lossy for replay fidelity in the specific case where the secret was
  itself the dangerous part (e.g. a credential in a URL). That case is recorded as a
  `Violation` at capture time so the *verdict* survives even though the *string* does
  not.

### D3 — The head hash is emitted on every write, so anchoring is by construction (answers FM2)

Every `--observe` invocation writes the new head hash to **stderr** as a single
`caro-rehearsal-head: <hex>` line, and `RehearsalReport` carries `head_hash` and
`record_count`. The caller therefore receives the anchor on every run without opting in,
and any external sink that captures stderr — CI logs, the agent's own transcript, a
scheduled task's output — is an anchor. `caro audit verify` (ADR-049) gains the ability
to check a corpus against a caller-supplied expected head.

We do not claim tamper-*proof*. We claim: truncation and rewrite are detectable by
anyone who kept any prior line of stderr, and that is a strictly stronger default than
shipping a verify command with no anchoring path.

### D4 — Unparseable means blocked, never "flagged" (answers FM4)

If Caro cannot resolve a command string, the verdict is `Block` with a
`caro:builtin:unparseable-command` violation. Not "unknown". Not "flagged". Not absent.
This is ADR-052's D4 and ADR-044 applied at the rehearsal boundary, and it is
deliberately stricter than Phinq's documented posture, because "we could not analyze it"
and "it is safe" must never serialize to the same thing.

**What "cannot resolve" means today, honestly.** ADR-007 (AST parser) is **Proposed and
unimplemented** — `grep -rn "yash-syntax\|conch-parser\|brush-parser" src/ Cargo.toml`
returns zero hits, and today's validator is regex over the raw string with a
quote-context heuristic (`is_dangerous_in_context`). So v1's unparseable predicate is a
bounded, explicitly-enumerated pre-check — unbalanced quotes, a command exceeding
`SafetyConfig::max_command_length`, or a non-UTF-8 byte sequence — and nothing more. It
is a small set, it is named in `docs/rehearsal-corpus.md` rather than implied, and it is
the *seam* the AST parser widens when ADR-007 lands. Claiming today's regex path
"resolves the command as bash will" would be the exact overclaim §3.7 of the 2026-08-20
memo warns against.

`--allow-unparseable` downgrades this to a warning. It is an explicit operator choice
and it is recorded in the report as `overrides: ["allow_unparseable"]`. There is no
`--fail-open`.

### D5 — Observe mode never alters process behaviour, and says so in the payload (answers the adoption problem)

The single most common way a shadow mode betrays its user is by not being entirely
shadow. So:

- `--observe` **cannot** change the exit code. Enforced by a unit test asserting that
  for every fixture, `exit_code(observe) == exit_code(--safety permissive equivalent)`.
- `--observe` and `--execute` compose: the command runs. That is the point.
- `--observe` is refused in combination with anything that would make the counterfactual
  meaningless (`--dry-run` is fine; a future `--enforce` is a usage error, exit 2).
- The envelope always carries `enforcement`, so a consumer can never mistake an observed
  run for an enforced one.

### D6 — No holds, no timers; the wait belongs to the caller (answers FM7)

Caro returns a verdict and an exit code, synchronously, and exits. It does not hold, does
not poll, does not run a 240-second timer, and does not own an approval channel. Callers
that need human-in-the-loop already have ADR-045 (strong-auth approval gate) and ADR-046
(approval exchange payload) for that, and those are *their* state, not Caro's.

The fail-closed property is therefore expressed as an exit code the caller must handle,
not as a timer Caro must survive. A subprocess that has already exited cannot fail open
by dying — which is precisely the failure mode a hold-in-a-daemon has.

### D7 — Every corpus record lands in `replayed` or `skipped`, and the gap is countable (answers FM3)

Borrowed wholesale from ADR-055's coverage invariant, for the same reason:

- `coverage.replayed + coverage.skipped.len() == coverage.total_records` is a golden-test
  invariant, not a comment.
- Any skip carries a `SkipReason` and sets `coverage.complete = false`.
- `coverage.complete == false` yields report verdict `Incomplete` and exit **16**, never
  `0`.
- Each record carries `source` (`"cli" | "hook" | "mcp" | "import"`) so a report can say
  *which* capture point covered what. This does not solve FM3 — nothing inside Caro can
  see what never reached it — but it makes the blind spot a number in the report instead
  of silence.

---

## New types

All live in **`src/cli/mod.rs`**, next to `HeadlessEnvelope` (ADR-024). No new module.
All derive `#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]` —
serializable from day one, per the standing constraint.

### `EnforcementMode`

```rust
/// Whether this invocation could change what the process did.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, schemars::JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EnforcementMode {
    /// Verdict computed and recorded; process behaviour unchanged.
    Observe,
    /// Verdict computed and acted on.
    Enforce,
}
```

### `RehearsalRecord`

One line of the corpus. Field #1 is `schema_version`; the file is NDJSON.

```rust
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct RehearsalRecord {
    pub schema_version: String,          // "1"
    pub seq: u64,                        // monotonic within the corpus
    pub ts: String,                      // RFC 3339, UTC
    /// Command as the shell would receive it, post-redaction. The unit of analysis.
    pub command: String,
    pub redacted: bool,
    pub redaction_profile: Option<String>,
    pub shell: ShellType,
    pub source: RecordSource,
    /// Verdict at capture time. Reuses existing safety types — no new verdict struct.
    pub verdict: SuggestedRouting,
    pub risk_level: RiskLevel,
    pub matched_patterns: Vec<String>,
    /// Pattern-set + config identity, so replay can report drift honestly.
    pub ruleset_digest: String,
    /// SHA-256 over the JCS-canonicalized previous record. Genesis = 64 zeros.
    pub prev_hash: String,
}
```

`RecordSource` is a closed enum — `Cli`, `Hook`, `Mcp`, `Import`, `Other` — with `Other`
as the escape hatch, per ADR-053's `StepKind` lesson about the cost of closed
vocabularies.

### `RehearsalDelta`

```rust
/// One record's verdict now versus at capture time.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct RehearsalDelta {
    pub seq: u64,
    pub command: String,
    pub was: SuggestedRouting,
    pub now: SuggestedRouting,
    /// Patterns matching now that did not match at capture, and vice versa.
    pub gained_patterns: Vec<String>,
    pub lost_patterns: Vec<String>,
}
```

A delta is emitted **only** when `was != now`. Identical verdicts are counted, not
listed — a 10,000-record corpus with three changes produces a three-line delta list.

### `CorpusCoverage`

```rust
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CorpusCoverage {
    pub total_records: u64,
    pub replayed: u64,
    pub skipped: Vec<SkippedRecord>,
    /// replayed + skipped.len() == total_records, and skipped.is_empty()
    pub complete: bool,
    /// Per-`RecordSource` counts, so the FM3 blind spot is visible.
    pub by_source: BTreeMap<String, u64>,
}
```

`SkippedRecord { seq: Option<u64>, reason: SkipReason, detail: String }` with
`SkipReason ∈ { MalformedJson, SchemaVersionUnsupported, ChainBroken, Unparseable, RedactedBeyondReplay }`.

### `RehearsalReport`

```rust
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct RehearsalReport {
    pub schema_version: String,           // "1"
    pub verdict: RehearsalVerdict,        // Clean | Changed | Incomplete
    pub caro_version: String,
    pub ruleset_digest: String,           // current, not the corpus's
    pub coverage: CorpusCoverage,
    /// Aggregate counts of the *current* verdict across replayed records.
    pub counts: BTreeMap<String, u64>,    // keyed by SuggestedRouting Display
    /// Would-block count — the headline number an operator flips enforcement on.
    pub would_block: u64,
    pub deltas: Vec<RehearsalDelta>,
    pub overrides: Vec<String>,
    pub head_hash: String,
    pub record_count: u64,
    pub timing: TimingInfo,               // reused
    pub exit_code: i32,                   // mirrored, per ADR-024
}
```

### Method contracts

- `RehearsalRecord::append(path: &Path, rec: &RehearsalRecord) -> io::Result<String>` —
  appends one line, returns the new head hash. `O_APPEND` single write; no read-modify-write.
- `RehearsalRecord::chain_next(prev: &str, rec: &RehearsalRecord) -> String` — JCS
  canonicalize, SHA-256. Pure; the golden test pins one known input to one known digest.
- `RehearsalReport::replay(records: &[RehearsalRecord], validator: &SafetyValidator) -> Self`
  — the only place replay logic lives. Calls `validate_batch` **once**. Note
  `SafetyValidator::validate_batch` is `async` (`src/safety/mod.rs:632`), so `replay` is
  `async` too and runs on the Tokio runtime `main.rs` already builds — this adds no
  runtime, and it is the reason D1's "one process, one init" claim is achievable without
  restructuring the validator.
- `RehearsalReport::render(&self, format: OutputFormat) -> String`.
- `HeadlessEnvelope` gains two additive fields (permitted within `schema_version: "1"`
  by ADR-024's additive rule): `enforcement: EnforcementMode` and
  `would_exit: Option<i32>`.

No changes to `ValidationResult`, `SafetyDecision`, `RiskLevel`, `SuggestedRouting`,
`SafetyValidator`, `SafetyConfig`, or `ConfigManager`. The rehearsal path is a consumer
of the existing validator, not a second one.

---

## Minimal set of files that change

No new modules. Five existing files plus two new files that are not modules:

1. **`src/cli/mod.rs`** — the six types above, their impls, and the two additive
   `HeadlessEnvelope` fields.
2. **`src/main.rs`** — add `--observe` to `Cli` (env `CARO_OBSERVE`); add
   `--allow-unparseable`; add the `rehearse` arm under the `audit` subcommand namespace
   claimed by ADR-049; route the observe tee at the existing post-validation point; exit
   via `ExitCode::as_process_code`.
3. **`src/backends/hybrid/sanitizer.rs`** — expose the existing redaction pass under a
   named profile so the corpus can record `redaction_profile`. Behaviour unchanged.
4. **`docs/adr/README.md`** — the ADR-056 table row.
5. **`Cargo.toml`** — no new dependency. `sha2` and `serde_json` are already in-tree;
   JCS canonicalization is ~40 LOC over `serde_json::Value` with sorted keys, and is
   shared with ADR-032 rather than duplicated.
6. **`docs/rehearsal-corpus.md`** (new doc, not a module) — publishes the record schema,
   the report schema, the exit-code row, and the stability promise.
7. **`tests/rehearsal_contract.rs`** (new integration test file) — below.

---

## Exit-code / output contract (what machines depend on)

`caro audit rehearse` reuses ADR-024's codes and adds exactly one.

| Exit | `verdict` | Meaning |
| --- | --- | --- |
| 0 | `Clean` | every record replayed; no verdict changed |
| 0 | `Changed` | every record replayed; ≥1 verdict changed. **Not** an error — drift is the product |
| 1 | — | internal/unexpected failure |
| 2 | — | usage error (bad flags, corpus path missing) |
| 16 | `Incomplete` | **new, claimed here.** `coverage.complete == false` — a record could not be replayed, or the chain broke |

`caro --observe <prompt>` returns **only** the exit code the run would have returned with
safety non-blocking. It can return 0, 1, 2, 5, or 6 (ADR-024) and **can never return 3,
4, or 16**. That is the whole promise of observe mode and it is a test, not a paragraph.

Stability promises, published in `docs/rehearsal-corpus.md`:

- Within `schema_version: "1"`, record and report fields are only **added**.
- A corpus written by version *N* is replayable by version *N+k*; unsupported
  `schema_version` is a `SkipReason`, never a silent skip, never a crash.
- `caro audit rehearse --output json` prints exactly one JSON object to stdout; all
  diagnostics go to stderr.
- `caro-rehearsal-head: <hex>` on stderr is a stable line format.

---

## Integration tests (deterministic input → fixed JSON + exit code)

In `tests/rehearsal_contract.rs`, all fixtures checked in under `tests/fixtures/rehearsal/`:

| # | Input | Expected |
| --- | --- | --- |
| 1 | 12-record clean corpus, unchanged ruleset | exit 0, `verdict: "clean"`, `deltas: []`, `coverage.complete: true`, byte-identical JSON across two runs |
| 2 | Same corpus, one new pattern injected via `patterns.toml` | exit 0, `verdict: "changed"`, exactly 1 delta with `was: "async_log"`, `now: "block"`, `gained_patterns` non-empty |
| 3 | Corpus with a mid-file byte flipped | exit 16, `verdict: "incomplete"`, `skipped[0].reason: "chain_broken"`, `skipped[0].seq` = the first bad record |
| 4 | Corpus truncated at the tail, caller supplies prior head via `--expect-head` | exit 16, `chain_broken` (this is the FM2 case; without `--expect-head` it exits 0, and the test asserts *both* branches so the limitation is pinned, not papered over) |
| 5 | Corpus containing `$(printf '\\x72\\x6d') -rf /` | **Characterization test, not an aspiration.** Asserts whatever v1 actually returns and pins it. If v1 returns anything other than `block`, the test name says so (`guardfall_shell_rewrite_currently_unblocked`) and the case becomes the first entry in the ADR-007 pattern backlog. Publishing the failure is the §3.7 credibility play; asserting a `block` we do not yet deliver would be the overclaim |
| 6 | Corpus record whose command the AST parser cannot resolve | exit 16, `unparseable` skip, `caro:builtin:unparseable-command` violation present (D4) |
| 7 | Same as #6 with `--allow-unparseable` | exit 0, warning present, `overrides: ["allow_unparseable"]` |
| 8 | `caro --observe "rm -rf /"` | **exit 0**, stdout envelope has `enforcement: "observe"`, `would_exit: 3`, `suggested_routing: "block"`; stderr has one `caro-rehearsal-head:` line; corpus gained exactly one record |
| 9 | `caro --observe --enforce` | exit 2, `error.kind: "usage"` |
| 10 | Golden chain test: one pinned record → one pinned SHA-256 | exact digest match (pins JCS canonicalization) |
| 11 | 10,000-record corpus | one `SafetyValidator` construction (asserted via a test-only counter), wall time recorded as a benchmark floor — measured, not assumed (ADR-053's note) |
| 12 | Empty corpus | exit 0, `verdict: "clean"`, `total_records: 0`, `complete: true` |

---

## Out of scope (next version)

- **Approval channels.** No Telegram, no Slack, no phone push. That is Phinq's product
  and ADR-045/046's territory.
- **Holds, pending state, and timeouts.** D6. If a caller wants a 240-second window it
  owns the window.
- **Any listener.** No proxy, no `/gate` endpoint, no port, no `PHINQ_GATE_TOKEN`
  analogue — because there is nothing to authenticate (FM5).
- **Signing and attestation.** ADR-049 owns DSSE/in-toto. This ADR produces the chained
  input that ADR-049 exports.
- **Automatic enforcement promotion.** Caro will not flip a user from observe to enforce
  on a threshold. Execlave's red-team promotion gate is interesting and it is not this.
- **Cross-session correlation and velocity.** ADR-053 owns order-aware multi-command
  detection; a rehearsal corpus is a legitimate *input* to it, and wiring the two is a
  follow-up.
- **The Phinq comparison benchmark** (Hermes §3.7). This ADR builds the corpus format
  that benchmark needs; running Caro's eval corpus against Phinq is a measurement task,
  not a feature, and it stays separate.
- **Importing foreign corpora.** `RecordSource::Import` is reserved so a future
  `caro audit rehearse --from phinq-toolcalls.jsonl` needs no schema change. Not built here.

---

## Consequences

### Benefits

- Caro becomes installable at zero blast radius, closing the single largest adoption gap
  against Phinq and Execlave, and closing it with an artifact neither produces: a chained
  corpus of **command strings**, not tool-call metadata.
- Every design constraint in this ADR removes a documented competitor limitation rather
  than matching a competitor feature. No daemon (FM5, FM7), one chained corpus (FM1),
  anchoring by construction (FM2), countable blind spot (FM3), fail-closed on
  unparseable (FM4).
- Produces exactly the evidence `.claude/rules/validation-discipline.md` Gate 1 needs,
  from real usage, without asking anyone to accept risk first.
- Reuses `validate_batch`, `ValidationResult`, `SuggestedRouting`, `RiskLevel`,
  `TimingInfo`, the sanitizer, and ADR-032's chain. One new exit code; no new dependency;
  no new module.

### Trade-offs

- **Redaction costs replay fidelity** in the narrow case where the secret was the danger
  (D2). Mitigated by recording the capture-time violation, not by pretending otherwise.
- **`ruleset_digest` drift is a reporting problem, not a solved one.** A report can say
  the ruleset changed; it cannot say the change was correct.
- **Corpus growth is unbounded.** No rotation is specified here. A busy agent will
  produce a large file and the first user to hit it will file the issue that specifies
  rotation. That is an accepted, named debt.
- **Three ADRs now touch the same hash chain** (032, 049, 056). The chain primitive needs
  a single owner in code — ADR-032 — or it will be implemented twice.
- **This ADR is blocked on ADR-024 and ADR-032, both unimplemented.** It cannot start
  until `ExitCode` and the chain exist. Stating that plainly is the point of this bullet.

### Risks

- *Observe mode leaks by accident* → the corpus contains command strings. Mitigation:
  redaction is on by default and off only via an explicit flag; the corpus path defaults
  under the config dir with `0600`; `redacted` is a queryable field so a report can prove
  the profile was applied.
- *Users treat a clean rehearsal as a safety guarantee* → it is a statement about the
  recorded corpus and nothing else. Mitigation: `coverage.by_source` and
  `coverage.complete` are mandatory fields in every report and in the doc's first
  paragraph, per ADR-048's lesson that limitations stated quietly get over-trusted.
- *Exit code 16 collides* → the registry is contended (see the exit-code note). Mitigation:
  renumber on merge; and this ADR formally proposes that ADR-024's `ExitCode` enum become
  the registry's owner.
- *`--observe` silently alters behaviour in some path we did not think of* → test #8 and
  the `exit_code(observe) == exit_code(permissive)` property test. If that property cannot
  be made to hold, the flag should not ship.

---

## Alternatives considered

### Alternative 1 — Run a Caro daemon and mirror Phinq's proxy

Amortizes init, enables real holds, enables session velocity natively. Rejected: it
imports FM5 and FM7 wholesale, contradicts the standing "pure subprocess, no daemon, no
state" constraint, and puts Caro on the exact ground the 2026-08-20 memo says to avoid
("be the content and the verdict; do not be the console"). D1 shows batch replay gets the
init amortization anyway.

### Alternative 2 — Log to the existing telemetry pipeline instead of a new corpus

`src/telemetry/events.rs` already has `SafetyValidation` events and a collector.
Rejected: telemetry is opt-in, aggressively privacy-scrubbed (no command text by design),
and unchained. It is the wrong artifact for a calibration replay in all three respects,
and widening it would break its own privacy promise.

### Alternative 3 — Two files, like Phinq: chained decisions plus an unchained corpus

Rejected as FM1. The split exists in Phinq because arguments are the sensitive part and
the chain is the integrity part; Caro can have both by redacting before hashing. Copying
the split would copy the weakness for none of the benefit.

### Alternative 4 — `--observe` as a config setting rather than a flag

Rejected: shadow mode must be trivially reversible and visible in the invocation. A
persistent setting that silently disables enforcement is a footgun with a long fuse.
`CARO_OBSERVE=1` covers the "set it for a session" case without hiding in a TOML file.

### Alternative 5 — Emit a delta line for every record, not just changed ones

Rejected on output size. A 10,000-record corpus would emit 10,000 lines to say "nothing
happened". Counts carry the aggregate; deltas carry the news.

---

## References

- Phinq docs, read 2026-08-21: [how it works](https://www.phinq.co/docs/concepts/how-phinq-works) ·
  [risk model](https://www.phinq.co/docs/concepts/risk-model) ·
  [watch mode and enforcement](https://www.phinq.co/docs/concepts/watch-mode-and-enforcement) ·
  [approvals](https://www.phinq.co/docs/concepts/approvals) ·
  [audit log](https://www.phinq.co/docs/concepts/audit-log) ·
  [CLI](https://www.phinq.co/docs/reference/cli) ·
  [HTTP gate](https://www.phinq.co/docs/reference/http-gate) ·
  [TypeScript SDK](https://www.phinq.co/docs/reference/typescript-sdk) ·
  [limitations](https://www.phinq.co/docs/security/limitations) ·
  [repo](https://github.com/phinq-co/phinq)
- GuardFall — *"GuardFall Exposes Open-Source AI Coding Agent Safety Bypasses"*, The
  Hacker News, 2026-06-30.
- Hermes strategy memo, 2026-08-20 — §3.6 (assessment contract), §3.7 (shell-semantics
  benchmark), §3.8 (observe mode, promoted to Now), Recommendation 1.
- In-tree: ADR-007, ADR-024, ADR-032, ADR-044, ADR-045, ADR-046, ADR-049, ADR-052,
  ADR-053, ADR-055; `.claude/rules/adr-numbering.md`, `.claude/rules/validation-discipline.md`.
