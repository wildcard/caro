//! Self-check for the trust-operation v1 contract (issue #1714 checkpoint 1,
//! revision 2 per the independent review on PR #1715).
//!
//! This test validates only that the frozen manifest, schemas, and byte-level
//! fixtures are internally consistent with each other, using a REAL JSON
//! Schema validator (`jsonschema`, dev-dependency only, `default-features =
//! false` so it pulls in no network/TLS machinery). It does NOT exercise any
//! detector, gate, or verifier behavior, because none exists yet in the
//! product: see `docs/trust-operation-contract.md` for the full contract and
//! `tests/fixtures/trust_operation_v1/` for the frozen artifacts.
//!
//! Every fixture under `.../plans`, `.../evidence`, and `.../reports` is
//! split into `valid/` (must pass schema validation) and `invalid/` (must
//! fail schema validation, or fail the dedicated Rust-level consistency
//! check named in its test when the defect is a cross-field or cross-file
//! rule JSON Schema draft-07 cannot express alone).

use jsonschema::Validator;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

const FX: &str = "tests/fixtures/trust_operation_v1";
const MANIFEST: &str = "tests/fixtures/trust_operation_v1/manifest.json";
const MANIFEST_SCHEMA: &str = "tests/fixtures/trust_operation_v1/schemas/manifest.schema.json";
const PLAN_SCHEMA: &str = "tests/fixtures/trust_operation_v1/schemas/plan.schema.json";
const EVIDENCE_SCHEMA: &str = "tests/fixtures/trust_operation_v1/schemas/evidence.schema.json";
const REPORT_SCHEMA: &str = "tests/fixtures/trust_operation_v1/schemas/report.schema.json";
const CONTRACT_DOC: &str = "docs/trust-operation-contract.md";

fn load_json(path: &str) -> Value {
    let text = fs::read_to_string(path).unwrap_or_else(|e| panic!("reading {path}: {e}"));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("parsing {path} as JSON: {e}"))
}

fn compiled(schema_path: &str) -> Validator {
    let schema = load_json(schema_path);
    jsonschema::validator_for(&schema)
        .unwrap_or_else(|e| panic!("{schema_path} is not a valid JSON Schema: {e}"))
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

/// Minimal lowercase-hex encoder so this test needs no extra `hex` crate.
mod hex {
    pub fn encode(bytes: impl AsRef<[u8]>) -> String {
        bytes.as_ref().iter().map(|b| format!("{b:02x}")).collect()
    }
}

fn list_json_files(dir: &str) -> Vec<String> {
    let mut out: Vec<String> = fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("reading dir {dir}: {e}"))
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("json"))
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    out.sort();
    out
}

fn frozen_reason_codes() -> BTreeSet<&'static str> {
    [
        "invalid_evidence",
        "missing_observation",
        "stale_observation",
        "inconsistent_evidence",
        "bundle_parse_failed",
        "backup_mismatch",
        "staging_mismatch",
        "final_content_mismatch",
        "metadata_mismatch",
        "unsupported_syntax",
        "unsupported_profile",
        "unsupported_transform",
        "missing_context",
        "authorization_required",
        "authorization_denied",
        "permission_unobserved",
        "execution_unobserved",
        "install_failed",
        "cleanup_failed",
        "already_satisfied",
    ]
    .into_iter()
    .collect()
}

fn frozen_acceptance_test_identifiers() -> BTreeSet<&'static str> {
    [
        "trust_mutation_requires_scoped_human_confirmation_across_modes",
        "guard_shadow_never_claims_authorization",
        "supplied_approval_never_authorizes_runtime",
        "shell_zero_without_observations_stays_unverified",
        "failed_install_survives_successful_cleanup",
        "verified_final_state_preserves_execution_failure",
        "malformed_observed_bundle_fails_postcondition",
        "invalid_evidence_stays_unverified",
        "cleanup_failure_preserves_verified_state",
        "absent_target_is_already_satisfied_without_write",
        "stale_bindings_never_verify",
        "legacy_executed_and_dry_run_semantics_unchanged",
        "default_minimal_and_judge_prompts_preserve_contract",
    ]
    .into_iter()
    .collect()
}

