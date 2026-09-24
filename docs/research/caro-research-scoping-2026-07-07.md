# caro-research--scoping-process — Run Report

**Date**: 2026-07-07
**Agent**: Automated scheduled task (`caro-research--scoping-process`)
**Status**: ✅ Completed — ADR-032 produced

---

## What Happened This Run

The task's SKILL.md still contains the unfilled `[FEATURE NAME]` placeholder
(outstanding since 2026-06-02). Resolved autonomously:

1. Read prior run reports (06-02 → 07-06) — ADR-015…031 already scoped.
2. Two independent signals converged on one target: the 07-06 report's
   suggested next target 2 (`agent_id` on lifecycle events, Hermes item D)
   and **today's Hermes 07-07 market scan opportunity 1** ("publish the
   assessment-payload + lifecycle-event schema as a spec", priority
   **Now**), with AEVS by Fetch.ai as the market evidence.
3. Mapped `[FEATURE NAME]` to the live reference implementations:
   **Fetch.ai AEVS** (signed execution receipts, SDK 0.2.2, beta —
   researched live via aevs.fetch.ai and its llms.txt on 2026-07-07) and
   **OTel GenAI agent semantic conventions** (`gen_ai.agent.id`,
   Development status).
4. Verified no overlap: no "receipt" concept anywhere in `docs/adr/` or
   `src/`; `agent_id` appears only in ADR-031's event-schema note;
   `docs/spec/` does not exist. ADR-032 confirmed as next number.

## New ADR Produced

**ADR-032** — `docs/adr/ADR-032-portable-execution-receipts.md`

- `ExecutionReceipt`: append-only, hash-chained (SHA-256 `prev_hash` +
  `seq`), schema-versioned NDJSON record per headless invocation at
  `$XDG_DATA_HOME/caro/receipts.ndjson` — embedding the safety
  `ValidationResult` verbatim (the judgment layer AEVS lacks) and the
  OS-level outcome as hashes (`ReceiptExecution`)
- New identity field `agent_id` (`--agent-id` / `CARO_AGENT_ID`, aligned
  with OTel `gen_ai.agent.id`) stamped on receipt, ADR-024 envelope,
  NDJSON `Init`, and ADR-031's `session_halted` — closes Hermes item D
- `caro receipt verify` (local recompute, exit **9 = ReceiptChainBroken**)
  and `caro receipt schema`; committed JSON Schema at
  `docs/spec/execution-receipt.schema.json` via existing `schemars` bin —
  this file is the publishable spec Hermes opportunity 1 asked for
- 8 files (2 new: `src/ai/receipts.rs`, `tests/receipt_chain.rs`); no new
  modules, **no new crates** (`sha2 0.10` + `schemars 0.8` already in
  tree); all types serde + `JsonSchema` from day one; pure subprocess
  (link = read last line, append, exit)
- 9 deterministic integration tests incl. fail-closed assertion (executor
  never invoked when the chain can't be appended)

## Key Research Findings

1. **AEVS is fail-open.** Bad credentials → silent "no-op mode" (agent
   runs, zero receipts); buffer overflow evicts oldest receipts with "gap
   markers". ADR-032 inverts this: `--receipt` makes evidence a
   precondition — unappendable chain aborts *before* execution.
2. **AEVS admits its attestation gap** in its own docs: verification
   proves "the SDK recorded the call and its reported result — not that
   an external system actually performed the action." Caro is the
   executor, so binding verdict + real exit code + output hashes into one
   record is free — the structural moat.
3. **AEVS receipts have no judgment layer** — tool/inputs/outputs/timing,
   but no risk tier, no policy verdict, no matched patterns. Caro's
   receipt embeds `ValidationResult` verbatim; nobody else has that data
   at sealing time.
4. **Worth copying from AEVS**: hash-chain + `seq` per session,
   `chain_status` failure taxonomy (adapted to `ChainBreakKind`),
   `proof_only` visibility (hashes leave the host, payloads never), and
   receipt identity split (client-generated ID available immediately).
5. **Worth copying from OTel GenAI**: the `gen_ai.agent.id` attribute
   name/semantics for caller-declared agent identity; span *export* is
   rejected for v1 (Development-status spec, collector-shaped).
6. **Signing deferred by rule, not oversight**: Ed25519 signing needs a
   new crate → external-sdk build-spike rule applies; an unsigned local
   chain has the same trust model as shell history and still beats a
   signed chain behind someone else's API for offline audit. `sig` field
   is additive within schema v1.

## Housekeeping Notes

- Files written to the **working tree only, not committed** (git-workflow
  rule; checkout is on `chore/dispatch-cycle-2026-07-04-0211` and the
  worktree's git metadata is not writable from this sandbox). Suggested
  branch: `feat/adr-032-portable-execution-receipts-scope`.
- Exit-code ledger after this ADR: 0–6 (ADR-024), 7 (ADR-027), 8
  (ADR-031), **9 (ADR-032)**, 30–32 (ADR-030), 201 (`EXIT_CODE_EDIT`,
  src/main.rs:938).
- `docs/adr/README.md` index still stale (ends at ADR-015); left
  untouched to avoid parallel-session collisions.
- P0 from 2026-06-02 remains open: fill `[FEATURE NAME]` in the scheduled
  task, or formally accept autonomous target selection (5 consecutive
  runs have now self-selected).

## Suggested Next Targets

- **MCP RC statefulness + incremental-scope-consent audit** before the
  Jul 28 final spec (Hermes 07-07 opportunity 2; freshness pass on
  ADR-015 `caro mcp serve` + phase-0 spike against the MCP Rust SDK RC)
- **AGT interop** (Hermes 07-07 opportunity 3): Caro verdicts as an Agent
  Governance Toolkit Ring-3 policy provider — Phase-1 issue on the
  existing PR #1103 beads epic
- **Receipt signing build-spike** (v2 of ADR-032) once a crypto crate is
  chosen — candidates: `ed25519-dalek`, `minisign-verify`

## Sources

- https://aevs.fetch.ai/ and https://aevs.fetch.ai/llms.txt (fetched 2026-07-07)
- https://github.com/fetchai/AEVS-sdk
- https://opentelemetry.io/docs/specs/semconv/gen-ai/gen-ai-spans/
- https://opentelemetry.io/blog/2026/genai-observability/
- `.hermes/digests/2026-07-07-agent-market-scan.md`
