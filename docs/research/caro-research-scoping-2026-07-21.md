# caro-research--scoping-process — Run Report

**Date**: 2026-07-21
**Agent**: Automated scheduled task (`caro-research--scoping-process`)
**Status**: ✅ Completed — ADR-038 produced + ADR-015 freshness pass

---

## What Happened This Run

The task's SKILL.md still contains the unfilled `[FEATURE NAME]` placeholder
(outstanding since 2026-06-02). Resolved autonomously:

1. Read prior run reports (06-02 → 07-14) and the ADR directory. Latest ADR
   on disk is ADR-037 (lifecycle event schema, 07-20). Next number: 038.
2. Selection signal was the strongest yet: Hermes 07-21 names the **MCP
   validator built against the RC** as #1 "Build/test next" for the *third
   consecutive week*, now date-anchored (spec finalizes **Jul 28** — 7 days
   from this run). The companion item "MCP RC statefulness audit before the
   Jul 28 final spec (freshness pass on ADR-015)" has been carried since
   07-06 and never executed. Hermes 07-21 additionally flags the **Tasks
   extension** as mapping "naturally onto long-running validation/approval
   flows" with "pending human approval as a protocol-native state."
3. Mapped `[FEATURE NAME]` to the **MCP 2026-07-28 release candidate**
   (stateless core) + the **Tasks extension**
   (`io.modelcontextprotocol/tasks`, SEP-2663) — a textbook fit for the
   template: shipped experimental in core 2025-11-25, redesigned under
   production use, graduated to an extension in the RC. Fetched the
   official RC announcement, the Tasks extension overview, and the SDK-beta
   announcement (all modelcontextprotocol.io properties).
