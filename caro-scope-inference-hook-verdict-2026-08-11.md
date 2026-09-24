# Implementation Scope — Anthropic Inference-Hook Verdict Adapter

**Feature under analysis:** Anthropic **Inference hooks** (Claude Enterprise
beta, launched ~2026-08-05) — every governed prompt is routed as a signed
HTTPS POST ("prompt frame") to an org-operated *AI security server*, which
returns an allow/deny verdict before inference proceeds. Docs fetched live
2026-08-11:
[overview](https://platform.claude.com/docs/en/manage-claude/inference-hooks) ·
[endpoint/integration](https://platform.claude.com/docs/en/manage-claude/inference-hooks-endpoint).

**Equivalent we are scoping for Caro:** `caro guard --host anthropic-hook` — a
pure-subprocess verdict engine that reads a prompt frame on stdin, extracts
command-bearing `tool_use` blocks, runs them through the existing
`SafetyValidator`, and emits a spec-conformant verdict JSON on stdout. The
HTTPS transport is a thin generic wrapper, explicitly out of scope for v1.

**Date:** 2026-08-11 · **Author:** `caro-research--scoping-process` (autonomous run)
**Companion ADR:** `docs/adr/ADR-048-anthropic-inference-hook-verdict-adapter.md`

> **Provenance note (autonomous run).** The task template left `[FEATURE NAME]`
> unbound and ran unattended. Target selection: the 2026-08-10 and 2026-08-11
> Hermes scans **both** name the inference-hook policy-server socket the #1
> build item ("the same ≤100-LOC `SafetyValidator` wrapper serves both …
> converging evidence, same artifact"). Coverage check: ADR-035 is caro as
> *host* of hooks; ADR-036 is caro as subprocess *guest* of Claude Code/Gemini
> CLI hooks; ADR-043 is the MCP stdio proxy; ADR-046 is the approval payload.
> No ADR covers the org-level HTTPS-callback socket or its frame/verdict
> schemas. The analog choice is a reviewable assumption.

---

## Phase 1 — Feature Research

### What problem it solves, and for whom

Enterprises don't trust vendor-side guardrails alone. Inference hooks let a
Claude Enterprise org route every governed request (claude.ai, Cowork, Claude
Code — web, desktop, CLI) through its **own** policy server before the model
sees it. Primary buyer: security/compliance teams (inline DLP is the flagship
use case); the *developer* audience is whoever builds the AI security server —
which is exactly the artifact Caro can be.

### Core architecture (verified against live docs)

- **Data flow:** user submits prompt → Anthropic sends signed HTTPS POST (the
  *prompt frame*) to the org's configured endpoint → server answers within the
  verdict timeout (default 5,000 ms; range 1–10,000 ms) → `allow` proceeds,
  `deny` rejects with a user-visible reason; denial recorded in the
  org's compliance Activity Feed.
- **Prompt frame** (request body): `type` (only `"prompt"` today),
  `request_id` (= `webhook-id` header, idempotency key), `tenant_id`,
  `actor {type:"user", id, email_address}`, `source {application}`
  (`claude-ai` | `claude-code` | `config-test`, open enum), `messages[]`
  (transcript up to inference point), `session_id`, `model`, `metadata`
  (reserved, empty).
- **Content blocks:** `text`, `tool_use {id, tool_name, input}`,
  `tool_result {content, is_error, tool_name, tool_use_id}`,
  `attachment {file_name, media_type, size_bytes, text}`. Raw bytes never
  sent; system prompts, tool definitions, and hidden reasoning never sent.
  Transcripts untruncated, up to **10 MB**.
- **Verdict:** HTTP 200 + `{"action":"allow"}` or
  `{"action":"deny","deny_reason":"≤500 chars","reference_id":"≤50 chars [A-Za-z0-9._:/-]"}`.
  **Unknown fields in the verdict body are ignored** — a server may return a
  richer object alongside. A deny is never discarded over formatting (oversize
  reason truncated, bad reference_id dropped); but any non-200, unparseable
  body, or unknown `action` is a **webhook failure**, not a deny.
- **Signing:** Standard Webhooks — `webhook-id`, `webhook-timestamp` (±5 min
  tolerance), `webhook-signature` (`v1,<base64>` HMAC-SHA256 over
  `{id}.{timestamp}.{raw body}`); secret is base64 after `whsec_` prefix
  (standard alphabet — URL-safe decoders derive wrong keys). Verify raw bytes,
  constant-time compare, accept any of the space-separated signatures.
- **Session/context lifecycle:** stateless request/response; one retry (100 ms
  delay) only on connection failure, same `webhook-id`/signature; keep-alive
  connections encouraged; dedupe on `webhook-id`. No redundant-initialization
  machinery beyond that — each frame carries the whole transcript.

### Why it is beta/limited — failure modes

1. **Binary verdicts, whole-request granularity.** No `ask` tier, no
   redaction/rewrite. The market's own number (AI Digest, Aug 10 scan): humans
   approve ~1 in 3 dangerous agent commands — and this socket can't even route
   to a human; it can only allow or hard-deny the entire request.
2. **Post-hoc for tool execution.** The frame that first contains a
   `tool_use` block arrives when the **tool result returns** (docs' flow
   diagram, step 5) — for Claude Code the shell command has already executed
   on the user's machine. The hook blocks *continuation*, not *execution*.
   Pre-execution blocking still requires a client-side hook (ADR-036).
3. **Statelessness × cumulative transcript = denial loops.** A dangerous
   block seen at turn N stays in the transcript forever; a naive
   whole-transcript scanner re-denies every subsequent frame of that
   conversation permanently.
4. **Fail-open is one org toggle away**, and an oversized body rejected by
   the server's own proxy (nginx 1 MB default vs 10 MB frames) counts as a
   webhook failure — under "allow" failure handling the prompt reaches the
   model uninspected. Sustained failures trip a circuit breaker that halts
   enforcement entirely.
5. **Beta contract drift:** field names/shapes may change before GA; legacy
   aliases already exist; unknown event types must be *allowed*, not errored.

### Structured output contract (what machines depend on)

Request: prompt frame above. Response: ≤64 KiB, uncompressed, HTTP 200,
verdict JSON. No exit codes (HTTP-level protocol); the subprocess equivalent
we define must supply its own (Phase 3).

---

## Phase 2 — Competitive Differentiation

### What they get right (replicate)

- **Forward compatibility as a hard rule** — ignore unknown fields/blocks/
  sources; *allow* unknown event types rather than error. We adopt this
  verbatim in the frame parser.
- **Asymmetric robustness:** a deny survives formatting problems; an error is
  never a deny. Clean separation of "verdict" from "failure" channels.
- **Raw-byte HMAC + idempotency via `webhook-id`** — correct Standard
  Webhooks usage, documented against the two classic bugs (re-encoded body,
  URL-safe base64).
- **Rollout ergonomics:** shadow mode, percentage rollout, role exclusions —
  vocabulary worth reusing when we later document deployment.

### Their gaps we avoid by design (schema-first)

- **No decision tiers.** Caro's `SafetyDecision` already carries
  `risk_level`, `suggested_routing`, `matched_patterns`, `confidence`. We
  project tiers onto the binary wire verdict deterministically *and* embed
  the full decision in the verdict body (spec-legal: unknown fields ignored),
  so org tooling reading our verdicts gets tiering the socket itself lacks.
- **Denial loops.** We evaluate only the **frame suffix** (deterministic:
  the final message, plus the immediately preceding assistant turn when the
  final user turn carries `tool_result` blocks referencing it). Stateless,
  idempotent, no permanent conversation poisoning. Residual risk documented
  in the ADR.
- **Whole-transcript O(10 MB) rescans.** Suffix evaluation bounds work per
  frame regardless of conversation length.
- **LLM-in-the-loop reviewers.** The week's competing pattern
  (Coldtea-style agents-watching-agents) is the fragile architecture both
  Hermes memos exclude; our reviewer is the deterministic 52+-pattern core.

### Our unique positioning

- **Same core, many sockets.** One `SafetyValidator` already scoped to answer
  Claude Code/Gemini hooks (ADR-036, pre-execution) and the MCP boundary
  (ADR-043). This adapter adds the org-level Anthropic socket. Finding worth
  stating plainly: **the inference hook cannot block local command execution
  before it happens; `caro guard` can.** They compose — client-side caro for
  prevention, org-side caro for detection/policy — a story neither a DLP
  vendor nor Anthropic tells.
- **Offline, deterministic, subprocess-pure.** No model call, no network
  dependency in the verdict path, sub-millisecond pattern matching against a
  5,000 ms budget.
- **Command-execution judgment, not DLP.** Per the Aug 10 memo we do not
  chase PII masking; Netskope/Zscaler own that lane. We are the shell-command
  reviewer those DLP servers can *also* call.

### Existing infrastructure that already covers part of this

`SafetyValidator` + `SafetyDecision`/`SuggestedRouting` (src/safety/mod.rs);
ADR-036's `caro guard` CLI shape, `AssessmentEnvelope`, fail-closed-by-
emission pattern, and exit-code registry; ADR-041 lifecycle events (audit
sink); ADR-046 AEP (`request_id` correlation vocabulary); ADR-040 policy file
(tier→action mapping the projection knob defaults can later read from).

---

## Phase 3 — Scope Definition

Full contract in **ADR-048**. Summary:

### Deliverable

`caro guard --host anthropic-hook`: stdin = prompt frame JSON → stdout =
verdict JSON, one shot, no daemon, no state. Optional `--verify` mode checks a
Standard Webhooks signature from env (`CARO_HOOK_WEBHOOK_ID`,
`CARO_HOOK_WEBHOOK_TIMESTAMP`, `CARO_HOOK_WEBHOOK_SIGNATURE`,
`CARO_HOOK_SECRET`) against the raw stdin bytes before parsing.

### Verdict projection (deterministic)

| `SuggestedRouting` | Wire `action` | Notes |
|---|---|---|
| `AutoApprove`, `AsyncLog` | `allow` | |
| `HumanGate` | `deny` by default; `--on-human-gate allow` opts out | socket has no ask tier; fail-closed default mirrors ADR-036 |
| `Block` | `deny` | `deny_reason` = caro explanation (≤500 chars, user-actionable) |

Every verdict body also carries `caro: {schema_version, decision:
SafetyDecision, evaluated_blocks, suffix_boundary}` — spec-legal enrichment.
`reference_id` = `caro:1:<16-hex BLAKE3 of request_id + action + matched_patterns>`
(≤50 chars, charset-conformant, content-free).

### Exit codes (extends ADR-036 registry; `schema_version: 1`)

| Exit | Meaning | stdout |
|---|---|---|
| 0 | verdict emitted (allow **or** deny) | verdict JSON |
| 3 | internal error → fail-closed | deny verdict, `caro.origin:"error"` |
| 5 | signature verification failed (only with `--verify`) | deny verdict, `caro.origin:"signature"` |
| 64 | usage error (bad flags) | none |

A wrapper maps exit 0 → HTTP 200 + body verbatim; exits 3/5 are still HTTP
200 + the emitted deny (fail-closed by emission — a caro crash must not
become a webhook failure that fail-open orgs sail through).

### Files changed (minimal set, no new modules)

1. `src/cli/guard.rs` — add `HostKind::AnthropicHook`, frame types
   (`PromptFrame`, `FrameMessage`, `ContentBlock` — tolerant serde with
   `#[serde(other)]`/untagged fallbacks), suffix extraction, command-bearing
   tool detection (`Bash`, `run_shell_command`, `shell`, configurable set),
   verdict emission, webhook verification fn (~300 LOC)
2. `src/safety/mod.rs` — `HookVerdict` + `CaroVerdictExt` types, serialize-
   from-day-one via serde + schemars (~70 LOC)
3. `src/main.rs` — `--host anthropic-hook`, `--on-human-gate`, `--verify`
   flags (~15 LOC)
4. `src/bin/generate-schema.rs` — register `HookVerdict` (~5 LOC)
5. `tests/guard_anthropic_hook.rs` — new integration test file

### Integration tests (known input → deterministic JSON + exit code)

1. Frame with benign `tool_use` (`ls -la`) → `allow`, exit 0.
2. Frame whose final assistant turn contains `rm -rf /` `tool_use` →
   `deny`, reason cites pattern, exit 0; **byte-identical on rerun**.
3. Dangerous block in *old* suffix, benign final message → `allow`
   (denial-loop regression test).
4. `type:"future-event"` → `allow` (forward-compat rule), exit 0.
5. Unknown block types / `source.application` / `actor.type` → parsed,
   ignored, exit 0.
6. Malformed JSON on stdin → deny + `origin:"error"`, exit 3.
7. `--verify` with doc's sample secret/signature vectors: valid → exit 0;
   tampered body → deny + `origin:"signature"`, exit 5; timestamp skew
   > 300 s → exit 5.
8. `HumanGate` command with default flags → deny; with
   `--on-human-gate allow` → allow + full decision in `caro` ext.
9. 10 MB frame → completes within budget (perf smoke, non-CI-gating).

### Out of scope (next version)

HTTP/TLS server binary and any daemon; circuit-breaker/retry handling
(Anthropic-side); shadow-mode analytics; text/attachment (DLP) scanning;
redaction; response-side hook events (don't exist yet); persistent
verdict store or `webhook-id` dedupe (wrapper concern); Activity Feed /
Compliance API joins; ADR-040 policy-file-driven tool-name sets; signing-
secret rotation dual-acceptance (wrapper concern); EU AI Act Art. 12/14
field-mapping doc (tracked separately per Aug 11 memo item C).

### Constraint compliance

Reuses `SafetyValidator`/`SuggestedRouting`/guard CLI — no duplication. All
new types `Serialize + Deserialize + JsonSchema` from day one. Pure
subprocess: one stdin read, one stdout write, exit. The Phase-1 failure modes
are solved by design: tiering via projection + enrichment (mode 1),
composition with ADR-036 documented (mode 2), suffix evaluation (mode 3),
fail-closed by emission (mode 4), tolerant parsing + allow-on-unknown-event
(mode 5).

---

*Method notes: non-interactive scheduled run. Anthropic docs fetched live
2026-08-11; beta contract may drift — the frame parser's tolerance rules are
the hedge. No write actions beyond this report and the companion ADR draft;
nothing committed (git-workflow.md requires a feature branch; a maintainer
should review, then land both via `bin/sk-new-feature`).*
