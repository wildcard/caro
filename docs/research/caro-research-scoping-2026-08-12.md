# caro-research--scoping-process — Run Report

**Date**: 2026-08-12
**Agent**: Automated scheduled task (`caro-research--scoping-process`)
**Status**: ✅ Completed — ADR-049 produced

---

## What Happened This Run

The task's SKILL.md still contains the unfilled `[FEATURE NAME]`
placeholder (outstanding since 2026-06-02). Resolved autonomously:

1. Read prior run reports (06-02 → 08-03) and the ADR directory. Latest
   ADR on disk is ADR-048 (Anthropic inference-hook verdict adapter,
   08-11). Next number: 049.
2. Selection signal: the Hermes **2026-08-12 weekly market scan** names
   **"attestable audit export ('evidence packet') for governance
   platforms"** as opportunity C (priority Next, complexity M) with the
   explicit next step "ADR defining the evidence-packet schema … then a
   `caro audit export` subcommand emitting JSONL", and repeats it as
   top-3 recommendation #3. Opportunity A is positioning content
   (outside this task's charter), B (MCP `validate_command`) is covered
   by ADR-043, and D is gated behind `validation-discipline.md` Gate 1
   transcripts by the memo itself. Grep confirmed no ADR covers audit
   export (ADR-032 receipts explicitly defers "export/ingest adapters"
   and signing to v2 — this run is that v2 scope).
3. Mapped `[FEATURE NAME]` to live reference implementations:
   **nono's tamper-evident audit trail** (nolabs-ai/nono, Apache-2.0 —
   Merkle-rooted NDJSON log, hash chain, domain-separated SHA-256,
   DSSE/in-toto attestation bundle, three-layer `nono audit verify`;
   blog deep-dive fetched live) read against the **in-toto Attestation
   Framework spec v1.2** (Statement/Predicate/Envelope/Bundle layers;
   spec README fetched live), plus the AEVS failure-mode findings
   already on file in ADR-032.
4. Ran a codebase survey (Explore agent): ADR-041 emitter and ADR-032
   receipts are still ADR-only; `SafetyDecision`/`SuggestedRouting`
   have zero non-defining call sites; `ExecutionResult` lacks
   `Serialize`; `DangerPattern` has no stable id (67 builtins, prose
   `matched_patterns` only; CVE rules have ids); no exit-code registry
   exists in code (highest implemented contract code: 1); `sha2` and
   `fd-lock` are already deps (v1 needs zero new crates); `caro export`
   is namespace-taken by CaroML, `caro audit` is free.

## Phase 1 findings (nono / in-toto / AEVS)

- **nono's construction** (replicate): supervisor-only audit custody;
  per-event `leaf_hash` + `chain_hash` (RFC-6962-style), Merkle root
  over canonical bytes, domain-separated hash contexts; in-toto
  Statement over the root, DSSE-wrapped; `audit verify` = three named
  independent checks returning `VERIFIED`/`MISMATCH` + reason.
- **nono's failure modes** (avoid by design): (1) without
  `--public-key-file`, verification trusts the key the artifact itself
  claims — a coordinated-rewrite adversary defeats it; (2) `.alpha`
  domain tags / predicateType — schema declared unstable; (3) signing
  drags a keyring/secret-store surface; (4) no external anchoring
  (host+key compromise unanswerable — honestly deferred).
- **AEVS** (avoid): fail-open evidence loss (silent no-op on bad
  credentials; buffer eviction with gap markers) and vendor-backend
  verification.
- **Lifecycle**: both references are session/daemon-shaped; caro's
  export/verify are pure batch reads over ADR-032's append-only file —
  no daemon, no state, idempotent, matching this task's constraints.

## New ADR Produced

**ADR-049** — `docs/adr/ADR-049-attestable-evidence-packet-export.md`

- **Packet = in-toto Statement/v1**, `predicateType
  https://caro.sh/attestation/evidence-packet/v1`, `schema_version: 1`,
  subject digest = Merkle root over canonical receipt-line bytes;
  deterministic output (no generation timestamp; golden-file test).
- **`PolicyIdentity` + stable pattern ids** (`CARO-P-001…067`,
  append-only; additive `matched_pattern_ids` on `ValidationResult`;
  `pattern_set_digest`) — closes the "which rule, under which ruleset?"
  gap neither reference implementation covers; caro's differentiation
  is attesting *decisions + deciding policy*, offline, standalone.
- **Fail-closed evidence** (AEVS's gap solved by contract): export
  never emits a packet it could not verify (broken chain ⇒ exit 9);
  missing source ⇒ exit 13, never silence; empty-but-healthy log ⇒
  valid `record_count: 0` packet, exit 0.
- **No in-band key trust** (nono's gap solved by contract): the future
  `signature` check runs only with an out-of-band key; otherwise
  reports `skipped`, and `--require-signature` turns skipped into
  exit 13.
- **Contract**: `caro audit export|verify`; exit 13 `EvidenceMismatch`
  claimed (0–12 owned by ADRs 024/027/029/032-or-034/039/040/044);
  materializes ADR-024's `ExitCode` registry in `src/cli/mod.rs` —
  first ADR to put it in code. 10 integration tests (fixtures →
  deterministic JSON + exit code) incl. determinism and
  unknown-schema-version gates.
- **v1 needs zero new dependencies** (sha2 + fd-lock already present).
  Signing (Ed25519 + DSSE) is v2, gated on an `ed25519-dalek` phase-0
  build spike per `external-sdk-integration.md`; the Statement/DSSE
  layering means the inner schema never changes when signing lands.
- **Out of scope**: signing (v2 spike), transparency-log/TSA anchoring,
  inclusion-proof CLI, cross-session host ledger, push adapters
  (Drata/Insygna/SIEM/OTel), ADR-003 enrichment, retention.

## ⚠️ Ledger flag for maintainers

**ADR-032 and ADR-034 both claim exit code 9** (`ReceiptChainBroken`
vs `DegradedGated`) — both were written as "next free slot" a week
apart. ADR-049 sides with ADR-032 (first writer, dependency of this
feature) and calls for a one-line amendment to ADR-034 (→ 10-next-free
at its date, or renumber against ADR-039/040's 10/11 claims) **before
either implements**. This is exactly the drift the ADR-numbering rule
exists for, applied to exit codes; a shared registry table in
ADR-024 (or a `docs/adr/exit-codes.md` ledger) would prevent a third
collision.

## Delivery Notes

- Files written to the working tree only — **no commits** (git-workflow
  rule: never commit to main; no user present to open a branch/PR).
  Next session: `bin/sk-new-feature "adr-049 evidence packet export"`,
  move both files in, update `docs/adr/README.md` index in the same PR,
  open PR per ADR-numbering rule (renumber if another ADR PR lands
  first).
- Reviewable assumptions flagged in the ADR: exit-9 arbitration,
  records-embedded vs digests-only packets, file-based PKCS#8 only for
  v2 signing.
- Implementation order: ADR-032's receipt writer is still unbuilt;
  ADR-049 assumes its contract, not its code — T1–T10 run on committed
  fixtures, so export/verify are implementable and testable first.

## Suggested next targets

1. **ADR-034 exit-code amendment** (S) — unblock 032/034/049.
2. **`ed25519-dalek` build spike** (S, ≤100 LOC) — unlocks ADR-049 v2
   signing and is prerequisite-free.
3. **Hermes 08-12 opportunity A** (evidence-memo content) — positioning
   work, belongs to a human/content session, not this task.
