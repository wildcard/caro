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
//!
//! ## Why there IS an external X.509 parsing crate here (unlike `jsonschema`)
//!
//! Round 6 added `x509-parser` (dev-dependency, `default-features = false`)
//! for `looks_like_complete_x509_der`, replacing round 4/5's hand-written
//! DER TLV-walking heuristics. This is NOT the same situation as
//! `jsonschema` above: `x509-parser`'s own dependency tree (`asn1-rs`,
//! `der-parser`, `oid-registry`, `rusticata-macros`, `data-encoding`, all
//! `MIT OR Apache-2.0`) clears `dependency-review.yml`'s license allowlist
//! cleanly -- no `MIT-0`, no `Unicode-3.0`, no exception needed. It is a
//! real, independently conformant DER/ASN.1 certificate parser (not
//! another round of child-tag/count heuristics), used ONLY to check
//! syntactic completeness (`rem.is_empty()` after parsing) -- the
//! `validate`/`verify`/`verify-aws` features (date-ordering/extension
//! rules, and chain/signature verification pulling in `ring`/`aws-lc-rs`)
//! are explicitly left disabled, so this performs no chain, trust,
//! expiry, or signature validation, matching this contract's existing
//! stated limits.

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

    /// Keywords `eval` actually interprets.
    const EVALUATED_KEYWORDS: &[&str] = &[
        "$ref",
        "type",
        "const",
        "enum",
        "pattern",
        "minLength",
        "minimum",
        "exclusiveMinimum",
        "required",
        "properties",
        "additionalProperties",
        "items",
        "minItems",
        "contains",
        "allOf",
        "anyOf",
        "not",
        "if",
        "then",
        "else",
    ];
    /// Keywords that carry no validation semantics for this evaluator --
    /// explicitly allow-listed per the #1715 round-4 review's point 1
    /// ("explicitly allow harmless annotations"), rather than silently
    /// tolerating anything unrecognized by omission.
    const HARMLESS_ANNOTATION_KEYWORDS: &[&str] = &[
        "$schema",
        "$id",
        "title",
        "description",
        "$comment",
        "$see",
        "definitions",
    ];

    /// Recursively preflight a schema document BEFORE it is ever used to
    /// validate an instance. Fixes the #1715 round-4 review's point 1
    /// finding that `Validator::new` previously only cloned the JSON with
    /// no checking at all, so a schema with an unsupported keyword, or a
    /// malformed value for a keyword this evaluator DOES support (e.g.
    /// `"type": 42` or `"maxLength": 0` -- `maxLength` isn't even
    /// evaluated, so silently accepting it as a no-op would be exactly
    /// the bug the review flagged), would be silently accepted and then
    /// validate every instance as trivially valid. This function panics
    /// (fails loud, at schema-compile time, matching the "preflight and
    /// reject" instruction) rather than returning a soft error, since a
    /// malformed schema in this repository's own frozen fixtures is a
    /// programming error in the contract itself, not recoverable input.
    fn preflight(schema: &Value, path: &str) {
        match schema {
            Value::Object(obj) => {
                for (key, value) in obj {
                    let sub_path = format!("{path}.{key}");
                    if HARMLESS_ANNOTATION_KEYWORDS.contains(&key.as_str()) {
                        if key == "definitions" {
                            if let Value::Object(defs) = value {
                                for (def_name, def_schema) in defs {
                                    preflight(def_schema, &format!("{sub_path}.{def_name}"));
                                }
                            } else {
                                panic!("{sub_path}: definitions must be an object, got {value}");
                            }
                        }
                        continue;
                    }
                    if !EVALUATED_KEYWORDS.contains(&key.as_str()) {
                        panic!("{sub_path}: unsupported keyword {key:?} -- minischema only understands {EVALUATED_KEYWORDS:?} plus the harmless annotations {HARMLESS_ANNOTATION_KEYWORDS:?}. Add real support before using this keyword, or use the independent jsonschema-backed check in scripts/validate_trust_operation_schemas.py if this is intentional.");
                    }
                    match key.as_str() {
                        "type" => match value {
                            Value::String(s) => {
                                const VALID: &[&str] = &["object", "array", "string", "integer", "number", "boolean", "null"];
                                if !VALID.contains(&s.as_str()) {
                                    panic!("{sub_path}: type {s:?} is not a recognized JSON Schema type");
                                }
                            }
                            Value::Array(arr) => {
                                for v in arr {
                                    preflight(&serde_json::json!({ "type": v }), &sub_path);
                                }
                            }
                            other => panic!("{sub_path}: type must be a string or array of strings, got {other}"),
                        },
                        "required" => match value {
                            Value::Array(arr) => {
                                for v in arr {
                                    if !v.is_string() {
                                        panic!("{sub_path}: required must be an array of strings, found {v}");
                                    }
                                }
                            }
                            other => panic!("{sub_path}: required must be an array, got {other}"),
                        },
                        "properties" => match value {
                            Value::Object(props) => {
                                for (k, v) in props {
                                    preflight(v, &format!("{sub_path}.{k}"));
                                }
                            }
                            other => panic!("{sub_path}: properties must be an object, got {other}"),
                        },
                        "additionalProperties" => {
                            if !value.is_boolean() {
                                panic!("{sub_path}: additionalProperties must be a boolean in this subset, got {value}");
                            }
                        }
                        "pattern" => {
                            let Some(s) = value.as_str() else {
                                panic!("{sub_path}: pattern must be a string, got {value}");
                            };
                            if Regex::new(s).is_err() {
                                panic!("{sub_path}: pattern {s:?} is not a valid regular expression");
                            }
                        }
                        "minLength" | "minItems" => {
                            if !value.is_u64() {
                                panic!("{sub_path}: {key} must be a non-negative integer, got {value}");
                            }
                        }
                        "minimum" | "exclusiveMinimum" => {
                            if value.as_f64().is_none() {
                                panic!("{sub_path}: {key} must be a number, got {value}");
                            }
                        }
                        "items" | "contains" | "not" | "if" | "then" | "else" => preflight(value, &sub_path),
                        "allOf" | "anyOf" => match value {
                            Value::Array(arr) => {
                                for (i, v) in arr.iter().enumerate() {
                                    preflight(v, &format!("{sub_path}[{i}]"));
                                }
                            }
                            other => panic!("{sub_path}: {key} must be an array of schemas, got {other}"),
                        },
                        "enum" => {
                            if !value.is_array() {
                                panic!("{sub_path}: enum must be an array, got {value}");
                            }
                        }
                        "$ref" => {
                            if !value.is_string() {
                                panic!("{sub_path}: $ref must be a string, got {value}");
                            }
                        }
                        "const" => {} // any JSON value is a legal const
                        _ => unreachable!("keyword {key:?} is in EVALUATED_KEYWORDS but has no preflight arm"),
                    }
                }
            }
            Value::Bool(b) => {
                // draft-07 allows `true`/`false` as a schema (always-valid /
                // always-invalid), but `eval` does not implement that
                // semantics -- it would silently treat a boolean schema as
                // "no constraints" (always valid) by calling .get() on a
                // non-object and finding nothing, which is wrong for
                // `false`. Per the #1715 round-5 review's small fix
                // ("implement correctly OR reject"), reject rather than
                // risk a silently-wrong accept. None of this contract's
                // four real schemas use a boolean schema.
                panic!("{path}: boolean schema ({b}) is not supported by this evaluator -- `eval` has no semantics for it. Rewrite as an equivalent object schema.");
            }
            other => panic!("{path}: a (sub)schema must be an object or boolean, got {other}"),
        }
    }

    impl Validator {
        pub fn new(schema: &Value) -> Validator {
            preflight(schema, "$");
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

/// Strict RFC 4648 standard-alphabet base64 decoder (no external crate).
/// Unlike a permissive decoder, this REJECTS: a non-multiple-of-4 length
/// (after stripping whitespace only), `=` padding anywhere but the final
/// 1-2 characters, more than 2 padding characters, and any byte outside the
/// standard alphabet. Fixes the #1715 round-4 review's point 3 finding that
/// the prior decoder silently tolerated malformed padding instead of
/// rejecting it (see `artifacts/malformed_base64_padding.pem.fixture`).
///
/// Also enforces the round-6 review's point 2 finding that this is a
/// SEPARATE strictness concern from DER BIT STRING unused-bits validation:
/// RFC 4648 section 3.5 requires the UNUSED trailing bits of the final
/// base64 sextet to be zero. A one-padding-character tail (`data_len % 4
/// == 3`) carries 18 encoded bits for 2 output bytes (16 bits), leaving 2
/// unused low bits; a two-padding-character tail (`data_len % 4 == 2`)
/// carries 12 encoded bits for 1 output byte (8 bits), leaving 4 unused
/// low bits. A non-canonical encoder can set those unused bits to
/// anything and this decoder would previously discard them silently --
/// which means two DIFFERENT base64 strings could decode to the SAME
/// bytes, undermining the "the fixture file's literal bytes are what we
/// hash" guarantee the rest of this contract relies on. Now rejected.
fn base64_decode_strict(input: &str) -> Option<Vec<u8>> {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut table = [255u8; 256];
    for (i, &c) in ALPHABET.iter().enumerate() {
        table[c as usize] = i as u8;
    }
    let clean: Vec<u8> = input.bytes().filter(|b| !b.is_ascii_whitespace()).collect();
    if clean.is_empty() || clean.len() % 4 != 0 {
        return None;
    }
    let pad_count = clean.iter().rev().take_while(|&&b| b == b'=').count();
    if pad_count > 2 {
        return None;
    }
    let data_len = clean.len() - pad_count;
    for (i, &b) in clean.iter().enumerate() {
        if b == b'=' {
            if i < data_len {
                return None; // '=' appearing before the trailing pad region
            }
        } else if table[b as usize] == 255 {
            return None; // byte outside the standard alphabet
        }
    }
    let mut out = Vec::with_capacity(data_len * 3 / 4 + 3);
    let mut buf: u32 = 0;
    let mut bits = 0u32;
    for &b in &clean[..data_len] {
        buf = (buf << 6) | table[b as usize] as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push(((buf >> bits) & 0xFF) as u8);
        }
    }
    if bits > 0 {
        let unused_mask = (1u32 << bits) - 1;
        if buf & unused_mask != 0 {
            return None; // non-canonical: unused trailing bits of the final sextet are not zero
        }
    }
    Some(out)
}

/// A parsed `-----BEGIN CERTIFICATE----- ... -----END CERTIFICATE-----` block.
struct PemBlock {
    /// Byte range in the containing file, [start, end), including the block's
    /// own trailing newline (if any) so adjacent blocks are exactly
    /// concatenable / removable without leaving stray blank lines.
    range: (usize, usize),
    der: Vec<u8>,
}

/// Independently parse every PEM certificate block in `data`. Does not
/// trust anything declared in a plan/evidence/manifest fixture.
///
/// Per the #1715 round-5 review's point 3, this tolerates (does NOT error
/// on) comment lines and blank/whitespace content before, between, and
/// after blocks -- the source issue (#1714) explicitly requires preserving
/// "all other raw bytes, comments, whitespace" verbatim, and round 4's
/// "must consume the whole file" fix over-corrected into rejecting that
/// legitimate content. What IS still rejected: a `BEGIN CERTIFICATE`
/// marker with no matching `END` (unmatched/truncated -- this is not
/// "comment text", it is an attempted block that never completes) and a
/// body that fails strict base64 decoding. Both LF and CRLF line endings
/// after `END CERTIFICATE-----` are recognized as the block's own trailing
/// newline.
fn find_pem_blocks_strict(data: &[u8]) -> Result<Vec<PemBlock>, String> {
    let text = std::str::from_utf8(data).map_err(|e| format!("not valid UTF-8: {e}"))?;
    const BEGIN: &str = "-----BEGIN CERTIFICATE-----";
    const END: &str = "-----END CERTIFICATE-----";
    let mut out = Vec::new();
    let mut pos = 0usize;
    while let Some(b_rel) = text[pos..].find(BEGIN) {
        let b = pos + b_rel;
        let Some(e_rel) = text[b..].find(END) else {
            return Err(format!("BEGIN CERTIFICATE at byte offset {b} has no matching END (unmatched/truncated block)"));
        };
        let mut end = b + e_rel + END.len();
        if text.as_bytes().get(end) == Some(&b'\r') && text.as_bytes().get(end + 1) == Some(&b'\n')
        {
            end += 2;
        } else if text.as_bytes().get(end) == Some(&b'\n') {
            end += 1;
        }
        let body_start = b + BEGIN.len();
        let body_end = b + e_rel;
        let body = &text[body_start..body_end];
        let der = base64_decode_strict(body)
            .ok_or_else(|| format!("block at byte offset {b} has malformed base64 (bad padding, bad length, or an invalid character)"))?;
        out.push(PemBlock {
            range: (b, end),
            der,
        });
        pos = end;
    }
    Ok(out)
}

/// Independently parse `der` as a COMPLETE, syntactically valid X.509
/// `Certificate` structure, using `x509-parser` (a real, independently
/// conformant DER/ASN.1 certificate parser) rather than another round of
/// hand-written tag/count heuristics.
///
/// Per the #1715 round-6 review's point 2: rounds 4 and 5's own
/// `parse_der_tlv_sequence_exact`-based checks were still just child-tag
/// and child-count heuristics (">= 6 children, >= 1 INTEGER, >= 4
/// SEQUENCE") rather than real syntax parsing of `tbsCertificate`'s actual
/// RFC 5280 field sequence (version, serialNumber, signature AlgorithmId,
/// issuer Name, validity, subject Name, subjectPublicKeyInfo). The
/// review's counterexample `30 1c 30 12 02 01 01 30 01 00 30 01 00 30 01
/// 00 30 01 00 02 01 01 30 03 06 01 2a 03 01 00` demonstrates this
/// concretely: its `tbsCertificate` has exactly 6 non-empty children
/// (INTEGER, 4x SEQUENCE, INTEGER) satisfying every one of the round-5
/// heuristic's counts, yet is not a real `tbsCertificate` at all (its
/// fields are not `issuer`/`validity`/`subject`/`subjectPublicKeyInfo` --
/// they're garbage `SEQUENCE{}`s). A fixture test below
/// (`child_tag_count_heuristic_counterexample_is_rejected_by_real_parsing`)
/// proves the real parser rejects these exact bytes while the round-5
/// heuristic (reconstructed in that test) would have accepted them.
///
/// `x509-parser` is a dev-dependency (test-only, never shipped in the
/// `caro` binary) added with `default-features = false`, so neither the
/// `validate` feature (RFC 5280 semantic rules: date ordering, extension
/// constraints) nor the `verify`/`verify-aws` features (signature/chain
/// verification, which pull in `ring`/`aws-lc-rs`) are enabled -- this is
/// pure DER/ASN.1 SYNTAX parsing, never chain, trust, expiry, or signature
/// validation, matching this contract's existing stated limits. Its own
/// transitive dependency tree (`asn1-rs`, `der-parser`, `oid-registry`,
/// `rusticata-macros`, `data-encoding`, all `MIT OR Apache-2.0`) clears
/// `dependency-review.yml`'s license allowlist -- unlike the `jsonschema`
/// crate round 2 tried and round 3 had to remove (see that section below),
/// this dependency is NOT a policy exception, it is a normal pass.
///
/// `rem.is_empty()` additionally enforces FULL consumption: a certificate
/// followed by any trailing byte, however small, is rejected -- this is
/// the same "consume the entire structure, no leftover" discipline this
/// contract already applies to PEM block scanning and manifest byte
/// ranges.
fn looks_like_complete_x509_der(der: &[u8]) -> bool {
    match x509_parser::parse_x509_certificate(der) {
        Ok((rem, _cert)) => rem.is_empty(),
        Err(_) => false,
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

/// Direct demonstration of the #1715 round-4 review's point 1 scenario:
/// `Validator::new` previously only cloned the JSON, so a schema with
/// `"type": 42` would silently compile and then accept every instance.
/// The preflight check added in response now rejects it at compile time.
#[test]
#[should_panic(expected = "type must be a string or array of strings")]
fn validator_new_rejects_a_schema_with_a_non_string_type_value() {
    let bad = serde_json::json!({ "type": 42 });
    let _ = minischema::Validator::new(&bad);
}

#[test]
#[should_panic(expected = "unsupported keyword")]
fn validator_new_rejects_an_unsupported_keyword_like_maxlength() {
    // maxLength is a real draft-07 keyword this evaluator does not
    // implement; silently ignoring it (as the round-3 evaluator did) would
    // let a schema claim a length bound that is never actually checked.
    let bad = serde_json::json!({ "type": "string", "maxLength": 0 });
    let _ = minischema::Validator::new(&bad);
}

#[test]
#[should_panic(expected = "is not a recognized JSON Schema type")]
fn validator_new_rejects_an_unrecognized_type_string() {
    let bad = serde_json::json!({ "type": "not-a-real-json-schema-type" });
    let _ = minischema::Validator::new(&bad);
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
    let blocks = find_pem_blocks_strict(&before_bytes)
        .expect("before_two_targets.pem.fixture must parse completely");
    assert_eq!(
        blocks.len(),
        4,
        "before_two_targets.pem.fixture must contain exactly 4 certificate blocks"
    );

    for b in &blocks {
        assert!(
            looks_like_complete_x509_der(&b.der),
            "block at {:?} does not decode to a complete X.509 structure",
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
    let independent_blocks = find_pem_blocks_strict(&before_bytes)
        .expect("before_two_targets.pem.fixture must parse completely");
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
            looks_like_complete_x509_der(&block.der),
            "block at ({start},{end}) is not a complete X.509 structure"
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
    let blocks = find_pem_blocks_strict(&bytes)
        .expect("a well-formed PEM envelope with a non-DER body still parses as a block");
    assert_eq!(blocks.len(), 1);
    assert!(
        !looks_like_complete_x509_der(&blocks[0].der),
        "invalid_der.pem.fixture must NOT decode to a complete X.509 structure"
    );
}

/// Point 3 of the #1715 round-4 review, each adversarial case named
/// explicitly: an empty `30 00` SEQUENCE, trailing bytes after a valid
/// SEQUENCE's declared length, malformed base64 padding, and a valid
/// certificate followed by an unmatched BEGIN. Each must be independently
/// rejected -- some at the PEM-parse stage, some at the DER-structure
/// stage, but all rejected.
#[test]
fn der_and_pem_adversarial_cases_are_each_independently_rejected() {
    // empty SEQUENCE (30 00): well-formed PEM envelope, valid base64, but
    // the decoded DER is an empty SEQUENCE -- parses as a block, fails the
    // "exactly 3 children" X.509 structure check.
    let bytes = read_artifact(&format!("{FX}/artifacts/empty_der_sequence.pem.fixture"));
    let blocks = find_pem_blocks_strict(&bytes)
        .expect("a well-formed PEM envelope around an empty DER SEQUENCE still parses as a block");
    assert_eq!(blocks.len(), 1);
    assert_eq!(
        blocks[0].der,
        vec![0x30, 0x00],
        "sanity: the decoded DER must actually be the empty SEQUENCE"
    );
    assert!(
        !looks_like_complete_x509_der(&blocks[0].der),
        "an empty 30 00 SEQUENCE must NOT be accepted as a complete X.509 structure"
    );

    // trailing bytes after a valid SEQUENCE's declared length: well-formed
    // PEM envelope, valid base64, decodes to a real certificate's DER plus
    // 4 extra junk bytes the outer length does not account for.
    let bytes = read_artifact(&format!("{FX}/artifacts/der_trailing_bytes.pem.fixture"));
    let blocks = find_pem_blocks_strict(&bytes).expect(
        "a well-formed PEM envelope around DER-plus-trailing-bytes still parses as a block",
    );
    assert_eq!(blocks.len(), 1);
    assert!(
        !looks_like_complete_x509_der(&blocks[0].der),
        "DER with trailing bytes past the declared SEQUENCE length must be rejected"
    );

    // malformed base64 padding: well-formed PEM envelope, but the body's
    // base64 is not a multiple of 4 characters and carries no valid
    // padding -- this must fail at the PEM/base64-decode stage itself.
    let bytes = read_artifact(&format!(
        "{FX}/artifacts/malformed_base64_padding.pem.fixture"
    ));
    let result = find_pem_blocks_strict(&bytes);
    assert!(
        result.is_err(),
        "malformed base64 padding must be rejected, not silently decoded to the wrong bytes"
    );

    // a valid certificate followed by an unmatched BEGIN: the first block
    // is completely well-formed, but trailing content starts a new BEGIN
    // with no matching END -- this must fail the "consume ALL bytes" check.
    let bytes = read_artifact(&format!(
        "{FX}/artifacts/trailing_unmatched_begin.pem.fixture"
    ));
    let result = find_pem_blocks_strict(&bytes);
    assert!(result.is_err(), "a trailing unmatched BEGIN after a valid certificate must be rejected, not silently dropped");
}

/// Point 3 of the #1715 round-5 review, the exact named attack bytes:
/// `30 06 30 00 30 00 03 00` -- an outer SEQUENCE whose declared length
/// (6) exactly matches three empty children (SEQUENCE, SEQUENCE, BIT
/// STRING), so round 4's one-level-deeper check (which only verified tag
/// and count, not content) accepted it. The round-5 recursive structural
/// check must reject it: tbsCertificate's content is empty (0 children,
/// fails >=6), and signatureValue's content is empty (fails the
/// BIT-STRING-must-have-data check).
#[test]
fn nested_empty_der_attack_is_rejected_by_the_recursive_structural_check() {
    let bytes = read_artifact(&format!("{FX}/artifacts/nested_empty_der.pem.fixture"));
    let blocks = find_pem_blocks_strict(&bytes)
        .expect("a well-formed PEM envelope around the attack bytes still parses as a block");
    assert_eq!(blocks.len(), 1);
    assert_eq!(
        blocks[0].der,
        vec![0x30, 0x06, 0x30, 0x00, 0x30, 0x00, 0x03, 0x00],
        "sanity: the decoded DER must actually be the named attack bytes"
    );
    // the ROUND-4 (shallow) check would have accepted this; prove the
    // round-5 (recursive) check does not.
    assert!(
        !looks_like_complete_x509_der(&blocks[0].der),
        "30 06 30 00 30 00 03 00 (empty nested structures) must be rejected"
    );
}

/// The #1715 round-6 review's exact counterexample: `30 1c 30 12 02 01 01
/// 30 01 00 30 01 00 30 01 00 30 01 00 02 01 01 30 03 06 01 2a 03 01 00`.
/// Its `tbsCertificate` has exactly 6 non-empty children (INTEGER, 4x
/// SEQUENCE, INTEGER), which satisfies every count the round-5 heuristic
/// checked (>= 6 children, all non-empty, >= 1 INTEGER tag, >= 4 SEQUENCE
/// tags) -- yet none of those SEQUENCE children are a real `issuer`,
/// `validity`, `subject`, or `subjectPublicKeyInfo`; they're just empty
/// placeholder SEQUENCEs. This proves two things: the round-5 heuristic
/// (reconstructed inline below, exactly as it shipped) WOULD have wrongly
/// accepted these bytes, and the real `x509-parser`-based
/// `looks_like_complete_x509_der` correctly rejects them.
#[test]
fn child_tag_count_heuristic_counterexample_is_rejected_by_real_parsing() {
    let bytes = read_artifact(&format!(
        "{FX}/artifacts/child_tag_count_heuristic_counterexample.pem.fixture"
    ));
    let blocks = find_pem_blocks_strict(&bytes).expect(
        "a well-formed PEM envelope around the counterexample bytes still parses as a block",
    );
    assert_eq!(blocks.len(), 1);
    assert_eq!(
        blocks[0].der,
        vec![
            0x30, 0x1c, 0x30, 0x12, 0x02, 0x01, 0x01, 0x30, 0x01, 0x00, 0x30, 0x01, 0x00, 0x30,
            0x01, 0x00, 0x30, 0x01, 0x00, 0x02, 0x01, 0x01, 0x30, 0x03, 0x06, 0x01, 0x2a, 0x03,
            0x01, 0x00
        ],
        "sanity: the decoded DER must actually be the review's named counterexample bytes"
    );

    // Reconstruct the round-5 heuristic exactly as it shipped, inline, to
    // PROVE it would have wrongly accepted this: a TLV walk checking only
    // outer SEQUENCE{SEQUENCE,SEQUENCE,BIT STRING} tiling, then
    // tbsCertificate child count/tag/non-emptiness -- never real field
    // semantics.
    fn round5_heuristic_would_have_accepted(der: &[u8]) -> bool {
        fn header(buf: &[u8]) -> Option<(u8, usize, usize)> {
            if buf.len() < 2 {
                return None;
            }
            let (tag, len_byte) = (buf[0], buf[1]);
            if len_byte < 0x80 {
                Some((tag, len_byte as usize, 2))
            } else {
                None // none of these fixtures need long-form lengths
            }
        }
        fn tlv_tile(buf: &[u8]) -> Option<Vec<(u8, &[u8])>> {
            let mut out = Vec::new();
            let mut pos = 0usize;
            while pos < buf.len() {
                let (tag, len, hlen) = header(&buf[pos..])?;
                let (start, end) = (pos + hlen, pos + hlen + len);
                if end > buf.len() {
                    return None;
                }
                out.push((tag, &buf[start..end]));
                pos = end;
            }
            (pos == buf.len()).then_some(out)
        }
        let Some(top) = tlv_tile(der) else {
            return false;
        };
        if top.len() != 1 || top[0].0 != 0x30 {
            return false;
        }
        let Some(children) = tlv_tile(top[0].1) else {
            return false;
        };
        if children.len() != 3
            || children[0].0 != 0x30
            || children[1].0 != 0x30
            || children[2].0 != 0x03
        {
            return false;
        }
        let Some(tbs) = tlv_tile(children[0].1) else {
            return false;
        };
        tbs.len() >= 6
            && tbs.iter().all(|(_, c)| !c.is_empty())
            && tbs.iter().any(|(t, _)| *t == 0x02)
            && tbs.iter().filter(|(t, _)| *t == 0x30).count() >= 4
    }

    assert!(
        round5_heuristic_would_have_accepted(&blocks[0].der),
        "sanity: the round-5 heuristic reconstruction must actually reproduce the gap this test exists to close"
    );
    assert!(
        !looks_like_complete_x509_der(&blocks[0].der),
        "the real x509-parser-based check must reject the counterexample bytes even though the round-5 heuristic would have accepted them"
    );
}

/// Point 3 of the #1715 round-5 review: comment lines and blank lines
/// around and between PEM blocks must be TOLERATED (per #1714's own
/// requirement to preserve "all other raw bytes, comments, whitespace"),
/// not rejected the way round 4's "consume the whole file" check did.
#[test]
fn commented_and_whitespace_bundle_parses_both_real_certificates() {
    let bytes = read_artifact(&format!(
        "{FX}/artifacts/commented_whitespace_bundle.pem.fixture"
    ));
    let blocks = find_pem_blocks_strict(&bytes)
        .expect("comments and blank lines around/between blocks must not be rejected");
    assert_eq!(
        blocks.len(),
        2,
        "both real certificate blocks must be found despite the surrounding comment text"
    );
    for b in &blocks {
        assert!(
            looks_like_complete_x509_der(&b.der),
            "both blocks in the commented bundle must still be valid, complete X.509 structures"
        );
    }
    // exact preservation: the comment/whitespace bytes outside the two
    // block ranges must still be present, byte-for-byte, in the file.
    assert!(String::from_utf8(bytes.clone())
        .unwrap()
        .contains("Bag Attributes"));
    assert!(String::from_utf8(bytes.clone())
        .unwrap()
        .contains("# a comment line between blocks"));
    assert!(String::from_utf8(bytes)
        .unwrap()
        .contains("trailing comment after the last block"));
}

/// Per the #1715 round-6 review's point 2: the preservation test above
/// only searches the ORIGINAL (untouched) file for substrings -- it never
/// actually exercises a removal against the commented bundle, so it
/// cannot catch a removal that corrupts or drops surrounding
/// comment/whitespace bytes. This test applies a removal of cert A's PEM
/// block (using `find_pem_blocks_strict`'s own byte range for block 0,
/// exactly as a real removal would -- delete those bytes, keep
/// everything else) and compares the result BYTE-FOR-BYTE against
/// `commented_whitespace_bundle_after_removing_cert_a.pem.fixture`, an
/// expected artifact authored independently (by direct line-level text
/// surgery in a throwaway script, never by running this contract's own
/// PEM/DER code) from the original bundle.
#[test]
fn removing_cert_a_from_the_commented_bundle_matches_an_independently_authored_expected_artifact() {
    let original = read_artifact(&format!(
        "{FX}/artifacts/commented_whitespace_bundle.pem.fixture"
    ));
    let blocks = find_pem_blocks_strict(&original)
        .expect("the commented bundle must still parse before removal");
    assert_eq!(blocks.len(), 2, "sanity: exactly cert A and cert B");

    let (remove_start, remove_end) = blocks[0].range;
    let mut after_removal = Vec::with_capacity(original.len());
    after_removal.extend_from_slice(&original[..remove_start]);
    after_removal.extend_from_slice(&original[remove_end..]);

    let expected = read_artifact(&format!(
        "{FX}/artifacts/commented_whitespace_bundle_after_removing_cert_a.pem.fixture"
    ));
    assert_eq!(
        after_removal, expected,
        "removing cert A's exact byte range must leave every other byte -- the \"Bag Attributes\" header, the blank lines, the inter-block comment, cert B, and the trailing comment -- untouched, matching the independently authored expected artifact exactly"
    );

    // The surviving bytes must still contain exactly one, still-valid
    // certificate (cert B), and the comment/whitespace text must still be
    // literally present -- not merely "the original file still has it"
    // (which the prior test already proved) but "the file AFTER REMOVAL
    // still has it."
    let remaining_blocks = find_pem_blocks_strict(&after_removal)
        .expect("the file must still parse cleanly after removal");
    assert_eq!(remaining_blocks.len(), 1, "exactly cert B must remain");
    assert!(looks_like_complete_x509_der(&remaining_blocks[0].der));
    let remaining_text = String::from_utf8(after_removal).unwrap();
    assert!(remaining_text.contains("Bag Attributes"));
    assert!(remaining_text.contains("# a comment line between blocks"));
    assert!(remaining_text.contains("trailing comment after the last block"));
    assert!(
        !remaining_text.contains("SAAwRQIgaENT3fELuW0cVfRn12MurnxK5t2R13vCDjKMaVzYL6M"),
        "cert A's own base64-encoded signature bytes must be gone, not merely cert A's PEM markers (the comment header naming cert A, which sits BEFORE cert A's own BEGIN marker, is legitimately preserved -- it was never part of the removed byte range)"
    );
}

/// Point 3 of the #1715 round-5 review: a PEM block whose lines end in
/// CRLF, not just LF, must still parse -- the round-4 parser only
/// recognized a single trailing `\n` after `END CERTIFICATE-----`.
#[test]
fn crlf_terminated_block_parses_correctly() {
    let bytes = read_artifact(&format!("{FX}/artifacts/crlf_block.pem.fixture"));
    let blocks =
        find_pem_blocks_strict(&bytes).expect("a CRLF-terminated PEM block must still parse");
    assert_eq!(blocks.len(), 1);
    assert!(
        looks_like_complete_x509_der(&blocks[0].der),
        "the CRLF block's DER must still be a valid, complete X.509 structure"
    );
    assert_eq!(
        blocks[0].range,
        (0, bytes.len()),
        "the single CRLF block must span the entire file"
    );
}

#[test]
#[should_panic(expected = "boolean schema")]
fn preflight_rejects_a_boolean_schema_rather_than_silently_mishandling_it() {
    // Per the #1715 round-5 review's small fix: a literal `false` schema
    // (which should make every instance invalid) would, with the old
    // code, silently evaluate as "no constraints" (always valid) because
    // `eval` only reads object-shaped schemas. Reject rather than mislead.
    let bad = serde_json::json!({ "anyOf": [false, { "type": "string" }] });
    let _ = minischema::Validator::new(&bad);
}

#[test]
fn reordered_bundle_has_the_same_fingerprint_multiset_but_different_byte_ranges() {
    // A reordered bundle is a legitimate negative for "order matters for
    // byte_ranges, even when the multiset is identical."
    let before = read_artifact(&format!("{FX}/artifacts/before_two_targets.pem.fixture"));
    let reordered = read_artifact(&format!("{FX}/artifacts/reordered_two_targets.pem.fixture"));
    let before_blocks = find_pem_blocks_strict(&before)
        .expect("before_two_targets.pem.fixture must parse completely");
    let reordered_blocks = find_pem_blocks_strict(&reordered)
        .expect("reordered_two_targets.pem.fixture must parse completely");

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

/// Per the #1715 round-6 review's point 2: base64 unused padding bits are
/// a SEPARATE strictness concern from DER BIT STRING unused bits (one is
/// an encoding-layer rule, RFC 4648 S3.5; the other is a DER-content-layer
/// rule). This exercises the encoding-layer rule directly, with no DER or
/// PEM involved at all.
#[test]
fn base64_decode_strict_rejects_non_canonical_unused_padding_bits() {
    // "cGxlYXN1cmU=" canonically decodes to "pleasure": the final sextet's
    // 2 unused bits (after the last full data char 'U', binary 010100)
    // are zero, as any conformant encoder produces.
    let canonical = "cGxlYXN1cmU=";
    assert_eq!(base64_decode_strict(canonical).unwrap(), b"pleasure");

    // Replacing the final data character 'U' (010100) with 'X' (010111)
    // shares the same top 4 bits -- so a permissive decoder (including
    // Python's own `base64.b64decode(..., validate=True)`) still silently
    // produces "pleasure" -- but sets the 2 unused low bits to a
    // non-zero value. This is non-canonical per RFC 4648 S3.5 and must be
    // rejected outright, not silently truncated to the same output bytes;
    // otherwise two different base64 strings could decode to the same
    // bytes, undermining this contract's "the fixture file's literal
    // bytes are what we hash" guarantee.
    let non_canonical = "cGxlYXN1cmX=";
    assert!(
        base64_decode_strict(non_canonical).is_none(),
        "non-canonical unused padding bits (RFC 4648 S3.5) must be rejected, not silently decoded"
    );
}

#[test]
fn malformed_truncated_artifact_has_no_end_marker_and_its_body_does_not_decode_to_valid_der() {
    let bytes = read_artifact(&format!("{FX}/artifacts/malformed_truncated.pem.fixture"));
    let text = String::from_utf8(bytes).unwrap();
    assert!(text.contains("-----BEGIN CERTIFICATE-----"));
    assert!(!text.contains("-----END CERTIFICATE-----"));
    assert!(
        find_pem_blocks_strict(text.as_bytes()).is_err(),
        "a block with no END marker must be rejected, not silently dropped"
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

/// Negative regression for R20 (#1715 round-6 review point 3): a report
/// claiming `cleanup.workflow_outcome: "not_attempted"` while
/// `execution.state` is `"started"` (with a step, `backup`, already
/// `completed`) is exactly the bug `stage_status_unknown_but_observed.json`
/// shipped with before this round -- a POSITIVE "nothing happened" claim
/// contradicting its own execution record. This fixture is the frozen
/// before-the-fix content of that file, kept forever as a regression
/// fixture so the bug cannot silently return.
#[test]
fn workflow_not_attempted_with_a_started_execution_is_schema_invalid() {
    let schema = load_json(REPORT_SCHEMA);
    let validator = minischema::Validator::new(&schema);
    let report = load_json(&format!(
        "{FX}/reports/invalid/workflow_not_attempted_with_started_execution.json"
    ));
    assert_eq!(report["execution"]["state"], "started");
    assert_eq!(
        report["execution"]["steps"]["backup"]["status"],
        "completed"
    );
    assert_eq!(report["cleanup"]["workflow_outcome"], "not_attempted");
    assert!(
        !validator.is_valid(&report),
        "R20 must reject workflow_outcome=not_attempted when execution.state is started (something demonstrably DID happen)"
    );
}

/// Negative regression for R20, tightened per the #1715 round-7 review's
/// point 2 ("R20 explicitly allows execution unobserved with workflow
/// not_attempted. Require execution EXACTLY not_attempted; add unobserved
/// negative"): a report claiming `cleanup.workflow_outcome: "not_attempted"`
/// while `execution.state` is `"unobserved"` (no information at all, NOT
/// positively established) must also be rejected -- round 6's R20 only
/// excluded started/completed/failed, leaving this gap open.
#[test]
fn workflow_not_attempted_with_an_unobserved_execution_is_schema_invalid() {
    let schema = load_json(REPORT_SCHEMA);
    let validator = minischema::Validator::new(&schema);
    let report = load_json(&format!(
        "{FX}/reports/invalid/workflow_not_attempted_with_unobserved_execution.json"
    ));
    assert_eq!(report["execution"]["state"], "unobserved");
    assert_eq!(report["cleanup"]["workflow_outcome"], "not_attempted");
    assert!(
        !validator.is_valid(&report),
        "R20 must reject workflow_outcome=not_attempted when execution.state is only unobserved (no information), not ESTABLISHED not_attempted"
    );
}

/// Retained dry-run state verification for R20: dry-run (and no-op)
/// reports legitimately pair `execution.state: "not_attempted"` with
/// `cleanup.workflow_outcome: "not_attempted"` (nothing ran, by design --
/// a dry run or already-satisfied no-op never executes). R20 must NOT
/// break this legitimate case while fixing the started/unobserved
/// contradictions above.
#[test]
fn dry_run_reports_retain_their_legitimate_not_attempted_pairing_under_r20() {
    let schema = load_json(REPORT_SCHEMA);
    let validator = minischema::Validator::new(&schema);
    for name in [
        "dry_run_independently_failed_verification.json",
        "dry_run_supplied_state_verified.json",
        "dry_run_unverified.json",
        "already_satisfied_no_op.json",
    ] {
        let report = load_json(&format!("{FX}/reports/valid/{name}"));
        assert_eq!(
            report["execution"]["state"], "not_attempted",
            "{name}: sanity, this fixture must actually be a dry-run/no-op (execution never started)"
        );
        assert_eq!(
            report["cleanup"]["workflow_outcome"], "not_attempted",
            "{name}: sanity, this fixture must actually claim workflow not_attempted"
        );
        assert!(
            validator.is_valid(&report),
            "{name}: R20 must not reject the legitimate not_attempted/not_attempted pairing for a genuine dry run or no-op"
        );
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
        all_of.len() >= 18,
        "expected at least 18 allOf conditionals (R1..R18 and their sub-rules), found {}",
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

// ===========================================================================
// Fixture-level case validator (plan + evidence -> derived verdict), per the
// #1715 round-4 review's point 4. This is a small, TEST-ONLY reference
// cross-check, not a preview of product behavior: it exists solely to prove
// that tampering with an evidence fixture while leaving its paired report
// fixture untouched is actually caught by this test suite. The review's
// exact example: swapping verified_removal's "final" observation to
// final_content_mismatch's bytes previously changed NOTHING any test
// checked, because nothing ever compared the final observation to the
// plan's expected_staged_hash.
// ===========================================================================

fn find_observation<'a>(evidence: &'a Value, step: &str) -> Option<&'a Value> {
    evidence["observations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|o| o["step"] == step)
}

fn find_all_observations_for_step<'a>(evidence: &'a Value, step: &str) -> Vec<&'a Value> {
    evidence["observations"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|o| o["step"] == step)
        .collect()
}

fn metadata_matches_expected(observed: &Value, expected: &Value) -> bool {
    !observed.is_null()
        && observed["owner"] == expected["owner"]
        && observed["group"] == expected["group"]
        && observed["mode"] == expected["mode"]
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct CaseVerdict {
    state: String,
    effect: String,
}

impl CaseVerdict {
    fn new(state: &str, effect: &str) -> Self {
        CaseVerdict {
            state: state.to_string(),
            effect: effect.to_string(),
        }
    }
}

/// Independently re-derive the DER fingerprint multiset of a fixture file's
/// bytes (sha256 of each PEM block's decoded DER, grouped by count). Shared
/// by the plan-level target-fingerprint check and `validate_fixture_case`'s
/// count checks, per the #1715 round-6 review's point 1 ("evidence counts
/// ... incompletely checked"): a parse failure is a hard `None`, never
/// silently treated as "zero certificates found" -- that silent-empty
/// conversion is exactly the no-op bug that review flagged.
///
/// Per the #1715 round-7 review's point 1 ("`independent_fingerprint_multiset`
/// never calls the new X.509 parser: invalid-DER no-op can become already
/// satisfied"): this now ALSO requires every block to be a complete, valid
/// X.509 structure per `looks_like_complete_x509_der`, not merely a block
/// whose base64 body decodes to SOME bytes. A block that decodes cleanly
/// from base64 but isn't a real certificate is a parse failure (`None`),
/// not a certificate with some other fingerprint.
fn independent_fingerprint_multiset(
    bytes: &[u8],
) -> Option<std::collections::BTreeMap<String, u64>> {
    let blocks = find_pem_blocks_strict(bytes).ok()?;
    let mut counts = std::collections::BTreeMap::new();
    for b in &blocks {
        if !looks_like_complete_x509_der(&b.der) {
            return None;
        }
        *counts.entry(sha256_hex(&b.der)).or_insert(0) += 1;
    }
    Some(counts)
}

fn declared_fingerprint_multiset(fingerprints: &Value) -> std::collections::BTreeMap<String, u64> {
    fingerprints
        .as_array()
        .map(|arr| {
            arr.iter()
                .map(|f| {
                    (
                        f["sha256"].as_str().unwrap().to_string(),
                        f["occurrence_count"].as_u64().unwrap(),
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Independently resolve an observation's content claim: `Ok(None)` when
/// `content_hash` is null (no content claim at all -- MISSING, not
/// contradictory); `Err(())` when `content_hash` is present but either no
/// real bytes back it (`artifact_ref` missing) or the REAL bytes at that
/// path do not hash to the declared value (a declared-but-wrong digest --
/// a genuine contradiction, never silently trusted); `Ok(Some(bytes))`
/// when the declared hash is independently confirmed against real bytes.
/// Per the #1715 round-7 review's point 1 ("omits existing artifact
/// hashing"): every content-bearing gate in `validate_fixture_case` below
/// routes through this, so a tampered `content_hash` that happens to
/// equal some OTHER declared value can never pass by matching the wrong
/// thing -- it is checked against the REAL bytes, not against another
/// field.
/// Per the #1715 round-9 review's point 3 ("Content resolver validates
/// content_hash but IGNORES artifact_ref.sha256 and size_bytes"): an
/// `artifact_ref` with its own `sha256`/`size_bytes` fields that disagree
/// with the real bytes (even though the OBSERVATION's own `content_hash`
/// happens to match those real bytes) is an internally inconsistent
/// evidence bundle -- the artifact reference itself is a declared claim
/// about the bytes, independently checked here, not merely a path to
/// fetch them from.
fn independently_resolve_content(obs: &Value) -> Result<Option<Vec<u8>>, ()> {
    let Some(declared_hash) = obs["content_hash"].as_str() else {
        return Ok(None);
    };
    let artifact_ref = &obs["artifact_ref"];
    let Some(path) = artifact_ref["fixture_path"].as_str() else {
        return Err(());
    };
    let bytes = read_artifact(path);
    let real_hash = sha256_hex(&bytes);
    if real_hash != declared_hash {
        return Err(());
    }
    if artifact_ref["sha256"].as_str() != Some(declared_hash) {
        return Err(());
    }
    if artifact_ref["size_bytes"].as_u64() != Some(bytes.len() as u64) {
        return Err(());
    }
    Ok(Some(bytes))
}

/// Independently resolve a PLAN-level `artifact_ref` (`before_artifact_ref`
/// / `expected_staged_artifact_ref`, both schema-required, never
/// nullable, unlike an evidence observation's `artifact_ref`) against
/// real bytes read from its OWN `fixture_path`, cross-checked against
/// the plan's own paired hash field (`before_hash` / `expected_staged_hash`).
/// Per the #1715 round-10 review's point 1: this is the step that was
/// missing entirely -- prior rounds only ever compared the plan's
/// DECLARED `artifact_ref` fields against each other or against
/// already-resolved EVIDENCE bytes, never against real bytes read from
/// the plan's own path. Called from the shared checker BEFORE either
/// the no-op or mutation branch, so a swapped `fixture_path` (pointing
/// at different, real, existing bytes) or a tampered `sha256`/
/// `size_bytes` field is caught regardless of which branch would
/// otherwise run.
fn resolve_plan_artifact_ref(artifact_ref: &Value, expected_hash: &str) -> Result<Vec<u8>, ()> {
    let Some(path) = artifact_ref["fixture_path"].as_str() else {
        return Err(());
    };
    let bytes = read_artifact(path);
    let real_hash = sha256_hex(&bytes);
    if real_hash != expected_hash {
        return Err(());
    }
    if artifact_ref["sha256"].as_str() != Some(expected_hash) {
        return Err(());
    }
    if artifact_ref["size_bytes"].as_u64() != Some(bytes.len() as u64) {
        return Err(());
    }
    Ok(bytes)
}

/// Per the #1715 round-7 review's point 1 ("omits ... destination/context
/// ... checks"): an observation claiming a different destination or
/// context than the plan it's bound to is a STALE observation (from a
/// different operation entirely), not a missing one -- a contradiction,
/// not an absence.
fn observation_binding_matches_plan(obs: &Value, plan: &Value) -> bool {
    obs["normalized_destination_binding"] == plan["normalized_destination"]
        && obs["context_binding"] == plan["context_identity"]
}

/// Independently apply `plan.approved_removal_ranges`' declared byte
/// ranges to `before_bytes` (delete each range, in sorted order) and
/// return the remainder. Returns `None` on any malformed/overlapping/
/// out-of-bounds range, OR on any binding failure between the plan's
/// DECLARED fingerprints/counts/ranges and the REAL, independently
/// re-parsed before bytes -- a malformed or dishonest plan must fail the
/// case, never panic and never silently apply unverified ranges.
///
/// Per the #1715 round-7 review's point 1 ("omits ... range
/// transformation ... checks"), this ties `before` to `stage` through
/// the plan's OWN declared transformation. Per the #1715 round-8
/// review's point 1 ("It applies ranges without binding declared
/// certificate fingerprint, occurrence count, or actual certificate
/// boundaries ... changing the approved fingerprint to an unrelated
/// value ... still verifies removal"): round 7's version used ONLY the
/// declared `start`/`end` byte offsets to slice `before_bytes` -- it
/// never checked that a `byte_ranges` entry's position actually
/// corresponds to a real, independently-parsed certificate boundary, or
/// that the DER there actually hashes to the entry's own declared
/// `target_sha256`, or that the declared `count` matches either the
/// number of `byte_ranges` entries OR the real occurrence count in the
/// before bytes, or that `plan.target_fingerprints` (the plan's
/// declared FULL multiset) matches the independently re-parsed FULL
/// multiset of the real before bytes at all. A plan could therefore
/// relabel `approved_removal_ranges[i].target_sha256` to an unrelated
/// value, recompute the scope-binding digest over the (now-tampered)
/// plan, and still pass -- because nothing ever cross-checked the label
/// against the real bytes at that position. All of that binding now
/// happens here, before any byte is ever removed.
fn bind_and_apply_approved_removal_ranges(before_bytes: &[u8], plan: &Value) -> Option<Vec<u8>> {
    // Full, independently-parsed multiset of the REAL before bytes --
    // every block must be a complete, valid X.509 structure (not merely
    // base64-decodable), and the plan's own declared `target_fingerprints`
    // must match this multiset EXACTLY, not just contain a subset.
    let blocks = find_pem_blocks_strict(before_bytes).ok()?;
    let mut block_at: std::collections::BTreeMap<(usize, usize), String> =
        std::collections::BTreeMap::new();
    let mut independent_counts: std::collections::BTreeMap<String, u64> =
        std::collections::BTreeMap::new();
    for b in &blocks {
        if !looks_like_complete_x509_der(&b.der) {
            return None;
        }
        let fp = sha256_hex(&b.der);
        block_at.insert(b.range, fp.clone());
        *independent_counts.entry(fp).or_insert(0) += 1;
    }
    let declared_target_counts = declared_fingerprint_multiset(&plan["target_fingerprints"]);
    if declared_target_counts != independent_counts {
        return None;
    }

    let mut ranges: Vec<(usize, usize)> = Vec::new();
    for range in plan["approved_removal_ranges"].as_array()? {
        let target_sha = range["target_sha256"].as_str()?;
        let declared_count = range["count"].as_u64()?;
        let byte_ranges = range["byte_ranges"].as_array()?;
        // The declared count must match BOTH the number of byte_ranges
        // entries AND the REAL occurrence count of this fingerprint in
        // the before bytes.
        if byte_ranges.len() as u64 != declared_count {
            return None;
        }
        if independent_counts.get(target_sha).copied().unwrap_or(0) != declared_count {
            return None;
        }
        for br in byte_ranges {
            let start = br["start"].as_u64()? as usize;
            let end = br["end"].as_u64()? as usize;
            if start > end || end > before_bytes.len() {
                return None;
            }
            // The declared byte_range must land on an ACTUAL,
            // independently-parsed certificate boundary, and the DER
            // there must actually hash to the declared target_sha256 --
            // not merely "some bytes at this offset."
            match block_at.get(&(start, end)) {
                Some(real_fp) if real_fp == target_sha => {}
                _ => return None,
            }
            ranges.push((start, end));
        }
    }
    ranges.sort_by_key(|(start, _)| *start);
    let mut cursor = 0usize;
    let mut kept = Vec::new();
    for (start, end) in &ranges {
        if *start < cursor {
            return None; // overlapping or unsorted
        }
        kept.extend_from_slice(&before_bytes[cursor..*start]);
        cursor = *end;
    }
    kept.extend_from_slice(&before_bytes[cursor..]);
    Some(kept)
}

/// A step_status in the failing family -- shared by both directions of
/// `report_steps_consistent_with_evidence` below.
const FAILING_STEP_STATUSES: &[&str] = &["failed", "timed_out", "signaled", "spawn_failed"];

/// Per the #1715 round-6 review's point 1 ("positive report claims filter
/// completed without filter evidence"), round-7's point 1 ("Report
/// checks omit cleanup/source agreement"), and round-8's point 2
/// ("Evidence-to-report propagation, not only report-to-evidence
/// claims"): this is now BIDIRECTIONAL.
///
/// Direction A (report -> evidence, rounds 6/7): for every execution
/// step the REPORT claims with a sourced (`observation_source != "none"`)
/// status, there must be a matching evidence observation for that step
/// with the SAME `step_status` AND the SAME `source` -- a report cannot
/// claim more than the evidence bundle actually backs, and cannot
/// misattribute WHO backed it either.
///
/// Direction B (evidence -> report, round 8): if the EVIDENCE itself
/// shows a mutation step (`backup`/`stage`/`filter`/`install`) in the
/// failing family, the REPORT cannot hide that behind
/// `unobserved`/`observation_source: "none"` while claiming
/// `execution.state`/`cleanup.workflow_outcome` completed -- a known
/// mutation failure MUST propagate upward into the report, not merely
/// never be contradicted by it. `cleanup` gets the same evidence->report
/// propagation, but deliberately does NOT force `execution.state`/
/// `workflow_outcome` to `failed` -- a failed cleanup step must become
/// VISIBLE in `cleanup.state` without erasing an otherwise independently
/// verified final mutation state (the same decoupling R12/R13 and the
/// `cleanup_failure_preserves_verified_state.json` fixture already
/// establish at the schema level).
fn report_steps_consistent_with_evidence(evidence: &Value, report: &Value) -> bool {
    for step in ["backup", "stage", "filter", "install"] {
        let step_report = &report["execution"]["steps"][step];
        let report_source = step_report["observation_source"].as_str().unwrap_or("none");
        let report_status = step_report["status"].as_str().unwrap_or("");
        let evidence_obs = find_observation(evidence, step);

        // Direction A: the report claims more than the evidence backs.
        if !step_report.is_null() && report_source != "none" {
            match evidence_obs {
                None => return false,
                Some(obs) => {
                    if obs["step_status"].as_str().unwrap_or("") != report_status {
                        return false;
                    }
                    if obs["source"].as_str().unwrap_or("") != report_source {
                        return false;
                    }
                }
            }
        }

        // Direction B: the evidence shows a real mutation-step failure
        // the report hides or contradicts.
        if let Some(obs) = evidence_obs {
            let obs_status = obs["step_status"].as_str().unwrap_or("");
            if FAILING_STEP_STATUSES.contains(&obs_status) {
                if !FAILING_STEP_STATUSES.contains(&report_status) {
                    return false; // report doesn't acknowledge the known failure
                }
                if report["execution"]["state"] != "failed" {
                    return false;
                }
                if report["cleanup"]["workflow_outcome"] != "failed" {
                    return false;
                }
            }
        }
    }

    let cleanup_source = report["cleanup"]["observation_source"]
        .as_str()
        .unwrap_or("none");
    let cleanup_obs = find_observation(evidence, "cleanup");
    // Direction A for cleanup.
    if cleanup_source != "none" {
        match cleanup_obs {
            None => return false,
            Some(obs) => {
                if obs["source"].as_str().unwrap_or("") != cleanup_source {
                    return false;
                }
                let obs_status = obs["step_status"].as_str().unwrap_or("");
                let agrees = match report["cleanup"]["state"].as_str().unwrap_or("") {
                    "succeeded" => obs_status == "completed",
                    "failed" => FAILING_STEP_STATUSES.contains(&obs_status),
                    "not_attempted" => obs_status == "not_attempted",
                    _ => false,
                };
                if !agrees {
                    return false;
                }
            }
        }
    }
    // Direction B for cleanup: a real evidence cleanup failure must be
    // VISIBLE in the report's cleanup.state AND force
    // cleanup.workflow_outcome=failed (R12) -- but (deliberately, unlike
    // the mutation steps above) does NOT force execution.state to
    // failed, and does not by itself erase an independently verified
    // mutation outcome (see `validate_fixture_case`'s own
    // `any_mutation_step_failed` check, which cleanup is excluded from).
    // "Workflow failed" here means the report's workflow_outcome field,
    // never the mutation's own effect.
    if let Some(obs) = cleanup_obs {
        let obs_status = obs["step_status"].as_str().unwrap_or("");
        if FAILING_STEP_STATUSES.contains(&obs_status) {
            if report["cleanup"]["state"] != "failed" {
                return false;
            }
            if report["cleanup"]["workflow_outcome"] != "failed" {
                return false;
            }
        }
    }
    true
}

/// A `verified` semantic verdict is only as trustworthy as the report that
/// will actually ship it: if the paired report claims more than the
/// evidence backs for any step (`report_steps_consistent_with_evidence`),
/// the case cannot be asserted `verified` -- demote to `unverified`. A
/// non-`verified` verdict passes through unchanged: the semantic gates
/// already found a concrete problem, and the report can't make that
/// worse or better.
fn finalize_against_report(verdict: CaseVerdict, evidence: &Value, report: &Value) -> CaseVerdict {
    if verdict.state == "verified" && !report_steps_consistent_with_evidence(evidence, report) {
        return CaseVerdict::new("unverified", "unknown");
    }
    verdict
}

/// THE single test-only fixture-integrity path, per the #1715 round-7
/// review's point 1 ("Complete AND ACTUALLY USE the shared test-only
/// fixture-integrity path ... Consolidate checks into ONE entry point and
/// route EVERY positive/adversarial paired case through it"). Round 6's
/// split between a narrower `validate_case` (plan+evidence only) and a
/// wrapping `validate_full_case` (plan+evidence+report) meant almost every
/// adversarial test called the narrower function directly, bypassing the
/// report/schema gates entirely -- THAT SPLIT NO LONGER EXISTS. Every test
/// in this suite that exercises a plan/evidence/report triad calls this
/// one function; there is no bypass path left.
///
/// Classification discipline (the round-7 review's closing instruction):
/// a field that is MISSING (null, or its observation doesn't exist at all,
/// or a real id it must bind to is itself null) can only ever derive
/// `unverified` -- we don't have the information needed to make a claim
/// either way. A field that IS PRESENT and CONTRADICTS what the plan/
/// schema/real-bytes require derives `failed`. Only a fully consistent
/// chain of PRESENT, independently re-verified evidence can derive
/// `verified`.
///
/// Gates, in order: (1) schema validity of all three documents; (2)
/// plan<->evidence operation binding; (3) same-step conflicts, even across
/// differing snapshot ids; (4) destination/context binding for every
/// observation present; (5) the no-op path (independent before-artifact
/// re-parse, including full X.509 parsing of every block, never just
/// base64 decoding); (6) a REQUIRED, independently-resolved `before`
/// observation for the mutating path (closes the round-7 review's named
/// false positive: a missing `before` paired with a null
/// `consumes_before_snapshot_id` can no longer "vacuously match", because
/// gate 6 returns `unverified` before gate 11 ever compares them); (7)
/// backup (when required); (8) stage, independently rehashed; (9) range
/// transformation -- applying the plan's OWN declared byte ranges to the
/// independently-rehashed `before` bytes must actually PRODUCE the
/// independently-rehashed `stage` bytes; (10) final (rehashed, parsed,
/// metadata, independently re-derived certificate counts); (11) install,
/// with NON-NULL exact-match snapshot bindings and freshness (install
/// must be captured strictly after what it claims to consume); (12) the
/// scoped-approval digest, when authorization claims `confirmed_scoped`;
/// (13) the report-consistency demotion. Still deliberately bounded --
/// this is fixture validation, not production verifier development.
fn validate_fixture_case(plan: &Value, evidence: &Value, report: &Value) -> CaseVerdict {
    // Gate 1: schema validity of every document, not just evidence.
    let plan_schema = load_json(PLAN_SCHEMA);
    let evidence_schema = load_json(EVIDENCE_SCHEMA);
    let report_schema = load_json(REPORT_SCHEMA);
    if !minischema::Validator::new(&plan_schema).is_valid(plan)
        || !minischema::Validator::new(&evidence_schema).is_valid(evidence)
        || !minischema::Validator::new(&report_schema).is_valid(report)
    {
        return CaseVerdict::new("unverified", "unknown");
    }

    // Gate 2: plan<->evidence operation binding.
    if plan["operation_id"] != evidence["operation_id"]
        || plan["evidence_id"] != evidence["evidence_id"]
    {
        return CaseVerdict::new("failed", "unknown");
    }

    // Gate 3: same-step conflicts, even across differing snapshot ids.
    // Per the #1715 round-8 review's point 2 ("Reject competing snapshots
    // differing in metadata/provenance as well as hash/status"): two
    // observations for the same step that agree on hash and status but
    // disagree on WHO backed them (`source`) or WHAT metadata they
    // carry are still competing, inconsistent claims about the same
    // step.
    for step in [
        "before", "backup", "stage", "filter", "install", "final", "cleanup",
    ] {
        let obs = find_all_observations_for_step(evidence, step);
        if obs.len() > 1 {
            let first = obs[0];
            let conflicting = obs.iter().any(|o| {
                o["content_hash"] != first["content_hash"]
                    || o["step_status"] != first["step_status"]
                    || o["metadata"] != first["metadata"]
                    || o["source"] != first["source"]
            });
            if conflicting {
                return CaseVerdict::new("unverified", "unknown");
            }
        }
    }

    // Gate 4: destination/context binding for every observation present.
    for step in [
        "before", "backup", "stage", "filter", "install", "final", "cleanup",
    ] {
        if let Some(obs) = find_observation(evidence, step) {
            if !observation_binding_matches_plan(obs, plan) {
                return CaseVerdict::new("failed", "unknown"); // stale_observation
            }
        }
    }

    // Gate 4.5: resolve EVERY plan-level artifact_ref (`before_artifact_ref`,
    // `expected_staged_artifact_ref`) independently against real bytes on
    // disk, cross-checked against the plan's own paired hash field --
    // BEFORE either the no-op or the mutation branch below ever trusts
    // them. Per the #1715 round-10 review's point 1: round 9's checks
    // only ever compared the PLAN's declared `artifact_ref` fields
    // against each other (`before_artifact_ref.sha256 == before_hash`)
    // or against the EVIDENCE's own already-resolved bytes -- never
    // against real bytes read from the PLAN's own `fixture_path`. That
    // meant: the no-op branch never looked at `before_artifact_ref` at
    // all (so a tampered `size_bytes` there still verified
    // already_satisfied); the mutation branch never actually opened
    // `before_artifact_ref.fixture_path` (so swapping it to a
    // DIFFERENT, real fixture file -- while leaving the declared
    // sha256/size_bytes fields untouched -- still verified removed,
    // since nothing ever re-read that path); and
    // `expected_staged_artifact_ref` was never checked at all, by
    // either branch.
    let plan_before_hash = plan["before_hash"].as_str().unwrap_or("");
    if resolve_plan_artifact_ref(&plan["before_artifact_ref"], plan_before_hash).is_err() {
        return CaseVerdict::new("failed", "unknown");
    }
    let plan_staged_hash = plan["expected_staged_hash"].as_str().unwrap_or("");
    if resolve_plan_artifact_ref(&plan["expected_staged_artifact_ref"], plan_staged_hash).is_err() {
        return CaseVerdict::new("failed", "unknown");
    }

    // Gate 5: the no-op path.
    if plan_is_noop(plan) {
        // Per the #1715 round-9 review's point 3 ("No-op rejects nonempty
        // removal ranges with ZERO approved counts"): `plan_is_noop`
        // classifies a plan as no-op purely from `count == 0` on every
        // range entry, but says nothing about whether that entry's own
        // `byte_ranges` array is also empty. A range entry claiming
        // count=0 while still declaring one or more non-empty
        // `byte_ranges` is internally inconsistent -- a plan cannot
        // simultaneously claim "nothing to remove" and "here are bytes to
        // remove" for the same target.
        let ranges_internally_consistent = plan["approved_removal_ranges"]
            .as_array()
            .unwrap()
            .iter()
            .all(|r| {
                r["byte_ranges"]
                    .as_array()
                    .map(|br| br.len() as u64)
                    .unwrap_or(0)
                    == r["count"].as_u64().unwrap_or(0)
            });
        if !ranges_internally_consistent {
            return finalize_against_report(
                CaseVerdict::new("failed", "unknown"),
                evidence,
                report,
            );
        }
        let verdict = match find_observation(evidence, "before") {
            None => CaseVerdict::new("unverified", "unknown"),
            Some(before) => match independently_resolve_content(before) {
                Err(()) => CaseVerdict::new("failed", "unknown"),
                Ok(None) => CaseVerdict::new("unverified", "unknown"),
                Ok(Some(before_bytes)) => {
                    if sha256_hex(&before_bytes) != plan["before_hash"].as_str().unwrap_or("") {
                        CaseVerdict::new("failed", "unknown")
                    } else {
                        match independent_fingerprint_multiset(&before_bytes) {
                            None => CaseVerdict::new("failed", "unknown"),
                            Some(present_counts) => {
                                let present: BTreeSet<String> =
                                    present_counts.into_keys().collect();
                                let plan_contradicted = plan["approved_removal_ranges"]
                                    .as_array()
                                    .unwrap()
                                    .iter()
                                    .any(|r| {
                                        present.contains(r["target_sha256"].as_str().unwrap())
                                    });
                                if plan_contradicted {
                                    CaseVerdict::new("failed", "unknown")
                                } else {
                                    CaseVerdict::new("verified", "already_satisfied")
                                }
                            }
                        }
                    }
                }
            },
        };
        return finalize_against_report(verdict, evidence, report);
    }

    // Gate 6: a REQUIRED, independently-resolved `before` observation.
    // This is the fix for the round-7 review's named false positive: with
    // `before` required here, a missing `before` observation can never
    // reach gate 11's binding comparison in the first place.
    let before_obs = match find_observation(evidence, "before") {
        None => return CaseVerdict::new("unverified", "unknown"),
        Some(b) => b,
    };
    let before_bytes = match independently_resolve_content(before_obs) {
        Err(()) => return CaseVerdict::new("failed", "unknown"),
        Ok(None) => return CaseVerdict::new("unverified", "unknown"),
        Ok(Some(bytes)) => bytes,
    };
    // Per the #1715 round-8 review's point 1 ("Before bytes are not
    // compared with plan.before_hash"): the evidence observation's OWN
    // declared hash was already independently confirmed against its own
    // real bytes above, but that says nothing about whether those bytes
    // are the ones the PLAN itself claims to be operating on. Bind them.
    // (`plan.before_artifact_ref` itself was already independently
    // resolved against real bytes on disk in gate 4.5, above.)
    if sha256_hex(&before_bytes) != plan_before_hash {
        return CaseVerdict::new("failed", "unknown");
    }

    // Gate 7: backup, when the plan requires one.
    if plan["backup_policy"]["required"] == true {
        match find_observation(evidence, "backup") {
            None => return CaseVerdict::new("unverified", "unknown"),
            Some(backup) => match independently_resolve_content(backup) {
                Err(()) => return CaseVerdict::new("failed", "unknown"),
                Ok(None) => return CaseVerdict::new("unverified", "unknown"),
                Ok(Some(bytes)) => {
                    if sha256_hex(&bytes) != plan["before_hash"].as_str().unwrap_or("") {
                        return CaseVerdict::new("failed", "unknown");
                    }
                    if backup["metadata"].is_null() {
                        return CaseVerdict::new("unverified", "unknown");
                    }
                    if !metadata_matches_expected(&backup["metadata"], &plan["expected_metadata"]) {
                        return CaseVerdict::new("failed", "unknown");
                    }
                    match backup["backup_readability"].as_str() {
                        Some("readable") => {}
                        Some("unreadable") => return CaseVerdict::new("failed", "unknown"),
                        _ => return CaseVerdict::new("unverified", "unknown"), // null/unobserved
                    }
                }
            },
        }
    }

    // Gate 8: stage, independently rehashed.
    let (stage_bytes, stage_obs) = match find_observation(evidence, "stage") {
        None => return CaseVerdict::new("unverified", "unknown"),
        Some(stage) => match independently_resolve_content(stage) {
            Err(()) => return CaseVerdict::new("failed", "unknown"),
            Ok(None) => return CaseVerdict::new("unverified", "unknown"),
            Ok(Some(bytes)) => {
                if sha256_hex(&bytes) != plan["expected_staged_hash"].as_str().unwrap_or("") {
                    return CaseVerdict::new("failed", "unknown");
                }
                (bytes, stage)
            }
        },
    };

    // Gate 9: range transformation -- applying the plan's OWN declared
    // byte ranges to the independently-rehashed `before` bytes must
    // actually PRODUCE the independently-rehashed `stage` bytes, with
    // every declared fingerprint/count/boundary bound to the real bytes
    // (see `bind_and_apply_approved_removal_ranges`).
    match bind_and_apply_approved_removal_ranges(&before_bytes, plan) {
        None => return CaseVerdict::new("failed", "unknown"),
        Some(transformed) => {
            if transformed != stage_bytes {
                return CaseVerdict::new("failed", "unknown");
            }
        }
    }

    // Gate 10: final -- rehashed, parsed, metadata, independently
    // re-derived certificate counts.
    let final_obs = match find_observation(evidence, "final") {
        None => return CaseVerdict::new("unverified", "unknown"),
        Some(f) => f,
    };
    let final_bytes = match independently_resolve_content(final_obs) {
        Err(()) => return CaseVerdict::new("failed", "unknown"),
        Ok(None) => return CaseVerdict::new("unverified", "unknown"),
        Ok(Some(bytes)) => bytes,
    };
    if sha256_hex(&final_bytes) != plan["expected_staged_hash"].as_str().unwrap_or("") {
        return CaseVerdict::new("failed", "unknown");
    }
    // Per the #1715 round-8 review's point 3 ("Explicitly classify
    // not_applicable parse outcomes ... as UNAVAILABLE versus
    // contradictory"): `not_applicable` means parsing was never
    // attempted/doesn't apply here -- that's MISSING information, not a
    // demonstrated contradiction. Only malformed/truncated/
    // unconsumed_blocks mean a parse WAS attempted and genuinely failed.
    match final_obs["parse_outcome"].as_str() {
        Some("ok") => {}
        Some("not_applicable") => return CaseVerdict::new("unverified", "unknown"),
        _ => return CaseVerdict::new("failed", "unknown"),
    }
    // Per the round-7 review's point 1 ("Missing final metadata/hash is
    // incorrectly FAILED rather than UNVERIFIED"): metadata that is simply
    // absent (null) is MISSING information, not a contradiction --
    // unverified, not failed. Only metadata that IS present and wrong
    // fails.
    if final_obs["metadata"].is_null() {
        return CaseVerdict::new("unverified", "unknown");
    }
    if !metadata_matches_expected(&final_obs["metadata"], &plan["expected_metadata"]) {
        return CaseVerdict::new("failed", "unknown");
    }
    // Per the round-8 review's point 3 ("null fingerprint declarations
    // ... UNAVAILABLE versus ... an asserted empty multiset"): a null
    // `certificate_fingerprints` means no declared count was ever
    // supplied -- that must be unverified, not silently treated as "an
    // asserted multiset of zero certificates" and compared (and likely
    // failed) against the real, non-empty bytes.
    if final_obs["certificate_fingerprints"].is_null() {
        return CaseVerdict::new("unverified", "unknown");
    }
    let declared_counts = declared_fingerprint_multiset(&final_obs["certificate_fingerprints"]);
    match independent_fingerprint_multiset(&final_bytes) {
        None => return CaseVerdict::new("failed", "unknown"),
        Some(independent_counts) => {
            if independent_counts != declared_counts {
                return CaseVerdict::new("failed", "unknown");
            }
        }
    }

    // Gate 11: install, with NON-NULL exact-match snapshot bindings and
    // freshness.
    let install = match find_observation(evidence, "install") {
        None => return CaseVerdict::new("unverified", "unknown"),
        Some(i) => i,
    };
    let before_snapshot_id = before_obs["step_snapshot_id"].clone();
    let stage_snapshot_id = stage_obs["step_snapshot_id"].clone();
    if !before_snapshot_id.is_string() || !stage_snapshot_id.is_string() {
        // A malformed bundle with no real id to bind against -- missing
        // information, not a demonstrated contradiction.
        return CaseVerdict::new("unverified", "unknown");
    }
    // Per the #1715 round-9 review's point 1 ("Null consumed before/
    // staging snapshot IDs must be UNVERIFIED/UNKNOWN even when a
    // snapshot exists. Existence does not prove which snapshot install
    // consumed."): install's OWN claim being null is a MISSING claim --
    // we don't know which snapshot (if any) it actually consumed, which
    // is different from install NAMING a specific, wrong snapshot (a
    // demonstrated contradiction). The round-8 code conflated these by
    // comparing null directly against the real id and treating the
    // resulting inequality as a contradiction (`failed`); null must be
    // checked FIRST and separately, as `unverified`.
    let install_before_claim = &install["consumes_before_snapshot_id"];
    let install_stage_claim = &install["consumes_staging_snapshot_id"];
    if install_before_claim.is_null() || install_stage_claim.is_null() {
        return CaseVerdict::new("unverified", "unknown");
    }
    if *install_before_claim != before_snapshot_id || *install_stage_claim != stage_snapshot_id {
        return CaseVerdict::new("failed", "unknown");
    }
    let before_seq = before_obs["capture_sequence"].as_i64().unwrap_or(-1);
    let stage_seq = stage_obs["capture_sequence"].as_i64().unwrap_or(-1);
    let install_seq = install["capture_sequence"].as_i64().unwrap_or(-1);
    if install_seq <= before_seq || install_seq <= stage_seq {
        // install claims to have consumed a snapshot captured AFTER
        // install itself -- a stale/impossible ordering.
        return CaseVerdict::new("failed", "unknown");
    }

    // Per the #1715 round-9 review's point 2 ("ANY MUTATION-STEP FAILURE
    // must retain effect UNKNOWN"): round 8 only ever consulted
    // `install["step_status"]` when deciding the effect -- a truthfully
    // reported `backup`/`stage`/`filter` failure, even with a completed
    // install and an independently verified final state, still produced
    // effect=removed, because nothing here ever looked at those steps'
    // own `step_status`. `cleanup` is deliberately excluded: a
    // cleanup-only failure may still retain effect=removed (R12/R13's
    // existing decoupling), it just forces `workflow_outcome=failed` at
    // the report level (enforced by R12 and by
    // `report_steps_consistent_with_evidence`'s cleanup propagation).
    let any_mutation_step_failed = ["backup", "stage", "filter"].iter().any(|step| {
        find_observation(evidence, step)
            .map(|o| FAILING_STEP_STATUSES.contains(&o["step_status"].as_str().unwrap_or("")))
            .unwrap_or(false)
    });
    let mutation_verdict = if install["step_status"] == "completed" && !any_mutation_step_failed {
        CaseVerdict::new("verified", "removed")
    } else {
        CaseVerdict::new("verified", "unknown")
    };

    // Gate 12: the scoped-approval digest, when authorization claims
    // confirmed_scoped (a report-level claim, independently recomputed
    // from the plan via the SAME versioned preimage
    // `confirmed_scoped_digest_must_match_the_versioned_preimage` checks
    // in isolation -- here it is wired into the one path, not left as a
    // separate, easy-to-bypass test).
    if report["authorization"]["state"] == "confirmed_scoped" {
        let expected_digest = compute_scope_binding_digest_v1(plan);
        if report["authorization"]["scope_binding_digest"].as_str()
            != Some(expected_digest.as_str())
        {
            return CaseVerdict::new("failed", "unknown");
        }
    }

    finalize_against_report(mutation_verdict, evidence, report)
}

fn plan_is_noop(plan: &Value) -> bool {
    plan["approved_removal_ranges"]
        .as_array()
        .unwrap()
        .iter()
        .all(|r| r["count"].as_u64().unwrap() == 0)
}

/// Derive (postcondition.state, verification_basis, effect) from a bound
/// (plan, evidence) pair, independently of whatever a report fixture
/// claims. Deliberately narrow: it implements exactly the before/final
/// byte-binding checks named in the review, not the full normative outcome
/// table (building that would be the actual behavioral implementation this
/// checkpoint is not allowed to ship).
fn derive_postcondition(plan: &Value, evidence: &Value) -> (String, String, String) {
    if plan_is_noop(plan) {
        return match find_observation(evidence, "before") {
            Some(before) if before["content_hash"] == plan["before_hash"] => (
                "verified".to_string(),
                "supplied_evidence".to_string(),
                "already_satisfied".to_string(),
            ),
            Some(_) => (
                "failed".to_string(),
                "supplied_evidence".to_string(),
                "unknown".to_string(),
            ),
            None => (
                "unverified".to_string(),
                "none".to_string(),
                "unknown".to_string(),
            ),
        };
    }
    match find_observation(evidence, "final") {
        None => (
            "unverified".to_string(),
            "none".to_string(),
            "unknown".to_string(),
        ),
        Some(final_obs) => {
            if final_obs["content_hash"] == plan["expected_staged_hash"] {
                (
                    "verified".to_string(),
                    "supplied_evidence".to_string(),
                    "removed".to_string(),
                )
            } else {
                (
                    "failed".to_string(),
                    "supplied_evidence".to_string(),
                    "unknown".to_string(),
                )
            }
        }
    }
}

#[test]
fn derived_postcondition_matches_the_actual_report_fixture_for_the_happy_path() {
    let plan = load_json(&format!(
        "{FX}/plans/valid/mutating_removal_with_backup_required.json"
    ));
    let evidence = load_json(&format!("{FX}/evidence/valid/verified_removal.json"));
    let report = load_json(&format!("{FX}/reports/valid/verified_removal.json"));
    let (state, basis, effect) = derive_postcondition(&plan, &evidence);
    assert_eq!(state, report["postcondition"]["state"]);
    assert_eq!(basis, report["postcondition"]["verification_basis"]);
    assert_eq!(effect, report["postcondition"]["effect"]);
}

#[test]
fn derived_postcondition_matches_the_actual_report_fixture_for_the_noop_case() {
    let plan = load_json(&format!("{FX}/plans/valid/no_op_targets_absent.json"));
    let evidence = load_json(&format!("{FX}/evidence/valid/already_satisfied_no_op.json"));
    let report = load_json(&format!("{FX}/reports/valid/already_satisfied_no_op.json"));
    let (state, basis, effect) = derive_postcondition(&plan, &evidence);
    assert_eq!(state, report["postcondition"]["state"]);
    assert_eq!(basis, report["postcondition"]["verification_basis"]);
    assert_eq!(effect, report["postcondition"]["effect"]);
}

/// THE fix for the #1715 round-4 review's point 4 headline bug: swap the
/// evidence bundle's "final" observation to point at mismatched bytes
/// (final_content_mismatch.pem.fixture) while changing nothing else, and
/// prove the derived postcondition actually flips to failed. Before this
/// test existed, nothing in this suite compared the final observation to
/// plan.expected_staged_hash at all, so this exact tamper was invisible.
#[test]
fn tampering_the_final_observation_flips_the_derived_postcondition_to_failed() {
    let plan = load_json(&format!(
        "{FX}/plans/valid/mutating_removal_with_backup_required.json"
    ));
    let mut evidence = load_json(&format!("{FX}/evidence/valid/verified_removal.json"));

    let mismatch_path =
        "tests/fixtures/trust_operation_v1/artifacts/final_content_mismatch.pem.fixture";
    let mismatch_bytes = read_artifact(mismatch_path);
    let mismatch_hash = sha256_hex(&mismatch_bytes);
    assert_ne!(
        mismatch_hash, plan["expected_staged_hash"],
        "sanity: the mismatch fixture must not accidentally equal expected_staged_hash"
    );

    let observations = evidence["observations"].as_array_mut().unwrap();
    let final_obs = observations
        .iter_mut()
        .find(|o| o["step"] == "final")
        .expect("verified_removal.json must have a final observation");
    final_obs["content_hash"] = Value::String(mismatch_hash.clone());
    final_obs["artifact_ref"] = serde_json::json!({
        "sha256": mismatch_hash,
        "size_bytes": mismatch_bytes.len(),
        "source": "fixture_file",
        "fixture_path": mismatch_path,
    });

    let (state, _basis, effect) = derive_postcondition(&plan, &evidence);
    assert_eq!(state, "failed", "swapping the final observation to mismatched bytes must flip the derived postcondition to failed, not stay verified");
    assert_ne!(
        effect, "removed",
        "a failed postcondition must never still claim effect=removed"
    );
}

#[test]
fn missing_final_observation_derives_to_unverified_not_verified() {
    let plan = load_json(&format!(
        "{FX}/plans/valid/mutating_removal_with_backup_required.json"
    ));
    let mut evidence = load_json(&format!("{FX}/evidence/valid/verified_removal.json"));
    let observations = evidence["observations"].as_array_mut().unwrap();
    observations.retain(|o| o["step"] != "final");
    let (state, basis, _effect) = derive_postcondition(&plan, &evidence);
    assert_eq!(state, "unverified");
    assert_eq!(basis, "none");
}

#[test]
fn a_stale_before_observation_derives_the_noop_case_to_failed_not_already_satisfied() {
    let plan = load_json(&format!("{FX}/plans/valid/no_op_targets_absent.json"));
    let mut evidence = load_json(&format!("{FX}/evidence/valid/already_satisfied_no_op.json"));
    let observations = evidence["observations"].as_array_mut().unwrap();
    let before_obs = observations
        .iter_mut()
        .find(|o| o["step"] == "before")
        .unwrap();
    before_obs["content_hash"] = Value::String("ab".repeat(32));
    let (state, _basis, effect) = derive_postcondition(&plan, &evidence);
    assert_eq!(state, "failed", "a before observation that doesn't match the plan's before_hash must not derive to already_satisfied");
    assert_ne!(effect, "already_satisfied");
}

// ===========================================================================
// validate_fixture_case adversarial tests, per the #1715 round-5/6/7
// reviews: "Validate the complete paired fixture case ... Consolidate
// checks into ONE entry point and route EVERY positive/adversarial paired
// case through it." Every test below calls `validate_fixture_case` with a
// (plan, evidence, report) triad -- there is no narrower function left to
// bypass. Each test starts from a known-good triad and tampers exactly
// one thing, asserting the EXACT classification the round-7 review
// requires: missing information => unverified; present-and-contradictory
// information => failed; only a fully consistent chain => verified.
// ===========================================================================

/// The review's own headline example for round 5's point 1: changing the
/// final observation's metadata.mode to "0000" (present, but wrong) must
/// FAIL -- this is a contradiction, not a missing field.
#[test]
fn validate_fixture_case_rejects_changed_final_metadata_even_when_final_hash_matches() {
    let plan = load_json(&format!(
        "{FX}/plans/valid/mutating_removal_with_backup_required.json"
    ));
    let mut evidence = load_json(&format!("{FX}/evidence/valid/verified_removal.json"));
    let report = load_json(&format!("{FX}/reports/valid/verified_removal.json"));
    let observations = evidence["observations"].as_array_mut().unwrap();
    let final_obs = observations
        .iter_mut()
        .find(|o| o["step"] == "final")
        .unwrap();
    assert_eq!(
        final_obs["content_hash"], plan["expected_staged_hash"],
        "sanity: final hash must still match before this test's tamper"
    );
    final_obs["metadata"]["mode"] = Value::String("0000".to_string());

    let verdict = validate_fixture_case(&plan, &evidence, &report);
    assert_eq!(
        verdict.state, "failed",
        "present-but-wrong final metadata is a contradiction, not a missing field"
    );
    assert_ne!(verdict.effect, "removed");
}

/// A backup observation whose content_hash does not match the REAL bytes
/// at its own artifact_ref (independently rehashed, not merely compared
/// to another declared field) must FAIL.
#[test]
fn validate_fixture_case_rejects_a_wrong_backup_content_hash() {
    let plan = load_json(&format!(
        "{FX}/plans/valid/mutating_removal_with_backup_required.json"
    ));
    let mut evidence = load_json(&format!("{FX}/evidence/valid/verified_removal.json"));
    let report = load_json(&format!("{FX}/reports/valid/verified_removal.json"));
    let observations = evidence["observations"].as_array_mut().unwrap();
    let backup_obs = observations
        .iter_mut()
        .find(|o| o["step"] == "backup")
        .unwrap();
    backup_obs["content_hash"] = Value::String("cd".repeat(32));

    let verdict = validate_fixture_case(&plan, &evidence, &report);
    assert_eq!(
        verdict.state, "failed",
        "a backup content_hash that doesn't match the real rehashed bytes is a contradiction"
    );
    assert_ne!(verdict.effect, "removed");
}

/// A backup observation with mismatched metadata (right bytes, wrong
/// owner/group/mode) must FAIL.
#[test]
fn validate_fixture_case_rejects_a_backup_with_mismatched_metadata() {
    let plan = load_json(&format!(
        "{FX}/plans/valid/mutating_removal_with_backup_required.json"
    ));
    let mut evidence = load_json(&format!("{FX}/evidence/valid/verified_removal.json"));
    let report = load_json(&format!("{FX}/reports/valid/verified_removal.json"));
    let observations = evidence["observations"].as_array_mut().unwrap();
    let backup_obs = observations
        .iter_mut()
        .find(|o| o["step"] == "backup")
        .unwrap();
    backup_obs["metadata"]["owner"] = Value::String("nobody".to_string());

    let verdict = validate_fixture_case(&plan, &evidence, &report);
    assert_eq!(
        verdict.state, "failed",
        "present-but-wrong backup metadata is a contradiction"
    );
    assert_ne!(verdict.effect, "removed");
}

/// Missing bindings: the final bytes match, but there is no install
/// observation at all. This is MISSING information => unverified, never
/// verified -- a matching final snapshot alone proves nothing about
/// whether install happened.
#[test]
fn validate_fixture_case_treats_a_missing_install_observation_as_unverified() {
    let plan = load_json(&format!(
        "{FX}/plans/valid/mutating_removal_with_backup_required.json"
    ));
    let mut evidence = load_json(&format!("{FX}/evidence/valid/verified_removal.json"));
    let report = load_json(&format!("{FX}/reports/valid/verified_removal.json"));
    let observations = evidence["observations"].as_array_mut().unwrap();
    observations.retain(|o| o["step"] != "install");

    let verdict = validate_fixture_case(&plan, &evidence, &report);
    assert_eq!(verdict.state, "unverified", "a matching final snapshot with no install observation at all must be unverified, not verified");
    assert_ne!(verdict.effect, "removed");
}

/// Conflicting same-step snapshots: two "stage" observations with
/// DIFFERENT step_snapshot_ids but different content -- competing claims
/// about the same step -- must be rejected (unverified: we don't know
/// which claim is real) even though the IDs differ (the review's explicit
/// requirement: "Reject conflicting same-step snapshots even when IDs
/// differ").
#[test]
fn validate_fixture_case_rejects_conflicting_same_step_observations_even_with_different_snapshot_ids(
) {
    let plan = load_json(&format!(
        "{FX}/plans/valid/mutating_removal_with_backup_required.json"
    ));
    let mut evidence = load_json(&format!("{FX}/evidence/valid/verified_removal.json"));
    let report = load_json(&format!("{FX}/reports/valid/verified_removal.json"));
    let observations = evidence["observations"].as_array_mut().unwrap();
    let stage_obs = observations
        .iter()
        .find(|o| o["step"] == "stage")
        .unwrap()
        .clone();
    let mut conflicting_stage = stage_obs.clone();
    conflicting_stage["step_snapshot_id"] = Value::String("stage-2-different-id".to_string());
    conflicting_stage["content_hash"] = Value::String("ef".repeat(32));
    observations.push(conflicting_stage);

    let verdict = validate_fixture_case(&plan, &evidence, &report);
    assert_eq!(verdict.state, "unverified", "two conflicting stage observations with different step_snapshot_ids must still be rejected as a conflict");
    assert_ne!(verdict.effect, "removed");
}

/// Failed execution with matching final bytes: the final observation's
/// bytes match plan.expected_staged_hash, but the install step itself is
/// recorded as failed, AND its report still claims install completed.
/// The evidence-only semantic gates would say verified/unknown (final
/// bytes line up, install exists), but the report now contradicts the
/// evidence -- `finalize_against_report` must demote this to unverified,
/// and the effect must never be reported as removed either way.
#[test]
fn validate_fixture_case_never_reports_removed_when_the_install_step_itself_failed() {
    let plan = load_json(&format!(
        "{FX}/plans/valid/mutating_removal_with_backup_required.json"
    ));
    let mut evidence = load_json(&format!("{FX}/evidence/valid/verified_removal.json"));
    let report = load_json(&format!("{FX}/reports/valid/verified_removal.json"));
    assert_eq!(
        report["execution"]["steps"]["install"]["status"], "completed",
        "sanity: the untouched report claims install completed"
    );
    let observations = evidence["observations"].as_array_mut().unwrap();
    let install_obs = observations
        .iter_mut()
        .find(|o| o["step"] == "install")
        .unwrap();
    install_obs["step_status"] = Value::String("failed".to_string());

    let verdict = validate_fixture_case(&plan, &evidence, &report);
    assert_eq!(verdict.state, "unverified", "a report claiming install completed while the evidence now shows failed is an inconsistency, demoted to unverified by finalize_against_report");
    assert_ne!(verdict.effect, "removed", "a failed install step must never be reported as effect=removed even when the final bytes happen to match");
}

/// No-op with targets present: the plan claims no-op (count=0 for every
/// approved target), and the declared before_hash/evidence agree with
/// each other -- but the ACTUAL bytes at that hash still contain one of
/// the approved-removal targets. `validate_fixture_case` independently
/// re-parses the before artifact's real DER fingerprints rather than
/// trusting the plan's claim, so this must derive to failed, not
/// already_satisfied.
#[test]
fn validate_fixture_case_rejects_a_noop_plan_whose_actual_before_bytes_still_contain_an_approved_target(
) {
    let mut plan = load_json(&format!("{FX}/plans/valid/no_op_targets_absent.json"));
    let contradicting_path =
        "tests/fixtures/trust_operation_v1/artifacts/before_two_targets.pem.fixture";
    let contradicting_bytes = read_artifact(contradicting_path);
    let contradicting_hash = sha256_hex(&contradicting_bytes);

    // The plan still claims this target has zero occurrences (a genuine
    // no-op), but the before bytes we're about to point it at actually
    // contain two occurrences of exactly that target.
    assert_eq!(
        plan["approved_removal_ranges"][0]["target_sha256"],
        "62e4ea45d3b967b26dea8c9b5cf2d350a15c78b127728f165cbb48628ae545a3",
        "sanity: this is the target this test contradicts"
    );
    plan["before_hash"] = Value::String(contradicting_hash.clone());
    plan["before_artifact_ref"] = serde_json::json!({
        "sha256": contradicting_hash,
        "size_bytes": contradicting_bytes.len(),
        "source": "fixture_file",
        "fixture_path": contradicting_path,
    });

    let mut evidence = load_json(&format!("{FX}/evidence/valid/already_satisfied_no_op.json"));
    let report = load_json(&format!("{FX}/reports/valid/already_satisfied_no_op.json"));
    let observations = evidence["observations"].as_array_mut().unwrap();
    let before_obs = observations
        .iter_mut()
        .find(|o| o["step"] == "before")
        .unwrap();
    before_obs["content_hash"] = Value::String(contradicting_hash.clone());
    before_obs["artifact_ref"] = serde_json::json!({
        "sha256": contradicting_hash,
        "size_bytes": contradicting_bytes.len(),
        "source": "fixture_file",
        "fixture_path": contradicting_path,
    });

    let verdict = validate_fixture_case(&plan, &evidence, &report);
    assert_eq!(verdict.state, "failed", "a no-op plan whose actual before bytes still contain an approved-removal target must derive to failed, not already_satisfied, even though the declared before_hash matches");
    assert_ne!(verdict.effect, "already_satisfied");
}

/// A final observation with `metadata: null` is MISSING information, not
/// a contradiction -- it must be unverified, never failed. Fixes the
/// round-6 `validate_case`'s own overcorrection (round 5 skipped the
/// check entirely on null; round 6 fixed that but landed on `failed`
/// instead of `unverified`, which the round-7 review named explicitly:
/// "Missing final metadata/hash is incorrectly FAILED rather than
/// UNVERIFIED").
#[test]
fn validate_fixture_case_treats_a_null_final_metadata_as_unverified_not_failed() {
    let plan = load_json(&format!(
        "{FX}/plans/valid/mutating_removal_with_backup_required.json"
    ));
    let mut evidence = load_json(&format!("{FX}/evidence/valid/verified_removal.json"));
    let report = load_json(&format!("{FX}/reports/valid/verified_removal.json"));
    let observations = evidence["observations"].as_array_mut().unwrap();
    let final_obs = observations
        .iter_mut()
        .find(|o| o["step"] == "final")
        .unwrap();
    final_obs["metadata"] = Value::Null;

    let verdict = validate_fixture_case(&plan, &evidence, &report);
    assert_eq!(
        verdict.state, "unverified",
        "a null (missing) final metadata must be unverified, not failed -- we simply don't have the information, we haven't demonstrated a contradiction"
    );
    assert_ne!(verdict.effect, "removed");
}

/// A final observation whose own `parse_outcome` admits the parse was not
/// clean (`malformed`) is present-and-contradictory information -- it
/// must FAIL, even when its `content_hash` still happens to equal
/// `plan.expected_staged_hash`.
#[test]
fn validate_fixture_case_rejects_a_final_observation_with_malformed_parse_outcome() {
    let plan = load_json(&format!(
        "{FX}/plans/valid/mutating_removal_with_backup_required.json"
    ));
    let mut evidence = load_json(&format!("{FX}/evidence/valid/verified_removal.json"));
    let report = load_json(&format!("{FX}/reports/valid/verified_removal.json"));
    let observations = evidence["observations"].as_array_mut().unwrap();
    let final_obs = observations
        .iter_mut()
        .find(|o| o["step"] == "final")
        .unwrap();
    final_obs["parse_outcome"] = Value::String("malformed".to_string());

    let verdict = validate_fixture_case(&plan, &evidence, &report);
    assert_eq!(
        verdict.state, "failed",
        "final.parse_outcome=malformed is a contradiction, never a verify"
    );
    assert_ne!(verdict.effect, "removed");
}

/// The `stage` observation (the bytes install is claimed to have actually
/// installed) is independently rehashed. A stage observation whose
/// content_hash does NOT match the REAL rehashed bytes must FAIL, even
/// though backup and final are both still fine on their own.
#[test]
fn validate_fixture_case_rejects_a_wrong_stage_content_hash() {
    let plan = load_json(&format!(
        "{FX}/plans/valid/mutating_removal_with_backup_required.json"
    ));
    let mut evidence = load_json(&format!("{FX}/evidence/valid/verified_removal.json"));
    let report = load_json(&format!("{FX}/reports/valid/verified_removal.json"));
    let observations = evidence["observations"].as_array_mut().unwrap();
    let stage_obs = observations
        .iter_mut()
        .find(|o| o["step"] == "stage")
        .unwrap();
    stage_obs["content_hash"] = Value::String("12".repeat(32));

    let verdict = validate_fixture_case(&plan, &evidence, &report);
    assert_eq!(
        verdict.state, "failed",
        "a stage content_hash that doesn't match the real rehashed bytes is a contradiction"
    );
    assert_ne!(verdict.effect, "removed");
}

/// A missing stage observation entirely is MISSING information: it must
/// be unverified.
#[test]
fn validate_fixture_case_treats_a_missing_stage_observation_as_unverified() {
    let plan = load_json(&format!(
        "{FX}/plans/valid/mutating_removal_with_backup_required.json"
    ));
    let mut evidence = load_json(&format!("{FX}/evidence/valid/verified_removal.json"));
    let report = load_json(&format!("{FX}/reports/valid/verified_removal.json"));
    let observations = evidence["observations"].as_array_mut().unwrap();
    observations.retain(|o| o["step"] != "stage");

    let verdict = validate_fixture_case(&plan, &evidence, &report);
    assert_eq!(verdict.state, "unverified");
    assert_ne!(verdict.effect, "removed");
}

/// The final observation's declared `certificate_fingerprints` must match
/// an INDEPENDENT re-parse of the final artifact's actual bytes -- a
/// declared count that disagrees with the real bytes must FAIL, even
/// though content_hash and metadata are both still correct.
#[test]
fn validate_fixture_case_rejects_final_certificate_fingerprints_that_disagree_with_the_real_bytes()
{
    let plan = load_json(&format!(
        "{FX}/plans/valid/mutating_removal_with_backup_required.json"
    ));
    let mut evidence = load_json(&format!("{FX}/evidence/valid/verified_removal.json"));
    let report = load_json(&format!("{FX}/reports/valid/verified_removal.json"));
    let observations = evidence["observations"].as_array_mut().unwrap();
    let final_obs = observations
        .iter_mut()
        .find(|o| o["step"] == "final")
        .unwrap();
    // The real final artifact contains exactly 1 occurrence of this
    // fingerprint; falsely declare 2.
    final_obs["certificate_fingerprints"][0]["occurrence_count"] = serde_json::json!(2);

    let verdict = validate_fixture_case(&plan, &evidence, &report);
    assert_eq!(verdict.state, "failed", "a declared certificate_fingerprints count that disagrees with an independent re-parse of the real bytes is a contradiction");
    assert_ne!(verdict.effect, "removed");
}

/// The install step's `consumes_before_snapshot_id`/`consumes_staging_snapshot_id`
/// must match the ACTUAL `before`/`stage` observation ids in this bundle,
/// not merely be present. A dangling/substituted reference must FAIL.
#[test]
fn validate_fixture_case_rejects_an_install_binding_that_does_not_match_the_real_before_snapshot_id(
) {
    let plan = load_json(&format!(
        "{FX}/plans/valid/mutating_removal_with_backup_required.json"
    ));
    let mut evidence = load_json(&format!("{FX}/evidence/valid/verified_removal.json"));
    let report = load_json(&format!("{FX}/reports/valid/verified_removal.json"));
    let observations = evidence["observations"].as_array_mut().unwrap();
    let install_obs = observations
        .iter_mut()
        .find(|o| o["step"] == "install")
        .unwrap();
    assert_eq!(install_obs["consumes_before_snapshot_id"], "before-1");
    install_obs["consumes_before_snapshot_id"] = Value::String("before-substituted".to_string());

    let verdict = validate_fixture_case(&plan, &evidence, &report);
    assert_eq!(verdict.state, "failed", "an install binding naming a before snapshot id that doesn't match the real before observation is a contradiction");
    assert_ne!(verdict.effect, "removed");
}

/// The round-7 review's named FALSE POSITIVE: remove the `before`
/// observation entirely AND set `install.consumes_before_snapshot_id` to
/// `null` -- under the round-6 code, a missing before observation made
/// `before_id` default to `Value::Null` via `.unwrap_or(Value::Null)`, so
/// `null == null` "matched" vacuously and the case still verified. Gate 6
/// now REQUIRES a real `before` observation for the mutating path before
/// gate 11's binding comparison is ever reached, so this can no longer
/// happen: the case is unverified (missing `before`) long before any
/// binding comparison runs.
#[test]
fn validate_fixture_case_requires_a_real_before_observation_rather_than_a_vacuous_null_equals_null_match(
) {
    let plan = load_json(&format!(
        "{FX}/plans/valid/mutating_removal_with_backup_required.json"
    ));
    let mut evidence = load_json(&format!("{FX}/evidence/valid/verified_removal.json"));
    let report = load_json(&format!("{FX}/reports/valid/verified_removal.json"));
    let observations = evidence["observations"].as_array_mut().unwrap();
    observations.retain(|o| o["step"] != "before");
    let install_obs = observations
        .iter_mut()
        .find(|o| o["step"] == "install")
        .unwrap();
    install_obs["consumes_before_snapshot_id"] = Value::Null;

    let verdict = validate_fixture_case(&plan, &evidence, &report);
    assert_eq!(verdict.state, "unverified", "a missing before observation must be unverified -- it must NEVER be papered over by a null-consumes-id vacuously matching a null default");
    assert_ne!(verdict.effect, "removed");
}

/// An observation whose `normalized_destination_binding`/`context_binding`
/// does not match the plan it's bound to is a STALE observation (from a
/// different operation) -- a contradiction, so this must FAIL.
#[test]
fn validate_fixture_case_rejects_a_before_observation_with_a_mismatched_destination_binding() {
    let plan = load_json(&format!(
        "{FX}/plans/valid/mutating_removal_with_backup_required.json"
    ));
    let mut evidence = load_json(&format!("{FX}/evidence/valid/verified_removal.json"));
    let report = load_json(&format!("{FX}/reports/valid/verified_removal.json"));
    let observations = evidence["observations"].as_array_mut().unwrap();
    let before_obs = observations
        .iter_mut()
        .find(|o| o["step"] == "before")
        .unwrap();
    before_obs["normalized_destination_binding"] =
        Value::String("/etc/ssl/some-other-cert.pem".to_string());

    let verdict = validate_fixture_case(&plan, &evidence, &report);
    assert_eq!(
        verdict.state, "failed",
        "a before observation bound to a different destination than the plan is a stale observation, a contradiction"
    );
    assert_ne!(verdict.effect, "removed");
}

/// Range transformation: independently applying the plan's OWN declared
/// `approved_removal_ranges` to the real `before` bytes must actually
/// PRODUCE the real `stage` bytes. Here the plan only declares ONE of the
/// two real target ranges while `stage`/`expected_staged_hash` still claim
/// the fuller (both-targets-removed) result -- applying only the
/// declared range to the real before bytes yields something that does
/// NOT match the real stage bytes, so this must FAIL even though stage's
/// own content_hash still matches `expected_staged_hash` (both were
/// edited together) and backup/final are untouched.
#[test]
fn validate_fixture_case_rejects_a_plan_whose_declared_ranges_do_not_actually_produce_the_staged_bytes(
) {
    let mut plan = load_json(&format!(
        "{FX}/plans/valid/mutating_removal_with_backup_required.json"
    ));
    let evidence = load_json(&format!("{FX}/evidence/valid/verified_removal.json"));
    let report = load_json(&format!("{FX}/reports/valid/verified_removal.json"));

    // Keep only the FIRST target's ranges (removing cert_A's two
    // occurrences), dropping the second target's range (cert_B's single
    // occurrence) entirely -- so applying these (fewer) ranges to the real
    // before bytes leaves cert_B still present, which does NOT match the
    // real staged bytes (which have both targets removed).
    let ranges = plan["approved_removal_ranges"].as_array().unwrap().clone();
    assert_eq!(
        ranges.len(),
        2,
        "sanity: the untouched plan declares ranges for both targets"
    );
    plan["approved_removal_ranges"] = serde_json::json!([ranges[0].clone()]);

    let verdict = validate_fixture_case(&plan, &evidence, &report);
    assert_eq!(verdict.state, "failed", "applying only the (incompletely) declared ranges to the real before bytes must not produce the real staged bytes -- a genuine transformation mismatch");
    assert_ne!(verdict.effect, "removed");
}

/// Freshness: an install observation whose own `capture_sequence` is NOT
/// strictly after the `stage` observation it claims to have consumed is a
/// stale/impossible ordering -- a contradiction, so this must FAIL.
#[test]
fn validate_fixture_case_rejects_an_install_that_claims_to_consume_a_snapshot_captured_after_itself(
) {
    let plan = load_json(&format!(
        "{FX}/plans/valid/mutating_removal_with_backup_required.json"
    ));
    let mut evidence = load_json(&format!("{FX}/evidence/valid/verified_removal.json"));
    let report = load_json(&format!("{FX}/reports/valid/verified_removal.json"));
    let observations = evidence["observations"].as_array_mut().unwrap();
    let stage_seq = observations.iter().find(|o| o["step"] == "stage").unwrap()["capture_sequence"]
        .as_i64()
        .unwrap();
    let install_obs = observations
        .iter_mut()
        .find(|o| o["step"] == "install")
        .unwrap();
    install_obs["capture_sequence"] = serde_json::json!(stage_seq); // not strictly after stage

    let verdict = validate_fixture_case(&plan, &evidence, &report);
    assert_eq!(verdict.state, "failed", "an install captured at the SAME sequence as (or before) the stage snapshot it claims to consume is a stale/impossible ordering");
    assert_ne!(verdict.effect, "removed");
}

/// Scoped-approval digest: `verified_removal.json`'s report genuinely
/// claims `authorization.state=confirmed_scoped` with a real, matching
/// digest already (not a synthetic addition for this test). Swapping in
/// the frozen `scope_binding_digest_does_not_match_preimage.json` negative
/// fixture's report (same plan/evidence, only a wrong digest) must FAIL
/// via the one shared path, not merely via the standalone digest test.
#[test]
fn validate_fixture_case_rejects_a_confirmed_scoped_report_whose_digest_does_not_match_the_plan() {
    let plan = load_json(&format!(
        "{FX}/plans/valid/mutating_removal_with_backup_required.json"
    ));
    let evidence = load_json(&format!("{FX}/evidence/valid/verified_removal.json"));
    let good_report = load_json(&format!("{FX}/reports/valid/verified_removal.json"));
    assert_eq!(
        good_report["authorization"]["state"], "confirmed_scoped",
        "sanity: the happy-path report genuinely uses confirmed_scoped"
    );

    let bad_report = load_json(&format!(
        "{FX}/reports/invalid/scope_binding_digest_does_not_match_preimage.json"
    ));
    assert_eq!(bad_report["authorization"]["state"], "confirmed_scoped");
    assert_ne!(
        bad_report["authorization"]["scope_binding_digest"],
        good_report["authorization"]["scope_binding_digest"],
        "sanity: the negative fixture must actually carry a different digest"
    );

    let verdict = validate_fixture_case(&plan, &evidence, &bad_report);
    assert_eq!(verdict.state, "failed", "a confirmed_scoped report whose digest doesn't match the recomputed v1 preimage over the plan is a contradiction");
    assert_ne!(verdict.effect, "removed");
}

/// The no-op path's independent re-parse must FAIL the case on a genuine
/// parse error, never silently treat "couldn't parse" as "zero
/// certificates found, so the no-op claim holds."
#[test]
fn validate_fixture_case_fails_a_noop_case_whose_before_artifact_fails_to_parse_rather_than_treating_it_as_already_satisfied(
) {
    let mut plan = load_json(&format!("{FX}/plans/valid/no_op_targets_absent.json"));
    let malformed_path =
        "tests/fixtures/trust_operation_v1/artifacts/malformed_truncated.pem.fixture";
    let malformed_bytes = read_artifact(malformed_path);
    assert!(
        find_pem_blocks_strict(&malformed_bytes).is_err(),
        "sanity: this fixture must actually fail to parse"
    );
    let malformed_hash = sha256_hex(&malformed_bytes);
    plan["before_hash"] = Value::String(malformed_hash.clone());
    plan["before_artifact_ref"] = serde_json::json!({
        "sha256": malformed_hash,
        "size_bytes": malformed_bytes.len(),
        "source": "fixture_file",
        "fixture_path": malformed_path,
    });

    let mut evidence = load_json(&format!("{FX}/evidence/valid/already_satisfied_no_op.json"));
    let report = load_json(&format!("{FX}/reports/valid/already_satisfied_no_op.json"));
    let observations = evidence["observations"].as_array_mut().unwrap();
    let before_obs = observations
        .iter_mut()
        .find(|o| o["step"] == "before")
        .unwrap();
    before_obs["content_hash"] = Value::String(malformed_hash.clone());
    before_obs["artifact_ref"] = serde_json::json!({
        "sha256": malformed_hash,
        "size_bytes": malformed_bytes.len(),
        "source": "fixture_file",
        "fixture_path": malformed_path,
    });

    let verdict = validate_fixture_case(&plan, &evidence, &report);
    assert_eq!(verdict.state, "failed", "a no-op case whose before artifact fails to parse must FAIL, never silently read as already_satisfied just because 'zero certificates were found'");
    assert_ne!(verdict.effect, "already_satisfied");
}

/// `validate_fixture_case` catches a REPORT claiming a step completed
/// with no matching evidence at all. `verified_removal.json`'s report
/// claims `filter` completed (sourced from `fixture`) -- stripping the
/// evidence bundle's `filter` observation must demote the case to
/// `unverified` via `finalize_against_report`, even though every
/// evidence-only semantic gate on its own would still say verified/removed.
#[test]
fn validate_fixture_case_catches_a_report_claiming_a_step_completed_with_no_matching_evidence() {
    let plan = load_json(&format!(
        "{FX}/plans/valid/mutating_removal_with_backup_required.json"
    ));
    let mut evidence = load_json(&format!("{FX}/evidence/valid/verified_removal.json"));
    let report = load_json(&format!("{FX}/reports/valid/verified_removal.json"));
    assert_eq!(
        report["execution"]["steps"]["filter"]["status"],
        "completed"
    );
    assert_eq!(
        report["execution"]["steps"]["filter"]["observation_source"],
        "fixture"
    );

    let observations = evidence["observations"].as_array_mut().unwrap();
    let had_filter = observations.iter().any(|o| o["step"] == "filter");
    assert!(had_filter, "sanity: the untampered fixture must have a filter observation for this test to actually remove something");
    observations.retain(|o| o["step"] != "filter");

    let verdict = validate_fixture_case(&plan, &evidence, &report);
    assert_eq!(verdict.state, "unverified", "a report claiming filter completed with no matching evidence observation must be demoted to unverified");
    assert_ne!(verdict.effect, "removed");
}

/// Report/evidence source misattribution: the evidence's `filter`
/// observation's own `source` is `fixture`, but the report claims
/// `observation_source=caro_observed` for that same step -- same status,
/// different claimed authority. Per the round-7 review's "Report checks
/// omit cleanup/source agreement," this must also be caught, not just a
/// `step_status` mismatch.
#[test]
fn validate_fixture_case_catches_a_report_misattributing_which_source_backs_a_step() {
    let plan = load_json(&format!(
        "{FX}/plans/valid/mutating_removal_with_backup_required.json"
    ));
    let evidence = load_json(&format!("{FX}/evidence/valid/verified_removal.json"));
    let mut report = load_json(&format!("{FX}/reports/valid/verified_removal.json"));
    assert_eq!(
        report["execution"]["steps"]["filter"]["observation_source"],
        "fixture"
    );
    report["execution"]["steps"]["filter"]["observation_source"] =
        Value::String("caro_observed".to_string());

    let verdict = validate_fixture_case(&plan, &evidence, &report);
    assert_eq!(verdict.state, "unverified", "a report claiming a different observation_source than the evidence actually carries for that step must be demoted to unverified");
    assert_ne!(verdict.effect, "removed");
}

/// Cleanup agreement: the report claims `cleanup.state=succeeded`, but the
/// paired evidence's own `cleanup` observation shows `step_status=failed`
/// -- the cleanup half of `report_steps_consistent_with_evidence`,
/// exercised here via the one shared path.
#[test]
fn validate_fixture_case_catches_a_report_claiming_cleanup_succeeded_when_evidence_shows_failed() {
    let plan = load_json(&format!(
        "{FX}/plans/valid/mutating_removal_with_backup_required.json"
    ));
    let mut evidence = load_json(&format!("{FX}/evidence/valid/verified_removal.json"));
    let report = load_json(&format!("{FX}/reports/valid/verified_removal.json"));
    assert_eq!(report["cleanup"]["state"], "succeeded");
    let observations = evidence["observations"].as_array_mut().unwrap();
    let cleanup_obs = observations
        .iter_mut()
        .find(|o| o["step"] == "cleanup")
        .unwrap();
    cleanup_obs["step_status"] = Value::String("failed".to_string());

    let verdict = validate_fixture_case(&plan, &evidence, &report);
    assert_eq!(verdict.state, "unverified", "a report claiming cleanup succeeded while the evidence shows a failed cleanup step must be demoted to unverified");
    assert_ne!(verdict.effect, "removed");
}

// ===========================================================================
// Round-8 adversarial tests, per the #1715 round-8 review's points 1-3.
// Each test tampers exactly one thing from the known-good
// `verified_removal`/`already_satisfied_no_op` triads and asserts the
// EXACT classification the review demands.
// ===========================================================================

/// The round-8 review's headline attack: relabel one
/// `approved_removal_ranges[i].target_sha256` to an UNRELATED value
/// (something that is not the real DER fingerprint at that byte range),
/// then RECOMPUTE the report's `scope_binding_digest` over the
/// (now-tampered) plan so the scoped-digest gate (12) also passes. Round
/// 7's `apply_approved_removal_ranges` used only the declared
/// `start`/`end` offsets to slice bytes -- it never checked the label
/// against the real bytes at that position -- so this attack used to
/// still verify removal. `bind_and_apply_approved_removal_ranges` now
/// checks the real DER at that byte range against the declared
/// `target_sha256`, so this must FAIL.
#[test]
fn validate_fixture_case_rejects_a_relabeled_target_fingerprint_even_with_a_recomputed_digest() {
    let mut plan = load_json(&format!(
        "{FX}/plans/valid/mutating_removal_with_backup_required.json"
    ));
    let evidence = load_json(&format!("{FX}/evidence/valid/verified_removal.json"));
    let mut report = load_json(&format!("{FX}/reports/valid/verified_removal.json"));

    let unrelated_fingerprint = "ab".repeat(32);
    assert_ne!(
        plan["approved_removal_ranges"][0]["target_sha256"], unrelated_fingerprint,
        "sanity: this must actually be a different value"
    );
    plan["approved_removal_ranges"][0]["target_sha256"] = Value::String(unrelated_fingerprint);
    // Recompute the digest over the TAMPERED plan, exactly as the review
    // describes -- so gate 12 (scoped digest) passes on its own terms,
    // and only gate 9's fingerprint binding can catch this.
    let recomputed_digest = compute_scope_binding_digest_v1(&plan);
    report["authorization"]["scope_binding_digest"] = Value::String(recomputed_digest);

    let verdict = validate_fixture_case(&plan, &evidence, &report);
    assert_eq!(verdict.state, "failed", "relabeling a target fingerprint to an unrelated value must fail even with a correctly recomputed scope-binding digest -- the byte range must bind to the REAL DER at that offset, not just to whatever label the plan claims");
    assert_ne!(verdict.effect, "removed");
}

/// `plan.target_fingerprints` (the plan's declared FULL multiset of the
/// before artifact) must match an independent re-parse of the REAL
/// before bytes, not just be internally consistent with
/// `approved_removal_ranges`. Tampering an unrelated-target's declared
/// occurrence count must fail the whole range-transformation gate.
#[test]
fn validate_fixture_case_rejects_a_plan_whose_declared_target_fingerprints_disagree_with_the_real_before_bytes(
) {
    let mut plan = load_json(&format!(
        "{FX}/plans/valid/mutating_removal_with_backup_required.json"
    ));
    let evidence = load_json(&format!("{FX}/evidence/valid/verified_removal.json"));
    let report = load_json(&format!("{FX}/reports/valid/verified_removal.json"));

    // The third target_fingerprints entry (not part of the approved
    // removal at all) declares occurrence_count=1; falsely claim 5.
    plan["target_fingerprints"][2]["occurrence_count"] = serde_json::json!(5);

    let verdict = validate_fixture_case(&plan, &evidence, &report);
    assert_eq!(verdict.state, "failed", "plan.target_fingerprints must match an independent re-parse of the real before bytes exactly, not merely be self-consistent with the approved ranges");
    assert_ne!(verdict.effect, "removed");
}

/// Before bytes must be bound to `plan.before_hash` directly, not merely
/// to the evidence observation's OWN declared `content_hash` (which can
/// be independently correct against ITS OWN artifact while still
/// disagreeing with what the PLAN claims the before state was).
#[test]
fn validate_fixture_case_rejects_a_plan_before_hash_that_disagrees_with_the_real_before_bytes() {
    let mut plan = load_json(&format!(
        "{FX}/plans/valid/mutating_removal_with_backup_required.json"
    ));
    let evidence = load_json(&format!("{FX}/evidence/valid/verified_removal.json"));
    let report = load_json(&format!("{FX}/reports/valid/verified_removal.json"));
    plan["before_hash"] = Value::String("cd".repeat(32));

    let verdict = validate_fixture_case(&plan, &evidence, &report);
    assert_eq!(verdict.state, "failed", "a plan.before_hash that disagrees with the real (independently rehashed) before bytes must fail, even though the evidence observation's own content_hash is internally correct");
    assert_ne!(verdict.effect, "removed");
}

/// The plan's own `before_artifact_ref` hash/size fields must themselves
/// be bound to `before_hash`/the real bytes, not merely declared.
#[test]
fn validate_fixture_case_rejects_a_plan_before_artifact_ref_with_a_wrong_declared_size() {
    let mut plan = load_json(&format!(
        "{FX}/plans/valid/mutating_removal_with_backup_required.json"
    ));
    let evidence = load_json(&format!("{FX}/evidence/valid/verified_removal.json"));
    let report = load_json(&format!("{FX}/reports/valid/verified_removal.json"));
    plan["before_artifact_ref"]["size_bytes"] = serde_json::json!(1);

    let verdict = validate_fixture_case(&plan, &evidence, &report);
    assert_eq!(
        verdict.state, "failed",
        "a before_artifact_ref whose declared size doesn't match the real before bytes must fail"
    );
    assert_ne!(verdict.effect, "removed");
}

/// Evidence-to-report propagation: the EVIDENCE shows `stage` genuinely
/// failed, but the REPORT hides this behind `unobserved`/`source: none`
/// while claiming `execution.state=completed` and
/// `cleanup.workflow_outcome=completed`. A known mutation failure MUST
/// propagate into the report -- this can no longer be hidden.
#[test]
fn validate_fixture_case_catches_a_known_stage_failure_hidden_behind_an_unobserved_report_claim() {
    let plan = load_json(&format!(
        "{FX}/plans/valid/mutating_removal_with_backup_required.json"
    ));
    let mut evidence = load_json(&format!("{FX}/evidence/valid/verified_removal.json"));
    let mut report = load_json(&format!("{FX}/reports/valid/verified_removal.json"));
    let observations = evidence["observations"].as_array_mut().unwrap();
    let stage_obs = observations
        .iter_mut()
        .find(|o| o["step"] == "stage")
        .unwrap();
    stage_obs["step_status"] = Value::String("failed".to_string());
    // The report hides this known failure.
    report["execution"]["steps"]["stage"]["status"] = Value::String("unobserved".to_string());
    report["execution"]["steps"]["stage"]["observation_source"] = Value::String("none".to_string());
    assert_eq!(report["execution"]["state"], "completed");
    assert_eq!(report["cleanup"]["workflow_outcome"], "completed");

    let verdict = validate_fixture_case(&plan, &evidence, &report);
    assert_eq!(verdict.state, "unverified", "a report that hides a known evidence-recorded stage failure behind an unobserved claim, while still claiming execution/workflow completed, must be demoted to unverified");
    assert_ne!(verdict.effect, "removed");
}

/// Cleanup failures must remain VISIBLE (propagate into the report)
/// without erasing an otherwise independently verified final mutation
/// state: when the evidence shows cleanup genuinely failed AND the
/// report correctly acknowledges it (`cleanup.state=failed`,
/// `workflow_outcome=failed`, per R12), the mutation's own
/// verified/removed outcome must be PRESERVED, not erased, because
/// cleanup failing says nothing about whether the mutation itself
/// succeeded.
#[test]
fn validate_fixture_case_preserves_verified_removed_when_cleanup_fails_but_is_honestly_reported() {
    let plan = load_json(&format!(
        "{FX}/plans/valid/mutating_removal_with_backup_required.json"
    ));
    let mut evidence = load_json(&format!("{FX}/evidence/valid/verified_removal.json"));
    let mut report = load_json(&format!("{FX}/reports/valid/verified_removal.json"));
    let observations = evidence["observations"].as_array_mut().unwrap();
    let cleanup_obs = observations
        .iter_mut()
        .find(|o| o["step"] == "cleanup")
        .unwrap();
    cleanup_obs["step_status"] = Value::String("failed".to_string());
    report["cleanup"]["state"] = Value::String("failed".to_string());
    report["cleanup"]["workflow_outcome"] = Value::String("failed".to_string());

    let verdict = validate_fixture_case(&plan, &evidence, &report);
    assert_eq!(verdict.state, "verified", "an honestly-reported cleanup failure must not erase the mutation's own independently verified state");
    assert_eq!(verdict.effect, "removed");
}

/// The round-9 review's headline point-2 scenario: `stage` is
/// TRUTHFULLY reported as failed (both evidence and report agree --
/// `execution.state=failed`, `workflow_outcome=failed`, matching
/// `report_steps_consistent_with_evidence`'s own propagation rule), yet
/// `install` still shows completed and the `final` bytes are
/// independently confirmed correct. Round 8 never consulted
/// backup/stage/filter's own `step_status` when computing the effect --
/// only `install`'s -- so this still produced effect=removed. Any
/// mutation-step failure (here, `stage`) must force effect=unknown,
/// regardless of what install/final look like on their own.
#[test]
fn validate_fixture_case_retains_unknown_effect_when_a_truthfully_reported_stage_failure_precedes_a_completed_install(
) {
    let plan = load_json(&format!(
        "{FX}/plans/valid/mutating_removal_with_backup_required.json"
    ));
    let mut evidence = load_json(&format!("{FX}/evidence/valid/verified_removal.json"));
    let mut report = load_json(&format!("{FX}/reports/valid/verified_removal.json"));
    let observations = evidence["observations"].as_array_mut().unwrap();
    let stage_obs = observations
        .iter_mut()
        .find(|o| o["step"] == "stage")
        .unwrap();
    // The bytes/hash are untouched (still independently correct) -- only
    // the step's own status is marked failed, e.g. a transient error
    // that was logged even though the write landed.
    stage_obs["step_status"] = Value::String("failed".to_string());
    report["execution"]["steps"]["stage"]["status"] = Value::String("failed".to_string());
    report["execution"]["state"] = Value::String("failed".to_string());
    // R11 requires a non-null failure_mode whenever execution.state=failed.
    report["execution"]["failure_mode"] = Value::String("step_failed".to_string());
    report["cleanup"]["workflow_outcome"] = Value::String("failed".to_string());
    // R1 requires execution.state=completed whenever postcondition.effect=
    // removed; keep the REPORT's own postcondition internally consistent
    // (schema-valid) with the truthful failure it now reports. This is
    // the report's own claim, not `validate_fixture_case`'s independently
    // computed verdict -- the whole point of this test is that the
    // Rust-computed verdict (asserted below) must say effect=unknown
    // regardless of what either document separately claims.
    report["postcondition"]["effect"] = Value::String("unknown".to_string());

    let verdict = validate_fixture_case(&plan, &evidence, &report);
    assert_eq!(
        verdict.state, "verified",
        "the final bytes are still independently confirmed correct, so the case itself is still verified"
    );
    assert_eq!(
        verdict.effect, "unknown",
        "a truthfully reported stage failure must force effect=unknown even though install completed and the final bytes check out -- a failed mutation step before install means we don't actually know removal is what produced this state"
    );
}

/// Point 3's second half: a no-op plan whose `approved_removal_ranges`
/// entry claims `count: 0` (making the plan classify as no-op) while
/// still declaring a NON-EMPTY `byte_ranges` array for that same entry
/// is internally inconsistent -- it cannot simultaneously claim "nothing
/// to remove" and "here are bytes to remove."
#[test]
fn validate_fixture_case_rejects_a_noop_plan_whose_zero_count_range_still_declares_nonempty_byte_ranges(
) {
    let mut plan = load_json(&format!("{FX}/plans/valid/no_op_targets_absent.json"));
    let evidence = load_json(&format!("{FX}/evidence/valid/already_satisfied_no_op.json"));
    let report = load_json(&format!("{FX}/reports/valid/already_satisfied_no_op.json"));
    assert!(
        plan_is_noop(&plan),
        "sanity: the untouched plan is a genuine no-op"
    );
    assert_eq!(plan["approved_removal_ranges"][0]["count"], 0);
    assert_eq!(
        plan["approved_removal_ranges"][0]["byte_ranges"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
    plan["approved_removal_ranges"][0]["byte_ranges"] =
        serde_json::json!([{"start": 0, "end": 10}]);

    let verdict = validate_fixture_case(&plan, &evidence, &report);
    assert_eq!(verdict.state, "failed", "a range entry claiming count=0 while still declaring a non-empty byte_ranges array is internally inconsistent and must fail, not pass through as a genuine no-op");
    assert_ne!(verdict.effect, "already_satisfied");
}

/// Point 3's first half: the `artifact_ref.sha256`/`size_bytes` fields
/// must themselves be independently checked against the real bytes, not
/// merely the observation's own `content_hash`. Tamper only
/// `artifact_ref.sha256` on the `backup` observation (leaving
/// `content_hash` itself correct) and confirm this is still caught.
#[test]
fn validate_fixture_case_rejects_an_artifact_ref_sha256_that_disagrees_with_the_real_bytes() {
    let plan = load_json(&format!(
        "{FX}/plans/valid/mutating_removal_with_backup_required.json"
    ));
    let mut evidence = load_json(&format!("{FX}/evidence/valid/verified_removal.json"));
    let report = load_json(&format!("{FX}/reports/valid/verified_removal.json"));
    let observations = evidence["observations"].as_array_mut().unwrap();
    let backup_obs = observations
        .iter_mut()
        .find(|o| o["step"] == "backup")
        .unwrap();
    let real_hash = backup_obs["content_hash"].as_str().unwrap().to_string();
    backup_obs["artifact_ref"]["sha256"] = Value::String("ef".repeat(32));
    assert_ne!(
        backup_obs["content_hash"],
        backup_obs["artifact_ref"]["sha256"]
    );
    assert_eq!(backup_obs["content_hash"], real_hash);

    let verdict = validate_fixture_case(&plan, &evidence, &report);
    assert_eq!(verdict.state, "failed", "artifact_ref.sha256 disagreeing with the real bytes/content_hash must fail, even though content_hash itself still matches the real bytes");
    assert_ne!(verdict.effect, "removed");
}

/// Same point, for `artifact_ref.size_bytes` on the `final` observation.
#[test]
fn validate_fixture_case_rejects_an_artifact_ref_size_bytes_that_disagrees_with_the_real_bytes() {
    let plan = load_json(&format!(
        "{FX}/plans/valid/mutating_removal_with_backup_required.json"
    ));
    let mut evidence = load_json(&format!("{FX}/evidence/valid/verified_removal.json"));
    let report = load_json(&format!("{FX}/reports/valid/verified_removal.json"));
    let observations = evidence["observations"].as_array_mut().unwrap();
    let final_obs = observations
        .iter_mut()
        .find(|o| o["step"] == "final")
        .unwrap();
    final_obs["artifact_ref"]["size_bytes"] = serde_json::json!(1);

    let verdict = validate_fixture_case(&plan, &evidence, &report);
    assert_eq!(verdict.state, "failed", "artifact_ref.size_bytes disagreeing with the real bytes must fail, even though content_hash itself still matches");
    assert_ne!(verdict.effect, "removed");
}

#[test]
fn validate_fixture_case_treats_a_null_install_binding_as_unverified_even_with_an_existing_before_snapshot(
) {
    // Per the #1715 round-9 review's point 1: this is the OPPOSITE
    // classification from round 8's own (wrong) test of this same
    // scenario. Round 8 argued a null consumes_before_snapshot_id was a
    // "contradiction" because a real before observation existed to
    // compare against -- but existence of a real snapshot does not
    // prove WHICH snapshot install actually consumed. A null claim is
    // simply a missing claim: unverified, not failed.
    let plan = load_json(&format!(
        "{FX}/plans/valid/mutating_removal_with_backup_required.json"
    ));
    let mut evidence = load_json(&format!("{FX}/evidence/valid/verified_removal.json"));
    let report = load_json(&format!("{FX}/reports/valid/verified_removal.json"));
    let observations = evidence["observations"].as_array_mut().unwrap();
    let install_obs = observations
        .iter_mut()
        .find(|o| o["step"] == "install")
        .unwrap();
    install_obs["consumes_before_snapshot_id"] = Value::Null;

    let verdict = validate_fixture_case(&plan, &evidence, &report);
    assert_eq!(verdict.state, "unverified", "a null consumes_before_snapshot_id is a MISSING claim, not a demonstrated contradiction, even when a real before observation genuinely exists -- existence alone doesn't prove which snapshot install consumed");
    assert_ne!(verdict.effect, "removed");
}

/// The companion positive case for point 1: a NON-NULL but WRONG
/// `consumes_staging_snapshot_id` (naming a specific, real-looking id
/// that is not the actual stage observation's own id) IS a demonstrated
/// contradiction -- failed, not unverified.
#[test]
fn validate_fixture_case_rejects_a_non_null_wrong_staging_snapshot_id() {
    let plan = load_json(&format!(
        "{FX}/plans/valid/mutating_removal_with_backup_required.json"
    ));
    let mut evidence = load_json(&format!("{FX}/evidence/valid/verified_removal.json"));
    let report = load_json(&format!("{FX}/reports/valid/verified_removal.json"));
    let observations = evidence["observations"].as_array_mut().unwrap();
    let install_obs = observations
        .iter_mut()
        .find(|o| o["step"] == "install")
        .unwrap();
    assert_eq!(install_obs["consumes_staging_snapshot_id"], "stage-1");
    install_obs["consumes_staging_snapshot_id"] = Value::String("stage-substituted".to_string());

    let verdict = validate_fixture_case(&plan, &evidence, &report);
    assert_eq!(verdict.state, "failed", "a non-null but wrong consumes_staging_snapshot_id is a demonstrated contradiction against the real stage observation's own id");
    assert_ne!(verdict.effect, "removed");
}

/// `final.parse_outcome=not_applicable` means parsing was never
/// attempted or doesn't apply -- MISSING information, never a
/// demonstrated contradiction. Must be unverified, not failed (unlike
/// `malformed`/`truncated`/`unconsumed_blocks`, which DO mean a parse
/// was attempted and genuinely failed).
#[test]
fn validate_fixture_case_treats_a_not_applicable_final_parse_outcome_as_unverified_not_failed() {
    let plan = load_json(&format!(
        "{FX}/plans/valid/mutating_removal_with_backup_required.json"
    ));
    let mut evidence = load_json(&format!("{FX}/evidence/valid/verified_removal.json"));
    let report = load_json(&format!("{FX}/reports/valid/verified_removal.json"));
    let observations = evidence["observations"].as_array_mut().unwrap();
    let final_obs = observations
        .iter_mut()
        .find(|o| o["step"] == "final")
        .unwrap();
    final_obs["parse_outcome"] = Value::String("not_applicable".to_string());

    let verdict = validate_fixture_case(&plan, &evidence, &report);
    assert_eq!(verdict.state, "unverified", "parse_outcome=not_applicable means no parse information is available, which is missing information, not a contradiction");
    assert_ne!(verdict.effect, "removed");
}

/// A `null` `certificate_fingerprints` declaration means no declared
/// count was ever supplied -- UNAVAILABLE, not an asserted empty
/// multiset to be compared (and fail) against the real, non-empty bytes.
#[test]
fn validate_fixture_case_treats_a_null_final_fingerprint_declaration_as_unverified_not_failed() {
    let plan = load_json(&format!(
        "{FX}/plans/valid/mutating_removal_with_backup_required.json"
    ));
    let mut evidence = load_json(&format!("{FX}/evidence/valid/verified_removal.json"));
    let report = load_json(&format!("{FX}/reports/valid/verified_removal.json"));
    let observations = evidence["observations"].as_array_mut().unwrap();
    let final_obs = observations
        .iter_mut()
        .find(|o| o["step"] == "final")
        .unwrap();
    final_obs["certificate_fingerprints"] = Value::Null;

    let verdict = validate_fixture_case(&plan, &evidence, &report);
    assert_eq!(verdict.state, "unverified", "a null certificate_fingerprints declaration is missing information, not an asserted empty multiset to fail against the real bytes");
    assert_ne!(verdict.effect, "removed");
}

/// Competing snapshots differing only in `metadata`/`source` (not hash or
/// status) for the same step must still be rejected as a conflict.
#[test]
fn validate_fixture_case_rejects_competing_snapshots_that_differ_only_in_metadata_or_source() {
    let plan = load_json(&format!(
        "{FX}/plans/valid/mutating_removal_with_backup_required.json"
    ));
    let mut evidence = load_json(&format!("{FX}/evidence/valid/verified_removal.json"));
    let report = load_json(&format!("{FX}/reports/valid/verified_removal.json"));
    let observations = evidence["observations"].as_array_mut().unwrap();
    let backup_obs = observations
        .iter()
        .find(|o| o["step"] == "backup")
        .unwrap()
        .clone();
    let mut competing_backup = backup_obs.clone();
    // Same content_hash and step_status as the original -- only the
    // claimed source differs (a genuinely different claim about WHO
    // backed this step, not just a relabeling).
    competing_backup["step_snapshot_id"] = Value::String("backup-2-different-id".to_string());
    competing_backup["source"] = Value::String("supplied".to_string());
    observations.push(competing_backup);

    let verdict = validate_fixture_case(&plan, &evidence, &report);
    assert_eq!(verdict.state, "unverified", "two backup observations agreeing on hash and status but disagreeing on source are still competing, inconsistent claims");
    assert_ne!(verdict.effect, "removed");
}

// ===========================================================================
// Round-10 adversarial tests, per the #1715 round-10 review's point 1:
// plan-level artifact_ref resolution (`resolve_plan_artifact_ref`),
// exercised for BOTH the no-op and mutation branches, with wrong hash,
// wrong size, and a swapped path to DIFFERENT EXISTING bytes.
// ===========================================================================

/// The review's exact no-op example: `before_artifact_ref.size_bytes`
/// tampered from 741 to 1 (leaving `sha256`/`fixture_path` untouched).
/// Round 9's checks never read `before_artifact_ref.fixture_path` at all
/// in the no-op branch, so this still verified `already_satisfied`.
#[test]
fn validate_fixture_case_rejects_a_noop_plan_before_artifact_ref_with_a_wrong_declared_size() {
    let mut plan = load_json(&format!("{FX}/plans/valid/no_op_targets_absent.json"));
    let evidence = load_json(&format!("{FX}/evidence/valid/already_satisfied_no_op.json"));
    let report = load_json(&format!("{FX}/reports/valid/already_satisfied_no_op.json"));
    assert_eq!(plan["before_artifact_ref"]["size_bytes"], 741);
    plan["before_artifact_ref"]["size_bytes"] = serde_json::json!(1);

    let verdict = validate_fixture_case(&plan, &evidence, &report);
    assert_eq!(verdict.state, "failed", "a no-op plan's before_artifact_ref.size_bytes disagreeing with the real bytes on disk must fail, caught before the no-op branch ever runs");
    assert_ne!(verdict.effect, "already_satisfied");
}

/// The no-op counterpart with a wrong `sha256` instead of `size_bytes`.
#[test]
fn validate_fixture_case_rejects_a_noop_plan_before_artifact_ref_with_a_wrong_declared_hash() {
    let mut plan = load_json(&format!("{FX}/plans/valid/no_op_targets_absent.json"));
    let evidence = load_json(&format!("{FX}/evidence/valid/already_satisfied_no_op.json"));
    let report = load_json(&format!("{FX}/reports/valid/already_satisfied_no_op.json"));
    plan["before_artifact_ref"]["sha256"] = Value::String("ab".repeat(32));

    let verdict = validate_fixture_case(&plan, &evidence, &report);
    assert_eq!(verdict.state, "failed", "a no-op plan's before_artifact_ref.sha256 disagreeing with the real bytes on disk must fail");
    assert_ne!(verdict.effect, "already_satisfied");
}

/// The no-op counterpart with `before_artifact_ref.fixture_path`
/// swapped to a DIFFERENT, real, existing fixture file (leaving the
/// declared `sha256`/`size_bytes` fields untouched, so an internal-
/// consistency-only check would miss this entirely).
#[test]
fn validate_fixture_case_rejects_a_noop_plan_before_artifact_ref_pointed_at_different_real_bytes() {
    let mut plan = load_json(&format!("{FX}/plans/valid/no_op_targets_absent.json"));
    let evidence = load_json(&format!("{FX}/evidence/valid/already_satisfied_no_op.json"));
    let report = load_json(&format!("{FX}/reports/valid/already_satisfied_no_op.json"));
    let swapped_path = "tests/fixtures/trust_operation_v1/artifacts/before_two_targets.pem.fixture";
    assert_ne!(
        plan["before_artifact_ref"]["fixture_path"], swapped_path,
        "sanity: this must actually be a different path"
    );
    plan["before_artifact_ref"]["fixture_path"] = Value::String(swapped_path.to_string());

    let verdict = validate_fixture_case(&plan, &evidence, &report);
    assert_eq!(verdict.state, "failed", "a before_artifact_ref.fixture_path swapped to different, real, existing bytes must fail -- the declared sha256/size_bytes fields being internally self-consistent is not enough, the path must actually resolve to bytes matching before_hash");
    assert_ne!(verdict.effect, "already_satisfied");
}

/// The review's exact mutation example: `before_artifact_ref.fixture_path`
/// changed to a DIFFERENT, real, existing fixture
/// (`before_targets_absent.pem.fixture`), with every other declared
/// value (`sha256`, `size_bytes`, `before_hash` itself) left unchanged.
/// Round 9's mutation-path check only ever compared
/// `before_artifact_ref.sha256`/`size_bytes` against `before_hash` and
/// the EVIDENCE's own resolved bytes -- it never opened the plan's own
/// `fixture_path` at all, so this still verified `removed`.
#[test]
fn validate_fixture_case_rejects_a_mutation_plan_before_artifact_ref_pointed_at_different_real_bytes(
) {
    let mut plan = load_json(&format!(
        "{FX}/plans/valid/mutating_removal_with_backup_required.json"
    ));
    let evidence = load_json(&format!("{FX}/evidence/valid/verified_removal.json"));
    let report = load_json(&format!("{FX}/reports/valid/verified_removal.json"));
    let swapped_path =
        "tests/fixtures/trust_operation_v1/artifacts/before_targets_absent.pem.fixture";
    assert_ne!(
        plan["before_artifact_ref"]["fixture_path"], swapped_path,
        "sanity: this must actually be a different path"
    );
    // Leave sha256/size_bytes/before_hash exactly as declared -- the
    // ONLY tamper is the path itself pointing somewhere else real.
    plan["before_artifact_ref"]["fixture_path"] = Value::String(swapped_path.to_string());

    let verdict = validate_fixture_case(&plan, &evidence, &report);
    assert_eq!(verdict.state, "failed", "a mutation plan's before_artifact_ref.fixture_path swapped to a different, real, existing fixture must fail, even though the declared sha256/size_bytes/before_hash fields are all left internally consistent with each other");
    assert_ne!(verdict.effect, "removed");
}

/// The mutation-path counterpart for `expected_staged_artifact_ref`,
/// which round 9 never checked at all: tamper only `size_bytes`.
#[test]
fn validate_fixture_case_rejects_a_mutation_plan_expected_staged_artifact_ref_with_a_wrong_declared_size(
) {
    let mut plan = load_json(&format!(
        "{FX}/plans/valid/mutating_removal_with_backup_required.json"
    ));
    let evidence = load_json(&format!("{FX}/evidence/valid/verified_removal.json"));
    let report = load_json(&format!("{FX}/reports/valid/verified_removal.json"));
    assert_eq!(plan["expected_staged_artifact_ref"]["size_bytes"], 741);
    plan["expected_staged_artifact_ref"]["size_bytes"] = serde_json::json!(1);

    let verdict = validate_fixture_case(&plan, &evidence, &report);
    assert_eq!(verdict.state, "failed", "expected_staged_artifact_ref.size_bytes disagreeing with the real bytes on disk must fail -- round 9 never checked this field at all");
    assert_ne!(verdict.effect, "removed");
}

/// Same field, tampered via `sha256` instead of `size_bytes`.
#[test]
fn validate_fixture_case_rejects_a_mutation_plan_expected_staged_artifact_ref_with_a_wrong_declared_hash(
) {
    let mut plan = load_json(&format!(
        "{FX}/plans/valid/mutating_removal_with_backup_required.json"
    ));
    let evidence = load_json(&format!("{FX}/evidence/valid/verified_removal.json"));
    let report = load_json(&format!("{FX}/reports/valid/verified_removal.json"));
    plan["expected_staged_artifact_ref"]["sha256"] = Value::String("cd".repeat(32));

    let verdict = validate_fixture_case(&plan, &evidence, &report);
    assert_eq!(
        verdict.state, "failed",
        "expected_staged_artifact_ref.sha256 disagreeing with the real bytes on disk must fail"
    );
    assert_ne!(verdict.effect, "removed");
}

/// And `expected_staged_artifact_ref.fixture_path` swapped to different,
/// real, existing bytes (the before-removal, two-target fixture).
#[test]
fn validate_fixture_case_rejects_a_mutation_plan_expected_staged_artifact_ref_pointed_at_different_real_bytes(
) {
    let mut plan = load_json(&format!(
        "{FX}/plans/valid/mutating_removal_with_backup_required.json"
    ));
    let evidence = load_json(&format!("{FX}/evidence/valid/verified_removal.json"));
    let report = load_json(&format!("{FX}/reports/valid/verified_removal.json"));
    let swapped_path = "tests/fixtures/trust_operation_v1/artifacts/before_two_targets.pem.fixture";
    assert_ne!(
        plan["expected_staged_artifact_ref"]["fixture_path"], swapped_path,
        "sanity: this must actually be a different path"
    );
    plan["expected_staged_artifact_ref"]["fixture_path"] = Value::String(swapped_path.to_string());

    let verdict = validate_fixture_case(&plan, &evidence, &report);
    assert_eq!(verdict.state, "failed", "expected_staged_artifact_ref.fixture_path swapped to different, real, existing bytes must fail");
    assert_ne!(verdict.effect, "removed");
}

/// Preserved outcome: tampering nothing at all still verifies, for both
/// branches -- proving the new gate 4.5 is not itself over-strict on the
/// genuine happy paths.
#[test]
fn validate_fixture_case_still_verifies_the_untampered_happy_paths_after_the_plan_artifact_ref_gate(
) {
    let plan = load_json(&format!(
        "{FX}/plans/valid/mutating_removal_with_backup_required.json"
    ));
    let evidence = load_json(&format!("{FX}/evidence/valid/verified_removal.json"));
    let report = load_json(&format!("{FX}/reports/valid/verified_removal.json"));
    let verdict = validate_fixture_case(&plan, &evidence, &report);
    assert_eq!(verdict.state, "verified");
    assert_eq!(verdict.effect, "removed");

    let noop_plan = load_json(&format!("{FX}/plans/valid/no_op_targets_absent.json"));
    let noop_evidence = load_json(&format!("{FX}/evidence/valid/already_satisfied_no_op.json"));
    let noop_report = load_json(&format!("{FX}/reports/valid/already_satisfied_no_op.json"));
    let noop_verdict = validate_fixture_case(&noop_plan, &noop_evidence, &noop_report);
    assert_eq!(noop_verdict.state, "verified");
    assert_eq!(noop_verdict.effect, "already_satisfied");
}

/// Happy-path sanity check: the untampered `verified_removal` and
/// `already_satisfied_no_op` triads must still agree with their frozen
/// report fixtures through THE single, fully-wired path -- schema +
/// bindings + destination/context + range transformation + rehashing +
/// full X.509 parsing + report-consistency, all of it. This guards
/// against the fuller validator becoming stricter than the frozen
/// contract itself.
#[test]
fn validate_fixture_case_agrees_with_the_frozen_report_fixtures_on_the_untampered_happy_paths() {
    let plan = load_json(&format!(
        "{FX}/plans/valid/mutating_removal_with_backup_required.json"
    ));
    let evidence = load_json(&format!("{FX}/evidence/valid/verified_removal.json"));
    let report = load_json(&format!("{FX}/reports/valid/verified_removal.json"));
    let verdict = validate_fixture_case(&plan, &evidence, &report);
    assert_eq!(verdict.state, report["postcondition"]["state"]);
    assert_eq!(verdict.effect, report["postcondition"]["effect"]);

    let noop_plan = load_json(&format!("{FX}/plans/valid/no_op_targets_absent.json"));
    let noop_evidence = load_json(&format!("{FX}/evidence/valid/already_satisfied_no_op.json"));
    let noop_report = load_json(&format!("{FX}/reports/valid/already_satisfied_no_op.json"));
    let noop_verdict = validate_fixture_case(&noop_plan, &noop_evidence, &noop_report);
    assert_eq!(noop_verdict.state, noop_report["postcondition"]["state"]);
    assert_eq!(noop_verdict.effect, noop_report["postcondition"]["effect"]);
}

/// Point 4: "real noop target absence" -- independently (via DER parsing,
/// not by trusting the plan's declared target_fingerprints) confirm the
/// no-op plan's before artifact genuinely contains ZERO occurrences of the
/// approved-removal targets (cert_A, cert_B), not just that the plan SAYS
/// count=0.
#[test]
fn noop_plans_before_artifact_genuinely_contains_zero_target_occurrences() {
    let plan = load_json(&format!("{FX}/plans/valid/no_op_targets_absent.json"));
    assert!(plan_is_noop(&plan));
    let before_bytes = read_artifact(
        plan["before_artifact_ref"]["fixture_path"]
            .as_str()
            .unwrap(),
    );
    let blocks = find_pem_blocks_strict(&before_bytes)
        .expect("before_targets_absent.pem.fixture must parse completely");
    let present_fingerprints: BTreeSet<String> =
        blocks.iter().map(|b| sha256_hex(&b.der)).collect();
    for range in plan["approved_removal_ranges"].as_array().unwrap() {
        let target = range["target_sha256"].as_str().unwrap();
        assert!(
            !present_fingerprints.contains(target),
            "no-op plan's before artifact must genuinely NOT contain target {target}, independently of what the plan declares"
        );
    }
}

/// Point 4: "unique nonconflicting snapshots" -- no two observations in the
/// same evidence bundle share a (step, step_snapshot_id) pair (each
/// snapshot id is scoped to its step and must be unique within it).
#[test]
fn every_evidence_bundle_has_unique_step_snapshot_ids_per_step() {
    for path in list_json_files(&format!("{FX}/evidence/valid")) {
        let evidence = load_json(&path);
        let mut seen: BTreeSet<(String, String)> = BTreeSet::new();
        for obs in evidence["observations"].as_array().unwrap() {
            let key = (
                obs["step"].as_str().unwrap().to_string(),
                obs["step_snapshot_id"].as_str().unwrap().to_string(),
            );
            assert!(
                seen.insert(key.clone()),
                "{path}: duplicate (step, step_snapshot_id) pair {key:?}"
            );
        }
    }
}

#[test]
fn evidence_with_wrong_schema_version_is_schema_invalid() {
    let validator = compiled(EVIDENCE_SCHEMA);
    let bad = load_json(&format!("{FX}/evidence/invalid/wrong_schema_version.json"));
    assert!(
        !validator.is_valid(&bad),
        "schema_version=2 must be rejected (schema pins const 1)"
    );
}

#[test]
fn backup_metadata_cannot_be_an_empty_object() {
    let validator = compiled(EVIDENCE_SCHEMA);
    let bad = load_json(&format!(
        "{FX}/evidence/invalid/backup_completed_with_empty_metadata.json"
    ));
    assert!(
        !validator.is_valid(&bad),
        "metadata={{}} (present but empty) must be rejected; owner/group/mode are all required"
    );
}
