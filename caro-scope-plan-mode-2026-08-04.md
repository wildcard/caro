# Implementation Scope — Plan Mode: Multi-Step Command Plans

**Feature under analysis:** Claude Code *plan mode* (`--permission-mode plan`,
`ExitPlanMode`, `~/.claude/plans/`) — propose-then-approve-then-execute for agent work.
**Equivalent we are scoping for Caro:** `caro plan "<task>"` → versioned JSON plan of
safety-validated steps; `caro plan run <plan.json>` → hash-verified, re-validated,
sequential execution.

**Date:** 2026-08-04 · **Author:** `caro-research--scoping-process` (autonomous run)
**Companion ADR:** `docs/adr/008-plan-mode-multi-step-contract.md`

> The task template left `[FEATURE NAME]` unfilled and ran unattended. Prior runs
> already scoped structured output (06-25), stream-json, lifecycle hooks (07-09),
> multi-backend council (07-16), agent memory (07-17), and context compaction
> (07-31). Plan mode was selected as the highest-value uncovered target: caro's
> agent prompt today *forbids* multi-command output (`src/agent/mod.rs:678`), and
> permission/approval territory is already covered by ADR-020/027/040/042/044.

---

## Phase 1 — Feature Research

### What problem it solves, and for whom
Plan mode separates *thinking* from *doing*: the agent explores read-only,
produces a full implementation plan, and only after explicit user approval does
execution begin. Audience: users running non-trivial, multi-step, risk-bearing
tasks who want to review the whole sequence before anything mutates their system.
For caro this is the natural answer to "set up a venv and install deps" — tasks
that are inherently N commands, not one.

### Core architecture (data flow, key types, separation of concerns)
- Activation: `Shift+Tab` ×2, `claude --permission-mode plan`, or `/plan`;
  headless: `claude --print --permission-mode plan "task"`.
- In plan mode the agent keeps read tools (Read, Glob, Grep, WebSearch, Todo*)
  and is denied write/execute tools by the permission layer.
- The `ExitPlanMode` tool ends planning: the plan text is echoed into the
  approval prompt so the model can reference it during implementation without
  re-reading the file.
- Plans persist as markdown in `~/.claude/plans/` with random names
  (`dreamy-orbiting-quokka.md`); they survive `/clear` and context compaction.
- Separation of concerns is **permission-state based**: same process, same
  session, different allowed-tool set per mode.

### Why it is experimental / limited — the failure modes
1. **Mode drift.** Read-only is enforced by session permission state, and Claude
   sometimes exits plan mode mid-session in long conversations — write tool
   calls appear unexpectedly. Guidance is literally "watch for it and re-toggle."
