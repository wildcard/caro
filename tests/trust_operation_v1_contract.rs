//! Self-check for the trust-operation v1 contract (issue #1714 checkpoint 1,
//! revision 2, round 3 fixes per the independent review on PR #1715).
//!
//! This test validates only that the frozen manifest, schemas, and byte-level
//! fixtures are internally consistent with each other. It does NOT exercise
//! any detector, gate, or verifier behavior, because none exists yet in the
//! product: see `docs/trust-operation-contract.md` for the full contract and
//! `tests/fixtures/trust_operation_v1/` for the frozen artifacts.
//!
//! ## Why there is no external JSON Schema crate here
//!
//! Round 2 used the `jsonschema` crate. Independent review found that it
//! fails this repository's `dependency-review.yml` license allowlist: every
//! `$ref`-capable JSON Schema crate checked (`jsonschema` 0.60.x via
//! `referencing`/`fluent-uri`/`borrow-or-share`, MIT-0; `jsonschema` 0.18.x
//! via `url`/`idna`/the icu4x stack, Unicode-3.0) pulls in a license string
//! outside the allowlist. Rather than editing `dependency-review.yml` to
//! weaken that policy (explicitly forbidden by the review), this file
//! implements a minimal, self-contained draft-07-SUBSET evaluator (module
//! `minischema` below) using only `serde_json` and `regex`, both of which
//! are already existing dependencies of this crate (zero new dependencies,
//! zero new license exposure). It supports exactly the keywords the four
//! schemas in this directory use: `type`, `const`, `enum`, `required`,
//! `properties`, `additionalProperties`, `pattern`, `minLength`, `minimum`,
//! `exclusiveMinimum`, `items`, `minItems`, `contains`, `allOf`, `anyOf`,
//! `not`, `if`/`then`, and local (`#/definitions/...`) `$ref`. It is NOT a
//! general-purpose, spec-compliant JSON Schema implementation: it has no
//! `oneOf`, no `patternProperties`, no remote `$ref` resolution, and no
//! meta-schema self-validation. That boundary is deliberate and documented
//! here rather than silently assumed.

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

// ===========================================================================
// minischema: a minimal draft-07-subset evaluator (see module doc above).
// ===========================================================================
mod minischema {
    use regex::Regex;
    use serde_json::Value;

    pub struct Validator {
        schema: Value,
    }

    impl Validator {
        pub fn new(schema: &Value) -> Validator {
            Validator {
                schema: schema.clone(),
            }
        }

        pub fn is_valid(&self, instance: &Value) -> bool {
            self.iter_errors(instance).is_empty()
        }

        pub fn iter_errors(&self, instance: &Value) -> Vec<String> {
            let mut errors = Vec::new();
            eval(&self.schema, instance, &self.schema, "$", &mut errors);
            errors
        }
    }

