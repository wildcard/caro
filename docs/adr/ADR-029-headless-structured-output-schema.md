# ADR-029: Headless Schema-Enforced Structured Output — Caller-Supplied JSON Schema with Typed Failure Provenance and a Constrained-Decoding Capability Hook

- **Status**: Proposed
- **Date**: 2026-07-02
- **Authors**: caro-research scoping process (automated scheduled run)
- **Target**: Hybrid (Community CLI + Enterprise CI / agent wrappers / `jq` pipelines)
- **Builds on / relates to**: ADR-024 (Headless JSON/NDJSON output contract),
  ADR-025 (Headless init snapshot cache), ADR-026 (Headless multi-turn agentic
  NDJSON session), ADR-027 (Headless permission resolution contract), ADR-028
  (Headless token streaming)

> **Provenance note (autonomous run).** Produced by the
> `caro-research--scoping-process` scheduled task, whose template left
> `[FEATURE NAME]` unbound and ran with no user present. The five prior runs
> scoped the headless output contract (ADR-024), init snapshot cache (ADR-025),
> multi-turn NDJSON session (ADR-026), permission resolution (ADR-027), and
> token streaming (ADR-028). Across those five, **`--json-schema` enforced
> structured output** is now the single most-repeatedly-deferred dimension —
> explicitly punted in ADR-024, ADR-025, ADR-026, and ADR-028 ("v1 *publishes*
> a schema; it does not *enforce* a caller's. Hooks are left (the `schemars`
> derive) so v2 can add it"). This run targets exactly that dimension. The
> competitor analog is Claude Code's headless `--json-schema` flag and the
> Agent SDK's `outputFormat: {type: "json_schema"}` structured-outputs feature.
> Treat the analog choice as a reviewable assumption, not a settled decision.

---

## Context

### Phase 1 — what the analog does, and where it is still rough

Claude Code headless mode accepts a caller-supplied JSON Schema:

```bash
claude -p "List countries with capitals" \
  --output-format json \
  --json-schema countries.schema.json
```