4. Verified ADR-038 as next sequential number; verified no `src/mcp/`, no
   `rmcp` in Cargo.toml (ADR-015's spike still unexecuted), MSRV 1.85.

## New ADR Produced

**ADR-038** — `docs/adr/ADR-038-mcp-rc-tasks-approval-alignment.md`

- **Closes the carried statefulness audit**: the RC's stateless core
  (handshake and session *removed*; capabilities via `server/discover` +
  per-request `_meta`) is the protocol converging on caro's architecture —
  ADR-015's tool surface, `SafetyAssessmentOutput`, and exit codes carry
  forward unchanged; its protocol-lifecycle assumptions are stale and
  tabulated as such.
- **Approval gate defaults to MRTR (SEP-2322), not Tasks**: HumanGate →
  `InputRequiredResult` with an `ApprovalGateState` echoed in
  `requestState`; the retry leg **re-validates the command from scratch**,
  so tampered state can only produce a mismatch error, never a weaker
  verdict. No task store, no state between legs — the Phase-1 failure mode
  (Tasks' durable Task Store obligation vs. caro's pure-subprocess
  constraint) is solved by design.
- **Tasks as optional compatibility layer**: only for clients advertising
  the extension; one in-process `HashMap`, process-lifetime semantics
  documented loudly; TTL expiry is deny-by-default; caro's instantaneous
  worker makes cooperative cancel TOCTOU-free. Cleanly detachable (D4) if
  review prefers MRTR-only.
- New types: `ApprovalGateState`, `McpTaskRecord`/`McpTaskStatus`/store,
  additive `approval: Option<ApprovalRecord>` on `SafetyAssessmentOutput`
  (schema_version stays 1). 7 files (only one beyond ADR-015's list);
  8 new deterministic integration tests + a conformance-suite CI step;
  binary exit codes 0/64/11 only — assessments are payloads, never exits.
- Build spike re-instantiated for the RC: rmcp is **not in the Tier-1 beta
  wave** (Python/TS/Go/C# only, 06-29 announcement) — spike step 4 now
  specifically probes `ProtocolVersion::V_2026_07_28` + MRTR/Tasks type
  presence, with the raw-JSON fallback documented.

Also updated ADR-015's "Last reviewed" header to point at ADR-038
(precedent: the 06-12 freshness pass).

## Key Research Findings

1. **MCP goes stateless on Jul 28** (RC locked 2026-05-21): initialize
   handshake and `Mcp-Session-Id` removed (SEP-2575/2567); every request
   self-contained; `server/discover` replaces capability exchange. The
   redundant-initialization question the task template asks is answered by
   *elimination*, and app state moves to explicit handles the model
   threads between calls.
2. **Tasks' redesign history is the failure-mode map**: blocking
   `tasks/result` → polling; `tasks/list` removed as unscopeable without
   sessions; task IDs became unguessable bearer tokens; cancellation is
   cooperative (TOCTOU for slow workers); the Task Store is a durability
   obligation that quietly reintroduces state into "stateless" servers —
   the exact trap ADR-038 routes around via MRTR-default.
3. **MRTR is the sleeper feature for approval gates**: `InputRequiredResult`
   + client-echoed `requestState` is stateless HITL. Caro is uniquely
   positioned for it: a deterministic validator can re-derive its verdict
   on the retry leg in <2ms, making the echoed state tamper-proof without
   signatures. LLM-judge safety tools cannot replay their verdict cheaply
   or deterministically — this is the differentiation argument.
4. **Contract-surface changes that touch caro**: JSON Schema 2020-12 for
   tool schemas; `-32002` → `-32602`; `-32003` reserved for missing client
   capability; MCP `logging` deprecated with stderr as the documented
   stdio path (caro's existing behavior, now spec-blessed); W3C Trace
   Context keys fixed in `_meta` (aligns with ADR-037's OTel direction).
5. **Rust SDK risk is real and now explicit**: the 06-29 Tier-1 beta wave
   is Python v2 / TypeScript v2 / Go / C#. rmcp main reports
   `V_2026_07_28` support but is unverified for MRTR/Tasks — the phase-0
   spike is the ≤1-hour instrument that converts this from schedule risk
   to known fact. Start it **before Jul 28**.

## Suggested Next Targets

- **Execute the rmcp phase-0 spike this week** (Hermes #1 for three weeks;
  ADR-015 + ADR-038 both blocked on it; hard date Jul 28).
- Tasks for embedded-LLM generation calls (minutes-long `working` states —
  the other natural Tasks fit; deferred in ADR-038's out-of-scope).
- CISO-rubric blog post (Hermes item C, carried; content not scope — decays
  fast).

## Housekeeping Notes (not fixed this run)

- `docs/adr/README.md` index still stale (flagged every run since 07-03;
  duplicate ADR-004/ADR-015 filename pairs persist — violates
  adr-numbering.md). Legacy lowercase `004-…`/`003-…` series also still
  coexists with the `ADR-NNN-…` series.
- SKILL.md's `[FEATURE NAME]` placeholder remains unfilled; runs keep
  selecting autonomously.
- Files written but **not committed**: per git-workflow.md a feature
  branch + PR is required, and prior runs found sandbox git access
  degraded. A human/interactive session should branch, commit ADR-038 +
  the ADR-015 header edit + this report, and open a PR.

## Sources

- https://blog.modelcontextprotocol.io/posts/2026-07-28-release-candidate/ (fetched 2026-07-21)
- https://tasks.extensions.modelcontextprotocol.io/ (fetched 2026-07-21)
- https://blog.modelcontextprotocol.io/posts/sdk-betas-2026-07-28/ (fetched 2026-07-21)
- `.hermes/digests/2026-07-21-weekly-agent-market-scan.md`
- `docs/adr/ADR-015-mcp-safety-server.md`, ADR-020, ADR-024, ADR-027, ADR-036, ADR-037
- `src/safety/mod.rs` (`SafetyDecision`), `src/models/mod.rs` (`SuggestedRouting`), `Cargo.toml` (MSRV 1.85, no rmcp)