    fn resolve_ref<'a>(root: &'a Value, ptr: &str) -> &'a Value {
        let ptr = ptr.trim_start_matches('#').trim_start_matches('/');
        let mut cur = root;
        if ptr.is_empty() {
            return cur;
        }
        for part in ptr.split('/') {
            cur = cur
                .get(part)
                .unwrap_or_else(|| panic!("broken $ref segment {part:?} while resolving {ptr:?}"));
        }
        cur
    }

    fn type_matches(ty: &str, v: &Value) -> bool {
        match ty {
            "object" => v.is_object(),
            "array" => v.is_array(),
            "string" => v.is_string(),
            "integer" => v.is_i64() || v.is_u64() || v.as_f64().is_some_and(|f| f.fract() == 0.0),
            "number" => v.is_number(),
            "boolean" => v.is_boolean(),
            "null" => v.is_null(),
            _ => false,
        }
    }

    /// Evaluate `schema` against `instance`. `root` is the whole document
    /// (for `$ref` resolution). Errors are pushed, not returned, so allOf
    /// branches can each contribute independently.
    pub fn eval(
        schema: &Value,
        instance: &Value,
        root: &Value,
        path: &str,
        errors: &mut Vec<String>,
    ) {
        if let Some(r) = schema.get("$ref").and_then(|v| v.as_str()) {
            let resolved = resolve_ref(root, r);
            eval(resolved, instance, root, path, errors);
            return;
        }

        if let Some(t) = schema.get("type") {
            let ok = match t {
                Value::String(s) => type_matches(s, instance),
                Value::Array(arr) => arr
                    .iter()
                    .any(|x| type_matches(x.as_str().unwrap_or(""), instance)),
                _ => true,
            };
            if !ok {
                errors.push(format!(
                    "{path}: type mismatch (schema wants {t}, instance is {instance})"
                ));
            }
        }

        if let Some(c) = schema.get("const") {
            if instance != c {
                errors.push(format!(
                    "{path}: const mismatch (expected {c}, got {instance})"
                ));
            }
        }

        if let Some(e) = schema.get("enum").and_then(|v| v.as_array()) {
            if !e.contains(instance) {
                errors.push(format!("{path}: enum mismatch ({instance} not in {e:?})"));
            }
        }

        if let Some(pat) = schema.get("pattern").and_then(|v| v.as_str()) {
            if let Some(s) = instance.as_str() {
                let re = Regex::new(pat).unwrap_or_else(|e| panic!("bad pattern {pat:?}: {e}"));
                if !re.is_match(s) {
                    errors.push(format!("{path}: pattern {pat:?} did not match {s:?}"));
                }
            }
        }

        if let Some(min_len) = schema.get("minLength").and_then(|v| v.as_u64()) {
            if let Some(s) = instance.as_str() {
                if (s.chars().count() as u64) < min_len {
                    errors.push(format!("{path}: minLength {min_len} violated by {s:?}"));
                }
            }
        }

        if let Some(min) = schema.get("minimum").and_then(|v| v.as_f64()) {
            if let Some(n) = instance.as_f64() {
                if n < min {
                    errors.push(format!("{path}: minimum {min} violated by {n}"));
                }
            }
        }

        if let Some(exmin) = schema.get("exclusiveMinimum").and_then(|v| v.as_f64()) {
            if let Some(n) = instance.as_f64() {
                if n <= exmin {
                    errors.push(format!("{path}: exclusiveMinimum {exmin} violated by {n}"));
                }
            }
        }

        if instance.is_object() {
            if let Some(req) = schema.get("required").and_then(|v| v.as_array()) {
                for r in req {
                    let key = r.as_str().unwrap();
                    if instance.get(key).is_none() {
                        errors.push(format!("{path}: missing required property {key:?}"));
                    }
                }
            }
            if let Some(props) = schema.get("properties").and_then(|v| v.as_object()) {
                for (k, subschema) in props {
                    if let Some(v) = instance.get(k) {
                        eval(subschema, v, root, &format!("{path}.{k}"), errors);
                    }
                }
            }
            if schema.get("additionalProperties") == Some(&Value::Bool(false)) {
                let allowed: std::collections::BTreeSet<&str> = schema
                    .get("properties")
                    .and_then(|v| v.as_object())
                    .map(|o| o.keys().map(|s| s.as_str()).collect())
                    .unwrap_or_default();
                if let Some(obj) = instance.as_object() {
                    for k in obj.keys() {
                        if !allowed.contains(k.as_str()) {
                            errors.push(format!("{path}: additional property {k:?} not allowed"));
                        }
                    }
                }
            }
        }

        if let Some(arr) = instance.as_array() {
            if let Some(items_schema) = schema.get("items") {
                for (i, item) in arr.iter().enumerate() {
                    eval(items_schema, item, root, &format!("{path}[{i}]"), errors);
                }
            }
            if let Some(min_items) = schema.get("minItems").and_then(|v| v.as_u64()) {
                if (arr.len() as u64) < min_items {
                    errors.push(format!(
                        "{path}: minItems {min_items} violated (len={})",
                        arr.len()
                    ));
                }
            }
            if let Some(contains_schema) = schema.get("contains") {
                let any_ok = arr.iter().any(|item| {
                    let mut e = Vec::new();
                    eval(contains_schema, item, root, path, &mut e);
                    e.is_empty()
                });
                if !any_ok {
                    errors.push(format!("{path}: contains violated, no item matches"));
                }
            }
        }

        if let Some(all_of) = schema.get("allOf").and_then(|v| v.as_array()) {
            for (i, sub) in all_of.iter().enumerate() {
                eval(sub, instance, root, &format!("{path}(allOf[{i}])"), errors);
            }
        }

        if let Some(any_of) = schema.get("anyOf").and_then(|v| v.as_array()) {
            let any_ok = any_of.iter().any(|sub| {
                let mut e = Vec::new();
                eval(sub, instance, root, path, &mut e);
                e.is_empty()
            });
            if !any_ok {
                errors.push(format!("{path}: anyOf violated, no branch matches"));
            }
        }

        if let Some(not_schema) = schema.get("not") {
            let mut e = Vec::new();
            eval(not_schema, instance, root, path, &mut e);
            if e.is_empty() {
                errors.push(format!(
                    "{path}: not violated (inner schema matched when it should not)"
                ));
            }
        }

        if let Some(if_schema) = schema.get("if") {
            let mut e = Vec::new();
            eval(if_schema, instance, root, path, &mut e);
            if e.is_empty() {
                if let Some(then_schema) = schema.get("then") {
                    eval(then_schema, instance, root, path, errors);
                }
            } else if let Some(else_schema) = schema.get("else") {
                eval(else_schema, instance, root, path, errors);
            }
        }
    }
}

fn load_json(path: &str) -> Value {
    let text = fs::read_to_string(path).unwrap_or_else(|e| panic!("reading {path}: {e}"));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("parsing {path} as JSON: {e}"))
}

fn compiled(schema_path: &str) -> minischema::Validator {
    minischema::Validator::new(&load_json(schema_path))
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex_encode(hasher.finalize())
}

fn hex_encode(bytes: impl AsRef<[u8]>) -> String {
    bytes.as_ref().iter().map(|b| format!("{b:02x}")).collect()
}

/// Minimal RFC 4648 standard-alphabet base64 decoder (no external crate).
/// PEM uses standard base64 with `=` padding; this is all we need to turn a
/// certificate's base64 body back into DER bytes.
fn base64_decode(input: &str) -> Vec<u8> {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut table = [255u8; 256];
    for (i, &c) in ALPHABET.iter().enumerate() {
        table[c as usize] = i as u8;
    }
    let clean: Vec<u8> = input
        .bytes()
        .filter(|b| !b.is_ascii_whitespace() && *b != b'=')
        .collect();
    let mut out = Vec::with_capacity(clean.len() * 3 / 4 + 3);
    let mut buf: u32 = 0;
    let mut bits = 0u32;
    for b in clean {
        let v = table[b as usize];
        assert_ne!(v, 255, "invalid base64 byte {b:?}");
        buf = (buf << 6) | v as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push(((buf >> bits) & 0xFF) as u8);
        }
    }
    out
}

/// A parsed `-----BEGIN CERTIFICATE----- ... -----END CERTIFICATE-----` block.
struct PemBlock {
    /// Byte range in the containing file, [start, end), including the block's
    /// own trailing newline (if any) so adjacent blocks are exactly
    /// concatenable / removable without leaving stray blank lines.
    range: (usize, usize),
    der: Vec<u8>,
}

/// Independently parse every PEM block in `data`. Does not trust anything
/// declared in a plan/evidence/manifest fixture: this re-derives the block
/// boundaries and DER bytes straight from the raw file content.
fn find_pem_blocks(data: &[u8]) -> Vec<PemBlock> {
    let text =
        std::str::from_utf8(data).expect("artifact must be UTF-8 for this contract's fixtures");
    const BEGIN: &str = "-----BEGIN CERTIFICATE-----";
    const END: &str = "-----END CERTIFICATE-----";
    let mut out = Vec::new();
    let mut pos = 0usize;
    while let Some(b_rel) = text[pos..].find(BEGIN) {
        let b = pos + b_rel;
        let Some(e_rel) = text[b..].find(END) else {
            break;
        };
        let mut end = b + e_rel + END.len();
        if text.as_bytes().get(end) == Some(&b'\n') {
            end += 1;
        }
        let body_start = b + BEGIN.len();
        let body_end = b + e_rel;
        let body = &text[body_start..body_end];
        let der = base64_decode(body);
        out.push(PemBlock {
            range: (b, end),
            der,
        });
        pos = end;
    }
    out
}

