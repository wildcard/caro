# ADR-016: Guardian Mode — LLM-Assisted Review for Borderline Commands

**Status**: Proposed

**Date**: 2026-03-25

**Authors**: @wildcard

**Target**: Community

**Depends on**:
- [ADR-015](./ADR-015-universal-agent-integration-protocol.md) — Universal Agent Integration Protocol (AssessmentPayload, validate subcommand)
- [ADR-010](./ADR-010-bubblewrap-sandbox-execution.md) — Sandbox execution

## Context

### The Limitation of Pure Pattern Matching

Caro's current safety system uses 52 pre-compiled regex patterns to detect dangerous commands.
This works excellently for clear-cut cases (`rm -rf /`, `mkfs`, fork bombs) but has a fundamental
gap: **context-blind binary decisions on borderline commands**.

Examples of commands that require semantic reasoning:
- `find . -delete` — dangerous when CWD is `/`, routine in a build cleanup script
- `chmod 777 /tmp/my-script` — likely fine; `chmod 777 /etc/passwd` — critical risk
- `dd if=/dev/urandom of=./testfile bs=1M count=10` — safe; `dd if=/dev/urandom of=/dev/sda` — destroys disk
- `kubectl delete pod my-pod` — routine; `kubectl delete namespace production` — catastrophic

Pattern matching alone cannot distinguish these. The result: either false positives
(blocking safe commands → user frustration) or false negatives (allowing dangerous
commands → security gap).

### The Codex Guardian Approach

OpenAI's Codex Guardian (PR #13860) solves a related problem: replacing human approval decisions
with an AI subagent reviewer. Guardian uses a "carefully prompted reviewer" that gathers context
and applies a risk-based decision framework.

