# ADR-048: Anthropic Inference-Hook Verdict Adapter — `caro guard --host anthropic-hook`

- **Status**: Proposed (scope only — no implementation in this document)
- **Date**: 2026-08-11
- **Produced by**: `caro-research--scoping-process` scheduled task
- **Feature researched**: Anthropic **Inference hooks** (Claude Enterprise
  beta, ~2026-08-05) — org-operated AI security server receiving signed
  prompt frames and returning allow/deny verdicts before inference. Docs
  fetched live 2026-08-11 (platform.claude.com/docs/en/manage-claude/
  inference-hooks and …/inference-hooks-endpoint).
- **Depends on**: ADR-020 (`SuggestedRouting` tier vocabulary), ADR-036
  (`caro guard` CLI, `AssessmentEnvelope`, fail-closed-by-emission, exit-code
  registry)
- **Relates to**: ADR-035 (caro as hook host — inverse), ADR-040 (policy
  file — later supplies projection defaults), ADR-041 (lifecycle-event sink),
  ADR-043 (MCP gateway — sibling socket), ADR-046 (AEP — `request_id`
  correlation)
- **Numbering note**: highest existing is ADR-047; per
  `.claude/rules/adr-numbering.md`, renumber on merge if another 048 lands
  first.

> **Provenance note (autonomous run).** Produced with no user present; the
> task template's `[FEATURE NAME]` was unbound. Target selection: the
> 2026-08-10 and 2026-08-11 Hermes scans both name this socket the #1 build
> item; no existing ADR covers it (035 = host, 036 = client-side subprocess
> guest, 043 = MCP proxy, 046 = approval payload). Treat the analog choice
> and every de-scoping decision below as reviewable.

---

## Context

Anthropic's inference hooks give every Claude Enterprise org an allow/deny
socket in front of inference — and no reviewer to plug into it. The protocol:
Anthropic POSTs a signed JSON *prompt frame* (full conversation transcript:
`text`, `tool_use`, `tool_result`, `attachment` blocks, plus `request_id`,
`actor`, `source`, `session_id`, `model`) to an org-configured HTTPS
endpoint; the server answers within a 5,000 ms default budget with
`{"action":"allow"}` or `{"action":"deny","deny_reason":…,"reference_id":…}`.
Signing follows Standard Webhooks (HMAC-SHA256 over
`{webhook-id}.{webhook-timestamp}.{raw body}`). Unknown fields in the verdict
body are explicitly ignored, so a server may return a richer object.

The socket's documented limits are Caro's opening:

1. **Binary verdicts** — no ask tier, no redaction; approval fatigue
   (humans approve ~1 in 3 dangerous commands) has no expression here.
2. **Post-hoc for tools** — the frame containing a `tool_use` block arrives
   when the tool *result* returns; for Claude Code the command already ran.
3. **Stateless cumulative transcripts** — a naive scanner that denies on any
   dangerous block in the transcript denies that conversation *forever*.
4. **Failure ≠ deny** — non-200/unparseable responses invoke org failure
   handling, which may be fail-open; sustained failures trip a circuit
   breaker that halts enforcement.
5. **Beta drift** — shapes may change; unknown event types must be allowed.

`caro guard` (ADR-036) already defines caro-as-guest with host adapters,
verdict mapping from `SuggestedRouting`, fail-closed-by-emission, and an
exit-code registry. This ADR adds one host kind; it does not introduce a new
subsystem.

## Decision

### 1. One new host adapter, no daemon

`caro guard --host anthropic-hook` reads one prompt frame from stdin, writes
one verdict to stdout, exits. The HTTPS/TLS surface, timeouts, keep-alive,
`webhook-id` dedupe, and circuit-breaker hygiene belong to a thin wrapper
(any reverse proxy or ≤50-line handler that shells out or links
`caro-safety`), out of scope here. This keeps the reviewer testable,
transport-agnostic, and compliant with the subprocess-purity constraint —
and the same binary later answers other HTTP-callback sockets.

### 2. Frame types (new, in `src/cli/guard.rs`)

