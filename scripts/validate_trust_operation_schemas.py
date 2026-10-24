#!/usr/bin/env python3
"""Independent conformance check for the trust-operation v1 contract
(issue #1714, checkpoint 1; independent review on PR #1715, point 1 of the
round-4 review).

tests/trust_operation_v1_contract.rs validates fixtures with a
hand-written, dependency-free draft-07-SUBSET evaluator (module
`minischema`) because no Rust `$ref`-capable JSON Schema crate clears this
repository's `dependency-review.yml` license allowlist (see that module's
doc comment for the full explanation). A hand-written evaluator can be
wrong in ways a spec implementation would catch -- the review's point 1
example: a schema with `"type": 42` would silently pass `minischema`
because it only recognizes `"type"` as a string or array, never checking
that the value itself is well-formed.

This script is the independent check that closes that gap, using the real
`jsonschema` PyPI package (not a Rust/Cargo dependency, so it is outside
`dependency-review.yml`'s scope, which only watches `**/Cargo.toml`,
`**/Cargo.lock`, `**/package.json`, `**/package-lock.json`, and
`**/pnpm-lock.yaml`; see .github/workflows/dependency-review.yml):

1. Meta-validates all four schema files against the official JSON Schema
   draft-07 meta-schema (`jsonschema.Draft7Validator.check_schema`). This
   rejects a malformed schema outright -- bad keyword types, unsupported
   keyword values -- before any fixture is even checked against it.
2. Validates every fixture under tests/fixtures/trust_operation_v1/ against
   its schema with the real validator, and asserts the result matches the
   fixture's own valid/ vs invalid/ directory placement -- the same
   ground truth tests/trust_operation_v1_contract.rs computes with
   `minischema`, computed independently here so the two must agree.
   A small, named set of fixtures is schema-valid by design (the
   Rust-only checks documented inline in the schemas, e.g. count vs
   byte_ranges.length); this script's SKIP_SCHEMA_ONLY_LIMITATIONS set
   mirrors the skip lists in the Rust test exactly, by filename.

This script does not implement any detector/gate/verifier behavior and
does not touch a real trust store. It is an offline, read-only check.
"""

from __future__ import annotations

import glob
import json
import os
import sys

try:
    import jsonschema
except ImportError:
    print(
        "ERROR: the 'jsonschema' PyPI package is required to run this script.\n"
        "Install it with: pip install jsonschema\n"
        "(This is a Python dev-tool dependency, not a Rust/Cargo dependency --\n"
        "it is not tracked by .github/workflows/dependency-review.yml.)",
        file=sys.stderr,
    )
    sys.exit(2)

REPO_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
FX = os.path.join(REPO_ROOT, "tests", "fixtures", "trust_operation_v1")

SCHEMAS = {
    "manifest": os.path.join(FX, "schemas", "manifest.schema.json"),
    "plan": os.path.join(FX, "schemas", "plan.schema.json"),
    "evidence": os.path.join(FX, "schemas", "evidence.schema.json"),
    "report": os.path.join(FX, "schemas", "report.schema.json"),
}

FIXTURE_DIRS = {
    "plan": [
        (os.path.join(FX, "plans", "valid"), True),
        (os.path.join(FX, "plans", "invalid"), False),
    ],
    "evidence": [
        (os.path.join(FX, "evidence", "valid"), True),
        (os.path.join(FX, "evidence", "invalid"), False),
    ],
    "report": [
        (os.path.join(FX, "reports", "valid"), True),
        (os.path.join(FX, "reports", "invalid"), False),
    ],
}

# Each entry here MUST have a matching named test in
# tests/trust_operation_v1_contract.rs that explains, and separately
# checks, why this specific fixture is schema-valid despite living under
# an invalid/ directory (a limitation of single-document JSON Schema
# validation, not an oversight). Keep this set exactly in sync with the
# `skip` lists passed to `assert_all_invalid` in that file.
SCHEMA_ONLY_LIMITATIONS = {
    "count_byte_ranges_mismatch.json",
    "overlapping_byte_ranges.json",
    "unrelated_target_fingerprint.json",
    "scope_binding_digest_does_not_match_preimage.json",
}


def load(path: str):
    with open(path, encoding="utf-8") as f:
        return json.load(f)


