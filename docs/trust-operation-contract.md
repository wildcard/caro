# Trust-Operation Contract (v1, checkpoint 1 — proposed, not yet accepted)

**Status**: Checkpoint 1 of issue [#1714](https://github.com/wildcard/caro/issues/1714),
**proposed, not yet accepted.** Three rounds of independent review on
[PR #1715](https://github.com/wildcard/caro/pull/1715) have found and fixed
static-contract defects (round 2: 6 classes; round 3: 7 classes, including
a real CI license-policy failure; round 4: 5 classes, including two real
bugs in this PR's own cross-checks). This document states, plainly, what
changed and why. It never uses the word "frozen" to describe the overall
contract's acceptance state — only this PR's own content, pending the next
review pass, which decides acceptance.

**No behavioral code implements this yet.** Nothing here changes
`CliResult`, the safety validator, the executor, or any CLI output. A later
PR adds behavior against this proposed contract, once accepted; this PR
exists so that PR has a reviewed target instead of an undefined one.

**Actual current shape, mechanically checked** (not asserted by hand — see
`tests/trust_operation_v1_contract.rs::manifest_has_33_rows_and_four_schema_files_as_actually_shipped`
and `::no_jsonschema_crate_dependency_exists`):

| Artifact | Count |
|---|---|
| `manifest.json` rows | **33** (T001–T033) |
| Schema files (`manifest`, `plan`, `evidence`, `report`) | **4** |
| Contract tests in `tests/trust_operation_v1_contract.rs` | **60** |
| New Cargo dependencies | **0** (round 2 added `jsonschema` as a dev-dependency; round 3 removed it — see below — so the net count is zero) |
| New Python dev-tool dependencies | **1** (`jsonschema`, via `pip install`; not a Cargo dependency, outside `dependency-review.yml`'s scope — see round 4 below) |

### What changed in round 4 (this revision)

Round 3's fixes were themselves found to have 5 further classes of defect:

1. **The custom `minischema` evaluator silently accepted invalid schemas.**
   `Validator::new` only cloned the input JSON with no checking at all, so
   a schema with `"type": 42` or an unsupported keyword like `maxLength`
   would compile and then validate everything as trivially valid. Fixed
   two ways: (a) added a `preflight` pass to `Validator::new` that
   recursively rejects any keyword outside an explicit allow-list (the
   keywords `eval` actually interprets, plus named harmless annotations
   like `$comment`) and any malformed value for a keyword it does support
   — three `#[should_panic]` tests prove this; (b) added
   `scripts/validate_trust_operation_schemas.py`, which meta-validates all
   four schema files and every fixture with the REAL `jsonschema` PyPI
   package (a Python dev-tool dependency, outside
   `dependency-review.yml`'s scope, which only watches Cargo/npm
   manifests) as an independent ground truth, including 3 conformance
   vectors proving that real validator rejects malformed schema forms.
   Wired into CI as its own job.
2. **Report-schema step semantics couldn't express "no information."**
   `not_attempted` was being used for both "we have no information" and
   "we positively know this didn't happen" — now split into `unobserved`
   (no info, `observation_source` must be `none`) and `not_attempted` (a
   sourced, positive claim). Added rules: failed execution forces
   `workflow_outcome=failed` (R15); `execution.state=completed` forbids
   any failing-family step directly, not only via `workflow_outcome` (R16);
   cleanup's `observation_source` is now coupled to its state the same way
   steps are (R17); `execution.state=not_attempted` forbids any step
   claiming `started`/`completed` (R18); and `authorization.state=denied`
   now requires a real provenance (R7b), mirroring `host_permission`.
3. **DER checks accepted garbage.** The shallow one-level length check
   accepted an empty `30 00` SEQUENCE, ignored trailing bytes past a
   declared length, and the base64 decoder tolerated malformed padding;
   a PEM scanner that hit an unmatched trailing `BEGIN` silently stopped
   instead of erroring. Replaced with: a strict base64 decoder that
   rejects bad padding/length/characters; a PEM parser that must consume
   the ENTIRE file with nothing left over (`find_pem_blocks_strict`,
   returns `Result`, not `Vec`); and a real (if shallow) X.509 structural
   parser (`looks_like_complete_x509_der`) that walks the DER as nested
   TLVs requiring an EXACT tiling (`Certificate ::= SEQUENCE { SEQUENCE,
   SEQUENCE, BIT STRING }`, consuming every byte at every level) rather
   than a one-level length check. Four new adversarial artifacts prove
   each case independently rejected.
4. **The pieces were never cross-checked against each other.** The
   headline bug: swapping `verified_removal`'s evidence `final`
   observation to point at `final_content_mismatch.pem.fixture`'s bytes
   changed nothing any test checked, because nothing ever compared the
   `final` observation's content to `plan.expected_staged_hash`. Added a
   bounded, TEST-ONLY `derive_postcondition(plan, evidence)` reference
   cross-check (explicitly NOT a preview of product behavior — see its
   doc comment) and a battery of tests built on it: the exact swap above
   now flips the derived verdict to `failed`; a missing `final`
   observation derives to `unverified`; a stale `before` observation on
   the no-op plan derives to `failed`, never `already_satisfied`. Also
   fixed: backup/evidence `metadata` previously accepted `{}` (present
   but empty) as satisfying "metadata is not null" — now requires all
   three fields; added independent (DER-based, not plan-trusting)
   verification that the no-op plan's before artifact genuinely contains
   zero target occurrences; added a uniqueness check for
   `(step, step_snapshot_id)` pairs within a bundle; added an
   `evidence/invalid/wrong_schema_version.json` fixture.
5. **CI wiring and stray wording.** `trust_operation_v1_contract` is now
   in the `unit-tests` job's "Run contract tests" step, alongside the
   other named contract tests, across all three OS runners. A separate
   `trust-operation-schema-conformance` CI job runs
   `scripts/validate_trust_operation_schemas.py`. This document's title
   and opening no longer mix "frozen" (acceptance) with "pending
   acceptance" in the same breath — see the top of this document.
   Pre-existing, unrelated CI flakiness (an Ubuntu utility-detection
   issue, other untriaged failures, website/docs-site build failures) is
   explicitly NOT attributed to this contract's changes.

### What changed in round 3

Round 2's fix for round 1 introduced, or left unfixed, seven further
defects an independent review found:

1. **Report schema `R5` was simply wrong**, and `R11` was too weak. `R5`
   forbade `postcondition=failed` whenever `execution=not_attempted` — but
   a dry-run CAN legitimately read current state via supplied evidence and
   find it does not match expectations, with no execution attempted at
   all. Fixed: `R5` now requires `verification_basis=supplied_evidence` for
   `failed` (same as `verified`), not a blanket ban. `R11` previously only
   checked a non-null `failure_mode` label with no corresponding failing
   step; it now also requires at least one of the four named steps to
   actually carry a failing-family status (`anyOf` over
   `backup`/`stage`/`filter`/`install`). `R12`/`R13` are tightened: a
   failed cleanup must report `workflow_outcome=failed` exactly (not
   merely "not completed"), and `workflow_outcome=completed` now also
   requires that none of the four steps carry a failing-family status. A
   new `unknown` step status and a step-level `observation_source`
   coupling rule (`R14`) were added. See `reports/valid/dry_run_independently_failed_verification.json`,
   `reports/valid/stage_status_unknown_but_observed.json`, and five new
   `reports/invalid/*.json` fixtures.
2. **The PEM artifacts were fake and the fingerprints were wrong.**
   Round 2's "synthetic certificates" decoded to a plain base64-encoded
   label, not DER — and `target_sha256` hashed the raw PEM text section
   (BEGIN/END markers, line wrapping and all), not the certificate's DER
   bytes. Both are fixed: `tests/fixtures/trust_operation_v1/artifacts/`
   now holds real (but throwaway, self-signed, keys never committed)
   X.509 certificates generated with `openssl req -x509`, and
   `target_sha256`/`target_fingerprints` are now true DER fingerprints
   (sha256 of the base64-decoded certificate body). The contract test
   independently re-parses every PEM block, re-decodes its DER, and
   re-hashes it — it does not trust the plan's claim. New negative
   fixtures: `invalid_der.pem.fixture` (valid PEM envelope, non-DER
   payload), `reordered_two_targets.pem.fixture` (same fingerprint
   multiset, different byte-range assignment).
3. **Bindings were still loose.** Evidence observations now carry
   `backup_readability` (`unobserved`/`readable`/`unreadable`) — a
   completed backup claim requires `readable`, not just present metadata —
   and `consumes_before_snapshot_id`/`consumes_staging_snapshot_id` on
   `install` observations, checked against the ACTUAL earlier observations
   in the same bundle by `capture_sequence` (a dangling or forward
   reference is rejected). A mutating plan can no longer set
   `backup_policy.must_match_before_hash=false` either (round 2 only fixed
   `required`).
4. **`caro_observed` cannot authenticate anything on its own.** Every
   schema touching `caro_observed` now says explicitly: v1 ships no live
   collector, so every fixture using `caro_observed` is a hypothetical
   stand-in, and a caller writing that label into a JSON file is no more
   authoritative than `supplied` until a real collector exists. The
   scoped-approval digest now has an explicit **versioned** preimage
   (`compute_scope_binding_digest_v1` in the test file:
   `"trust_operation_scope_binding_v1\n" + command_digest + "\n" +
   normalized_destination + "\n" + approved_transformation + "\n" +
   sorted "sha256:count" pairs`), and a dedicated test recomputes it and
   rejects a well-formed-but-wrong digest.
5. **Manifest fixture rigor.** Every row now has EXACTLY ONE of a literal
   `input` or a fully-specified `input_construction` (never both, never
   neither — enforced by the metaschema). `input_construction` rows
   (`T030`, `T032`) now carry a frozen `expected_length`/`expected_sha256`
   that the test actually constructs (right-padding; repeated POSIX
   single-quote-escaped `sh -c '...'` wrapping) and checks, rather than
   trusting the row's own claim. `plan_ref` is now bound to the row's own
   command bytes (`command_digest` is sha256 of the row's literal text,
   not a placeholder), destination, context, and manifest version — all
   cross-checked. Report fixtures' `assessment.manifest_version` now
   matches `manifest.json`'s actual `manifest_version` (`1.1.0`; round 2
   left this at a stale `1.0.0`).
6. **`inline_base64` is removed entirely** from `artifact_ref` in both
   `plan.schema.json` and `evidence.schema.json`. It was speculative,
   unused by any fixture, and its exclusivity with `fixture_file` was
   asserted only in prose. `fixture_file` is now the only branch — trivially
   exclusive, nothing left to enforce.
7. **The `jsonschema` dev-dependency added in round 2 failed this
   repository's real `dependency-review.yml` license allowlist** (`MIT-0`
   via `fluent-uri`/`borrow-or-share`). Checking an alternative (`jsonschema`
   0.18.x) found it trades that failure for a different one (`Unicode-3.0`
   via `url`/`idna`/the icu4x stack) — every `$ref`-capable JSON Schema
   crate checked pulls in a license string outside the allowlist. Per the
   review's explicit instruction not to weaken that policy, **the
   dependency is removed entirely.** `tests/trust_operation_v1_contract.rs`
   now implements a minimal, self-contained draft-07-SUBSET evaluator
   (module `minischema`) using only `serde_json` and `regex` — both
   already-existing dependencies, zero new ones, zero new license
   exposure. Its documented boundary (what keywords it supports, what it
   deliberately does not implement) is in that module's doc comment.

## Why this exists

Apple's [eMudhra CA advisory](https://support.apple.com/en-us/149555)
(23 October 2026) publishes an optional command-line remediation that
backs up a certificate bundle, stages a filtered copy, checks it is
nonempty, installs it with explicit ownership/mode, and cleans up. A shell
pipeline that ends in `cleanup; exit 0` can report success even when the
install step failed. Caro's job in v1 is to say, honestly, which of six
separate questions it can answer about such an operation — and to refuse a
false "it worked."

The six separate questions, each with its own state in the frozen report
schema (`tests/fixtures/trust_operation_v1/schemas/report.schema.json`):

1. **Assessment** — did we recognize a trust mutation in the command text?
2. **Authorization** — did a human give fresh, scoped approval?
3. **Host permission** — did the OS actually let the operation proceed?
4. **Execution** — did the steps run, and which ones?
5. **Postcondition** — is the resulting file state what was approved?
6. **Cleanup** — did the cleanup step itself succeed, separately from 1–5?

`execution` itself carries three distinct signals, never collapsed into
one: `state` (the granular claim), `whole_shell_status` (the raw,
legacy-style exit-code signal, reported but never itself sufficient to set
`state`), and `steps` (a required object with `backup`/`stage`/`filter`/
`install` keys, each carrying its own `status` and `observation_source`).
`execution.state=completed` requires `steps.install.status=completed`;
`execution.state=failed` requires a non-null `failure_mode`. This is what
makes "the enclosing shell exited 0" structurally unable to stand in for
"the install step completed" — the exact gap the Apple-advisory pipeline
(`stage; install; cleanup; exit 0`) can otherwise hide. See
`report.schema.json` rules R10/R11 and the
`shell_exit_zero_alone_claims_completed.json` negative fixture.
## Links

- Manifest (frozen support boundary): `tests/fixtures/trust_operation_v1/manifest.json`
- Manifest metaschema: `tests/fixtures/trust_operation_v1/schemas/manifest.schema.json`
- Plan schema: `tests/fixtures/trust_operation_v1/schemas/plan.schema.json`
- Evidence schema: `tests/fixtures/trust_operation_v1/schemas/evidence.schema.json`
- Report schema: `tests/fixtures/trust_operation_v1/schemas/report.schema.json`
- Byte-level artifacts: `tests/fixtures/trust_operation_v1/artifacts/`. Real,
  valid, self-signed X.509 (EC P-256) certificates generated with `openssl
  req -x509` for this contract only (subjects named `synthetic-test-cert-*.invalid`,
  org `Caro Test Fixtures (not real)`). Their private keys were generated
  only to produce the certificate and were never written to this repository
  or saved anywhere — these files carry no secret material, and the
  certificates establish no trust relationship with anything.
- Plan / evidence / report instance fixtures (valid + invalid): `tests/fixtures/trust_operation_v1/{plans,evidence,reports}/{valid,invalid}/`
- Contract self-check (real schema validation + byte-level + cross-document checks): `tests/trust_operation_v1_contract.rs`
- Source issue: [#1714](https://github.com/wildcard/caro/issues/1714)
- Coordinate, don't duplicate: [#1684](https://github.com/wildcard/caro/pull/1684)
  (protected-path policy for file tools — different surface, the PreToolUse
  file-tool hook) and [#1687](https://github.com/wildcard/caro/pull/1687)
  (normalization views for encoding evasions — different surface, the
  shell-text pattern scanners). Neither touches `tests/fixtures/trust_operation_v1/`,
  `docs/trust-operation-contract.md`, or `CliResult`.

## Frozen reason codes

Exactly these twenty. A later PR may propose adding to this list only
through a reviewed diff to this document and the report schema together;
it must never repurpose an existing code for a new meaning.

| Code | Meaning |
|---|---|
| `invalid_evidence` | Evidence envelope/schema is malformed or an unknown schema version. Never treated as a negative or positive observation. |
| `missing_observation` | A required observation for this evidence/result combination was not supplied. |
| `stale_observation` | An observation's binding (operation/before/staging) does not match what the plan expects. |
| `inconsistent_evidence` | Two supplied observations contradict each other. |
| `bundle_parse_failed` | A certificate bundle observation is malformed, truncated, or has unconsumed blocks. |
| `backup_mismatch` | The backup does not match the bound before-snapshot, or fails the declared metadata policy. |
| `staging_mismatch` | Staged bytes do not match the approved staging hash. |
| `final_content_mismatch` | Final bytes do not match the approved staged bytes. |
| `metadata_mismatch` | Final owner/group/mode does not match the named profile. |
| `unsupported_syntax` | The command form is outside the manifest's supported syntax rows. |
| `unsupported_profile` | The target platform/profile is outside the manifest (e.g. no Windows trust-store support in v1). |
| `unsupported_transform` | The transformation kind is not `remove_approved_certificates_only`. |
| `missing_context` | A relative destination or a variable destination has no supplied cwd/context binding. |
| `authorization_required` | An identified or possible trust mutation has not yet received a fresh scoped decision. |
| `authorization_denied` | A human explicitly denied the scoped decision. |
| `permission_unobserved` | v1 has no live host-permission collector; this is the ordinary non-fixture state. |
| `execution_unobserved` | No execution observation was supplied or captured for this step. |
| `install_failed` | The install step itself failed, independent of cleanup or the enclosing shell's exit code. |
| `cleanup_failed` | The cleanup step failed, reported separately from the mutation result. |
| `already_satisfied` | Every approved target was already absent in a valid bound before-snapshot; no write was attempted. |

## Mode/entrypoint table (normative, frozen)

Deterministic `requires_scoped_human_confirmation` behavior per entrypoint.
No generic `auto`, `-y`, allowlist, judge confidence, vendor provenance, or
supplied approval claim may discharge this requirement. Every stronger
existing deny is preserved.

| Entrypoint/mode | Required v1 behavior |
|---|---|
| Generate, explain, or dry-run | Assess/report only. Never imply approval or execution. |
| Interactive execution (existing CLI path, including edit-then-execute) | Require a fresh explicit decision bound to the exact command digest, normalized destination, and intended trust effect. A changed command/target/effect invalidates the approval. If scope cannot be established, stop — never fall back to a blanket permission. |
| Noninteractive execution, including `auto`/`-y` | Refuse execution with confirmation required. v1 adds no persistent/scoped authorization channel, so there is no noninteractive exception. |
| Guard shadow | Report/log the static finding only. Authorization and execution stay unobserved. It does not enforce a human decision. |
| Guard enforce | Emit `ask`, or preserve a stronger existing `deny`. Never emit `allow`. Never assert the host actually prompted. |

## Normative evidence/outcome table (frozen)

Each row is a fixture to implement under `tests/fixtures/trust_operation_v1/`
in the behavioral PR, pinned to one of the acceptance-test identifiers below.

| Evidence/result combination | Required report |
|---|---|
| No execution requested / dry-run | `execution=not_attempted`; `host_permission=unobserved`; `postcondition=unverified` unless a separately supplied state-verification report is explicitly labeled as such. No claim of removal. |
| Stage/filter/install fails; cleanup succeeds; enclosing shell exits 0 | `execution=failed`, `workflow_outcome=failed`, failing step retained. Shell/cleanup status proves nothing about the postcondition. Missing final-state evidence stays `unverified`. |
| Install completed, matching complete post-read and all invariants; cleanup succeeded | `postcondition=verified`, `effect=removed`, `verification_basis=supplied_evidence`; cleanup succeeded. Authorization and host permission reported independently. |
| Same verified mutation, cleanup fails | Keep `postcondition=verified`/`effect=removed`; `cleanup=failed`, `workflow_outcome=failed`. Never rewritten as an install failure or unconditional success. |
| All approved targets already absent in a valid bound before-snapshot; approved effective removal count zero; no mutation attempted | `postcondition.effect=already_satisfied`, never `removed`; `execution=not_attempted`. State verification may be `verified` against supplied evidence. Backup not required for this no-write branch. A stale/changed before snapshot cannot be relabeled as this no-op (see `stale_observation`). |
| Missing observations, unknown provenance/bindings, stale or unverifiably fresh input | `postcondition=unverified` with the precise reason code(s). No success inferred from an overall exit code. |
| Evidence contradicts before/staged/final hash, exact multiset, raw preservation, backup, or metadata contract | `postcondition=failed` with the precise mismatch reason code(s). |
| Install reports failure but a complete bound post-read satisfies every state invariant | `execution=failed`, `workflow_outcome=failed`; `postcondition` may be `verified` against supplied evidence, but `effect=unknown` in v1. This alone is not `inconsistent_evidence`. |
| Correctly bound complete observation contains malformed/truncated/unparseable PEM | `postcondition=failed` with `bundle_parse_failed`. A complete negative observation is not a missing one. |
| Unsupported syntax/profile/transform | `postcondition=unverified` with the corresponding `unsupported_*` reason. The underlying safety finding and confirmation requirement are preserved. |
| Malformed evidence envelope or unknown schema version | `postcondition=unverified` with `invalid_evidence`. Never treated as a valid negative or positive bundle observation. |

A `verified` result establishes only the bundle-state contract above
against the supplied observations. It does not prove TLS remediation
effectiveness, that every app honored the change, authentic host
observation, or atomic enforcement.

## Support matrix summary

The reviewed boundary lives in `tests/fixtures/trust_operation_v1/manifest.json`.
Summary (see the manifest for the literal rows):

- **Profile**: `macos-bsd-cert-pem-v1` only (`/etc/ssl/cert.pem`, root/wheel,
  0644). No Windows trust-store support.
- **Shells**: `sh`, `bash`, `zsh`. Not `fish`, `csh`, `tcsh`, `ksh`, `nu`,
  PowerShell, or any other dialect.
- **Supported syntax**: literal quoted/unquoted words; `install` with
  enumerated BSD `-m/-o/-g` short forms; `cp` with enumerated `-f`/`-p`
  short forms; `>`/`>>` redirection; bare/absolute executable paths;
  `sudo`/`command`/`env` prefixes; literal `sh -c '…'` payloads.
  Long-option equivalents (`--mode=`, `--owner=`, `--group=`) are not
  enumerated and fall to `possible`/`unsupported_syntax`. **`tee -a` and
  `>>` (append) are `unsupported_transform`, not supported** (see
  "Detection vs. understanding" below) — this reverses revision 1, which
  incorrectly marked them `supported`.
- **Path handling**: lexical normalization of repeated separators, `.`,
  and `..`; a relative destination requires an explicitly supplied cwd. No
  shell evaluation or filesystem probing resolves a destination.
- **Explicitly unsupported** (not a gap — a frozen boundary; see manifest
  rows T020–T032): alias expansion, function expansion, live symlink
  resolution, variable substitution, command substitution, dynamic
  destinations, unsupported shells, unenumerated option forms,
  unsupported profiles, unsupported transformation kinds, missing cwd,
  over-limit input (`T030`, padded to `config_bounds.max_command_length_strict + 1`
  = 1001 bytes — the project's actually-configured strict limit from
  `src/safety/mod.rs`, not an invented number), and wrapper nesting past
  the configured bound (`T032`, nested one level past
  `config_bounds.max_wrapper_depth` = 3, from `src/safety/segments.rs::MAX_DEPTH`).
  `tests/trust_operation_v1_contract.rs::source_safety_config_still_matches_the_pinned_config_bounds`
  fails if either source constant ever drifts from these pinned values.
- **`possible` vs `not_detected`**: `possible` requires observable evidence
  — a recognized write-shaped operation with a protected-target token, or
  supplied context mapping an opaque destination to the protected target,
  with incomplete supported resolution. An arbitrary opaque alias with no
  such evidence is `not_detected`, not `possible` — Caro does not claim to
  have inspected every alias.
- **Detection vs. understanding, and why `plan_ref` exists.**
  `mutation_detection` (identified/possible/not_detected) is about
  recognizing a write-shaped operation against the protected target from
  command text alone. `transformation_understanding` (supported/unknown/
  unsupported) is a stronger claim — that Caro knows *what the bytes will
  become* — and **can only be `supported` when the manifest row carries a
  `plan_ref`** pointing at a real, byte-bound plan fixture under
  `tests/fixtures/trust_operation_v1/plans/valid/` (enforced by
  `manifest.schema.json`'s `plan_ref` conditional, not just by convention).
  Plain command-text recognition (`mutation_detection=identified`, no bound
  plan) is `unknown`, never `supported` — this is why T001, T003, and
  T006–T014 report `unknown` rather than the `supported` revision 1
  claimed. T002, T028, and T031 are the same `cp -f` shape with three
  different filenames: T002 carries a `plan_ref` and is `supported`; T028
  and T031 carry none and are both `unknown`, proving the filename itself
  never drives the verdict (see
  `tests/trust_operation_v1_contract.rs::filename_alone_never_changes_the_verdict_for_an_unbound_cp_form`).
  An append (`tee -a`, `>>`) is a categorically different transformation
  kind than `remove_approved_certificates_only` — it can only add bytes —
  so it is `unsupported_transform` regardless of any plan. Read-only
  inspection, hashing, and copying a bundle out for review remain benign
  controls (manifest rows T017–T019), not replacements for the mutation
  check.
- **Context provenance in fixtures that need out-of-band mappings**
  (T016, T020–T022): these carry `context_provenance: "supplied_fixture"`
  to make explicit that the mapping (an alias body, a function body, a
  symlink target) is injected ONLY so the fixture can exercise the
  `not_detected`/`unsupported` path deterministically. A real run has no
  such mapping — v1 ships no live alias/function/symlink resolver — and
  must not invent one; this is documented per-row rather than left
  ambiguous.
- **Conservative fallback preserves the existing safety verdict**
  (T020, T021, T025): a row whose shell or form puts it outside this
  contract's declared coverage carries a `verdict_floor_note` stating
  explicitly that `src/safety/`'s own pattern-based risk assessment of the
  literal command text is independent of, and never downgraded by, a
  trust-operation `not_detected`/`unsupported` result. Being unable to
  assess a *transformation* never means falling open on the existing
  *safety* verdict.

## Acceptance-test identifiers (frozen names, not yet implemented)

These names are reserved. The behavioral PR implements exactly these
tests against this contract; it does not rename, drop, or add to this
list without a reviewed diff to this document.

- `trust_mutation_requires_scoped_human_confirmation_across_modes`
- `guard_shadow_never_claims_authorization`
- `supplied_approval_never_authorizes_runtime`
- `shell_zero_without_observations_stays_unverified`
- `failed_install_survives_successful_cleanup`
- `verified_final_state_preserves_execution_failure`
- `malformed_observed_bundle_fails_postcondition`
- `invalid_evidence_stays_unverified`
- `cleanup_failure_preserves_verified_state`
- `absent_target_is_already_satisfied_without_write`
- `stale_bindings_never_verify`
- `legacy_executed_and_dry_run_semantics_unchanged`
- `default_minimal_and_judge_prompts_preserve_contract`

## Explicit non-goals (v1)

No live trust-store read/write, no privileged runner/collector/remediation,
no provider/model loading or inference, no host-permission collector, no
new runtime knowledge-index or telemetry ingestion, no global
trust-management service, no automatic rollback, no autonomous trust
decision. `caro guard test`'s existing JSON keys, exit codes, and PreToolUse
hook wire shapes are unchanged by this contract.