// ---------------------------------------------------------------------------
// Schema self-validity
// ---------------------------------------------------------------------------

#[test]
fn all_four_schemas_are_themselves_valid_json_schema_draft7() {
    for path in [MANIFEST_SCHEMA, PLAN_SCHEMA, EVIDENCE_SCHEMA, REPORT_SCHEMA] {
        let _ = compiled(path);
    }
}

// ---------------------------------------------------------------------------
// Manifest: metaschema validation, plus real negative-instance tests
// ---------------------------------------------------------------------------

#[test]
fn manifest_validates_against_its_metaschema() {
    let validator = compiled(MANIFEST_SCHEMA);
    let manifest = load_json(MANIFEST);
    let errors: Vec<String> = validator
        .iter_errors(&manifest)
        .map(|e| e.to_string())
        .collect();
    assert!(
        errors.is_empty(),
        "manifest.json failed metaschema validation: {errors:#?}"
    );
}

#[test]
fn manifest_rejects_a_flipped_confirmation_flag_on_an_identified_row() {
    let validator = compiled(MANIFEST_SCHEMA);
    let mut manifest = load_json(MANIFEST);
    let rows = manifest["rows"].as_array_mut().unwrap();
    let row = rows
        .iter_mut()
        .find(|r| r["id"] == "T002")
        .expect("T002 must exist");
    row["requires_scoped_human_confirmation"] = Value::Bool(false);
    assert!(
        !validator.is_valid(&manifest),
        "flipping requires_scoped_human_confirmation to false on an identified mutation (T002) \
         must be rejected by the metaschema, per the non-relaxable confirmation rule"
    );
}

#[test]
fn manifest_rejects_a_supported_row_with_plan_ref_deleted() {
    let validator = compiled(MANIFEST_SCHEMA);
    let mut manifest = load_json(MANIFEST);
    let rows = manifest["rows"].as_array_mut().unwrap();
    let row = rows
        .iter_mut()
        .find(|r| r["id"] == "T002")
        .expect("T002 must exist");
    row.as_object_mut().unwrap().remove("plan_ref");
    assert!(
        !validator.is_valid(&manifest),
        "deleting plan_ref from a transformation_understanding=supported row (T002) must be \
         rejected: supported requires a bound plan, not just command-text recognition"
    );
}

#[test]
fn manifest_rejects_supported_without_plan_ref_even_when_plan_ref_is_merely_null() {
    let validator = compiled(MANIFEST_SCHEMA);
    let mut manifest = load_json(MANIFEST);
    let rows = manifest["rows"].as_array_mut().unwrap();
    let row = rows
        .iter_mut()
        .find(|r| r["id"] == "T002")
        .expect("T002 must exist");
    row["plan_ref"] = Value::Null;
    assert!(
        !validator.is_valid(&manifest),
        "a supported row with plan_ref explicitly null must still be rejected"
    );
}

#[test]
fn manifest_row_ids_are_unique() {
    let manifest = load_json(MANIFEST);
    let rows = manifest["rows"].as_array().unwrap();
    let mut seen = BTreeSet::new();
    for row in rows {
        let id = row["id"].as_str().expect("row.id must be a string");
        assert!(
            seen.insert(id.to_string()),
            "duplicate manifest row id: {id}"
        );
    }
}

#[test]
fn manifest_has_both_identified_possible_and_not_detected_rows() {
    let manifest = load_json(MANIFEST);
    let mut seen = BTreeSet::new();
    for row in manifest["rows"].as_array().unwrap() {
        seen.insert(row["mutation_detection"].as_str().unwrap().to_string());
    }
    for expected in ["identified", "possible", "not_detected"] {
        assert!(
            seen.contains(expected),
            "manifest.rows never exercises mutation_detection={expected}"
        );
    }
}