The result payload then carries a `structured_output` field with data validated
against the schema ([headless docs](https://code.claude.com/docs/en/headless)).
The Agent SDK exposes the same mechanism as `outputFormat` / `output_format`
with Zod/Pydantic sugar ([structured-outputs
docs](https://code.claude.com/docs/en/agent-sdk/structured-outputs)). The
history matters: users asked for **constrained decoding** — guaranteed
schema compliance at token-generation level, like OpenAI's `strict: true`
([claude-code#9058](https://github.com/anthropics/claude-code/issues/9058),
Oct 2025) — and what shipped instead is **post-hoc validation with bounded
re-prompting**. The docs are explicit: *"Schema validation happens after
Claude finishes. It's not constrained generation during inference."*

Phase-1 research surfaced four rough edges that define the failure modes to
solve by design:

1. **Ambiguous failure provenance.** On retry exhaustion the result subtype is
   `error_max_structured_output_retries` — but the SDK docs warn the *same*
   subtype fires when a model fallback "retract[s] an already-completed output
   mid-stream" with no validation failure at all. The consumer must go inspect
   a separate `errors` field to tell "your schema is too hard" apart from
   "infra retracted the answer". Two unrelated causes share one terminal state.
2. **Silent absence on flag mis-combination.** `--json-schema` only produces
   `structured_output` when `--output-format json` is also passed; in text
   mode the field is silently missing. Same activation-coupling trap as the
   three-flag streaming friction ADR-028 catalogued: miss a verbosity flag,
   silently get nothing.
3. **Provider-defined schema subset, remotely validated.** The supported JSON
   Schema feature set is a documented-but-partial subset ("see JSON Schema
   limitations"), enforcement lives server-side, and an unsupported construct
   surfaces late — after a paid, non-deterministic inference round-trip. There
   is no offline pre-flight "is my schema acceptable" check.
4. **No constrained decoding — structurally.** Claude Code drives a remote API;
   it *cannot* mask logits during sampling. Validate-and-re-prompt is the only
   lever a remote-API wrapper has. #9058 asked for the guarantee; the
   architecture cannot deliver it.

Session/context lifecycle: schema handling is per-invocation (schema is read,
sent with the request, validated on the result); nothing is cached across
calls, and each retry replays the full context — acceptable for a paid API
wrapper, expensive for a local model without ADR-025's snapshot reuse.

### Phase 2 — what caro already has, and what only caro can do

Codebase inventory (2026-07-02):

| Capability | Exists today | File |
| --- | --- | --- |
| Result envelope + `schema_version` | ✅ ADR-024 envelope | `src/cli/mod.rs` |
| `schemars` dep + schema-gen binary | ✅ `schema_for!(UserConfiguration)` | `Cargo.toml`, `src/bin/generate-schema.rs` |
| Model-side JSON discipline | ✅ `CommandOutput {cmd}` + `format_chat_json()` primes `{"cmd": "` | `src/prompts/smollm_prompt.rs` |
| Malformed-output recovery | ✅ JSON-extraction fallback, `PromptResponse` enum, `GeneratorError::ParseError` | `src/prompts/smollm_prompt.rs`, `src/backends/mod.rs` |
| Safety validator (52+ patterns) | ✅ `SafetyValidator::validate_command` | `src/safety/mod.rs` |
| Backend capability reporting | ✅ `BackendInfo` (+ ADR-028's `supports_streaming` precedent) | `src/backends/mod.rs` |
| Deterministic offline test backend | ✅ static/mock backend | `src/backends/static_matcher.rs` |

What their implementation gets right, to replicate: post-hoc validation with
**bounded, violation-fed re-prompting** (feed the validator's error list back
to the model — cheap, effective); a **distinct terminal state** for schema
failure rather than a generic error; **schema-as-file or inline** ergonomics;
Zod/Pydantic-style "derive the schema from your types" (caro users get this
for free via `schemars`).

What only caro can do:

- **True constrained decoding is architecturally reachable.** caro's embedded
  backend owns the sampler. Grammar-constrained sampling (GBNF/logit masking)
  can *guarantee* schema conformance at generation time — the exact thing
  #9058 asked for and a remote-API wrapper structurally cannot ship. v1 does
  not wire it (see out-of-scope) but designs the contract so it lands
  additively: an `enforcement` provenance field and a
  `supports_constrained_output` capability flag, mirroring ADR-028's honest
  `supports_streaming` pattern.
- **Offline, deterministic, full-draft validation.** Validation runs locally
  via the `jsonschema` crate (JSON Schema draft 2020-12, pure Rust, MIT):
  schema problems are caught by compiling the schema *before any inference*,
  not after a round-trip. No provider subset, no network.
- **Safety advisory scan over structured output.** caro's 52+ pattern
  validator can sweep every string leaf of the structured output and attach
  advisory risk findings — a shell-safety layer over caller-shaped data that
  no generic agent CLI offers.

### The failure mode we must design around

The analog's compound hazard is **untyped, ambiguous failure**: one error
subtype for two unrelated causes, silent field absence on flag
mis-combination, and schema acceptability discovered only after inference.
Machines branching on the result cannot distinguish "fix your schema" from
"retry the run" from "you forgot a flag". We solve this by design: one flag
that fully activates the feature or exits loudly, schema compilation before
any model work, a typed `cause` enum, and a dedicated exit code.

---

## Decision

Introduce **headless schema-enforced structured output**: a `--json-schema
<path|inline-json>` flag that adds a caller-shaped, locally-validated
`structured_output` field to the ADR-024 result envelope, produced by
schema-injected prompting with bounded violation-fed retries, and reported
with typed provenance. Five design commitments:

### 1. One flag, loud activation — no silent absence

`--json-schema` is the only switch. It requires `--output json` or `--output
ndjson`; combined with `--output plain` (or yaml) caro exits `2` with
`error.kind = "schema_requires_json_output"` — never a silently missing field
(closes Phase-1 rough edge 2). The argument is a file path or an inline JSON
string (auto-detected: leading `{`/`[` ⇒ inline), matching the analog's
ergonomics.

### 2. Compile the schema before any inference

At startup the schema is parsed and compiled via `jsonschema` (draft 2020-12).
A malformed or unsupported schema (including any **external `$ref`** — caro is
offline; remote refs are rejected) exits `2` with `error.kind =
"schema_invalid"` and a pointer to the offending construct, **before** the
model loads (and before an ADR-025 snapshot is even read). Closes rough edge 3:
pre-flight is offline and free.

### 3. Post-hoc validation with violation-fed bounded retries; typed provenance

v1 enforcement is post-hoc (like the analog), but honest about it and typed
about failure:

- The prompt builder injects the schema plus a "respond with JSON matching
  this schema" fragment (extending the existing `format_chat_json()`
  JSON-priming discipline).
- The response is parsed (reusing the existing JSON-extraction fallback) and
  validated locally. On violation, up to `--schema-retries N` (default 2)
  re-prompts are issued, each carrying the concrete violation list
  (`instance_path`, `schema_path`, `message`).
- Success ⇒ `structured_output` (raw `serde_json::Value`) +
  `structured_output_meta { enforcement: "post_hoc", attempts, advisory_findings }`.
- Exhaustion ⇒ exit `8`, `status: "schema_failed"`, and a typed
  `structured_output_error` whose `cause` is an enum —
  `retries_exhausted` (model kept violating; the violation list is attached)
  vs `generation_failed` (backend/parse error before validation ever ran).
  Two causes, two variants, machine-distinguishable (closes rough edge 1).

### 4. The command stays canonical and safety-gated; structured output is scanned, not trusted

Non-negotiable invariant: `payload.command` remains the **only** authoritative
command and always passes the `SafetyValidator` + ADR-027 permission
resolution, schema or no schema. `structured_output` is a *supplementary
view*; the contract documents that executing strings out of it bypasses caro's
gate. As defense-in-depth, every string leaf of the validated output is swept
by the existing pattern validator; hits are attached as **advisory**
`advisory_findings: [{json_path, risk_level, pattern_id}]` in
`structured_output_meta` — they never change the exit code, but a CI caller
can branch on them. This is the "superior, not equivalent" layer.

### 5. Constrained decoding is a declared capability, not a promise

`BackendInfo` gains `supports_constrained_output: bool` (default `false` for
every backend in v1), and `structured_output_meta.enforcement` is an enum
(`post_hoc` today, `constrained` reserved). When GBNF/logit-masking lands for
the embedded backend (v2), it flips the flag and the provenance value — zero
contract change, and consumers can already tell guaranteed output from
best-effort output. Same honest-capability pattern ADR-028 established for
`supports_streaming`.

---

## New types (all `Serialize + Deserialize + JsonSchema` from day one)

In `src/cli/mod.rs` (beside the ADR-024 envelope types):

```rust
/// How the structured output's schema conformance was achieved.
#[derive(Serialize, Deserialize, JsonSchema, Clone, Copy, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SchemaEnforcement { PostHoc, Constrained }

/// One concrete schema violation (subset of jsonschema's error, stabilized).
#[derive(Serialize, Deserialize, JsonSchema, Clone)]
pub struct SchemaViolation {
    pub instance_path: String, // JSON Pointer into the candidate output
    pub schema_path: String,   // JSON Pointer into the caller schema
    pub message: String,
}

/// Advisory safety finding inside structured output (never affects exit code).
#[derive(Serialize, Deserialize, JsonSchema, Clone)]
pub struct AdvisoryFinding {
    pub json_path: String,     // JSON Pointer to the string leaf
    pub risk_level: RiskLevel, // reused from src/models
    pub pattern_id: String,    // reused pattern naming from src/safety/patterns.rs
}

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
pub struct StructuredOutputMeta {
    pub enforcement: SchemaEnforcement,
    pub attempts: u8,                       // 1 = first try succeeded
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub advisory_findings: Vec<AdvisoryFinding>,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
#[serde(rename_all = "snake_case", tag = "cause")]
pub enum StructuredOutputError {
    RetriesExhausted { attempts: u8, violations: Vec<SchemaViolation> },
    GenerationFailed { attempts: u8, detail: String },
}
```

Envelope additions (additive under `schema_version: "1"` per ADR-024's
stability promise — fields only added, never renamed):

```rust
#[serde(skip_serializing_if = "Option::is_none")]
pub structured_output: Option<serde_json::Value>,
#[serde(skip_serializing_if = "Option::is_none")]
pub structured_output_meta: Option<StructuredOutputMeta>,
#[serde(skip_serializing_if = "Option::is_none")]
pub structured_output_error: Option<StructuredOutputError>,
```

In `src/backends/mod.rs`: `BackendInfo` gains
`#[serde(default)] pub supports_constrained_output: bool`. No new
`GeneratorError` variant — `ParseError`/`GenerationFailed` already cover the
pre-validation failures; validation failures are a runner-level concern, not a
backend error.

Method contracts: `SchemaViolation::from(&jsonschema::ValidationError)` (the
stabilization boundary — the crate's error type never leaks into the
envelope); `fn scan_value_leaves(&SafetyValidator, &Value) ->
Vec<AdvisoryFinding>` as a free helper in `src/safety/mod.rs` reusing the
compiled pattern set.

New dependency: `jsonschema` (MIT, pure Rust). Per
`.claude/rules/external-sdk-integration.md` this is small enough to skip the
full spike PR, but the implementation PR's first commit must still record the
license check, MSRV (≥ crate's `rust-version`, currently 1.83) verification,
and `cargo deny check licenses` for transitives. Disable its network-resolver
features (`resolve-http` off) so external `$ref` cannot dial out.

---

## Minimal set of files to change (no new `src` modules)

1. **`src/cli/mod.rs`** — `--json-schema <arg>` + `--schema-retries <n>` on
   the arg struct; the five types above; envelope field additions; the
   inline-vs-path detection.
2. **`src/main.rs`** — compile schema at startup (exit 2 paths); thread the
   compiled schema into the generation call; map outcome → exit code 8 at the
   single existing envelope-emit site (ADR-024).
3. **`src/prompts/mod.rs`** — `fn schema_instruction(schema: &Value) -> String`
   and `fn schema_retry_instruction(violations: &[SchemaViolation]) -> String`,
   composed into the existing prompt builders (extends, does not fork,
   `format_chat_json()`).
4. **`src/backends/mod.rs`** — `supports_constrained_output` on `BackendInfo`
   (defaulted; every v1 backend reports `false`).
5. **`src/safety/mod.rs`** — `scan_value_leaves` helper reusing the compiled
   pattern set (no new patterns, no validator changes).
6. **`docs/headless-contract.md`** — exit 8 row, the three new envelope
   fields, the "structured output is not safety-gated for execution"
   invariant, and the `enforcement` provenance semantics.
7. **`tests/structured_output_contract.rs`** — new integration-test *binary*
   (a `tests/` file is a separate crate, not a `src` module).

Reused unchanged: `SafetyValidator`, pattern set, `ConfigManager`,
`GeneratedCommand`, ADR-024 envelope machinery, ADR-025 snapshot, ADR-026
session events, ADR-027 permission resolution, static/mock backend.

---

## Exit-code / output contract (what machines depend on)

Extends the ADR-024 table (0–6) and ADR-027 (7) under `schema_version: "1"`:

| Exit | Meaning | Envelope signal |
| --- | --- | --- |
| 0 | generated; if `--json-schema` given, `structured_output` present and valid | `structured_output` + `structured_output_meta` |
| 2 | usage: schema malformed / external `$ref` / wrong output format / unreadable file | `error.kind ∈ {schema_invalid, schema_requires_json_output}` — **emitted before any inference** |
| 8 | schema enforcement failed after retries, or generation failed under schema mode | `status: "schema_failed"`, `structured_output_error.cause ∈ {retries_exhausted, generation_failed}` |

Promises: exit 8 is reserved exclusively for structured-output failure; the
`cause` tag is stable; `structured_output` is either absent (flag not given)
or schema-valid (exit 0) — there is no state where it is present but invalid;
`advisory_findings` never affects the exit code; all three new fields are
additive and absent when the flag is unused, so every existing ADR-024
consumer is byte-compatible.

---

## Integration tests (known inputs → deterministic JSON + exit code)

`tests/structured_output_contract.rs`, driven through the static/mock backend
(offline, byte-reproducible), mock seeded per-case:

1. **Happy path** — schema `{type: object, properties: {cmd: {type: string}}, required: [cmd]}`, mock emits conforming JSON ⇒ exit 0; `structured_output_meta == {enforcement: "post_hoc", attempts: 1}`; envelope asserted byte-exact.
2. **Pre-flight schema rejection** — inline schema `{"type": "nope"}` ⇒ exit 2, `error.kind = "schema_invalid"`; assert the mock backend was **never invoked** (no inference before pre-flight).
3. **External `$ref` rejection** — schema with `"$ref": "https://…"` ⇒ exit 2, `schema_invalid`, offline guarantee upheld.
4. **Flag mis-combination** — `--json-schema … --output plain` ⇒ exit 2, `error.kind = "schema_requires_json_output"`.
5. **Retries then success** — mock emits a violating payload once, conforming on 2nd call ⇒ exit 0, `attempts: 2`; assert the retry prompt contained the violation's `instance_path`.
6. **Retries exhausted** — mock always violates, `--schema-retries 1` ⇒ exit 8, `cause: "retries_exhausted"`, `attempts: 2`, non-empty `violations` with stable paths.
7. **Generation failure under schema mode** — mock returns unparseable text every attempt ⇒ exit 8, `cause: "generation_failed"` (distinguishable from case 6 — the analog's ambiguity, disproven by test).
8. **Advisory scan** — conforming output containing string leaf `"rm -rf /"` ⇒ exit 0 (advisory only) with `advisory_findings[0].risk_level == "critical"`; `payload.command` untouched and still safety-gated.
9. **Determinism** — case 1 run twice ⇒ identical bytes (ADR-024 property preserved).

---

## Consequences

### Positive

- Clears the dimension deferred by four prior ADRs, completing the headless
  surface: envelope (024) + warm init (025) + multi-turn (026) + permissions
  (027) + streaming (028) + **caller-shaped output (029)**.
- Fail-fast offline schema pre-flight and typed failure provenance — both
  impossible or unshipped in the analog — plus the advisory safety sweep no
  competitor offers.
- Pure subprocess, no daemon, no state: schema in argv, result on stdout,
  verdict in the exit code; composes with `--session` and the ADR-025 cache
  without touching either.
- Contract-ready for constrained decoding: when GBNF lands, only a bool and an
  enum value change.

### Trade-offs / risks

- Post-hoc + retry burns local inference time on small models that struggle
  with big schemas; mitigated by fail-fast pre-flight, default 2 retries, and
  documented "keep schemas focused" guidance (same as the analog's).
- One new dependency (`jsonschema`); mitigated by the license/MSRV/deny
  checklist in the first commit and disabled network features.
- Advisory scan may surface false-positive-ish findings on prose fields
  (e.g., an explanation *describing* `rm -rf`); acceptable because advisory
  findings never gate — documented, and pattern IDs let consumers filter.
- Small-model schema fidelity is unproven; `attempts` telemetry in the
  envelope gives the eval suite (`src/eval/`) a direct success-rate metric.

## Out of scope (explicitly deferred)

- **Wiring constrained decoding** (GBNF / logit masking in the embedded
  backend). The contract hooks (`enforcement`, `supports_constrained_output`)
  ship now; the sampler work is its own PR against `src/backends/embedded/`.
- **Per-turn schemas in ADR-026 multi-turn sessions.** v1 applies one schema
  per invocation; per-turn schema switching is additive later.
- **Schema-aware streaming** (validating or shaping ADR-028 deltas). Deltas
  stay advisory text; only the final result is schema-checked.
- **Caller schemas over the envelope itself.** The envelope is caro-owned
  (ADR-024); `--json-schema` shapes only the `structured_output` field.
- **Zod/Pydantic-style client sugar.** `schemars` already covers Rust callers;
  other-language sugar belongs in wrapper libraries, not the CLI.
- **Cost/usage accounting** — still deferred from ADR-024; unaffected here.

## Alternatives considered

1. **Copy the analog exactly (validate-retry, single opaque error subtype).**
   Rejected: reproduces the ambiguous-provenance failure mode we found in
   Phase 1; typed `cause` costs one enum.
2. **Wait and ship constrained decoding directly (skip post-hoc v1).**
   Rejected: sampler-level grammar work is backend-specific and long-pole;
   the contract, validation, retries, and tests are backend-agnostic and
   deliver value now. Shipping post-hoc first with an `enforcement` field is
   honest and additive.
3. **Constrain the model to the caller schema *instead of* the `{cmd}`
   discipline, replacing the command payload.** Rejected: breaks the
   safety invariant — `payload.command` must remain the single gated,
   canonical artifact. Structured output is a view, never the command.
4. **Validate with `schemars`' generated-schema tooling only (no `jsonschema`
   dep).** Rejected: `schemars` *generates* schemas from types; it does not
   *validate* arbitrary caller schemas against instances. Wrong tool.
5. **Hard-fail on advisory findings (exit non-zero on dangerous string
   leaves).** Rejected: structured output is data, not an execution request;
   hard-failing on prose mentioning `rm -rf /` makes the feature unusable for
   safety-reporting use cases (the validator describing danger *is* the
   payload). Advisory-only, with pattern IDs for caller-side policy.

## References

- Claude Code headless mode — `--json-schema`: https://code.claude.com/docs/en/headless
- Agent SDK structured outputs (retry semantics, `error_max_structured_output_retries`, model-fallback retraction): https://code.claude.com/docs/en/agent-sdk/structured-outputs
- Feature request for constrained decoding, claude-code#9058: https://github.com/anthropics/claude-code/issues/9058
- API structured outputs (provider-side JSON Schema limitations): https://platform.claude.com/docs/en/build-with-claude/structured-outputs
- ADR-024 §Out-of-scope, ADR-025/026/028 deferrals — the repeated punt this ADR clears
- `.claude/rules/external-sdk-integration.md` — dependency checklist applied to `jsonschema`