Key architectural insights from Guardian:
1. **Session-scoped LLM instance** — Guardian reuses the same reviewer subagent across
   multiple approval requests in a session (PR #14668), sharing prompt cache for latency
2. **Structured assessment payload** — Returns `risk_score`, `rationale`, `decision`
3. **Complementary to pattern matching** — Guardian is an overlay, not a replacement for
   policy-based rules

Key reason Guardian is still experimental:
- Assessment payload generation can fail silently → silent allow bugs
- Trust calibration not yet tuned → both over-protective and permissive in different scenarios
- Protocol design was finalized mid-implementation (payload schema was designed late)

### The Opportunity for Caro

Caro already has an embedded LLM backend. Unlike Codex Guardian which calls an external API,
Caro can run the reviewer locally — no latency from network calls, no API cost, no privacy
concern about sending commands to a remote service.

The Dogma rule engine (v2.0.0 roadmap) is Caro's long-term answer to advanced safety rules.
Guardian Mode is a focused, earlier implementation of the LLM-review concept that:
- Ships sooner (targeted for v2.0.0 alongside Dogma, as a lighter parallel track)
- Solves the immediate borderline command problem
- Validates the LLM-as-reviewer architecture before Dogma's broader scope

## Decision

We will add **Guardian Mode** — an optional LLM-assisted review layer that supplements
pattern matching for borderline commands.

### Activation

Guardian Mode is **opt-in** and explicitly configured:
```toml
# ~/.config/caro/config.toml
[safety]
guardian_mode = true          # Enable LLM-assisted review
guardian_threshold = 0.4      # Invoke LLM when pattern confidence < 0.4 (0.0-1.0)
guardian_backend = "embedded" # "embedded" | "ollama" | "none"
guardian_timeout_ms = 2000    # Max time to wait for LLM review
```

Or via flag:
```bash
caro --guardian "find . -delete"
caro validate --json --guardian "chmod 777 /tmp/script.sh"
```

### Decision Flow

```
Input command
     │
     ▼
Pattern Matching (52 patterns)
     │
     ├─ CRITICAL/HIGH match → Block immediately (no LLM needed, high confidence)
     │
     ├─ MODERATE match + guardian_mode=true → LLM Review
     │         │
     │         ▼
     │   Guardian LLM: "Is this safe in context?"
     │         │
     │         ├─ LLM says BLOCK → Block + explain
     │         ├─ LLM says WARN → Warn + rationale
     │         └─ LLM says ALLOW → Allow + confidence note
     │
     ├─ No match + confidence < threshold + guardian_mode=true → LLM Review
     │         (catches novel patterns not in database)
     │
     └─ No match + high confidence → Allow (no LLM needed)
```

### Session-Scoped LLM Instance (Learning from Guardian PR #14668)

The critical performance optimization from Codex Guardian: **reuse the LLM instance across
multiple validations in a session**.

```rust
pub struct GuardianSession {
    backend: Arc<dyn InferenceBackend>,
    /// System prompt loaded once, cached for the session
    system_prompt: String,
    /// Session ID for audit trail correlation
    session_id: Uuid,
}

impl GuardianSession {
    /// Create once per Caro session (not per command)
    pub fn new(backend: Arc<dyn InferenceBackend>) -> Self { ... }

    /// Review a command — reuses the loaded model context
    pub async fn review(&self, command: &str, context: &ExecutionContext)
        -> Result<GuardianAssessment> { ... }
}
```

This avoids re-loading the model for every command. For the embedded backend, model loading
is the expensive operation (200-500ms); subsequent inferences are fast (50-150ms).

### Guardian Prompt Design

The system prompt is carefully designed to produce well-structured output every time
(solving the silent payload failure that plagues Codex Guardian):

```
You are a shell command safety reviewer. Your job is to assess whether a given shell command
is safe to execute in the described context.

Respond ONLY with a JSON object in exactly this format (no other text):
{
  "decision": "allow" | "warn" | "block",
  "risk_score": <0-100 integer>,
  "rationale": "<one sentence explaining the decision>",
  "suggested_alternative": "<safer command, or null>"
}

Rules:
- "allow" (risk_score 0-30): command is routine and unlikely to cause harm
- "warn" (risk_score 31-70): command could cause harm in some contexts; human should verify
- "block" (risk_score 71-100): command is likely to cause irreversible harm

Context provided: operating system, current directory, shell type.
If the command is clearly safe (e.g., ls, cat, echo), respond with allow and risk_score < 10.
If uncertain, prefer "warn" over "block" to avoid false positives.
```

Key design decisions to prevent Codex Guardian's payload bug:
1. **JSON-only response** — Prompt explicitly forbids non-JSON text
2. **Strict schema** — All fields required, no optional fields that could be omitted
3. **Parse validation** — If response doesn't parse as valid JSON with all fields, it's a
   validator error (exit code 3) — never a silent allow
4. **Timeout with fallback** — If LLM doesn't respond within `guardian_timeout_ms`,
   fall back to pattern-matching-only result (never silent allow)

### AssessmentPayload Integration

Guardian Mode output feeds into the same `AssessmentPayload` type defined in ADR-015:

```rust
pub struct AssessmentPayload {
    pub decision: Decision,       // allow | warn | block
    pub risk_score: u8,           // 0-100
    pub risk_level: RiskLevel,    // safe | low | moderate | high | critical
    pub rationale: String,        // Human-readable explanation
    pub pattern_matched: Option<String>,
    pub suggested_alternative: Option<String>,
    pub confidence_score: f32,    // 0.0-1.0
    pub reviewed_by: ReviewedBy,  // NEW: patterns | guardian | patterns+guardian
    pub execution_time_ms: u64,
    pub caro_version: String,
}

pub enum ReviewedBy {
    PatternsOnly,
    GuardianOnly,
    PatternsAndGuardian,
}
```

The `reviewed_by` field tells callers how the decision was made — important for audit trails.

## Rationale

### Why LLM Review Instead of More Patterns?

Patterns are fast and deterministic but cannot capture semantic context. The combinatorial
explosion of "dangerous in context X, safe in context Y" makes exhaustive patterns
impractical. An LLM naturally handles this:

- `find . -delete` in `/home/user/project` → low risk
- `find . -delete` in `/` or `/etc` → critical
- `dd ... of=/dev/sda` → critical
- `dd ... of=./disk.img` → safe

A pattern for each combination is unmaintainable. The LLM reasons naturally about context.

### Why Opt-In (Not Default)?

1. **Latency**: LLM adds 50-150ms per review — acceptable for interactive use, not for hot loops
2. **Privacy**: Even the embedded backend processes the command text — users should consent
3. **Reliability**: Pattern matching is 100% deterministic; LLM adds nondeterminism
4. **Gradual rollout**: Opt-in lets us tune the guardian threshold before making it default

### Why Not Default Guardian for Everything?

Pattern matching is fast, deterministic, and highly accurate for known dangerous patterns.
Guardian Mode is an overlay for borderline cases — it should not replace the pattern
database for clear-cut dangerous commands. The combination is:
- Fast + accurate for known patterns
- Context-aware for borderline cases

### Why Session-Scoped vs Per-Command LLM?

Per-command initialization has O(n×model_load_time) cost. With 5-10 commands in a typical
session, that's 1-5 seconds of total model loading overhead. Session-scoped initialization
pays the model load cost once and amortizes it across all commands.

## Consequences

### Benefits

1. **Near-zero false positives**: LLM can reason about context that patterns cannot
2. **Novel attack detection**: LLM may catch dangerous patterns not in the 52-pattern database
3. **Better user explanations**: LLM generates human-readable rationale (vs pattern description)
4. **Suggested alternatives**: LLM can recommend safer commands
5. **Local privacy**: Embedded backend — no command text sent to external services

### Trade-offs

1. **Latency increase**: +50-150ms for commands reviewed by guardian
2. **Nondeterism**: LLM may give different answers to identical inputs (mitigated by prompt design)
3. **Model dependency**: Quality depends on embedded model capability
4. **Complexity**: Two code paths for safety decisions

### Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| LLM generates non-JSON | Silent allow bug | Parse validation + timeout fallback → error, not allow |
| LLM is over-protective | User frustration | Tune threshold; default threshold set conservatively |
| LLM is under-protective | Security compromise | Pattern matching always runs first; LLM only for borderline |
| Model unavailable | Guardian skipped | Configurable: `guardian_unavailable = "warn"/"skip"/"block"` |
| Prompt injection via command | Manipulation | Commands sent as data fields, not as instructions |

## Alternatives Considered

### Alternative 1: More Patterns (Extend Pattern Database)

Add context-aware patterns: `find.*-delete` with CWD context, `dd.*of=/dev/` etc.

- **Pros**: Deterministic, fast, no LLM dependency
- **Cons**: Combinatorial explosion, can't capture all contexts, maintenance burden
- **Why not chosen**: Context-aware patterns become indistinguishable from a DSL — better to use LLM for this

### Alternative 2: AST-Based Analysis (ADR-007)

Parse shell commands into AST and reason about their effects symbolically.

- **Pros**: Deterministic, fast, no LLM dependency
- **Cons**: Shell parsing is complex (ADR-007 found this to be hard), incomplete coverage
- **Decision**: ADR-007 (AST parser) is a complementary track, not a replacement for Guardian Mode

### Alternative 3: Dogma Rule Engine Only (v2.0.0)

Wait for Dogma's full rule engine to handle borderline cases.

- **Pros**: Unified approach, more powerful
- **Cons**: Dogma is more complex and further out; Guardian Mode can ship earlier with less scope
- **Why not chosen**: Guardian Mode is a focused MVP; Dogma is the long-term platform

### Alternative 4: External LLM API

Call Claude API or OpenAI API for guardian reviews.

- **Pros**: Highest quality reasoning, no local model required
- **Cons**: Network latency, API cost, privacy (commands sent externally), offline not supported
- **Why not chosen**: Conflicts with Caro's offline-first, privacy-first philosophy

## Implementation Notes

### Phase 1: Foundation (Alongside AssessmentPayload in ADR-015)

- Add `AssessmentPayload` type with `reviewed_by` field
- Add `GuardianSession` struct (disabled by default)
- Add guardian-related fields to `SafetyConfig`

### Phase 2: Embedded Backend Integration

- Wire `GuardianSession` to embedded backend
- Implement system prompt + JSON response parsing
- Add timeout + fallback logic
- Unit tests: mock LLM responses, test JSON parsing robustness

### Phase 3: Integration + Threshold Tuning

- Connect to `validate` subcommand (ADR-015)
- Add `--guardian` CLI flag and `guardian_mode` config
- Benchmark: measure latency overhead vs without guardian
- Integration tests: known borderline commands → verify correct decisions

### Module Structure

```
src/safety/
├── mod.rs           # SafetyValidator, SafetyConfig, ValidationResult, AssessmentPayload (ADR-015)
├── patterns.rs      # Pattern database (unchanged)
├── guardian/        # NEW
│   ├── mod.rs       # GuardianSession, GuardianAssessment
│   ├── prompt.rs    # System prompt templates
│   └── parser.rs    # JSON response parsing + validation
└── validator.rs     # Orchestrator: patterns → optional guardian → AssessmentPayload
```

### Testing Strategy

1. **Unit**: JSON response parsing — test all valid/invalid response shapes
2. **Unit**: Timeout fallback — verify guardian timeout returns pattern-only result
3. **Integration**: End-to-end with embedded backend (requires model)
4. **Property**: Known borderline commands evaluated by guardian (snapshot/regression tests)
5. **Security**: Prompt injection attempts via crafted command strings

## Success Metrics

| Metric | Target |
|--------|--------|
| False positive rate reduction | >50% reduction on borderline commands vs patterns-only |
| Guardian review latency | <200ms p95 (after session warm-up) |
| JSON parse reliability | 100% — no silent allows from parse failures |
| Timeout reliability | 100% — timeouts always return pattern-only result, never silent allow |
| User opt-in rate | >20% of users enable guardian mode within 6 months of release |

## References

- [Codex Guardian PR #13860](https://github.com/openai/codex/pull/13860) — Smart approvals
- [Codex Guardian session caching PR #14668](https://github.com/openai/codex/pull/14668) — Session reuse
- [Codex Guardian Issue #15341](https://github.com/openai/codex/issues/15341) — Assessment payload failure (what to avoid)
- [ADR-007](./ADR-007-ast-parser-shell-validation.md) — AST-based validation (complementary)
- [ADR-015](./ADR-015-universal-agent-integration-protocol.md) — Universal Agent Integration Protocol
- [ADR-010](./ADR-010-bubblewrap-sandbox-execution.md) — Sandbox execution

## Revision History

| Date | Author | Changes |
|------|--------|---------|
| 2026-03-25 | @wildcard | Initial draft, informed by Codex Guardian research |