#[test]
fn manifest_has_both_supported_and_unknown_and_unsupported_transformation_rows() {
    let manifest = load_json(MANIFEST);
    let mut seen = BTreeSet::new();
    for row in manifest["rows"].as_array().unwrap() {
        seen.insert(
            row["transformation_understanding"]
                .as_str()
                .unwrap()
                .to_string(),
        );
    }
    for expected in ["supported", "unknown", "unsupported"] {
        assert!(
            seen.contains(expected),
            "manifest.rows never exercises transformation_understanding={expected}"
        );
    }
}

#[test]
fn append_shaped_rows_are_unsupported_transform_not_supported() {
    // Direct regression guard for the #1715 review's point 5 finding.
    let manifest = load_json(MANIFEST);
    for id in ["T004", "T005"] {
        let row = manifest["rows"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == id)
            .unwrap_or_else(|| panic!("{id} must exist"));
        assert_eq!(
            row["transformation_understanding"], "unsupported",
            "{id} (append) must be unsupported"
        );
        assert_eq!(
            row["unsupported_reason"], "unsupported_transform",
            "{id} (append) must cite unsupported_transform"
        );
        assert!(row["plan_ref"].is_null(), "{id} must not carry a plan_ref");
    }
}

#[test]
fn filename_alone_never_changes_the_verdict_for_an_unbound_cp_form() {
    // Direct regression guard for the #1715 review's point 5 finding: T028
    // previously differed from T002 only by a suggestive filename. T028 and
    // T031 now use neutral/different filenames with the same (lack of)
    // plan_ref and must agree with each other; T002 differs from both ONLY
    // by plan_ref, and must disagree with them on transformation_understanding.
    let manifest = load_json(MANIFEST);
    let rows = manifest["rows"].as_array().unwrap();
    let get = |id: &str| {
        rows.iter()
            .find(|r| r["id"] == id)
            .unwrap_or_else(|| panic!("{id} must exist"))
    };
    let t002 = get("T002");
    let t028 = get("T028");
    let t031 = get("T031");

    assert_eq!(t028["mutation_detection"], t031["mutation_detection"]);
    assert_eq!(
        t028["transformation_understanding"],
        t031["transformation_understanding"]
    );
    assert!(t028["plan_ref"].is_null());
    assert!(t031["plan_ref"].is_null());

    assert_eq!(
        t002["mutation_detection"], t028["mutation_detection"],
        "detection should not depend on plan_ref"
    );
    assert_ne!(
        t002["transformation_understanding"], t028["transformation_understanding"],
        "T002 (plan-bound) must report a different transformation_understanding than the \
         otherwise-identical unbound T028/T031, and the difference must trace to plan_ref, not filename"
    );
    assert!(t002["plan_ref"].is_string());
}