```rust
/// Tolerant by construction: unknown fields ignored (serde default),
/// unknown block/actor/source values preserved as raw variants.
#[derive(Debug, Deserialize)]
pub struct PromptFrame {
    pub r#type: String,              // "prompt" | future values → allow
    pub request_id: String,
    pub tenant_id: Option<String>,
    pub actor: Option<serde_json::Value>,   // discriminated union; pass-through
    pub source: Option<FrameSource>,
    pub messages: Vec<FrameMessage>,
    pub session_id: Option<String>,
    pub model: Option<String>,
    #[serde(default)]
    pub metadata: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct FrameMessage { pub role: String, pub content: Vec<ContentBlock> }

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum ContentBlock {
    #[serde(rename = "text")]        Text { text: String },
    #[serde(rename = "tool_use")]    ToolUse { id: String, tool_name: String,
                                               input: serde_json::Value },
    #[serde(rename = "tool_result")] ToolResult { tool_use_id: String,
                                               tool_name: Option<String>,
                                               is_error: Option<bool> },
    #[serde(rename = "attachment")]  Attachment { file_name: Option<String> },
    #[serde(other)]                  Unknown,
}
```

### 3. Suffix evaluation (the denial-loop fix, by design)

Evaluated set = command-bearing `tool_use` blocks in the **final message**,
plus, when the final message is a `user` turn containing `tool_result`
blocks, the `tool_use` blocks in the immediately preceding `assistant` turn
whose ids those results reference. Deterministic, stateless, bounded per
frame. Earlier transcript content is never re-judged, so a conversation
denied at turn N is not poisoned for turns N+1… . Residual risk: a dangerous
command in an *older* suffix was already denied when it was new; if the org
ran shadow mode at that time it goes unflagged later — acceptable, and
recorded here as the deliberate trade.

Command-bearing tools default to `{"Bash", "bash", "run_shell_command",
"shell", "exec"}`; `--tool-names a,b,c` overrides. Input string extracted
from `input.command` falling back to `input` as string. Non-command blocks
(text, attachments) are **not** scanned: caro judges command execution, not
DLP content (deliberate market boundary; Aug 10 memo).

### 4. Verdict projection and enrichment (new types in `src/safety/mod.rs`)

```rust
/// Wire verdict. Serializable from day one; schema registered.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct HookVerdict {
    pub action: HookAction,                       // Allow | Deny
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deny_reason: Option<String>,              // ≤500 chars, user-actionable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_id: Option<String>,             // caro:1:<16-hex hash>, ≤50 chars
    pub caro: CaroVerdictExt,                     // spec-legal unknown field
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CaroVerdictExt {
    pub schema_version: u32,                      // 1
    pub origin: VerdictOrigin,                    // Policy | Error | Signature
    pub decision: Option<SafetyDecision>,         // full tiered decision
    pub evaluated_blocks: u32,
    pub suffix_boundary: Option<String>,          // last-evaluated tool_use id
}
```

Projection from `SuggestedRouting` (per evaluated block; worst wins):
`Block` → deny; `HumanGate` → deny by default, `--on-human-gate allow` to
downgrade (socket has no ask tier — fail-closed default mirrors ADR-036 §3);
`AutoApprove`/`AsyncLog` → allow. `deny_reason` is the caro explanation plus
the offending command, truncated to 500 chars. `reference_id` is
content-free: BLAKE3 over `request_id + action + matched_patterns`,
hex-truncated — joinable against the org's Activity Feed without leaking
prompt content.

### 5. Signature verification (pure function, optional)

`fn verify_standard_webhook(secret, id, timestamp, signatures, body, now,
tolerance) -> Result<(), SigError>` — raw-byte HMAC, standard-alphabet
base64 (the docs call out URL-safe decoding as the classic bug), ±300 s
tolerance, constant-time compare, any-of semantics. Enabled with `--verify`;
inputs from `CARO_HOOK_WEBHOOK_ID` / `…_TIMESTAMP` / `…_SIGNATURE` /
`CARO_HOOK_SECRET` env vars so the wrapper stays a dumb pipe. Rotation
dual-acceptance is the wrapper's concern.

