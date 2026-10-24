//! Self-check for the trust-operation v1 contract (issue #1714 checkpoint 1).
//!
//! This test validates only that the frozen manifest, schemas, and docs
//! are internally consistent with each other. It does NOT exercise any
//! detector, gate, or verifier behavior, because none exists yet in the
//! product: see `docs/trust-operation-contract.md` for the full contract
//! and `tests/fixtures/trust_operation_v1/` for the frozen artifacts.

use serde_json::Value;
use std::collections::BTreeSet;
use std::fs;

const MANIFEST: &str = "tests/fixtures/trust_operation_v1/manifest.json";
const PLAN_SCHEMA: &str = "tests/fixtures/trust_operation_v1/schemas/plan.schema.json";
const EVIDENCE_SCHEMA: &str = "tests/fixtures/trust_operation_v1/schemas/evidence.schema.json";
const REPORT_SCHEMA: &str = "tests/fixtures/trust_operation_v1/schemas/report.schema.json";
const CONTRACT_DOC: &str = "docs/trust-operation-contract.md";

fn load_json(path: &str) -> Value {
    let text = fs::read_to_string(path).unwrap_or_else(|e| panic!("reading {path}: {e}"));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("parsing {path} as JSON: {e}"))
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

#[test]
fn manifest_is_valid_json_with_schema_version_one() {
    let manifest = load_json(MANIFEST);
    assert_eq!(manifest["schema_version"], 1);
    assert!(manifest["manifest_version"].is_string());
    assert!(manifest["rows"].is_array());
    assert!(
        !manifest["rows"].as_array().unwrap().is_empty(),
        "manifest must freeze at least one support-boundary row"
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
fn every_manifest_row_declares_required_fields() {
    let manifest = load_json(MANIFEST);
    let rows = manifest["rows"].as_array().unwrap();
    let valid_detection: BTreeSet<&str> = ["identified", "possible", "not_detected"].into();
    let valid_understanding: BTreeSet<&str> = ["supported", "unsupported", "unknown"].into();
    let known_shells: BTreeSet<&str> = manifest["shells"]
        .as_array()
        .expect("manifest.shells must be an array")
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();

    for row in rows {
        let id = row["id"].as_str().expect("row.id missing");
        let detection = row["mutation_detection"]
            .as_str()
            .unwrap_or_else(|| panic!("{id}: missing mutation_detection"));
        assert!(
            valid_detection.contains(detection),
            "{id}: mutation_detection {detection:?} is not one of identified|possible|not_detected"
        );

        let understanding = row["transformation_understanding"]
            .as_str()
            .unwrap_or_else(|| panic!("{id}: missing transformation_understanding"));
        assert!(
            valid_understanding.contains(understanding),
            "{id}: transformation_understanding {understanding:?} is not one of supported|unsupported|unknown"
        );

        assert!(
            row["requires_scoped_human_confirmation"].is_boolean(),
            "{id}: requires_scoped_human_confirmation must be a bool"
        );

        let shell = row["shell"]
            .as_str()
            .unwrap_or_else(|| panic!("{id}: missing shell"));
        // Rows are allowed to exercise an unsupported shell (e.g. fish) on
        // purpose, as long as they say so via unsupported_reason.
        if row["unsupported_reason"].is_null() {
            assert!(
                known_shells.contains(shell),
                "{id}: shell {shell:?} is not declared in manifest.shells and has no unsupported_reason"
            );
        }

        // A row that claims a mutation but falls short of full support
        // (identified+supported, or possible) must name why. A plain
        // not_detected row may be a benign control with no reason at all,
        // or a documented limitation with one — both are legitimate, so
        // not_detected alone never forces unsupported_reason.
        if detection == "possible" {
            assert!(
                row["unsupported_reason"].is_string(),
                "{id}: detection=possible but unsupported_reason is not set"
            );
        }
        if detection == "identified" && understanding != "supported" {
            assert!(
                row["unsupported_reason"].is_string(),
                "{id}: detection=identified understanding={understanding} but unsupported_reason is not set"
            );
        }
    }
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
fn manifest_profiles_referenced_by_rows_are_declared_or_explicitly_unsupported() {
    let manifest = load_json(MANIFEST);
    let declared_profiles: BTreeSet<&str> = manifest["profiles"]
        .as_array()
        .expect("manifest.profiles must be an array")
        .iter()
        .map(|p| p["name"].as_str().unwrap())
        .collect();

    for row in manifest["rows"].as_array().unwrap() {
        let id = row["id"].as_str().unwrap();
        let profile = row["profile"]
            .as_str()
            .unwrap_or_else(|| panic!("{id}: missing profile"));
        if !declared_profiles.contains(profile) {
            assert_eq!(
                row["unsupported_reason"].as_str(),
                Some("unsupported_profile"),
                "{id}: profile {profile:?} is not declared in manifest.profiles and \
                 unsupported_reason is not unsupported_profile"
            );
        }
    }
}

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
        "report.schema.json reason_code enum has drifted from the frozen list in \
         tests/trust_operation_v1_contract.rs and docs/trust-operation-contract.md"
    );
}

#[test]
fn report_schema_is_valid_json_and_pins_schema_version_to_one() {
    let schema = load_json(REPORT_SCHEMA);
    assert_eq!(schema["properties"]["schema_version"]["const"], 1);
    for section in [
        "assessment",
        "authorization",
        "host_permission",
        "execution",
        "postcondition",
        "cleanup",
    ] {
        assert!(
            schema["properties"][section].is_object(),
            "report schema is missing required top-level section: {section}"
        );
    }
}

#[test]
fn authorization_confirmed_scoped_requires_caro_interactive_provenance_in_schema_docs() {
    // This is a documentation/consistency check, not a behavioral one: it
    // asserts the schema's own description still states the non-relaxable
    // rule, so a future edit cannot silently drop it without failing CI.
    let schema = load_json(REPORT_SCHEMA);
    let state_desc = schema["definitions"]["authorization"]["properties"]["state"]["description"]
        .as_str()
        .unwrap_or("");
    assert!(
        state_desc.contains("confirmed_scoped"),
        "authorization.state description must still document the confirmed_scoped provenance rule"
    );
}

#[test]
fn plan_and_evidence_schemas_are_valid_json_pinned_to_schema_version_one() {
    let plan = load_json(PLAN_SCHEMA);
    assert_eq!(plan["properties"]["schema_version"]["const"], 1);
    assert!(plan["properties"]["target_fingerprints"].is_object());
    assert!(plan["properties"]["backup_policy"].is_object());

    let evidence = load_json(EVIDENCE_SCHEMA);
    assert_eq!(evidence["properties"]["schema_version"]["const"], 1);
    assert!(evidence["definitions"]["observation"]["properties"]["source"].is_object());
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
        assert!(
            doc.contains(test_id),
            "docs/trust-operation-contract.md is missing frozen acceptance-test identifier: {test_id}"
        );
    }
}

#[test]
fn manifest_has_both_identified_and_not_detected_and_possible_rows() {
    // Guards against a manifest that drifts to cover only the easy case.
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