/// Shallow DER sanity check: a real DER-encoded certificate is a SEQUENCE
/// (tag 0x30) whose length octets describe (approximately) the remaining
/// bytes. This is NOT an ASN.1/X.509 parser -- it does not look inside the
/// SEQUENCE -- but it is enough to tell a real certificate's DER envelope
/// apart from arbitrary non-DER bytes stuffed into a PEM-shaped wrapper
/// (see artifacts/invalid_der.pem.fixture).
fn looks_like_valid_der_sequence(der: &[u8]) -> bool {
    if der.len() < 2 || der[0] != 0x30 {
        return false;
    }
    let len_byte = der[1];
    if len_byte < 0x80 {
        // short form: length is len_byte, content follows immediately.
        2 + len_byte as usize <= der.len()
    } else {
        let num_len_bytes = (len_byte & 0x7F) as usize;
        if num_len_bytes == 0 || num_len_bytes > 4 || der.len() < 2 + num_len_bytes {
            return false;
        }
        let mut length: usize = 0;
        for &b in &der[2..2 + num_len_bytes] {
            length = (length << 8) | b as usize;
        }
        2 + num_len_bytes + length <= der.len()
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

/// The exact versioned preimage/encoding for a scoped-approval digest
/// (#1715 review point 4: "Define VERSIONED scoped-approval digest
/// preimage/encoding over command, destination, intended effect plus
/// mutation vectors"). Any change to this encoding is a new version
/// ("trust_operation_scope_binding_v2\n..."), never a silent reinterpretation
/// of v1 digests.
fn compute_scope_binding_digest_v1(plan: &Value) -> String {
    let command_digest = plan["command_digest"].as_str().unwrap();
    let dest = plan["normalized_destination"].as_str().unwrap();
    let transform = plan["approved_transformation"].as_str().unwrap();
    let mut pairs: Vec<(String, u64)> = plan["approved_removal_ranges"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| {
            (
                r["target_sha256"].as_str().unwrap().to_string(),
                r["count"].as_u64().unwrap(),
            )
        })
        .collect();
    pairs.sort_by(|a, b| a.0.cmp(&b.0));
    let csv = pairs
        .iter()
        .map(|(s, c)| format!("{s}:{c}"))
        .collect::<Vec<_>>()
        .join(",");
    let preimage =
        format!("trust_operation_scope_binding_v1\n{command_digest}\n{dest}\n{transform}\n{csv}");
    sha256_hex(preimage.as_bytes())
}

// ===========================================================================
// Schema self-validity
// ===========================================================================

#[test]
fn all_four_schemas_parse_as_json_and_construct_a_validator_without_panicking() {
    for path in [MANIFEST_SCHEMA, PLAN_SCHEMA, EVIDENCE_SCHEMA, REPORT_SCHEMA] {
        let _ = compiled(path);
    }
}

// ===========================================================================
// Manifest: metaschema validation, negative-instance tests
// ===========================================================================

#[test]
fn manifest_validates_against_its_metaschema() {
    let validator = compiled(MANIFEST_SCHEMA);
    let manifest = load_json(MANIFEST);
    let errors = validator.iter_errors(&manifest);
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
        "flipping confirmation on T002 must be rejected"
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
        "deleting plan_ref from a supported row must be rejected"
    );
}

#[test]
fn manifest_rejects_a_row_with_both_input_and_input_construction() {
    let validator = compiled(MANIFEST_SCHEMA);
    let mut manifest = load_json(MANIFEST);
    let rows = manifest["rows"].as_array_mut().unwrap();
    let t030 = rows.iter().find(|r| r["id"] == "T030").unwrap().clone();
    let row = rows.iter_mut().find(|r| r["id"] == "T002").unwrap();
    row["input_construction"] = t030["input_construction"].clone();
    assert!(
        !validator.is_valid(&manifest),
        "a row with both input and input_construction must be rejected"
    );
}

#[test]
fn manifest_rejects_a_row_with_neither_input_nor_input_construction() {
    let validator = compiled(MANIFEST_SCHEMA);
    let mut manifest = load_json(MANIFEST);
    let rows = manifest["rows"].as_array_mut().unwrap();
    let row = rows.iter_mut().find(|r| r["id"] == "T002").unwrap();
    row.as_object_mut().unwrap().remove("input");
    assert!(
        !validator.is_valid(&manifest),
        "a row with neither input nor input_construction must be rejected"
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
fn manifest_has_33_rows_and_four_schema_files_as_actually_shipped() {
    // Direct regression guard for the final review instruction: "update
    // owner PR body/docs: ACTUAL 33 rows, 4 schemas, N tests, dependency
    // status". This pins the row/schema counts so the PR body claim can be
    // checked mechanically rather than asserted by hand.
    let manifest = load_json(MANIFEST);
    assert_eq!(manifest["rows"].as_array().unwrap().len(), 33);
    for schema in [MANIFEST_SCHEMA, PLAN_SCHEMA, EVIDENCE_SCHEMA, REPORT_SCHEMA] {
        assert!(Path::new(schema).is_file(), "missing schema file: {schema}");
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
fn writer_detection_is_independent_of_transformation_understanding() {
    // Point 5 of the #1715 review: "Writer detection != transform
    // understanding." Prove it with real rows rather than asserting it in
    // prose: there must be at least one row where the writer IS detected
    // (mutation_detection=identified) but the transform is NOT understood
    // (transformation_understanding=unknown), and at least one row where
    // both are true together -- showing identified does not determine the
    // transformation_understanding verdict either way.
    let manifest = load_json(MANIFEST);
    let rows = manifest["rows"].as_array().unwrap();
    let identified_unknown = rows.iter().any(|r| {
        r["mutation_detection"] == "identified" && r["transformation_understanding"] == "unknown"
    });
    let identified_supported = rows.iter().any(|r| {
        r["mutation_detection"] == "identified" && r["transformation_understanding"] == "supported"
    });
    let identified_unsupported = rows.iter().any(|r| {
        r["mutation_detection"] == "identified"
            && r["transformation_understanding"] == "unsupported"
    });
    assert!(
        identified_unknown,
        "no row demonstrates identified+unknown (writer detected, transform not understood)"
    );
    assert!(
        identified_supported,
        "no row demonstrates identified+supported (writer detected AND transform understood)"
    );
    assert!(identified_unsupported, "no row demonstrates identified+unsupported (writer detected, transform categorically out of scope)");
}

#[test]
fn append_shaped_rows_are_unsupported_transform_not_supported() {
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
        t002["transformation_understanding"],
        t028["transformation_understanding"]
    );
    assert!(t002["plan_ref"].is_string());
}

#[test]
fn over_limit_and_depth_rows_cite_the_actually_configured_bounds() {
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
}

#[test]
fn source_safety_config_still_matches_the_pinned_config_bounds() {
    let mod_rs = fs::read_to_string("src/safety/mod.rs").expect("reading src/safety/mod.rs");
    for needle in [
        "max_command_length: 1000",
        "max_command_length: 5000",
        "max_command_length: 10000",
    ] {
        assert!(
            mod_rs.contains(needle),
            "src/safety/mod.rs no longer contains {needle:?}"
        );
    }
    let segments_rs =
        fs::read_to_string("src/safety/segments.rs").expect("reading src/safety/segments.rs");
    assert!(
        segments_rs.contains("pub const MAX_DEPTH: usize = 3;"),
        "MAX_DEPTH is no longer 3"
    );
}

/// Point 5 of the #1715 review: "assert emitted bytes, length, escaping,
/// hash" for the fully-specified constructor rows. This actually RUNS the
/// documented construction algorithm (right-pad for T030; POSIX
/// single-quote escaping + repeated `sh -c '...'` wrapping for T032) and
/// checks the result against the frozen expected_length/expected_sha256 --
/// it does not just trust the manifest's own claim.
#[test]
fn input_construction_rows_actually_construct_the_claimed_bytes() {
    let manifest = load_json(MANIFEST);
    let rows = manifest["rows"].as_array().unwrap();
    let get = |id: &str| rows.iter().find(|r| r["id"] == id).unwrap().clone();

    // T030: right-pad T002's literal command with pad_char to total_length.
    let t030 = get("T030");
    let base = get(t030["input_construction"]["base_row_id"].as_str().unwrap());
    let base_input = base["input"].as_str().unwrap();
    let pad_char = t030["input_construction"]["pad_char"].as_str().unwrap();
    let total_length = t030["input_construction"]["total_length"].as_u64().unwrap() as usize;
    let mut constructed = base_input.to_string();
    constructed.push_str(&pad_char.repeat(total_length - base_input.chars().count()));
    assert_eq!(
        constructed.len(),
        t030["input_construction"]["expected_length"]
            .as_u64()
            .unwrap() as usize
    );
    assert_eq!(
        sha256_hex(constructed.as_bytes()),
        t030["input_construction"]["expected_sha256"]
    );

    // T032: wrap T011's command in `sh -c '<escaped>'` additional_wraps_applied times.
    let t032 = get("T032");
    let base011 = get(t032["input_construction"]["base_row_id"].as_str().unwrap());
    let mut cur = base011["input"].as_str().unwrap().to_string();
    let wraps = t032["input_construction"]["additional_wraps_applied"]
        .as_u64()
        .unwrap();
    for _ in 0..wraps {
        let escaped = cur.replace('\'', "'\\''");
        cur = format!("sh -c '{escaped}'");
    }
    assert_eq!(
        cur.len(),
        t032["input_construction"]["expected_length"]
            .as_u64()
            .unwrap() as usize
    );
    assert_eq!(
        sha256_hex(cur.as_bytes()),
        t032["input_construction"]["expected_sha256"]
    );
}

/// Point 5 of the #1715 review: "Bind plan_ref to actual command
/// digest/destination/context/manifest version." For every manifest row
/// claiming transformation_understanding=supported, independently recompute
/// sha256 of the row's own literal command text and assert it equals the
/// bound plan's command_digest -- and that destination/context/manifest
/// version actually agree too.
#[test]
fn supported_rows_plan_ref_is_bound_to_the_rows_own_command_bytes() {
    let manifest = load_json(MANIFEST);
    let manifest_version = manifest["manifest_version"].as_str().unwrap();
    for row in manifest["rows"].as_array().unwrap() {
        if row["transformation_understanding"] != "supported" {
            continue;
        }
        let id = row["id"].as_str().unwrap();
        let input = row["input"]
            .as_str()
            .unwrap_or_else(|| panic!("{id}: supported row must have a literal input"));
        let plan = load_json(row["plan_ref"].as_str().unwrap());

        assert_eq!(
            sha256_hex(input.as_bytes()),
            plan["command_digest"],
            "{id}: plan_ref command_digest does not match this row's own command bytes"
        );
        assert!(
            input.contains(plan["normalized_destination"].as_str().unwrap()),
            "{id}: row's command text does not literally contain the plan's normalized_destination"
        );
        assert_eq!(
            row["context_supplied_cwd"], plan["context_identity"]["cwd_supplied"],
            "{id}: context_supplied_cwd mismatch with plan_ref"
        );
        if let Some(cwd) = row.get("supplied_cwd").and_then(|v| v.as_str()) {
            assert_eq!(
                Some(cwd),
                plan["context_identity"]["cwd"].as_str(),
                "{id}: supplied_cwd mismatch with plan_ref"
            );
        }
        assert_eq!(plan["manifest_version"], manifest_version, "{id}: plan_ref manifest_version does not match the current manifest.json manifest_version");
    }
}

#[test]
fn every_supported_manifest_row_plan_ref_resolves_and_validates() {
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
            let errors = plan_validator.iter_errors(&plan);
            assert!(
                errors.is_empty(),
                "{id}: plan_ref {plan_ref:?} fails plan schema validation: {errors:#?}"
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
                "{id}: unsupported_reason {reason:?} not in frozen enum"
            );
        }
    }
}

// ===========================================================================
// Plan / evidence / report fixtures: schema validation, valid + invalid
// ===========================================================================

fn assert_all_valid(validator: &minischema::Validator, dir: &str) {
    for path in list_json_files(dir) {
        let instance = load_json(&path);
        let errors = validator.iter_errors(&instance);
        assert!(
            errors.is_empty(),
            "{path} was expected to validate but did not: {errors:#?}"
        );
    }
}

fn assert_all_invalid(validator: &minischema::Validator, dir: &str, skip: &[&str]) {
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
fn plan_fixtures_invalid_directory_fails_schema_or_is_a_documented_rust_only_check() {
    // Each of these is schema-valid BY DESIGN -- draft-07 cannot express the
    // check without a custom keyword -- and is instead caught by a named
    // Rust-level test below. Each skip entry here has a matching #[test].
    assert_all_invalid(
        &compiled(PLAN_SCHEMA),
        &format!("{FX}/plans/invalid"),
        &[
            "count_byte_ranges_mismatch.json", // -> approved_removal_ranges_count_matches_byte_ranges_length
            "overlapping_byte_ranges.json",    // -> byte_ranges_are_non_overlapping_and_sorted
            "unrelated_target_fingerprint.json", // -> byte_ranges_actually_slice_out_bytes_matching_target_sha256 (negative half)
        ],
    );
}

#[test]
fn approved_removal_ranges_count_matches_byte_ranges_length() {
    let bad = load_json(&format!(
        "{FX}/plans/invalid/count_byte_ranges_mismatch.json"
    ));
    let mismatched = bad["approved_removal_ranges"]
        .as_array()
        .unwrap()
        .iter()
        .any(|r| {
            r["count"].as_u64().unwrap() as usize != r["byte_ranges"].as_array().unwrap().len()
        });
    assert!(
        mismatched,
        "fixture must actually contain a count/byte_ranges mismatch"
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
fn byte_ranges_are_non_overlapping_and_sorted() {
    fn all_ranges(plan: &Value) -> Vec<(usize, usize)> {
        let mut v: Vec<(usize, usize)> = plan["approved_removal_ranges"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|r| r["byte_ranges"].as_array().unwrap().iter())
            .map(|br| {
                (
                    br["start"].as_u64().unwrap() as usize,
                    br["end"].as_u64().unwrap() as usize,
                )
            })
            .collect();
        v.sort();
        v
    }

    for path in list_json_files(&format!("{FX}/plans/valid")) {
        let plan = load_json(&path);
        let ranges = all_ranges(&plan);
        for i in 1..ranges.len() {
            assert!(
                ranges[i - 1].1 <= ranges[i].0,
                "{path}: byte_ranges overlap: {:?} vs {:?}",
                ranges[i - 1],
                ranges[i]
            );
        }
    }

    let bad = load_json(&format!("{FX}/plans/invalid/overlapping_byte_ranges.json"));
    let ranges = all_ranges(&bad);
    let overlaps = (1..ranges.len()).any(|i| ranges[i - 1].1 > ranges[i].0);
    assert!(
        overlaps,
        "overlapping_byte_ranges.json must actually contain an overlap"
    );
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
fn report_fixtures_invalid_directory_fails_schema_or_is_a_documented_rust_only_check() {
    assert_all_invalid(
        &compiled(REPORT_SCHEMA),
        &format!("{FX}/reports/invalid"),
        &[
            "scope_binding_digest_does_not_match_preimage.json", // -> confirmed_scoped_digest_must_match_the_versioned_preimage
        ],
    );
}

#[test]
fn confirmed_scoped_digest_must_match_the_versioned_preimage() {
    // Point 4 of the #1715 review: define a VERSIONED digest preimage, and
    // prove a wrong-but-well-formed digest is still caught. JSON Schema
    // cannot recompute a hash, so this is necessarily a Rust-level check.
    let plan = load_json(&format!(
        "{FX}/plans/valid/mutating_removal_with_backup_required.json"
    ));
    let real_digest = compute_scope_binding_digest_v1(&plan);

    let good = load_json(&format!("{FX}/reports/valid/verified_removal.json"));
    assert_eq!(
        good["authorization"]["scope_binding_digest"], real_digest,
        "verified_removal.json's digest must match the recomputed v1 preimage"
    );

    let bad = load_json(&format!(
        "{FX}/reports/invalid/scope_binding_digest_does_not_match_preimage.json"
    ));
    let claimed = bad["authorization"]["scope_binding_digest"]
        .as_str()
        .unwrap();
    assert_ne!(
        claimed, real_digest,
        "the negative fixture must actually carry a WRONG digest to be a meaningful test"
    );
    assert_eq!(bad["authorization"]["state"], "confirmed_scoped");
}

/// Preview/regression-guard for the frozen acceptance-test identifier
/// `supplied_approval_never_authorizes_runtime` (point 4 of the review):
/// no valid report fixture may combine state=confirmed_scoped with a
/// supplied_claim provenance, and the schema must actively reject it if one
/// tried.
#[test]
fn supplied_approval_never_authorizes_runtime() {
    for path in list_json_files(&format!("{FX}/reports/valid")) {
        let report = load_json(&path);
        if report["authorization"]["state"] == "confirmed_scoped" {
            assert_eq!(
                report["authorization"]["provenance"], "caro_interactive",
                "{path}: confirmed_scoped must never be reached via supplied_claim"
            );
        }
    }
    let validator = compiled(REPORT_SCHEMA);
    let bad = load_json(&format!(
        "{FX}/reports/invalid/confirmed_scoped_supplied_claim.json"
    ));
    assert!(
        !validator.is_valid(&bad),
        "schema must reject confirmed_scoped+supplied_claim"
    );
}

#[test]
fn every_forbidden_report_combination_from_the_review_has_a_fixture() {
    let required = [
        "confirmed_scoped_missing_digest.json",
        "confirmed_scoped_supplied_claim.json",
        "host_permission_granted_none_provenance.json",
        "postcondition_verified_basis_none.json",
        "postcondition_failed_basis_none.json",
        "removed_effect_without_completed_execution.json",
        "cleanup_failed_workflow_completed.json",
        "cleanup_failed_workflow_unverified_not_failed.json",
        "completed_workflow_with_a_failed_step.json",
        "execution_completed_without_install_step.json",
        "execution_failed_without_any_failing_step.json",
        "already_satisfied_with_execution_started.json",
        "scope_digest_present_without_confirmed.json",
        "shell_exit_zero_alone_claims_completed.json",
        "step_status_unknown_without_observation_source.json",
        "scope_binding_digest_does_not_match_preimage.json",
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
    let required = [
        "already_satisfied_no_op.json",
        "failed_install_independently_verified.json",
        "dry_run_supplied_state_verified.json",
        "dry_run_unverified.json",
        "dry_run_independently_failed_verification.json",
        "cleanup_failure_preserves_verified_state.json",
        "verified_removal.json",
        "stage_status_unknown_but_observed.json",
    ];
    let present = list_json_files(&format!("{FX}/reports/valid"));
    for name in required {
        assert!(
            present.iter().any(|p| p.ends_with(name)),
            "missing required positive report fixture: {name}"
        );
    }
}

/// Point 1 of the #1715 review: the ORIGINAL R5 wrongly rejected
/// execution=not_attempted + independently-failed supplied-state
/// verification. Prove the fixed schema now accepts exactly that, while
/// still rejecting a bare (evidence-free) failed claim.
#[test]
fn dry_run_can_independently_fail_verification_without_conflating_execution_state() {
    let validator = compiled(REPORT_SCHEMA);
    let good = load_json(&format!(
        "{FX}/reports/valid/dry_run_independently_failed_verification.json"
    ));
    assert_eq!(good["execution"]["state"], "not_attempted");
    assert_eq!(good["postcondition"]["state"], "failed");
    assert_eq!(
        good["postcondition"]["verification_basis"],
        "supplied_evidence"
    );
    assert!(
        validator.is_valid(&good),
        "a dry-run with independently-failed supplied evidence must validate"
    );

    let bad = load_json(&format!(
        "{FX}/reports/invalid/postcondition_failed_basis_none.json"
    ));
    assert!(
        !validator.is_valid(&bad),
        "postcondition=failed with no supplied evidence must still be rejected"
    );
}

// ===========================================================================
// Byte-level grounding: independent PEM/DER parsing, not trusted claims.
// ===========================================================================

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
        "{context}: only fixture_file artifact refs exist in v1"
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
        for key in ["before_artifact_ref", "expected_staged_artifact_ref"] {
            if !plan[key].is_null() {
                check_artifact_ref(&plan[key], &format!("{path} {key}"));
            }
        }
        let before_bytes = read_artifact(
            plan["before_artifact_ref"]["fixture_path"]
                .as_str()
                .unwrap(),
        );
        assert_eq!(sha256_hex(&before_bytes), plan["before_hash"]);
        let staged_bytes = read_artifact(
            plan["expected_staged_artifact_ref"]["fixture_path"]
                .as_str()
                .unwrap(),
        );
        assert_eq!(sha256_hex(&staged_bytes), plan["expected_staged_hash"]);
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
                    &format!("{path} step={}", obs["step"]),
                );
                let bytes = read_artifact(obs["artifact_ref"]["fixture_path"].as_str().unwrap());
                assert_eq!(sha256_hex(&bytes), hash);
            }
        }
    }
}

/// Point 2 of the #1715 review, in full: independently parse the real PEM
/// bundle, decode each block's base64 body to DER, hash the DER (NOT the
/// raw PEM section) to get the fingerprint, and check the plan's declared
/// fingerprints against what is actually in the file -- never trusting the
/// plan's own claim.
#[test]
fn target_fingerprints_are_der_fingerprints_independently_rederived_from_real_bytes() {
    let plan = load_json(&format!(
        "{FX}/plans/valid/mutating_removal_with_backup_required.json"
    ));
    let before_path = plan["before_artifact_ref"]["fixture_path"]
        .as_str()
        .unwrap();
    let before_bytes = read_artifact(before_path);
    let blocks = find_pem_blocks(&before_bytes);
    assert_eq!(
        blocks.len(),
        4,
        "before_two_targets.pem.fixture must contain exactly 4 certificate blocks"
    );

    for b in &blocks {
        assert!(
            looks_like_valid_der_sequence(&b.der),
            "block at {:?} does not decode to a valid DER SEQUENCE envelope",
            b.range
        );
    }

    let mut independent_counts: std::collections::BTreeMap<String, u64> =
        std::collections::BTreeMap::new();
    for b in &blocks {
        *independent_counts.entry(sha256_hex(&b.der)).or_insert(0) += 1;
    }

    let declared: std::collections::BTreeMap<String, u64> = plan["target_fingerprints"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| {
            (
                f["sha256"].as_str().unwrap().to_string(),
                f["occurrence_count"].as_u64().unwrap(),
            )
        })
        .collect();
    assert_eq!(independent_counts, declared, "plan.target_fingerprints does not match the independently re-derived DER fingerprint multiset");

    // Also confirm the independently-derived hashes differ from a naive
    // raw-PEM-section hash -- this is the exact bug the review found.
    for b in &blocks {
        let raw_section_hash = sha256_hex(&before_bytes[b.range.0..b.range.1]);
        let der_hash = sha256_hex(&b.der);
        assert_ne!(raw_section_hash, der_hash, "raw-PEM-section hash and DER fingerprint must differ (that they previously didn't is the bug)");
    }
}

/// Point 2: "validate non-overlapping section ranges, apply ranges, compare
/// exact staged/final bytes." Independently re-derive the block ranges from
/// the real file (not from the plan's byte_ranges), then check the plan's
/// declared byte_ranges actually line up with those real block boundaries,
/// that each range's DER hashes to its declared target_sha256, and that
/// removing exactly those ranges reconstructs the expected staged bytes
/// byte-for-byte.
#[test]
fn byte_ranges_actually_slice_out_bytes_matching_target_sha256_and_apply_to_exact_staged_bytes() {
    let plan = load_json(&format!(
        "{FX}/plans/valid/mutating_removal_with_backup_required.json"
    ));
    let before_bytes = read_artifact(
        plan["before_artifact_ref"]["fixture_path"]
            .as_str()
            .unwrap(),
    );
    let independent_blocks = find_pem_blocks(&before_bytes);
    let independent_ranges: BTreeSet<(usize, usize)> =
        independent_blocks.iter().map(|b| b.range).collect();

    let mut all_declared_ranges: Vec<(usize, usize, String)> = Vec::new();
    for range in plan["approved_removal_ranges"].as_array().unwrap() {
        let target_sha = range["target_sha256"].as_str().unwrap();
        for br in range["byte_ranges"].as_array().unwrap() {
            let start = br["start"].as_u64().unwrap() as usize;
            let end = br["end"].as_u64().unwrap() as usize;
            all_declared_ranges.push((start, end, target_sha.to_string()));
        }
    }

    for (start, end, target_sha) in &all_declared_ranges {
        assert!(
            independent_ranges.contains(&(*start, *end)),
            "declared byte_range ({start},{end}) does not match any independently-parsed PEM block boundary"
        );
        let block = independent_blocks
            .iter()
            .find(|b| b.range == (*start, *end))
            .unwrap();
        assert!(
            looks_like_valid_der_sequence(&block.der),
            "block at ({start},{end}) is not valid DER"
        );
        assert_eq!(
            &sha256_hex(&block.der),
            target_sha,
            "DER at ({start},{end}) does not hash to the declared target_sha256"
        );
    }

    // apply ranges: remove every declared range from before_bytes, in order,
    // and check the remainder matches expected_staged_hash exactly.
    let mut sorted = all_declared_ranges.clone();
    sorted.sort_by_key(|(s, _, _)| *s);
    let mut cursor = 0usize;
    let mut kept = Vec::new();
    for (start, end, _) in &sorted {
        assert!(
            *start >= cursor,
            "ranges are not in non-overlapping sorted order"
        );
        kept.extend_from_slice(&before_bytes[cursor..*start]);
        cursor = *end;
    }
    kept.extend_from_slice(&before_bytes[cursor..]);
    assert_eq!(
        sha256_hex(&kept),
        plan["expected_staged_hash"],
        "removing the declared byte_ranges does not reconstruct expected_staged_hash"
    );

    // negative half: the "unrelated target fingerprint" plan claims a
    // target_sha256 that matches NEITHER cert_A nor cert_B nor cert_C.
    let bad = load_json(&format!(
        "{FX}/plans/invalid/unrelated_target_fingerprint.json"
    ));
    let claimed = bad["approved_removal_ranges"][0]["target_sha256"]
        .as_str()
        .unwrap();
    let real_hashes: BTreeSet<String> = independent_blocks
        .iter()
        .map(|b| sha256_hex(&b.der))
        .collect();
    assert!(!real_hashes.contains(claimed), "unrelated_target_fingerprint.json's claimed hash must NOT match any real block (that's the point of the fixture)");
}

#[test]
fn invalid_der_artifact_independently_fails_the_der_envelope_check() {
    let bytes = read_artifact(&format!("{FX}/artifacts/invalid_der.pem.fixture"));
    let blocks = find_pem_blocks(&bytes);
    assert_eq!(blocks.len(), 1);
    assert!(
        !looks_like_valid_der_sequence(&blocks[0].der),
        "invalid_der.pem.fixture must NOT decode to a valid DER SEQUENCE"
    );
}

#[test]
fn reordered_bundle_has_the_same_fingerprint_multiset_but_different_byte_ranges() {
    // A reordered bundle is a legitimate negative for "order matters for
    // byte_ranges, even when the multiset is identical."
    let before = read_artifact(&format!("{FX}/artifacts/before_two_targets.pem.fixture"));
    let reordered = read_artifact(&format!("{FX}/artifacts/reordered_two_targets.pem.fixture"));
    let before_blocks = find_pem_blocks(&before);
    let reordered_blocks = find_pem_blocks(&reordered);

    let fp = |blocks: &[PemBlock]| -> BTreeSet<String> {
        blocks.iter().map(|b| sha256_hex(&b.der)).collect()
    };
    assert_eq!(
        fp(&before_blocks),
        fp(&reordered_blocks),
        "reordering must not change the fingerprint SET"
    );

    // Every certificate here is exactly 741 bytes, so the byte-range SLOTS
    // ((0,741), (741,1482), ...) are identical before and after reordering
    // -- only WHICH fingerprint sits in which slot changes. That is exactly
    // what makes order a real, independent hazard from the multiset: a
    // byte_ranges list is only meaningful bound to its own specific file.
    let fp_at = |blocks: &[PemBlock], idx: usize| sha256_hex(&blocks[idx].der);
    let any_slot_changed =
        (0..before_blocks.len()).any(|i| fp_at(&before_blocks, i) != fp_at(&reordered_blocks, i));
    assert!(
        any_slot_changed,
        "reordering must change which fingerprint occupies at least one byte-range slot"
    );

    // the plan's byte_ranges (derived from before_two_targets) must NOT line
    // up with the reordered file's block boundaries for the SAME fingerprint.
    let plan = load_json(&format!(
        "{FX}/plans/valid/mutating_removal_with_backup_required.json"
    ));
    let cert_a_range = &plan["approved_removal_ranges"][0]["byte_ranges"][0];
    let (s, e) = (
        cert_a_range["start"].as_u64().unwrap() as usize,
        cert_a_range["end"].as_u64().unwrap() as usize,
    );
    let reordered_block_at_same_offset =
        reordered_blocks.iter().find(|b| b.range == (s, e)).unwrap();
    assert_ne!(
        sha256_hex(&reordered_block_at_same_offset.der),
        plan["approved_removal_ranges"][0]["target_sha256"],
        "the plan's byte range, reapplied to the reordered file, must land on a DIFFERENT certificate -- proving order matters"
    );
}

#[test]
fn malformed_truncated_artifact_has_no_end_marker_and_its_body_does_not_decode_to_valid_der() {
    let bytes = read_artifact(&format!("{FX}/artifacts/malformed_truncated.pem.fixture"));
    let text = String::from_utf8(bytes).unwrap();
    assert!(text.contains("-----BEGIN CERTIFICATE-----"));
    assert!(!text.contains("-----END CERTIFICATE-----"));
    assert!(
        find_pem_blocks(text.as_bytes()).is_empty(),
        "a block with no END marker must not be parsed as a complete block at all"
    );
}

// ===========================================================================
// Cross-document binding (plan<->evidence), order/freshness checks
// ===========================================================================

#[test]
fn plan_and_evidence_context_identity_definitions_are_byte_identical() {
    let plan_schema = load_json(PLAN_SCHEMA);
    let evidence_schema = load_json(EVIDENCE_SCHEMA);
    let plan_def = &plan_schema["definitions"]["context_identity"];
    let evidence_def = &evidence_schema["definitions"]["context_identity"];
    assert!(!plan_def.is_null() && !evidence_def.is_null());
    assert_eq!(
        plan_def, evidence_def,
        "context_identity definitions have drifted between plan and evidence schemas"
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
        assert_eq!(plan["operation_id"], evidence["operation_id"]);
        assert_eq!(plan["evidence_id"], evidence["evidence_id"]);
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
                obs["normalized_destination_binding"],
                plan["normalized_destination"]
            );
            assert_eq!(obs["context_binding"], plan["context_identity"]);
        }
    }
}