def conformance_vectors() -> list[str]:
    """Independent conformance vectors (#1715 round-4 review point 1): prove
    the REAL validator rejects malformed schema forms that a hand-written
    evaluator could plausibly accept. These are deliberately malformed
    COPIES of a real schema fragment, never applied to the actual schema
    files on disk.
    """
    failures = []
    base = load(SCHEMAS["report"])

    bad_type = json.loads(json.dumps(base))
    bad_type["properties"]["schema_version"] = {"type": 42}
    try:
        jsonschema.Draft7Validator.check_schema(bad_type)
        failures.append("conformance vector 'type: 42' was NOT rejected by the real validator (it should be)")
    except jsonschema.exceptions.SchemaError:
        pass  # expected: a non-string/array `type` value is invalid draft-07

    bad_min_length = json.loads(json.dumps(base))
    bad_min_length["definitions"]["authorization"]["properties"]["scope_binding_digest"] = {"minLength": "not-a-number"}
    try:
        jsonschema.Draft7Validator.check_schema(bad_min_length)
        failures.append("conformance vector 'minLength: \"not-a-number\"' was NOT rejected by the real validator (it should be)")
    except jsonschema.exceptions.SchemaError:
        pass

    bad_required = json.loads(json.dumps(base))
    bad_required["required"] = "schema_version"  # must be an array, not a bare string
    try:
        jsonschema.Draft7Validator.check_schema(bad_required)
        failures.append("conformance vector 'required: <string>' was NOT rejected by the real validator (it should be)")
    except jsonschema.exceptions.SchemaError:
        pass

    return failures


def main() -> int:
    failures = []

    print("== Independent conformance vectors (malformed-schema rejection) ==")
    vector_failures = conformance_vectors()
    if vector_failures:
        failures.extend(vector_failures)
        for f in vector_failures:
            print(f"FAIL  {f}")
    else:
        print("OK    all 3 malformed-schema conformance vectors were correctly rejected")
    print()

    print("== Meta-validating all 4 schemas against draft-07 ==")
    compiled = {}
    for name, path in SCHEMAS.items():
        schema = load(path)
        try:
            jsonschema.Draft7Validator.check_schema(schema)
        except jsonschema.exceptions.SchemaError as e:
            failures.append(f"{path} is not a valid draft-07 schema: {e}")
            print(f"FAIL  {path}: {e}")
            continue
        compiled[name] = jsonschema.Draft7Validator(schema)
        print(f"OK    {path}")

    if len(compiled) != len(SCHEMAS):
        print("\nAborting fixture validation: not all schemas are valid draft-07.")
        for f in failures:
            print(f" - {f}")
        return 1

    print("\n== Validating fixtures against their schema (real jsonschema, not minischema) ==")
    manifest = load(os.path.join(FX, "manifest.json"))
    manifest_result = list(compiled["manifest"].iter_errors(manifest))
    if manifest_result:
        failures.append(f"manifest.json failed metaschema validation: {manifest_result[0]}")
        print(f"FAIL  manifest.json: {manifest_result[0]}")
    else:
        print("OK    manifest.json")

    for kind, dirs in FIXTURE_DIRS.items():
        validator = compiled[kind]
        for directory, want_valid in dirs:
            for path in sorted(glob.glob(os.path.join(directory, "*.json"))):
                name = os.path.basename(path)
                instance = load(path)
                errors = list(validator.iter_errors(instance))
                is_valid = len(errors) == 0

                if is_valid == want_valid:
                    print(f"OK    {os.path.relpath(path, REPO_ROOT)}")
                    continue

                if not want_valid and is_valid and name in SCHEMA_ONLY_LIMITATIONS:
                    print(f"OK*   {os.path.relpath(path, REPO_ROOT)} (documented schema-only limitation, see tests/trust_operation_v1_contract.rs)")
                    continue

                failures.append(
                    f"{os.path.relpath(path, REPO_ROOT)}: expected valid={want_valid}, "
                    f"got valid={is_valid}" + (f" ({errors[0]})" if errors else "")
                )
                print(f"FAIL  {os.path.relpath(path, REPO_ROOT)}: expected valid={want_valid}, got valid={is_valid}")

    print()
    if failures:
        print(f"{len(failures)} failure(s):")
        for f in failures:
            print(f" - {f}")
        return 1

    print("All schemas are valid draft-07, and every fixture's real-jsonschema verdict matches its valid/invalid placement.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
