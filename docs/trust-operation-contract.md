# Trust-Operation Contract (v1, checkpoint 1 — proposed, not yet accepted)

**Status**: Checkpoint 1 of issue [#1714](https://github.com/wildcard/caro/issues/1714),
**proposed, not yet accepted.** Ten rounds of independent review on
[PR #1715](https://github.com/wildcard/caro/pull/1715) have found and fixed
static-contract defects (round 2: 6 classes; round 3: 7 classes, including
a real CI license-policy failure; round 4: 5 classes, including two real
bugs in this PR's own cross-checks; round 5: 4 classes plus a small fix,
including a real Windows CI failure; round 6: 3 classes, including
replacing the hand-written X.509 heuristic with a real conformant parser;
round 7: 2 classes, consolidating the test-only fixture-integrity path
into one function every test now routes through; round 8: 4 classes,
binding the range-transformation check to real certificate boundaries,
adding evidence-to-report propagation, correcting the missing-vs-
contradictory taxonomy further, and reverting round 7's own wrong fixture
fix; round 9: 3 classes, correcting round 8's own OVER-correction on null
snapshot bindings, forcing effect=unknown on any mutation-step failure,
and binding `artifact_ref`'s own hash/size fields; round 10: 1 class,
independently resolving the PLAN's own `before_artifact_ref`/
`expected_staged_artifact_ref` against real bytes on disk, for both
branches, before either trusts them; round 11: 1 class, a regression
round 10 itself introduced — the new gate unconditionally rejected an
absent (schema-optional) plan-level `artifact_ref`, instead of validating
it only when present). This document states, plainly,
what changed and why. It never uses the word "frozen" to describe the
overall contract's acceptance state — only this PR's own content,
pending the next review pass, which decides acceptance.

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
| Contract tests in `tests/trust_operation_v1_contract.rs` | **120** |
| New Cargo dev-dependencies | **1** (`x509-parser`, `default-features = false` — round 2 also added `jsonschema`; round 3 removed it, see below, so `x509-parser` is the only one actually shipping; both are dev-only, never linked into the `caro` binary) |
| New Python dev-tool dependencies | **1** (`jsonschema`, via `pip install`; not a Cargo dependency, outside `dependency-review.yml`'s scope — see round 4 below) |

### What changed in round 11 (this revision)

Round 10's own fix introduced one regression:

1. **Gate 4.5 unconditionally rejected an absent plan-level `artifact_ref`.**
   The unchanged `plan.schema.json` makes `before_artifact_ref` and
   `expected_staged_artifact_ref` OPTIONAL -- neither is in the schema's
   top-level `required` list, so a conforming plan may omit either one.
   Round 10's `resolve_plan_artifact_ref` calls ran unconditionally, so
   a valid plan that simply omitted `before_artifact_ref` (with
   everything else -- a complete, bound evidence chain -- correct) hit
   `artifact_ref["fixture_path"]` on a null value, got `Err(())`, and
   fell into `failed`/`unknown` even though nothing was actually wrong.
   Fixed by guarding each call on `!plan["<field>"].is_null()`: an
   absent field is now simply skipped (no check, no requirement),
   leaving the evidence-derived verdict from the no-op/mutation branch
   as the sole determinant. A PRESENT field is still fully checked --
   wrong hash, wrong size, and wrong path (pointed at different real
   bytes) all still fail exactly as round 10 left them.

4 new tests cover the omitted-before and omitted-staged cases in both
the no-op and mutation branches, each removing the key entirely (not
setting it to `null`, which would itself be schema-invalid for a
present-but-wrong-typed field) and asserting the untampered,
evidence-derived verdict survives unchanged.

Also corrected: the round-10 self-review undercounted its own added
tests as 9; the actual diff added 8 new test functions (confirmed via
`git diff` against the round-10 commit range). The GitHub comment has
been corrected to match.

Preserved per the review's closing instruction: everything from rounds
7-10 not named above (unified entry point, R20, range/fingerprint/count
binding, bidirectional report consistency, the null-binding
classification fix, the mutation-failure effect fix, the real
x509-parser, strict base64 padding, round 10's real-bytes-backed
`resolve_plan_artifact_ref` gate for PRESENT fields, Windows/macOS
116/116 passing locally before this round, Python schema conformance).

### What changed in round 10

Round 9's own fixes left one residual artifact-binding gap:

1. **The shared checker never independently resolved the PLAN's own
   `before_artifact_ref`/`expected_staged_artifact_ref` against real
   bytes read from THEIR OWN `fixture_path`.** Round 9's checks only
   ever compared the plan's declared `artifact_ref` fields against each
   other (`before_artifact_ref.sha256 == before_hash`) or against
   already-resolved EVIDENCE bytes -- never against real bytes read
   from the plan's own path. Concretely: the no-op branch never
   consulted `before_artifact_ref` at all, so a tampered
   `size_bytes`/`sha256` there still verified `already_satisfied`; the
   mutation branch's checks were internal-consistency-only, so swapping
   `before_artifact_ref.fixture_path` to a DIFFERENT, real, existing
   fixture file (leaving `sha256`/`size_bytes`/`before_hash` all
   untouched and mutually consistent with EACH OTHER) still verified
   `removed`; and `expected_staged_artifact_ref` was never checked by
   either branch, at all. Fixed with a new `resolve_plan_artifact_ref`
   helper -- reads the real bytes at the artifact_ref's own
   `fixture_path`, independently rehashes them, and cross-checks both
   the declared `sha256` and `size_bytes` fields against those real
   bytes -- called as a new gate (4.5) BEFORE either the no-op or
   mutation branch runs, so both `before_artifact_ref` and
   `expected_staged_artifact_ref` are resolved unconditionally, for
   every case, regardless of which branch would otherwise apply. The
   round-9 per-field checks inside the mutation branch (which only ever
   compared declared fields to each other) are now redundant and
   removed, leaving one shared, real-bytes-backed check.

8 new tests cover both the no-op and mutation paths for all three
tamper shapes the review named (wrong hash, wrong size, path swapped to
different existing bytes), plus a sanity test proving the new gate
doesn't become stricter than the frozen happy paths. (Corrected in
round 11 below: the original round-10 self-review miscounted this as 9.)

Preserved per the review's closing instruction: everything from rounds
7-9 not named above (unified entry point, R20, range/fingerprint/count
binding, bidirectional report consistency, the null-binding
classification fix, the mutation-failure effect fix, the real
x509-parser, strict base64 padding, Windows/macOS 116/116 passing
locally, Python schema conformance).

### What changed in round 9

Round 8's own fixes were found to have 3 further classes of defect:

1. **Round 8 itself OVER-corrected on null snapshot bindings.** Round 8
   added a test asserting a null `consumes_before_snapshot_id` -- when a
   real `before` observation genuinely exists -- is a contradiction
   (`failed`). That was WRONG: existence of a real snapshot does not
   prove WHICH snapshot `install` actually consumed. A null claim is a
   MISSING claim, not a demonstrated contradiction naming a specific
   wrong snapshot. Gate 11 now checks `is_null()` on install's own claim
   FIRST and separately, returning `unverified`; only a NON-NULL but
   WRONG id (naming a real-looking id that isn't the actual observation's
   own) still returns `failed`. The round-8 test is corrected (renamed,
   inverted assertion) and a new companion test proves the non-null-wrong
   case still fails.
2. **Any mutation-step failure must force `effect=unknown`, not just
   `install`'s own failure.** Round 8's effect computation only ever
   consulted `install["step_status"]` -- a truthfully reported
   `backup`/`stage`/`filter` failure (evidence AND report agreeing,
   satisfying the round-8 propagation rule) with a completed `install`
   and an independently verified `final` state still produced
   `effect=removed`. Fixed: `backup`/`stage`/`filter`'s own step_status
   is now checked too; any one of them in the failing family forces
   `effect=unknown` regardless of what `install`/`final` look like on
   their own. `cleanup` is deliberately excluded from this check -- a
   cleanup-only failure may still retain `effect=removed` (R12/R13's
   existing decoupling) -- but the cleanup propagation check now also
   explicitly requires `workflow_outcome=failed` in code (previously
   relying only on R12 at the schema level, which the round-8 code
   comment described imprecisely as "not forcing execution/workflow,"
   conflating `execution.state` -- correctly not forced -- with
   `workflow_outcome` -- which R12 does force).
3. **The content resolver validated `content_hash` but ignored
   `artifact_ref.sha256`/`size_bytes`.** `independently_resolve_content`
   rehashed the real bytes and compared against the observation's own
   `content_hash`, but never checked whether `artifact_ref`'s own
   declared `sha256`/`size_bytes` fields agreed with those same real
   bytes -- an internally inconsistent evidence bundle (right
   `content_hash`, wrong `artifact_ref` fields) passed undetected. Fixed
   in the one shared helper, so both the mutation path and the no-op path
   get the fix automatically. Separately: a no-op plan's range entry
   claiming `count: 0` while still declaring a non-empty `byte_ranges`
   array is internally inconsistent (simultaneously claiming "nothing to
   remove" and "here are bytes to remove"); `plan_is_noop` alone can't
   catch this (it only looks at `count`), so a new consistency check
   runs inside the no-op gate itself before trusting the no-op
   classification.

5 new/corrected tests cover all three points, exercised entirely through
the one shared `validate_fixture_case` entry point (no bypass path).

Preserved per the review's closing instruction: everything from rounds
7-8 not named above (the unified entry point, R20's tightening, the real
x509-parser, strict base64 padding, the range/fingerprint/count binding,
bidirectional report consistency, Windows/macOS 108/108 passing locally,
Python schema conformance). The pre-existing Ubuntu utility-detection
failure remains separate.

### What changed in round 8

Round 7's fully-consolidated `validate_fixture_case` and R20 were found
to have 4 further classes of defect:

1. **Range transformation applied byte ranges without binding them to
   real certificate fingerprints, occurrence counts, or boundaries.**
   `apply_approved_removal_ranges` used ONLY the declared `start`/`end`
   offsets to slice `before_bytes` -- it never checked that a
   `byte_ranges` entry actually lands on a real, independently-parsed
   certificate boundary, that the DER there actually hashes to the
   entry's own declared `target_sha256`, that the declared `count`
   matches the real occurrence count, or that `plan.target_fingerprints`
   (the declared FULL multiset) matches the real before bytes at all.
   Concretely: relabeling `approved_removal_ranges[i].target_sha256` to
   an unrelated value and recomputing the scope-binding digest over the
   tampered plan still verified removal, because nothing cross-checked
   the label against the real bytes at that position. Replaced with
   `bind_and_apply_approved_removal_ranges`, which independently re-parses
   the before bytes (full X.509 parsing, not just base64), binds
   `plan.target_fingerprints` to that real multiset exactly, binds each
   range's declared `count` to both its own `byte_ranges.len()` and the
   real occurrence count, and binds each `byte_ranges` entry's
   `target_sha256` to the ACTUAL DER fingerprint at that byte offset.
   Separately, `before_bytes` is now also bound directly to
   `plan.before_hash` (not merely to the evidence observation's own
   declared `content_hash`), and `plan.before_artifact_ref`'s own
   `sha256`/`size_bytes` fields are bound to the same real bytes.
2. **Report-to-evidence consistency was one-way.** The round-7 check
   only verified a report never claims MORE than the evidence backs; it
   never checked the reverse -- evidence showing a real mutation-step
   failure could be hidden behind a report claiming `unobserved`/
   `source: none` while `execution.state`/`cleanup.workflow_outcome`
   claimed completed. `report_steps_consistent_with_evidence` is now
   bidirectional: a known evidence failure in `backup`/`stage`/`filter`/
   `install` MUST propagate into the report (acknowledged status, AND
   `execution.state=failed`, AND `workflow_outcome=failed`). `cleanup`
   gets the same propagation WITHOUT forcing execution/workflow to
   failed -- a failed cleanup must become visible in `cleanup.state`
   without erasing an otherwise independently verified mutation outcome
   (the same decoupling R12/R13 already establish). Separately, the
   same-step conflict check (gate 3) now also rejects competing
   observations that differ only in `metadata` or `source`, not just
   hash/status.
3. **Missing-vs-contradictory classification had two more gaps.** A
   `null` `consumes_before_snapshot_id`/`consumes_staging_snapshot_id`
   when a REAL before/stage observation genuinely exists is a
   contradiction (FAILED) -- there IS something to bind to, and
   install's own claim is null; this already worked correctly, now with
   an explicit regression test. `final.parse_outcome=not_applicable`
   was being treated the same as `malformed` (FAILED); corrected to
   UNVERIFIED -- `not_applicable` means no parse was ever attempted
   (missing information), while `malformed`/`truncated`/
   `unconsumed_blocks` mean a parse WAS attempted and genuinely failed.
   A `null` `certificate_fingerprints` declaration was being converted
   to an asserted EMPTY multiset and compared (and usually failed)
   against the real, non-empty bytes; corrected to UNVERIFIED -- null
   means no declared count was ever supplied, not "zero, asserted."
4. **Round 7's own fixture fix was wrong, and is reverted.** Round 7
   changed `authorization_denied_with_provenance.json`'s
   `execution.state` from `unobserved` to `not_attempted`, reasoning
   that a denied authorization "genuinely establishes" non-execution.
   That reasoning does not hold: the denial's `provenance` is
   `supplied_claim` -- a CLAIM about authorization, not an actual Caro
   observation of non-execution. A supplied claim can be wrong, stale,
   or bypassed; only a DISTINCT, actually-observed non-execution signal
   would establish genuine `not_attempted`. Reverted `execution.state`
   back to `unobserved` and `cleanup.workflow_outcome` to `unverified`
   (R20 is NOT weakened -- it still correctly rejects
   `unobserved`+`not_attempted`; this fixture simply no longer claims
   `not_attempted` at all). 10 new round-8 adversarial tests cover all
   four points, including the exact relabel-and-recompute-digest attack
   and a test proving an honestly-reported cleanup failure does NOT
   erase an independently verified `removed` mutation outcome.

Preserved per the review's closing instruction: the unified
`validate_fixture_case` entry point and R20's tightening (both from
round 7, confirmed repaired and left in place), Windows/macOS 103/103
passing locally, and Python schema conformance (re-run clean against
every round-8 change). The pre-existing Ubuntu utility-detection failure
remains separate and unattributed to this contract.

### What changed in round 7

Round 6's `validate_case`/`validate_full_case` split, and R20, were found
to have 2 further classes of defect:

1. **The shared test-only fixture-integrity path existed but wasn't
   actually used everywhere, and had real gaps of its own.** Round 6's
   `validate_full_case` wrapped a narrower `validate_case`, but almost
   every adversarial test called `validate_case` directly — the wrapping
   function had only 3 call sites. That split is now gone entirely: a
   single `validate_fixture_case(plan, evidence, report)` is the ONLY
   entry point, and every test in the suite that exercises a triad calls
   it. Along the way, five concrete gaps were closed: (a) a FALSE
   POSITIVE the review named explicitly — removing the `before`
   observation while setting `install.consumes_before_snapshot_id = null`
   used to "verify," because a missing before id defaulted to
   `Value::Null` and compared equal to the tampered field's own null;
   `before` is now a REQUIRED, independently-resolved observation on the
   mutating path, checked before any binding comparison runs, so a
   missing `before` can never reach that comparison at all; (b) content
   hashes were previously compared only to OTHER declared fields (e.g.
   `backup.content_hash != plan.before_hash`), never independently
   rehashed from the real artifact bytes — a new
   `independently_resolve_content` helper reads the real bytes at
   `artifact_ref.fixture_path` and rehashes them for every content-bearing
   observation (before/backup/stage/final), so a tampered `content_hash`
   that happens to equal some OTHER wrong value can no longer pass; (c)
   `independent_fingerprint_multiset` grouped PEM blocks by their DER
   sha256 without ever calling the new X.509 parser on them, so a block
   that decoded cleanly from base64 but was NOT a real certificate could
   still contribute a fingerprint count — fixed to call
   `looks_like_complete_x509_der` per block, failing the whole re-parse on
   any invalid block; (d) `validate_case` never checked destination/
   context binding, range transformation (that applying the plan's OWN
   declared byte ranges to the real `before` bytes actually produces the
   real `stage` bytes), freshness (install's own `capture_sequence`
   strictly after what it claims to consume), or the scoped-approval
   digest (when authorization claims `confirmed_scoped`) — all four are
   now wired into the one path, with the digest check exercised for real
   against `verified_removal.json`'s own genuine `confirmed_scoped`
   claim, not a synthetic addition; (e) the report-consistency check
   (`report_steps_consistent_with_evidence`) only checked
   `backup`/`stage`/`filter`/`install` step STATUS equality, never
   `cleanup` and never WHICH source backed a claim — extended to check
   `cleanup` (via its own `succeeded`/`failed`/`not_attempted` vocabulary
   mapped onto the observation `step_status` enum) and to require the
   report's `observation_source` match the evidence observation's own
   `source` field, not just its status. Classification was also corrected
   per the review's closing instruction: a final observation with `null`
   metadata is MISSING information (`unverified`), not a demonstrated
   contradiction (`failed`, round 6's own overcorrection) — the taxonomy
   is now applied consistently throughout: missing/null fields derive
   `unverified`; present-but-wrong fields derive `failed`. 18
   `validate_fixture_case_*` tests exercise this, each with an EXACT
   classification assertion (`assert_eq!` on state, not a weaker
   `assert_ne!(.., "verified")`), including the false-positive fix, the
   destination-binding check, the range-transformation check, the
   freshness check, the scoped-digest check, and the extended
   cleanup/source report check.
2. **R20 (round 6) explicitly allowed `execution.state=unobserved` paired
   with `workflow_outcome=not_attempted`.** Tightened to require
   `execution.state` be EXACTLY `not_attempted` (ESTABLISHED, not merely
   "no information"). Round 7's own fixture fix for this was WRONG and
   is corrected again in round 8 (see round 8's point 4 below):
   `authorization_denied_with_provenance.json`'s `execution.state` is
   `"unobserved"`, not `"not_attempted"`, because a *supplied_claim*
   denial of authorization does not itself prove execution never
   occurred -- it is a claim about authorization, not an actual Caro
   observation of non-execution. A new negative fixture,
   `workflow_not_attempted_with_unobserved_execution.json`, and a
   dedicated test keep the `unobserved` gap from reopening; the retained
   dry-run test also covers `already_satisfied_no_op.json` (a GENUINE
   no-op, independently confirmed by DER re-parsing, not merely a
   supplied claim).

Preserved per the review's closing instruction: the real `x509-parser`
with full consumption, strict base64 padding-bit rejection, the
independent retained-comment artifact comparison, 93/93 passing locally
(Windows/macOS CI confirmation from round 5/6 not reopened), and Python
schema conformance (re-run clean against every round-7 change). This
document does NOT claim `cargo-deny` has verified the new dependency --
that job's own pre-existing, unrelated failure (deprecated config keys)
is a separate, already-known issue, not something this PR's changes
caused or can speak to; the license check here is `cargo metadata`
against `dependency-review.yml`'s allowlist directly (see round 6 above).

### What changed in round 6

Round 5's fixes were themselves found to have 3 further classes of defect:

1. **The fixture-case validator still had five real gaps.** `validate_case`'s
   final-metadata check was written as `if !metadata.is_null() &&
   !metadata_matches_expected(..)`, which SKIPPED the check entirely when
   metadata was null instead of failing it — the exact opposite of
   correct, since `metadata_matches_expected` already returns `false` for
   null on its own. Fixed by removing the redundant-and-wrong guard.
   `final.parse_outcome` was never checked at all, so a final observation
   admitting `malformed` still verified; now checked. The `stage`
   observation (the bytes install is claimed to have actually installed)
   was never independently checked against `plan.expected_staged_hash` at
   all; now checked, with a dedicated missing/mismatched-stage test each.
   The final observation's declared `certificate_fingerprints` were never
   cross-checked against an independent re-parse of the real final
   artifact bytes; now checked via a shared
   `independent_fingerprint_multiset` helper. The install step's
   `consumes_before_snapshot_id`/`consumes_staging_snapshot_id` were
   required to be PRESENT by the schema but never checked for EXACT
   equality against the real `before`/`stage` observation ids in
   `validate_case` itself; now checked. Separately, the no-op path's
   `find_pem_blocks_strict(..).unwrap_or_default()` silently converted a
   genuine parse failure into "zero certificates found" — exactly
   backwards, since a malformed before artifact must never read as
   already-satisfied; fixed to fail the case on a parse error. Finally, a
   new `validate_full_case(plan, evidence, report)` — the single path ALL
   positive/adversarial paired triads now route through — adds a report
   `report_steps_consistent_with_evidence` cross-check and caught a real
   bug in `verified_removal.json` itself: its report claimed `filter`
   completed (sourced) while its evidence bundle had NO `filter`
   observation at all. Fixed by adding the missing `filter` observation
   to the evidence fixture (reflecting that a transformation step
   logically did happen) rather than weakening the report's claim. 9 new
   adversarial tests plus 2 happy-path sanity tests for the fuller path.
2. **X.509 checking was still child-tag/count heuristics.** Round 5's
   "deepened" check (≥6 non-empty `tbsCertificate` children, ≥1 INTEGER,
   ≥4 SEQUENCE) is satisfied by the review's counterexample DER (`30 1c 30
   12 02 01 01 30 01 00 30 01 00 30 01 00 30 01 00 02 01 01 30 03 06 01 2a
   03 01 00`) despite its "fields" being empty placeholder SEQUENCEs, not
   real `issuer`/`validity`/`subject`/`subjectPublicKeyInfo`. Replaced the
   entire hand-written DER-TLV-walking approach with `x509-parser` (a
   real, independently conformant DER/ASN.1 certificate parser,
   `default-features = false` so no chain/trust/expiry/signature
   validation is enabled — pure syntax parsing only), requiring full byte
   consumption (`rem.is_empty()`). A new test reconstructs the round-5
   heuristic inline to PROVE it would have accepted the counterexample,
   then proves the real parser rejects it. Separately: base64 unused
   padding bits (RFC 4648 §3.5) are a distinct strictness concern from DER
   BIT STRING unused bits — `base64_decode_strict` now rejects non-zero
   unused trailing bits in the final sextet, with a dedicated unit test
   exercising the encoding layer alone (no DER/PEM involved). And the
   comment/whitespace preservation test only ever searched the ORIGINAL
   (untouched) file for substrings, so it could never catch a removal
   that corrupts surrounding bytes; added a new test that actually applies
   a removal (cert A's exact byte range) to the commented bundle and
   compares the result byte-for-byte against an independently authored
   (hand-constructed, never run through this contract's own code)
   expected artifact,
   `commented_whitespace_bundle_after_removing_cert_a.pem.fixture`.
3. **`stage_status_unknown_but_observed.json` had execution.state=started
   (with `backup` already completed) paired with
   `cleanup.workflow_outcome=not_attempted`** — `not_attempted` is a claim
   that nothing happened at all, which directly contradicts a `started`
   execution. Fixed the fixture (`workflow_outcome` → `unverified`) and
   added report-schema **R20**: `workflow_outcome=not_attempted` now
   requires `execution.state` to NOT be `started`/`completed`/`failed`
   (both `unobserved`, no information, and `not_attempted`, positively
   known, remain legal). A new negative-regression fixture
   (`workflow_not_attempted_with_started_execution.json`, the frozen
   before-the-fix content) and a dedicated test keep the bug from
   silently returning; a second test confirms the three genuine dry-run
   report fixtures retain their legitimate `not_attempted`/`not_attempted`
   pairing under R20.

Preserved per the review's closing instruction: no Cargo dependency
*policy* change (the dependency-review allowlist itself is untouched;
`x509-parser` is a normal pass, not an exception), the exact DER
fingerprint domain, manifest bytes, and the "proposed, not yet accepted"
status language throughout. The round-5 Windows/macOS fix, Python
conformance, backup-omission/failure-aggregation fixes, and byte
preservation are explicitly RESOLVED and not reopened this round.

### What changed in round 5

Round 4's fixes were themselves found to have 4 further classes of defect,
plus one small fix:

1. **`derive_postcondition` only ever compared final-vs-expected hash
   equality**, so tampering `verified_removal`'s final `metadata.mode` to
   `"0000"` still derived to `verified`/`removed`. Added a fuller,
   still-bounded, TEST-ONLY `validate_case(plan, evidence)` reference
   cross-check (`derive_postcondition` stays as-is for its own existing
   tests) that additionally checks: final metadata equality against
   `plan.expected_metadata`; backup binding (content hash, metadata,
   observed readability) whenever the plan requires a backup; the
   install step's own status (a final-bytes match with no install
   observation at all, or a `failed` install step, is never
   `verified`/`removed`); conflicting same-step observations, rejected
   even when their `step_snapshot_id`s differ; and, for a no-op plan, an
   INDEPENDENT re-parse of the before artifact's real DER fingerprints
   (not the plan's own claim) to catch a no-op plan whose actual bytes
   still contain an approved-removal target. Eight new adversarial tests
   (`validate_case_rejects_*`, `validate_case_treats_*`,
   `validate_case_never_reports_*`) each start from a known-good fixture
   pair, tamper exactly one thing, and assert the broken case is never
   granted `verified`/`removed`; a ninth test
   (`validate_case_agrees_with_the_frozen_report_fixtures_on_the_untampered_happy_paths`)
   guards that the fuller validator doesn't become stricter than the
   frozen contract on the untampered happy paths.
2. **Report-schema aggregation was one-way, and a metadata-presence rule
   had a gap.** R11 required a failing step whenever execution/workflow
   were failed, but not the reverse: a `sourced` `failed` install step
   could coexist with an `unobserved`/`unverified` workflow. Added R19
   (one `allOf` entry per step — backup/stage/filter/install) forcing
   `execution.state=failed` AND `cleanup.workflow_outcome=failed`
   whenever that step's own status is in the failing family, with a new
   `reports/invalid/failing_step_without_failed_execution_and_workflow.json`
   fixture. Separately, the evidence schema's backup+completed rule used
   `"not": {"required": ["metadata"]}`, which only fires when the key is
   *present* — so an OMITTED `metadata` key (not merely `null` or `{}`)
   still satisfied it. Changed to a direct `"required": ["metadata"]` at
   the `then` level, with two new invalid fixtures
   (`backup_completed_metadata_omitted.json`,
   `backup_completed_metadata_partial.json`). Also fixed
   `shell_status_present_without_granular_observations.json`, which
   incorrectly used `cleanup.workflow_outcome: "not_attempted"` when a
   whole-shell status WAS observed; corrected to `"unverified"`.
3. **DER/PEM checks still had real gaps.** `looks_like_complete_x509_der`
   accepted `30 06 30 00 30 00 03 00` (empty nested `tbsCertificate`/
   `signatureAlgorithm`/`signatureValue`) because it only checked the
   outer three-child tiling, never recursing in. Deepened it to require
   `tbsCertificate` to have ≥6 non-empty children including an INTEGER
   and ≥4 SEQUENCE-tagged fields, `signatureAlgorithm` to start with a
   non-empty OID, and `signatureValue` to be a canonical DER BIT STRING
   (new `is_valid_der_bit_string_content`, checking the unused-bits
   count). A new `nested_empty_der_attack_is_rejected_by_the_recursive_structural_check`
   test proves the exact attack bytes are now rejected. Separately,
   round 4's PEM scanner had become OVER-strict: it required zero bytes
   between blocks, rejecting the comments and inter-block whitespace
   issue #1714 itself requires preserving, and had no CRLF-after-`END`
   support. Rewrote `find_pem_blocks_strict` to tolerate arbitrary
   comment/whitespace text before, between, and after blocks (an
   unmatched trailing `BEGIN` is still an error) and to recognize both
   LF and CRLF block terminators. Three new fixtures/tests prove this:
   `commented_whitespace_bundle.pem.fixture` (two real certificates
   separated by comment lines and blank lines, with the comment text
   itself asserted still present byte-for-byte) and
   `crlf_block.pem.fixture` (CRLF-terminated block parses correctly).
4. **Windows CI genuinely failed** (observed: job `120861400481` in run
   `40090290433`, 5/60 tests failing with byte-count mismatches — 754 vs
   741, 3016 vs 2964 — consistent with CRLF line-ending conversion during
   checkout). Added a `.gitattributes` entry marking
   `tests/fixtures/trust_operation_v1/artifacts/*.pem.fixture` as
   `binary` (disabling line-ending conversion on every platform, not
   normalizing bytes before hashing), verified locally with
   `git check-attr -a`.

**Small fix**: the `minischema::preflight` pass accepted boolean schemas
(`true`/`false`) without `eval` having any semantics for them. Per the
review's explicit either/or, chose to reject: `preflight` now panics on
any boolean schema value, with a new
`preflight_rejects_a_boolean_schema_rather_than_silently_mishandling_it`
`#[should_panic]` test.

Preserved unchanged: the independently-passing Python schema validator
(re-run against every round-5 schema/fixture change —
`scripts/validate_trust_operation_schemas.py` reports all green), zero
Cargo dependency or policy changes, the exact DER fingerprint domain
(sha256 of base64-decoded certificate bytes, never raw-PEM-section
bytes), and the "proposed, not yet accepted" status language throughout.

### What changed in round 4

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