/// Point 3 of the #1715 review: "explicit pre-install before/staging
/// snapshot bindings and freshness/order checks." For every install
/// observation that names a before/staging snapshot id, that id must
/// actually belong to a before/stage observation present EARLIER (by
/// capture_sequence) in the same bundle -- not a dangling or forward
/// reference.
#[test]
fn install_snapshot_bindings_reference_real_earlier_observations() {
    let evidence = load_json(&format!("{FX}/evidence/valid/verified_removal.json"));
    let observations = evidence["observations"].as_array().unwrap();

    for obs in observations {
        if obs["step"] != "install" {
            continue;
        }
        let install_seq = obs["capture_sequence"].as_u64().unwrap();
        for (field, expected_step) in [
            ("consumes_before_snapshot_id", "before"),
            ("consumes_staging_snapshot_id", "stage"),
        ] {
            let referenced_id = obs[field]
                .as_str()
                .unwrap_or_else(|| panic!("install observation must name {field}"));
            let referent = observations
                .iter()
                .find(|o| o["step"] == expected_step && o["step_snapshot_id"] == referenced_id);
            let referent = referent.unwrap_or_else(|| panic!("{field}={referenced_id:?} does not name any {expected_step} observation in this bundle"));
            assert!(
                referent["capture_sequence"].as_u64().unwrap() < install_seq,
                "{field} must reference an observation captured BEFORE the install step (stale/forward reference otherwise)"
            );
        }
    }
}