2. **Modal state-machine bugs.** With `--dangerously-skip-permissions` +
   manual Shift+Tab into plan mode, `ExitPlanMode` approval fails to transition
   back — the session is stuck read-only (anthropics/claude-code#32934).
3. **Plan artifact is prose.** Markdown, random filename, no schema, no version,
   not machine-parseable, not diffable against execution.
4. **Approval ≈ text echo, not binding.** Nothing ties the approved plan
   content to subsequent actions — a review/execute TOCTOU gap.
5. **Headless plan mode produces a plan but the resume-to-execute loop is
   string glue** (scrape session_id, re-invoke) — same gap ADR-003 catalogued.

### Structured output / lifecycle
- Plan content: free markdown. Approval: interactive prompt (or auto in some
  permission configs). No dedicated exit codes for "plan produced" vs
  "plan rejected". Session lifecycle is the standard resume mechanism; the plan
  file itself is the only cross-session artifact — and it is unversioned.

## Phase 2 — Competitive Differentiation

### What they get right (replicate)
- Hard separation of propose vs execute, with explicit user approval between.
- Plan persistence across context loss — the plan outlives the conversation.
- Echoing the plan into the approval step (we replicate as: the JSON plan **is**
  the approval artifact).

### Their gaps (avoid by design)
| Their gap | Our design answer |
|---|---|
| Mode drift (policy-enforced read-only) | `caro plan` has no executor in its code path — read-only **by construction** |
| Stuck modal transitions (#32934) | No modes: two independent subprocess invocations over a document |
| Prose plans, no schema | `CommandPlan` serde type under envelope `schema_version` |
| Approval not bound to content | `plan_hash` (SHA-256, canonical serialization); `plan run` refuses mismatch (exit 7) |
| No per-step risk visibility | Every `PlanStep` carries a full `ValidationResult` + `RiskLevel` |
| Validation only at generation | Re-validate every step again at execution time |

### Our unique positioning
- **Offline/local:** plan generation via embedded backend — no API, no cost field
  needed; deterministic static-matcher steps are possible (Claude Code cannot be
  deterministic).
- **Safety-native:** 52+ pattern validator applied per step, twice. A plan is a
  *safety document*, not a to-do list.
- **Standalone tool:** the plan JSON is a portable artifact any orchestrator
  (Hermes, CI, CaroML) can generate on one machine, review in a PR, and run on
  another.

### Existing infrastructure already covering part of this
- `AgentLoop` generation + repair loop (`src/agent/mod.rs`) — reuse; add a
  list-output prompt variant. Its `CommandValidator` already validates
  candidates.
- `safety::SafetyValidator` → `ValidationResult` (serde, `src/safety/mod.rs:174`).
- `execution::CommandExecutor` + `ExecutionResult` (`src/execution/executor.rs`)
  — needs `Serialize` derive (same gap ADR-003 found for `AiOutcome`).
- ADR-003 `CommandEnvelope` (proposed, unimplemented) — plan mode is its second
  consumer; implement envelope first or land the minimal envelope subset with
  this feature.
- `ApprovalMode {prompt, auto, smart}` (`src/models/mod.rs:288`) + ADR-020 tiers
  — plan approval maps onto them; no new approval concept.
- `dialoguer::Confirm` flow in `main.rs` — reused for interactive plan approval.

## Phase 3 — Scope Definition

### New types (all in `src/models/mod.rs`; serde from day one)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandPlan {
    pub schema_version: u32,          // = 1, shared with envelope
    pub task: String,                 // original NL task
    pub created_at: String,           // RFC3339
    pub shell: String,                // target shell at generation time
    pub backend_used: String,
    pub steps: Vec<PlanStep>,
    pub plan_hash: String,            // hex SHA-256 over canonical steps JSON
    pub overall_risk: RiskLevel,      // max of step risks
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanStep {
    pub index: u32,
    pub purpose: String,              // one-line rationale
    pub command: GeneratedCommand,    // existing type, already serde
    pub validation: ValidationResult, // existing type, already serde
    pub depends_on: Vec<u32>,         // v1: always [index-1]; field reserved
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepOutcome {
    pub index: u32,
    pub revalidation: ValidationResult,
    pub execution: Option<ExecutionResult>, // None if blocked/skipped
    pub status: StepStatus,           // ok | blocked | failed | skipped
}
```

Method contracts:
- `CommandPlan::compute_hash(&self) -> String` — SHA-256 over
  `serde_json::to_vec(&self.steps)` (canonical: struct field order; document that
  field reordering is a schema_version bump).
- `CommandPlan::verify_hash(&self) -> Result<(), PlanIntegrityError>`.
- `CommandPlan::from_agent(task, Vec<GeneratedCommand>, &SafetyValidator) -> Self`.
- `impl FromStr`/`try_from` file loader with schema_version check.
- Derive `Serialize` on `execution::ExecutionResult` (one-line change).

### Files that change (no new modules)

| File | Change |
|---|---|
| `src/models/mod.rs` | Add `CommandPlan`, `PlanStep`, `StepOutcome`, `StepStatus`, `PlanIntegrityError` |
| `src/agent/mod.rs` | `generate_plan(&self, task) -> Result<Vec<GeneratedCommand>, GeneratorError>`: list-output prompt variant (lift the "single command" instruction at `:678` for this path only); parse numbered list; per-item repair reusing existing repair loop |
| `src/prompts/command_templates.rs` | New plan prompt template (ordered steps, one command per line, no prose) |
| `src/execution/executor.rs` | `#[derive(Serialize, Deserialize)]` on `ExecutionResult` |
| `src/main.rs` | `plan` subcommand (generate) + `plan run` (execute); exit-code constants `EXIT_CODE_BLOCKED = 3` (shared w/ ADR-003), `EXIT_CODE_PLAN_INVALID = 7`; wire `-o json|plain` for both paths |
| `src/cli/mod.rs` | Plumb plan paths through `CliResult`/output switch (or emit envelope directly if ADR-003 lands first) |

Estimated net new code ≈ 600–800 LOC + tests. No new crates (sha2 is already a
transitive dep of the cache integrity code — verify; else add `sha2`, MIT/Apache).

### Exit code / output contract (what machines depend on)

- `caro plan -o json "<task>"` → stdout: envelope containing `CommandPlan`;
  exit 0 (all steps allowed) / 3 (≥1 step blocked; plan still emitted with
  blocked steps marked) / 1,2,4,5,6 per ADR-003.
- `caro plan run -o json plan.json` → stdout: envelope containing
  `steps: Vec<StepOutcome>`; exit 0 (all ran, all exit 0) / 3 (step blocked at
  re-validation) / 7 (hash mismatch or unsupported schema_version — nothing
  executed) / 1 (step spawn failure). A step's non-zero child exit halts the
  plan; envelope reports the child exit code; caro exits with the child's code
  only if `--propagate-exit` (deferred — v1 exits 1 and reports).
- Plain mode: human-readable numbered plan on stdout, risk annotations on
  stderr; interactive `Confirm` before `plan run` unless `-y`.
- Determinism guarantee (ADR-003 §5): identical task + backend + flags ⇒
  identical plan modulo `created_at`/timings; `plan_hash` excludes `created_at`.

### Integration tests (known input → deterministic output)

1. **Static-matcher plan:** task decomposing to template-covered steps → exact
   JSON snapshot (steps, risks, hash), exit 0.
2. **Blocked step at generation:** task inducing `rm -rf /` step → step marked
   `allowed:false`, `overall_risk: critical`, exit 3, plan still valid JSON.
3. **Tamper test:** generate plan, mutate one command byte in the file,
   `plan run` → exit 7, envelope `error.kind = "plan_invalid"`, zero executions.
4. **Re-validation catch:** handcraft a valid-hash plan containing a dangerous
   command (hash computed honestly) → `plan run` exits 3 at re-validation before
   spawn.
5. **Halt-on-failure:** 3-step plan where step 2 exits 1 → step 3 `skipped`,
   caro exit 1, `StepOutcome`s present for steps 1–3.
6. **Schema version:** plan with `schema_version: 99` → exit 7.
7. **Round-trip:** `CommandPlan` serde round-trip property test.

### Out of scope (next version)
- Agentic replanning / repair-on-failure mid-run; `--keep-going`.
- Parallel step execution and real `depends_on` DAGs (field reserved).
- Plan registry / managed storage; plan diffing UX.
- Rollback integration with ADR-033 undo snapshots.
- `--propagate-exit` child-code passthrough.
- Streaming step events (belongs to ADR-004 stream-json once implemented).
- Cross-shell plan translation (plan records its target shell; translation is v2).

### Constraint check
- Reuses validator/safety/config/executor — no duplication. ✔
- All new types serde from day one. ✔
- Pure subprocess: two stateless invocations over a caller-owned file; no
  daemon, no managed state. ✔
- Phase-1 failure modes solved by design: no modes (drift/stuck impossible),
  typed+versioned plan (no prose), hash-bound approval + double validation
  (no TOCTOU). ✔