### 6. Fail-closed by emission + exit codes (extends ADR-036 registry)

| Exit | Meaning | stdout |
|---|---|---|
| 0 | verdict emitted (allow or deny) | `HookVerdict` JSON |
| 3 | internal error (parse, IO, panic-caught) | deny, `origin:"error"` |
| 5 | signature invalid (`--verify` only) | deny, `origin:"signature"` |
| 64 | usage error | none |

The wrapper returns HTTP 200 + stdout verbatim for exits 0/3/5. Rationale:
a caro failure surfaced as a non-200 becomes a *webhook failure*, and under
org fail-open handling the request proceeds uninspected — the exact Phase-1
failure mode 4. Emitting a deny keeps the failure inside the verdict
channel. Unknown top-level `type` values emit `allow` (docs mandate this;
erroring would feed the circuit breaker).

### 7. Session/context lifecycle

None. One frame per invocation; idempotent (identical frame → byte-identical
verdict, asserted in tests); dedupe belongs to the wrapper keyed on
`webhook-id`. Pattern compilation cost per spawn is the accepted trade
(ADR-036 measured it acceptable; the wrapper may link `caro-safety` (ADR-022)
in-process later if spawn overhead matters at org scale — that is the v2
escape hatch, not this ADR).

## Consequences

**Positive:** first-mover reviewer for a week-old socket from the
highest-credibility vendor; tiered decisions smuggled (legally) into a
binary protocol; composes with ADR-036 into a prevention+detection story no
DLP vendor tells; zero new modules; every Phase-1 failure mode answered
structurally.

**Negative / accepted:** cannot block Claude Code command *execution*
(post-hoc socket — ADR-036 remains the prevention layer; docs must say so
plainly or users will over-trust it); suffix rule can skip old shadow-mode
misses; beta drift may force parser updates (tolerance rules are the hedge);
no HTTP surface shipped means orgs need the wrapper before this is usable
end-to-end.

## Alternatives considered

- **Ship the HTTP server binary now** — rejected: violates the
  subprocess-purity constraint, drags TLS/cert/timeout surface into safety-
  critical review, and the wrapper is trivial for any adopter.
- **Whole-transcript scanning** — rejected: denial loops (failure mode 3)
  and O(10 MB) rescans; suffix evaluation is deterministic and bounded.
- **Scan text/attachment blocks for secrets/PII** — rejected: DLP is
  Netskope/Zscaler's lane and off-thesis (Aug 10 memo); we are the command
  reviewer a DLP server can also call.
- **LLM-assisted verdicts** — rejected: reintroduces the fragile
  LLM-checks-LLM dependency both Hermes memos exclude; deterministic core
  only.
- **New `src/hooks/` module** — rejected: ADR-036's guard module is the
  established home for host adapters; constraint says no new modules unless
  unavoidable.

## Integration tests (deterministic)

See scope doc §Phase 3: nine cases covering allow/deny, denial-loop
regression, forward-compat (unknown event/block/source/actor), malformed
stdin → exit 3, signature vectors → exit 0/5, `--on-human-gate` both ways,
and a 10 MB perf smoke. Golden files under `tests/fixtures/anthropic_hook/`.

## Out of scope (v2+)

HTTP/TLS wrapper binary; shadow-mode analytics; response-side hook events;
verdict persistence/dedup; DLP content scanning; redaction (protocol can't);
policy-file-driven tool sets (ADR-040 wiring); secret-rotation windows;
EU AI Act Art. 12/14 mapping doc (separate item C, Aug 11 memo);
`caro-safety` in-process wrapper linkage.

## References

- Anthropic Inference hooks overview + endpoint docs (fetched 2026-08-11)
- Standard Webhooks specification — standardwebhooks.com
- Hermes scans 2026-08-10 (§B, top build item) and 2026-08-11 (§4, unchanged)
- ADR-020, ADR-022, ADR-035, ADR-036, ADR-040, ADR-041, ADR-043, ADR-046
- Companion scope: `caro-scope-inference-hook-verdict-2026-08-11.md`
