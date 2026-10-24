# Trust-Operation Contract (v1, checkpoint 1, revision 2 — frozen, not implemented)

**Status**: Checkpoint 1 of issue
[#1714](https://github.com/wildcard/caro/issues/1714), revision 2 after the
independent review on [PR #1715](https://github.com/wildcard/caro/pull/1715).
This document, the manifest, and the schemas it links are the frozen support
boundary and report contract. **No behavioral code implements this yet.**
Nothing here changes `CliResult`, the safety validator, the executor, or any
CLI output. A later PR adds behavior against this frozen contract; this PR
exists so that PR has a reviewed target instead of an undefined one.

### What changed in revision 2

The first draft of this contract was rejected on independent review for six
classes of static defect: forbidden report states were representable
(`confirmed_scoped` with a `supplied_claim` or a missing digest; `granted`
host permission with no provenance; `verified` postcondition with no basis;
`workflow_outcome=completed` after a failed cleanup); the evidence/plan
schemas carried claimed hashes with no real bytes behind them; plan↔evidence
bindings were inconsistent (no shared `evidence_id`, no canonical
context-equality rule, optional step status); whole-shell status could not
be told apart from step-level status; the manifest confused *detecting* a
mutation with *understanding* its transformation (append rows wrongly
claimed `supported`; two identical `cp` commands got different verdicts
based only on a filename); and the manifest invented a `4096`-byte bound
instead of citing the project's actually-configured limits. All six are
fixed in this revision — see the `$comment` fields inline in the schemas,
the `revision 2` notes in `manifest.json`, and the new fixture-backed tests
in `tests/trust_operation_v1_contract.rs` (34 tests, up from 11, now using a
real JSON Schema validator — `jsonschema`, dev-dependency only,
`default-features = false`, so no network/TLS code is pulled in — instead of
checking only that the right words appear in prose).

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
- Synthetic byte-level artifacts (NOT real certificates): `tests/fixtures/trust_operation_v1/artifacts/`
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