#[test]
fn capture_sequence_is_monotonic_in_canonical_step_order() {
    fn step_rank(step: &str) -> u8 {
        match step {
            "before" => 0,
            "backup" => 1,
            "stage" => 2,
            "filter" => 3,
            "install" => 4,
            "final" => 5,
            "cleanup" => 6,
            other => panic!("unknown step {other}"),
        }
    }
    for path in list_json_files(&format!("{FX}/evidence/valid")) {
        let evidence = load_json(&path);
        let mut last_seq = -1i64;
        let mut last_rank = -1i8;
        for obs in evidence["observations"].as_array().unwrap() {
            let seq = obs["capture_sequence"].as_i64().unwrap();
            let rank = step_rank(obs["step"].as_str().unwrap()) as i8;
            assert!(
                seq > last_seq,
                "{path}: capture_sequence must be strictly increasing"
            );
            assert!(rank >= last_rank, "{path}: steps must appear in canonical order (before<backup<stage<filter<install<final<cleanup)");
            last_seq = seq;
            last_rank = rank;
        }
    }
}

// ===========================================================================
// Reason codes / frozen enum drift checks
// ===========================================================================

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
    assert_eq!(enum_values, frozen);
}

#[test]
fn contract_doc_names_every_frozen_reason_code_and_test_identifier() {
    let doc =
        fs::read_to_string(CONTRACT_DOC).unwrap_or_else(|e| panic!("reading {CONTRACT_DOC}: {e}"));
    for code in frozen_reason_codes() {
        assert!(
            doc.contains(code),
            "docs/trust-operation-contract.md missing frozen reason code: {code}"
        );
    }
    for test_id in frozen_acceptance_test_identifiers() {
        assert!(
            doc.contains(test_id),
            "docs/trust-operation-contract.md missing frozen acceptance-test identifier: {test_id}"
        );
    }
}

#[test]
fn authorization_confirmed_scoped_still_restricted_to_caro_interactive_in_schema() {
    let schema = load_json(REPORT_SCHEMA);
    let state_desc = schema["definitions"]["authorization"]["properties"]["state"]["description"]
        .as_str()
        .unwrap_or("");
    assert!(state_desc.contains("confirmed_scoped"));
    let all_of = schema["allOf"]
        .as_array()
        .expect("report schema must have allOf rules");
    assert!(
        all_of.len() >= 13,
        "expected at least 13 allOf conditionals (R1..R13+), found {}",
        all_of.len()
    );
}

#[test]
fn no_jsonschema_crate_dependency_exists() {
    // Direct regression guard for point 7 of the #1715 review: once removed,
    // it must stay removed rather than silently creeping back in.
    let cargo_toml = fs::read_to_string("Cargo.toml").unwrap();
    assert!(!cargo_toml.contains("jsonschema"), "Cargo.toml must not reintroduce the jsonschema crate (license policy conflict, see module doc above)");
}