#[test]
fn over_limit_and_depth_rows_cite_the_actually_configured_bounds() {
    // Direct regression guard for the #1715 review's point 6 finding: the
    // prior revision invented "4096" against the actually-configured
    // 1000/5000/10000 (src/safety/mod.rs) and MAX_DEPTH=3 (src/safety/segments.rs).
    let manifest = load_json(MANIFEST);
    let bounds = &manifest["config_bounds"];
    assert_eq!(bounds["max_command_length_strict"], 1000);
    assert_eq!(bounds["max_command_length_moderate"], 5000);
    assert_eq!(bounds["max_command_length_permissive"], 10000);
    assert_eq!(bounds["max_wrapper_depth"], 3);

    let t030 = manifest["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == "T030")
        .unwrap();
    assert_eq!(
        t030["input_construction"]["bound_reference"],
        "config_bounds.max_command_length_strict + 1"
    );
    assert_eq!(t030["input_construction"]["total_length"], 1001);

    let t032 = manifest["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == "T032")
        .unwrap();
    assert_eq!(
        t032["input_construction"]["bound_reference"],
        "config_bounds.max_wrapper_depth + 1"
    );
    assert_eq!(t032["input_construction"]["nesting_depth"], 4);
}

#[test]
fn source_safety_config_still_matches_the_pinned_config_bounds() {
    // If src/safety/mod.rs or src/safety/segments.rs ever change these
    // constants, this manifest's config_bounds (and T030/T032) go stale
    // silently unless this test catches the drift.
    let mod_rs = fs::read_to_string("src/safety/mod.rs").expect("reading src/safety/mod.rs");
    for needle in [
        "max_command_length: 1000",
        "max_command_length: 5000",
        "max_command_length: 10000",
    ] {
        assert!(
            mod_rs.contains(needle),
            "src/safety/mod.rs no longer contains {needle:?}; update manifest.json config_bounds"
        );
    }
    let segments_rs =
        fs::read_to_string("src/safety/segments.rs").expect("reading src/safety/segments.rs");
    assert!(
        segments_rs.contains("pub const MAX_DEPTH: usize = 3;"),
        "src/safety/segments.rs::MAX_DEPTH is no longer 3; update manifest.json config_bounds"
    );
}

#[test]
fn manifest_unsupported_reasons_are_within_the_frozen_enum() {
    let manifest = load_json(MANIFEST);
    let codes = frozen_reason_codes();
    for row in manifest["rows"].as_array().unwrap() {
        if let Some(reason) = row["unsupported_reason"].as_str() {
            let id = row["id"].as_str().unwrap();
            assert!(
                codes.contains(reason),
                "{id}: unsupported_reason {reason:?} is not in the frozen reason-code enum"
            );
        }
    }
}

#[test]
fn every_supported_manifest_row_plan_ref_resolves_and_validates() {
    // Cross-file check the metaschema documents as its own limitation:
    // plan_ref must point at a real, schema-valid plan fixture.
    let manifest = load_json(MANIFEST);
    let plan_validator = compiled(PLAN_SCHEMA);
    for row in manifest["rows"].as_array().unwrap() {
        if row["transformation_understanding"] == "supported" {
            let id = row["id"].as_str().unwrap();
            let plan_ref = row["plan_ref"]
                .as_str()
                .unwrap_or_else(|| panic!("{id}: plan_ref must be a string"));
            assert!(
                Path::new(plan_ref).is_file(),
                "{id}: plan_ref {plan_ref:?} does not exist"
            );
            let plan = load_json(plan_ref);
            let errors: Vec<String> = plan_validator
                .iter_errors(&plan)
                .map(|e| e.to_string())
                .collect();
            assert!(
                errors.is_empty(),
                "{id}: plan_ref {plan_ref:?} fails plan schema validation: {errors:#?}"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Plan / evidence / report fixtures: real schema validation, valid + invalid
// ---------------------------------------------------------------------------

fn assert_all_valid(validator: &Validator, dir: &str) {
    for path in list_json_files(dir) {
        let instance = load_json(&path);
        let errors: Vec<String> = validator
            .iter_errors(&instance)
            .map(|e| e.to_string())
            .collect();
        assert!(
            errors.is_empty(),
            "{path} was expected to validate but did not: {errors:#?}"
        );
    }
}

fn assert_all_invalid(validator: &Validator, dir: &str, skip: &[&str]) {
    for path in list_json_files(dir) {
        if skip.iter().any(|s| path.ends_with(s)) {
            continue;
        }
        let instance = load_json(&path);
        assert!(
            !validator.is_valid(&instance),
            "{path} was expected to FAIL validation but passed"
        );
    }
}

#[test]
fn plan_fixtures_valid_directory_all_validate() {
    assert_all_valid(&compiled(PLAN_SCHEMA), &format!("{FX}/plans/valid"));
}

#[test]
fn plan_fixtures_invalid_directory_all_fail_schema_or_are_documented_schema_level_limitations() {
    // count_byte_ranges_mismatch.json is schema-valid by design (JSON Schema
    // draft-07 cannot compare array length to a sibling integer without a
    // custom keyword; plan.schema.json documents this). It is caught instead
    // by approved_removal_ranges_count_matches_byte_ranges_length below.
    assert_all_invalid(
        &compiled(PLAN_SCHEMA),
        &format!("{FX}/plans/invalid"),
        &["count_byte_ranges_mismatch.json"],
    );
}

#[test]
fn approved_removal_ranges_count_matches_byte_ranges_length() {
    let bad = load_json(&format!(
        "{FX}/plans/invalid/count_byte_ranges_mismatch.json"
    ));
    let ranges = bad["approved_removal_ranges"].as_array().unwrap();
    let mismatched = ranges.iter().any(|r| {
        r["count"].as_u64().unwrap() as usize != r["byte_ranges"].as_array().unwrap().len()
    });
    assert!(
        mismatched,
        "count_byte_ranges_mismatch.json fixture must actually contain a mismatch"
    );

    for path in list_json_files(&format!("{FX}/plans/valid")) {
        let plan = load_json(&path);
        for range in plan["approved_removal_ranges"].as_array().unwrap() {
            let count = range["count"].as_u64().unwrap() as usize;
            let len = range["byte_ranges"].as_array().unwrap().len();
            assert_eq!(
                count, len,
                "{path}: count ({count}) must equal byte_ranges.len() ({len})"
            );
        }
    }
}

#[test]
fn evidence_fixtures_valid_directory_all_validate() {
    assert_all_valid(&compiled(EVIDENCE_SCHEMA), &format!("{FX}/evidence/valid"));
}

#[test]
fn evidence_fixtures_invalid_directory_all_fail_schema() {
    assert_all_invalid(
        &compiled(EVIDENCE_SCHEMA),
        &format!("{FX}/evidence/invalid"),
        &[],
    );
}

#[test]
fn report_fixtures_valid_directory_all_validate() {
    assert_all_valid(&compiled(REPORT_SCHEMA), &format!("{FX}/reports/valid"));
}

#[test]
fn report_fixtures_invalid_directory_all_fail_schema() {
    assert_all_invalid(
        &compiled(REPORT_SCHEMA),
        &format!("{FX}/reports/invalid"),
        &[],
    );
}

/// Every forbidden combination named in the #1715 review must have a
/// corresponding invalid/ fixture. This guards against someone quietly
/// deleting the fixture that proves a rule works.
#[test]
fn every_forbidden_report_combination_from_the_review_has_a_fixture() {
    let required = [
        "confirmed_scoped_missing_digest.json",
        "confirmed_scoped_supplied_claim.json",
        "host_permission_granted_none_provenance.json",
        "postcondition_verified_basis_none.json",
        "removed_effect_without_completed_execution.json",
        "cleanup_failed_workflow_completed.json",
        "execution_completed_without_install_step.json",
        "already_satisfied_with_execution_started.json",
        "scope_digest_present_without_confirmed.json",
        "shell_exit_zero_alone_claims_completed.json",
    ];
    let present = list_json_files(&format!("{FX}/reports/invalid"));
    for name in required {
        assert!(
            present.iter().any(|p| p.ends_with(name)),
            "missing required negative report fixture: {name}"
        );
    }
}

#[test]
fn every_required_outcome_scenario_from_the_review_has_a_valid_fixture() {
    // "Cover no-op, failed install with independently verified state,
    // dry-run supplied-state report, missing digest/provenance, and
    // no-execution removal." (point 1 of the #1715 review)
    let required = [
        "already_satisfied_no_op.json",               // no-op
        "failed_install_independently_verified.json", // failed install, independently verified state
        "dry_run_supplied_state_verified.json",       // dry-run supplied-state report
        "dry_run_unverified.json",                    // the default dry-run (no claim)
        "cleanup_failure_preserves_verified_state.json",
        "verified_removal.json",
    ];
    let present = list_json_files(&format!("{FX}/reports/valid"));
    for name in required {
        assert!(
            present.iter().any(|p| p.ends_with(name)),
            "missing required positive report fixture: {name}"
        );
    }
    // "missing digest/provenance" is exercised as a NEGATIVE case (reports/invalid),
    // because a report with confirmed_scoped and a missing digest/provenance is,
    // by this contract, invalid, not a legitimate reportable state.
    let invalid_present = list_json_files(&format!("{FX}/reports/invalid"));
    assert!(invalid_present
        .iter()
        .any(|p| p.contains("missing_digest") || p.contains("confirmed_scoped_missing_digest")));
}

// ---------------------------------------------------------------------------
// Byte-level grounding: every claimed hash must resolve to real, re-hashable
// bytes. This is the direct fix for the #1715 review's point 2 finding
// ("claimed hashes ... but no raw bytes").
// ---------------------------------------------------------------------------

fn read_artifact(fixture_path: &str) -> Vec<u8> {
    fs::read(fixture_path).unwrap_or_else(|e| panic!("reading artifact {fixture_path}: {e}"))
}

fn check_artifact_ref(ar: &Value, context: &str) {
    if ar.is_null() {
        return;
    }
    let declared_sha = ar["sha256"]
        .as_str()
        .unwrap_or_else(|| panic!("{context}: artifact_ref.sha256 missing"));
    let declared_size = ar["size_bytes"]
        .as_u64()
        .unwrap_or_else(|| panic!("{context}: artifact_ref.size_bytes missing"));
    assert_eq!(
        ar["source"], "fixture_file",
        "{context}: only fixture_file artifact refs are used in these fixtures"
    );
    let path = ar["fixture_path"]
        .as_str()
        .unwrap_or_else(|| panic!("{context}: fixture_path missing"));
    assert!(
        path.starts_with("tests/fixtures/trust_operation_v1/artifacts/"),
        "{context}: fixture_path escapes the frozen artifacts dir: {path}"
    );
    let bytes = read_artifact(path);
    assert_eq!(
        bytes.len() as u64,
        declared_size,
        "{context}: {path} size does not match artifact_ref.size_bytes"
    );
    assert_eq!(
        sha256_hex(&bytes),
        declared_sha,
        "{context}: {path} content does not match artifact_ref.sha256"
    );
}

#[test]
fn every_plan_artifact_ref_resolves_to_real_bytes_matching_its_declared_hash() {
    for path in list_json_files(&format!("{FX}/plans/valid")) {
        let plan = load_json(&path);
        if !plan["before_artifact_ref"].is_null() {
            check_artifact_ref(
                &plan["before_artifact_ref"],
                &format!("{path} before_artifact_ref"),
            );
            let bytes = read_artifact(
                plan["before_artifact_ref"]["fixture_path"]
                    .as_str()
                    .unwrap(),
            );
            assert_eq!(
                sha256_hex(&bytes),
                plan["before_hash"].as_str().unwrap(),
                "{path}: before_hash must equal sha256 of before_artifact_ref bytes"
            );
        }
        if !plan["expected_staged_artifact_ref"].is_null() {
            check_artifact_ref(
                &plan["expected_staged_artifact_ref"],
                &format!("{path} expected_staged_artifact_ref"),
            );
            let bytes = read_artifact(
                plan["expected_staged_artifact_ref"]["fixture_path"]
                    .as_str()
                    .unwrap(),
            );
            assert_eq!(sha256_hex(&bytes), plan["expected_staged_hash"].as_str().unwrap(), "{path}: expected_staged_hash must equal sha256 of expected_staged_artifact_ref bytes");
        }
    }
}

#[test]
fn every_evidence_content_hash_resolves_to_real_bytes_via_its_artifact_ref() {
    for path in list_json_files(&format!("{FX}/evidence/valid")) {
        let evidence = load_json(&path);
        for obs in evidence["observations"].as_array().unwrap() {
            if let Some(hash) = obs["content_hash"].as_str() {
                check_artifact_ref(
                    &obs["artifact_ref"],
                    &format!("{path} observation step={}", obs["step"]),
                );
                let bytes = read_artifact(obs["artifact_ref"]["fixture_path"].as_str().unwrap());
                assert_eq!(
                    sha256_hex(&bytes),
                    hash,
                    "{path}: observation content_hash must match sha256 of its artifact_ref bytes"
                );
            }
        }
    }
}

#[test]
fn approved_removal_byte_ranges_actually_slice_out_bytes_matching_target_sha256() {
    // Binds the abstract (target_sha256, count) pair to real byte offsets
    // within the real before-mutation artifact, per the review's point 2
    // ("bind actual approved byte ranges and occurrence identity to before bytes").
    for path in list_json_files(&format!("{FX}/plans/valid")) {
        let plan = load_json(&path);
        if plan["before_artifact_ref"].is_null() {
            continue;
        }
        let before_bytes = read_artifact(
            plan["before_artifact_ref"]["fixture_path"]
                .as_str()
                .unwrap(),
        );
        for range in plan["approved_removal_ranges"].as_array().unwrap() {
            let target_sha = range["target_sha256"].as_str().unwrap();
            for br in range["byte_ranges"].as_array().unwrap() {
                let start = br["start"].as_u64().unwrap() as usize;
                let end = br["end"].as_u64().unwrap() as usize;
                assert!(
                    end <= before_bytes.len() && start <= end,
                    "{path}: byte range [{start},{end}) out of bounds"
                );
                let slice_hash = sha256_hex(&before_bytes[start..end]);
                assert_eq!(
                    slice_hash, target_sha,
                    "{path}: before[{start}:{end}] does not hash to the declared target_sha256"
                );
            }
        }
    }
}

#[test]
fn malformed_truncated_artifact_is_actually_structurally_truncated() {
    // Not a parser implementation: just confirms the fixture used to exercise
    // bundle_parse_failed really is missing its END marker, so the scenario
    // it is meant to represent (truncated certificate block) is real.
    let bytes = read_artifact(&format!("{FX}/artifacts/malformed_truncated.pem.fixture"));
    let text = String::from_utf8(bytes).unwrap();
    assert!(text.contains("-----BEGIN CERTIFICATE-----"));
    assert!(
        !text.contains("-----END CERTIFICATE-----"),
        "malformed_truncated.pem must NOT contain an END marker"
    );
}

#[test]
fn before_two_targets_artifact_actually_contains_duplicate_and_distinct_targets() {
    // Grounds the duplicate-target-count scenario in real, re-countable bytes.
    let bytes = read_artifact(&format!("{FX}/artifacts/before_two_targets.pem.fixture"));
    let cert_a_marker = b"U1lOVEhFVElDLUEtMDAxLW5vdC1hLXJlYWwtY2VydGlmaWNhdGUtZm9yLXRlc3Rz";
    let occurrences = bytes
        .windows(cert_a_marker.len())
        .filter(|w| *w == cert_a_marker)
        .count();
    assert_eq!(
        occurrences, 2,
        "before_two_targets.pem must contain exactly two occurrences of the cert_a marker"
    );
}

// ---------------------------------------------------------------------------
// Cross-document binding: plan<->evidence id correspondence, and the
// context_identity definition must stay byte-identical between the two
// schema files (the #1715 review's point 3 "canonical binding rule").
// ---------------------------------------------------------------------------

#[test]
fn plan_and_evidence_context_identity_definitions_are_byte_identical() {
    let plan_schema = load_json(PLAN_SCHEMA);
    let evidence_schema = load_json(EVIDENCE_SCHEMA);
    let plan_def = &plan_schema["definitions"]["context_identity"];
    let evidence_def = &evidence_schema["definitions"]["context_identity"];
    assert!(
        !plan_def.is_null() && !evidence_def.is_null(),
        "both schemas must define context_identity"
    );
    assert_eq!(
        plan_def, evidence_def,
        "plan.schema.json and evidence.schema.json definitions.context_identity have drifted; \
         they are the canonical binding rule between a plan's context_identity and an \
         evidence observation's context_binding and MUST stay identical"
    );
}

#[test]
fn bound_plan_and_evidence_pairs_share_operation_id_and_evidence_id() {
    let pairs = [
        (
            "plans/valid/mutating_removal_with_backup_required.json",
            "evidence/valid/verified_removal.json",
        ),
        (
            "plans/valid/no_op_targets_absent.json",
            "evidence/valid/already_satisfied_no_op.json",
        ),
    ];
    for (plan_path, evidence_path) in pairs {
        let plan = load_json(&format!("{FX}/{plan_path}"));
        let evidence = load_json(&format!("{FX}/{evidence_path}"));
        assert_eq!(
            plan["operation_id"], evidence["operation_id"],
            "{plan_path} <-> {evidence_path}: operation_id mismatch"
        );
        assert_eq!(
            plan["evidence_id"], evidence["evidence_id"],
            "{plan_path} <-> {evidence_path}: evidence_id mismatch"
        );
    }
}

#[test]
fn every_evidence_observation_binds_to_its_paired_plans_destination_and_context() {
    let pairs = [
        (
            "plans/valid/mutating_removal_with_backup_required.json",
            "evidence/valid/verified_removal.json",
        ),
        (
            "plans/valid/no_op_targets_absent.json",
            "evidence/valid/already_satisfied_no_op.json",
        ),
    ];
    for (plan_path, evidence_path) in pairs {
        let plan = load_json(&format!("{FX}/{plan_path}"));
        let evidence = load_json(&format!("{FX}/{evidence_path}"));
        for obs in evidence["observations"].as_array().unwrap() {
            assert_eq!(
                obs["normalized_destination_binding"], plan["normalized_destination"],
                "{evidence_path}: observation step={} is not bound to {plan_path}'s destination",
                obs["step"]
            );
            assert_eq!(
                obs["context_binding"], plan["context_identity"],
                "{evidence_path}: observation step={} context_binding does not structurally match {plan_path}'s context_identity",
                obs["step"]
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Reason codes / frozen enum drift checks (carried over from revision 1)
// ---------------------------------------------------------------------------

#[test]
fn report_schema_reason_code_enum_matches_frozen_list_exactly() {
    let schema = load_json(REPORT_SCHEMA);
    let enum_values: BTreeSet<String> = schema["definitions"]["reason_code"]["enum"]
        .as_array()
        .expect("report schema must define definitions.reason_code.enum")
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    let frozen: BTreeSet<String> = frozen_reason_codes()
        .into_iter()
        .map(String::from)
        .collect();
    assert_eq!(
        enum_values, frozen,
        "report.schema.json reason_code enum has drifted from the frozen list"
    );
}

#[test]
fn contract_doc_names_every_frozen_reason_code_and_test_identifier() {
    let doc =
        fs::read_to_string(CONTRACT_DOC).unwrap_or_else(|e| panic!("reading {CONTRACT_DOC}: {e}"));
    for code in frozen_reason_codes() {
        assert!(
            doc.contains(code),
            "docs/trust-operation-contract.md is missing frozen reason code: {code}"
        );
    }
    for test_id in frozen_acceptance_test_identifiers() {
        assert!(doc.contains(test_id), "docs/trust-operation-contract.md is missing frozen acceptance-test identifier: {test_id}");
    }
}

#[test]
fn authorization_confirmed_scoped_still_restricted_to_caro_interactive_in_schema() {
    let schema = load_json(REPORT_SCHEMA);
    let state_desc = schema["definitions"]["authorization"]["properties"]["state"]["description"]
        .as_str()
        .unwrap_or("");
    assert!(state_desc.contains("confirmed_scoped"));
    // The real enforcement is the allOf R6/R7 rules, checked directly above
    // via the invalid report fixtures; this only guards the description text.
    let all_of = schema["allOf"]
        .as_array()
        .expect("report schema must have allOf rules");
    assert!(
        all_of.len() >= 13,
        "expected at least 13 allOf conditionals (R1..R13) in report.schema.json, found {}",
        all_of.len()
    );
}
