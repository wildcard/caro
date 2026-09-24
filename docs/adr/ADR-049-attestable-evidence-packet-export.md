# ADR-049: Attestable Evidence Packet Export — `caro audit export` / `caro audit verify`

- **Status**: Proposed (scope only — no implementation in this document)
- **Date**: 2026-08-12
- **Produced by**: `caro-research--scoping-process` scheduled task
- **Feature researched**: **nono's tamper-evident audit trail**
  (nolabs-ai/nono, Apache-2.0; blog deep-dive 2026-04-22 — Merkle-rooted
  NDJSON event log, hash chain, `ExecutableIdentity`, DSSE/in-toto signed
  attestation bundle, three-layer `nono audit verify`), read against the
  **in-toto Attestation Framework spec v1.2** (in-toto/attestation:
  Statement / Predicate / DSSE Envelope / Bundle layers) and the AEVS
  findings already on file in ADR-032.
- **Depends on**: ADR-032 (portable execution receipts — `ExecutionReceipt`,
  hash chain, `receipts.ndjson`; this ADR is its export/verify layer),
  ADR-024 (headless JSON contract — exit-code registry), ADR-020
  (`SuggestedRouting` tier vocabulary)
- **Relates to**: ADR-041 (caro-events emitter — a *stream* sink; this ADR
  is the *packet* artifact), ADR-003 (enterprise audit trail — superset,
  still deferred), ADR-037 (lifecycle event schema), Hermes memo
  2026-08-12 (opportunity C: "attestable audit export for governance
  platforms", priority Next, complexity M; recommendation 3: "Start the
  attestable audit export ADR"), EU AI Act Art. 12 record-keeping
  (enforcement began 2026-08-02; mapped in ADR-045/-048 context sections)

---

## Context

### The market problem

Attestation entered agent procurement the week of 2026-08-05: Insygna's
Agent Report Card scores agents pre-deployment, Drata shipped an agent
governance module, and Sirion published a government assessment scheme.
All of them score agents **statically**. None can attest *what a safety
layer actually decided at runtime*. Hermes 08-12: "Runtime enforcement
products that can also emit attestable evidence get pulled into these
channels for free."

Caro already produces the decision that matters — a deterministic safety
verdict per command — and ADR-032 scoped the durable, hash-chained record
of it (`ExecutionReceipt`, `receipts.ndjson`). What is missing is the
**artifact an auditor or governance platform ingests**: a single,
self-describing, independently verifiable evidence packet over a set of
receipts — {command, assessment, tier, decision, policy version} — plus
the command that verifies one offline. ADR-032 explicitly deferred
"receipt export/ingest adapters" and signing to v2. This ADR is that
follow-up, designed schema-first against two live reference
implementations.

### Phase 1 — the reference implementations

**nono** (OSS, Rust, Apache-2.0) is the closest architecture to what caro
needs, and the most instructive:

- **Two-process custody.** A trusted supervisor is the sole audit writer;
  the sandboxed child cannot touch the log. Caro's analogue is simpler
  and stronger for its niche: caro *is* the pre-execution gate, a pure
  subprocess — the audited command never runs inside caro's trust
  boundary at all.
- **Record structure.** `audit-events.ndjson` — one JSON line per event,
  each carrying its own `leaf_hash` and `chain_hash`
  (`chain[i] = H(chain[i-1] ‖ leaf[i])`), with **domain-separated**
  hashing (`"nono.audit.event.alpha"` etc.) to prevent cross-context
  hash confusion. A Merkle root commits the set; the chain commits the
  order. Session metadata (`session.json`) records
  `{hash_algorithm, event_count, chain_head, merkle_root}`.
- **Attestation.** At session end, an in-toto Statement
  (`subject.digest.sha256 = <merkle root>`,
  `predicateType = "https://nono.sh/attestation/audit-session/alpha"`)
  is signed and wrapped in a DSSE envelope → `audit-attestation.bundle`.
- **Verification.** `nono audit verify <session> [--public-key-file]
  [--json]` runs three independent checks (log integrity replay, global
  ledger inclusion, attestation signature) and returns
  `VERIFIED`/`MISMATCH` with a specific reason.

**Why it is experimental / its failure modes** (each drives a design
decision below):

1. **In-band key trust.** Without `--public-key-file`, verification uses
   the public key *the session itself claims*. nono's own threat table
   admits this "catches honest corruption but not a coordinated-rewrite
   adversary": an attacker who rewrites the session can also rewrite the
   embedded key. Trust anchored in the artifact being verified is not
   trust. (→ D4)
2. **Alpha-versioned domain tags.** Domain strings end in `.alpha` and
   the predicateType is `/alpha` — the schema is declared unstable, so
   nothing downstream can build on it yet. (→ D2: `schema_version` and a
   versioned predicateType from day one.)
3. **Signing is optional and key-management-shaped.** `--audit-sign-key`
   reaches into keyrings/1Password/K8s secrets — a large dependency
   surface caro must not swallow in one PR (external-sdk build-spike
   rule). (→ D5)
4. **No external anchoring.** A host attacker with the signing key can
   forge sessions; nono defers transparency logs/TSA to future work.
   Same honest deferral here. (→ out of scope)

**AEVS** (Fetch.ai, researched in ADR-032) contributes the anti-pattern:
fail-open evidence loss — invalid credentials put the SDK in silent
no-op mode, and buffer overflow evicts receipts with a "gap marker". An
evidence system that silently stops collecting evidence is the exact
failure auditors care about. AEVS also requires `api.aevs.fetch.ai` for
anchoring and verification — evidence that needs the vendor's backend.

**in-toto Attestation Framework** (spec v1.2) is the packet format
target. Four independent layers: Predicate (type-specific payload),
Statement (binds predicate to subject digests; `_type =
https://in-toto.io/Statement/v1`), Envelope (DSSE — authentication),
Bundle (grouping). It is consumed today by policy engines (in-toto-verify,
Binary Authorization) and by Sigstore. Adopting the Statement shape means
governance platforms that already parse in-toto/SLSA artifacts can parse
caro packets with zero custom work — and the layering lets caro ship the
Statement now and add the DSSE signature later without changing the
inner schema.

### Session/context lifecycle

nono's audit layer is session-scoped with a per-host chained ledger
across sessions; AEVS drains a SQLite buffer via a background thread.
Caro's constraint is stricter: **pure subprocess, no daemon, no
background state**. ADR-032 already solved the write side within that
constraint (append one receipt per invocation under `fd-lock`). Export
and verify are therefore *batch reads* over that file — no lifecycle to
manage, no redundant initialization to avoid, idempotent by
construction.

### What already exists in caro (codebase survey, 2026-08-12)

- **Types that exist**: `safety::ValidationResult`
  (src/safety/mod.rs:175 — `allowed, risk_level, explanation, warnings,
  matched_patterns, confidence_score`; Serialize+Deserialize),
  `SafetyDecision` (src/safety/mod.rs:189), `RiskLevel`
  (src/models/mod.rs:152; JsonSchema), `SuggestedRouting` +
  `from_risk_and_safety` (src/models/mod.rs:189/198 — the tier source of
  truth). `ExecutionResult` (src/execution/executor.rs:11) is **not**
  `Serialize` yet.
- **Sinks**: ADR-041 emitter and ADR-032 receipts are **ADR-only** —
  nothing in `src/` writes assessment records today. The reusable prior
  art is the JSONL appender in src/caroml/history.rs (append-only,
  `sha256:<hex>` digests) and the no-op-safe global collector in
  src/telemetry/mod.rs:65-86. `fd-lock = "4.0"` and `sha2 = "0.10"` are
  already dependencies — **the entire v1 needs zero new crates**.
- **Identity gap**: `DangerPattern` (src/safety/mod.rs:334) has **no
  stable id** — `ValidationResult.matched_patterns` carries prose
  descriptions (67 builtin patterns; only CVE rules have ids via
  dogma's `CompiledPattern.id`). Prose is a fragile join key for
  evidence. There is also **no pattern-set digest**: the closest thing
  to a "policy version" is `VersionInfo` (src/version.rs — git hash,
  build date, `cve_rule_count`).
- **Contract gap**: no exit-code registry exists in code (highest
  implemented contract code is 1; `EXIT_CODE_EDIT = 201` is a shell
  handshake). ADRs allocate 0–12 (0–6 ADR-024, 7 ADR-027, 8 ADR-029,
  9 ADR-032/034 — **collision, see D7**, 10 ADR-039, 11 ADR-040,
  12 ADR-044).
- **Namespace**: `caro export` is taken (CaroML runbooks,
  src/main.rs:588). `caro audit` is free.

### Phase 2 — what to replicate, what to avoid, where caro wins

Replicate from nono: the two-artifact split (append-log + sealed
attestation over its root), domain-separated hashing, chain + Merkle
root as complementary properties, canonical-bytes hashing (verify binds
to the exact bytes on disk, never a re-serialization), a single `verify`
subcommand with named independent checks and machine-readable output.

Avoid (by schema, not by workaround): AEVS's fail-open evidence loss
(D6); nono's in-band key trust (D4); nono's alpha-tag schema instability
(D2); both tools' binding evidence to *actions only* — caro's packet
binds each record to the **decision and the policy that produced it**
(pattern ids + policy digest), which is the field auditors actually ask
for ("what rule blocked this, and was it in force at the time?").

Caro's unique positioning: **verdict evidence, offline, standalone.**
nono attests what a sandboxed process did; AEVS attests tool calls with
a cloud anchor; Insygna/Drata score agents statically. Caro attests
*pre-execution safety decisions* — the layer the UK AISI sandbox-escape
report and the 1-in-3 human-approval data (Hermes 08-12, scan #1) just
showed is missing — with no daemon, no backend, no key server, and
verification that runs on an air-gapped laptop. Caro *feeds* the
governance platforms rather than competing with them.

---

## Decision

Ship an evidence-packet layer over ADR-032 receipts as `caro audit
export` and `caro audit verify`, with in-toto Statement/v1 as the packet
shape, tamper-evidence (chain + Merkle root) in v1, and cryptographic
signing deferred to a phase-gated follow-up behind an `ed25519-dalek`
build spike.

### D1 — Packet shape: in-toto Statement/v1, caro predicate

`caro audit export` emits one JSON document:

```json
{
  "_type": "https://in-toto.io/Statement/v1",
  "subject": [{
    "name": "caro-audit:<first_seq>-<last_seq>",
    "digest": { "sha256": "<merkle_root over included receipt lines>" }
  }],
  "predicateType": "https://caro.sh/attestation/evidence-packet/v1",
  "predicate": {
    "schema_version": 1,
    "hash_algorithm": "sha256",
    "record_count": 42,
    "chain_head": "<hex>",
    "merkle_root": "<hex>",
    "window": { "from": "<RFC3339 of first record>", "to": "<of last>" },
    "policy": { /* PolicyIdentity, D3 */ },
    "records": [ /* verified ExecutionReceipt objects, in seq order */ ]
  }
}
```

- The subject digest is the Merkle root computed over the **canonical
  NDJSON line bytes** of each included receipt (nono's exact-bytes rule).
  Domain separation: `"caro.audit.leaf.v1"`, `"caro.audit.chain.v1"`,
  `"caro.audit.merkle.v1"`.
- **Export verifies before it packs.** A packet is only emitted if the
  included receipt range replays cleanly (leaf hashes, chain, root). A
  broken chain is exit 9 (`ReceiptChainBroken`, ADR-032), never a packet.
- **Determinism**: the Statement contains **no generation timestamp**
  (in-toto puts time in predicates; caro's window comes from record
  timestamps). Serialization is `serde_json` with ordered struct fields.
  Same input file + same filter ⇒ byte-identical packet. This is a
  golden-file test (T8).

### D2 — Versioning from day one

`schema_version: 1` inside the predicate, `/v1` in the predicateType
URI. A consumer that sees an unknown major version must reject, not
guess. No `.alpha` anywhere — the schema committed by this ADR is the
schema, and changes follow the ADR process.

### D3 — `PolicyIdentity`: the "policy version" field, made real

New type (in `src/models/mod.rs`, next to `RiskLevel`):

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PolicyIdentity {
    pub caro_version: String,        // CARGO_PKG_VERSION
    pub git_hash: String,            // VersionInfo.git_hash_full
    pub pattern_set_digest: String,  // "sha256:<hex>", see below
    pub cve_rule_count: u32,
    pub safety_level: String,        // effective SafetyLevel at validation time
    pub config_digest: Option<String>, // sha256 of config.toml + patterns.toml bytes, if present
}
```

`pattern_set_digest` requires the missing identity primitive:

- `DangerPattern` gains `pub id: &'static str` — stable ids `CARO-P-001`
  … `CARO-P-067`, assigned once in `src/safety/patterns.rs` in
  declaration order, append-only forever (retired patterns keep their
  id reserved; the ADR-numbering no-gaps discipline, applied to
  patterns). CVE rules already have ids (`CVE-…`).
- `ValidationResult` gains **additive** `matched_pattern_ids:
  Vec<String>` (`#[serde(default)]`); `matched_patterns` (prose) is
  unchanged for compatibility.
- `pattern_set_digest` = SHA-256 over the sorted `(id, pattern,
  risk_level)` triples of builtin + CVE + user patterns, computed
  lazily once (`Lazy<String>` beside `DANGEROUS_PATTERNS`).

This closes the audit question "which rule, under which ruleset?" with
two machine-comparable fields — and it is the piece neither nono nor
AEVS has: their evidence identifies *actions*; caro's identifies the
*deciding policy*.

### D4 — Verification never trusts in-band keys (solving nono's gap)

`caro audit verify <packet.json> [--receipts <file>] [--json]` runs
named, independent checks:

| Check | What it proves |
|---|---|
| `schema` | `_type`/`predicateType`/`schema_version` are known |
| `records` | every embedded record's leaf hash matches its canonical bytes |
| `chain` | records re-chain to `chain_head`, in order, no gaps in `seq` |
| `root` | rebuilt Merkle tree matches `merkle_root` and `subject.digest` |
| `source` (optional) | with `--receipts`, every packet record is present and identical in the live log (nono's "ledger inclusion", minus the daemon) |
| `signature` (v2) | DSSE signature valid **for a key supplied out-of-band** |

Output: a serializable `VerifyReport { status, checks:
Vec<CheckResult>, reason: Option<String> }`; `status ∈ {verified,
mismatch}`. Rule inherited from nono's failure mode: when signing
lands (v2), **there is no key-from-packet mode at all** — `verify`
requires `--public-key-file` (or a pinned key path in config) to run
the `signature` check; absent that, the check reports `skipped`, never
`passed`. A skipped check can never produce `verified` if the caller
passed `--require-signature`.

### D5 — Signing is a phase, not a field, and the schema is ready for it

v1 packets are tamper-evident but unsigned. Signing (Ed25519, DSSE
envelope wrapping the Statement exactly as nono does) requires a new
crypto dependency ⇒ `external-sdk-integration.md` applies: a ≤100-LOC
phase-0 build spike for `ed25519-dalek` (license MIT/Apache-2.0 — OK;
MSRV check; optional dep behind new feature `audit-sign`; `pub fn
smoke()`; two verification builds) must merge before any signing PR.
Because DSSE wraps the Statement *unchanged* (base64 payload +
signatures array), v1 packets and v2 signed bundles share one inner
schema — consumers written today keep working. The signing key
strategy (file-based PKCS#8 only in v2; no keyring integrations) is
deliberately narrower than nono's.

### D6 — Fail-closed evidence (solving AEVS's gap)

Two rules, both contract-level:

1. `caro audit export` **never emits a packet it could not verify**
   (D1). No gap markers, no best-effort packets. Partial evidence is
   worse than absent evidence with a loud exit code.
2. An **empty but healthy** log exports a valid packet with
   `record_count: 0` and exits 0 — "nothing was decided" is itself
   evidence. A **missing** receipts file is exit 13, not 0: the caller
   asked for evidence that does not exist, and silence would be
   AEVS-style fail-open.

### D7 — Contract: subcommand, flags, exit codes

New subcommand in `src/main.rs` `Commands` (existing `#[derive(Parser)]`
convention):

```
caro audit export [--receipts <path>] [--from <RFC3339>] [--to <RFC3339>]
                  [--session <id>] [--output <path>|stdout]
caro audit verify <packet> [--receipts <path>] [--public-key-file <pem>]
                  [--require-signature] [--json]
```

Exit codes (this ADR materializes the registry ADR-024 designed — a
`#[repr(i32)] pub enum ExitCode` in `src/cli/mod.rs`, since no code
registry exists yet):

| Code | Name | Meaning |
|---|---|---|
| 0 | Success | packet written / packet verified |
| 1 | OperationalError | I/O, parse, bad flags (existing convention) |
| 9 | ReceiptChainBroken | source log fails replay during export (ADR-032) |
| 13 | EvidenceMismatch | `verify` found tampering, or export source missing (**new — claimed here**) |

Codes 2–8 and 10–12 belong to prior ADRs and are untouched. ⚠️ **Ledger
flag**: ADR-032 (§exit codes) and ADR-034 both claim 9 — written a week
apart, both "next free slot". This ADR sides with ADR-032 (first
writer, and `ReceiptChainBroken` is in this feature's dependency
chain); the collision must be resolved by a one-line amendment PR to
ADR-034 before either implements. Machines depending on this contract:
CI compliance jobs (`caro audit verify --json && …`), governance-platform
ingest scripts, and the future `caro guard` hosts that archive packets
per session.

### D8 — Files that change (minimal set, no new modules)

| File | Change |
|---|---|
| `src/safety/patterns.rs` | `id` on all 67 builtin patterns; `pattern_set_digest()` |
| `src/safety/mod.rs` | `DangerPattern.id`; `ValidationResult.matched_pattern_ids` (additive); populate in `validate_command` |
| `src/models/mod.rs` | `PolicyIdentity`, `EvidencePacket`, `EvidencePredicate`, `VerifyReport`, `CheckResult` (all Serialize+Deserialize+JsonSchema; lives beside `ExecutionReceipt` per ADR-032's placement) |
| `src/execution/executor.rs` | add `Serialize, Deserialize` to `ExecutionResult` (receipt embedding; flagged by survey) |
| `src/cli/mod.rs` | `ExitCode` registry enum (0–13, gaps documented per owning ADR) |
| `src/main.rs` | `Audit { Export, Verify }` subcommand + dispatch |
| `src/bin/generate-schema.rs` | emit `docs/schemas/evidence-packet.v1.schema.json` |

Implementation-order note (same posture as ADR-044): ADR-032's receipt
writer is not yet implemented. This ADR assumes its *contract*, not its
code. If export lands first, it operates on receipts.ndjson files
produced by tests/fixtures; the write path remains ADR-032's scope.

### D9 — Integration tests (known input → deterministic output + exit)

Fixtures in `tests/fixtures/audit/` (committed NDJSON files).

| # | Input | Expected |
|---|---|---|
| T1 | 5-receipt valid log | packet matches golden file byte-for-byte; exit 0 |
| T2 | same, `--from/--to` window selecting 2 | packet with `record_count: 2`, correct window; exit 0 |
| T3 | empty (0-line) log | valid packet, `record_count: 0`; exit 0 |
| T4 | missing receipts file | no packet; exit 13 |
| T5 | log with one flipped byte mid-file | export refuses; exit 9 |
| T6 | T1's packet → `verify` | `VerifyReport{status: verified}`, all checks `passed`/`skipped`; exit 0 |
| T7 | T1's packet with one record's field edited | `mismatch`, failing check named (`records`); exit 13 |
| T8 | export T1 twice | byte-identical output (determinism gate) |
| T9 | packet with `schema_version: 99` → `verify` | `mismatch`, check `schema`; exit 13 |
| T10 | `verify --require-signature` on unsigned packet | `mismatch`, check `signature: skipped→required`; exit 13 |

Plus unit: pattern-id uniqueness/append-only assertion in
`src/safety/patterns.rs` tests (extends the existing count asserts),
and `pattern_set_digest` stability across two computations.

### Out of scope (explicitly v2+)

- **Signing & DSSE envelope** — next phase, gated on the
  `ed25519-dalek` build spike (D5). Schema is forward-compatible now.
- **External anchoring** (transparency log, RFC-3161 TSA, remote
  witness) — nono defers this too; it is the only answer to
  host+key compromise, and it needs a network story caro must not
  bake into a pure-subprocess tool.
- **Merkle inclusion proofs as a CLI feature** (`caro audit prove
  <seq>`) — valuable, but no consumer exists yet; the root commits
  everything in v1.
- **Cross-session host ledger** (nono's `ledger.ndjson` layer) — the
  `source` check covers the single-log case; a host ledger reintroduces
  shared mutable state across invocations and needs its own design.
- **Push adapters** (Drata/Insygna/SIEM ingest, OTel spans) — the
  committed JSON Schema is the interface; adapters are downstream
  glue, per ADR-032's same call.
- **ADR-003 enterprise enrichment fields** (machine/user/department) —
  join keys (`session_id`, `agent_id`) are in the receipt already.
- **Retention/rotation** of receipts.ndjson — unchanged from ADR-032.

## Consequences

**Positive.** Caro gets the artifact the attestation channel wants,
built from two deps it already has (`sha2`, `fd-lock`) and one schema
standard the ecosystem already parses (in-toto Statement). The two
failure modes found in Phase 1 — fail-open evidence loss (AEVS) and
in-band key trust (nono) — are impossible by contract, not mitigated by
care. Pattern ids + `PolicyIdentity` fix a latent audit gap
(prose-only `matched_patterns`) that would have bitten every future
evidence consumer. The exit-code registry finally materializes in code.

**Negative / risks.** (a) Two ADR dependencies (032 receipts, 024
registry) are still unimplemented — this ADR adds a third layer of
specified-but-unbuilt contract; the mitigation is that T1–T10 run on
fixtures, so this feature is implementable and testable *first*.
(b) Unsigned v1 packets prove integrity relative to themselves and the
source log, not provenance — a sophisticated adversary who controls the
host can regenerate both. This is documented honestly in `verify`
output (`signature: skipped`) rather than papered over. (c) The
exit-9 collision (D7) must be settled in an ADR-034 amendment before
either ships. (d) 67 hand-assigned pattern ids invite copy-paste
mistakes — mitigated by the uniqueness unit test.

**Reviewable assumptions.** (1) Siding with ADR-032 on exit 9. (2)
`records` embedded in the packet (self-contained, auditor-friendly)
rather than referenced by hash only (smaller, privacy-friendlier) — a
`--digests-only` mode is a plausible v1.1 flag if packet size or
command-text sensitivity becomes an issue; the schema tolerates it
(records array + per-record hashes already present). (3) File-based
PKCS#8 only for v2 signing.

## Alternatives considered

1. **Custom packet schema (no in-toto).** Simpler to write, invisible
   to every existing attestation consumer. Rejected: the entire point
   is ingestion by governance platforms; in-toto Statement costs three
   wrapper fields.
2. **Sign in v1 with HMAC (sha2 already present, no new dep).**
   Symmetric keys mean the verifier holds the signing secret —
   third-party attestation is impossible. Rejected; tamper-evidence
   now, asymmetric signing next.
3. **Ship the OTel GenAI span exporter instead** (ADR-032's other
   deferred adapter). Observability pipelines are commoditizing
   (Hermes 08-12 scan #8: third entrant in two weeks); attestation is
   the open gap. Deferred, not rejected.
4. **Wait for ADR-032 implementation before scoping export.** Rejected:
   schema-first is this task's charter — designing the packet now
   surfaced two receipt-schema requirements (stable pattern ids,
   `PolicyIdentity`) that are cheaper to honor before the writer exists
   than after receipts are in the wild.
5. **Adopt nono outright** (Apache-2.0, Rust) as a dependency. Its audit
   layer is inseparable from its supervisor/sandbox architecture and
   would drag Landlock/Seatbelt machinery into a tool whose whole value
   is running before any sandbox. Rejected; adopt the construction
   (RFC-6962-style chain + root + DSSE), not the code.
