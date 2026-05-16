#!/usr/bin/env bash
# Detect drift between the shipped caro-demo video and the source files
# it depends on. Reads .baseline-manifest.json (captured at the last
# successful render) and compares each watched file's current SHA-256.
#
# Exit codes:
#   0  No drift — video is up to date.
#   1  Drift detected — re-render warranted. See JSON output for details.
#   2  Tooling error (jq missing, manifest missing, etc.).
#
# Output: JSON on stdout. The CI workflow and the agent both parse this.
#
# Usage:
#   demos/remotion-video/scripts/check-drift.sh
#   demos/remotion-video/scripts/check-drift.sh --human   (pretty-print)
set -euo pipefail

# Resolve script-relative paths so it works from any cwd.
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_DIR="$SCRIPT_DIR/.."
REPO_ROOT="$PROJECT_DIR/../.."
MANIFEST="$PROJECT_DIR/.baseline-manifest.json"

HUMAN=0
[[ "${1:-}" == "--human" ]] && HUMAN=1

if ! command -v jq >/dev/null 2>&1; then
  echo '{"error":"jq not installed"}' >&2
  exit 2
fi

if [[ ! -f "$MANIFEST" ]]; then
  echo "{\"error\":\"manifest not found at $MANIFEST\"}" >&2
  exit 2
fi

# Portable SHA-256 — shasum is on macOS, sha256sum on Linux.
sha256() {
  if command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$1" | cut -d' ' -f1
  else
    sha256sum "$1" | cut -d' ' -f1
  fi
}

# Build a JSON array of drift entries. Each entry has: path, expected_sha,
# actual_sha, purpose, status (DRIFTED | MISSING | OK).
ENTRIES=$(jq -c '.watched_files[]' "$MANIFEST")
DRIFT_REPORT='[]'
HAS_DRIFT=0

while IFS= read -r entry; do
  path=$(jq -r '.path' <<<"$entry")
  expected=$(jq -r '.sha256' <<<"$entry")
  purpose=$(jq -r '.purpose' <<<"$entry")
  abs="$REPO_ROOT/$path"

  if [[ ! -f "$abs" ]]; then
    status="MISSING"
    actual="null"
    HAS_DRIFT=1
  else
    actual=$(sha256 "$abs")
    if [[ "$actual" == "$expected" ]]; then
      status="OK"
    else
      status="DRIFTED"
      HAS_DRIFT=1
    fi
  fi

  DRIFT_REPORT=$(jq --arg path "$path" \
                    --arg status "$status" \
                    --arg expected "$expected" \
                    --arg actual "$actual" \
                    --arg purpose "$purpose" \
                    '. + [{path: $path, status: $status, expected_sha: $expected, actual_sha: $actual, purpose: $purpose}]' \
                    <<<"$DRIFT_REPORT")
done <<<"$ENTRIES"

RENDERED_AT=$(jq -r '.rendered_at' "$MANIFEST")
RENDERED_COMMIT=$(jq -r '.rendered_commit' "$MANIFEST")

OUTPUT=$(jq -n \
  --arg rendered_at "$RENDERED_AT" \
  --arg rendered_commit "$RENDERED_COMMIT" \
  --argjson has_drift "$HAS_DRIFT" \
  --argjson files "$DRIFT_REPORT" \
  '{
    drift_detected: ($has_drift == 1),
    last_rendered_at: $rendered_at,
    last_rendered_commit: $rendered_commit,
    files: $files,
    summary: {
      total: ($files | length),
      ok: ($files | map(select(.status == "OK")) | length),
      drifted: ($files | map(select(.status == "DRIFTED")) | length),
      missing: ($files | map(select(.status == "MISSING")) | length)
    }
  }')

if [[ "$HUMAN" == "1" ]]; then
  echo "$OUTPUT" | jq -r '
    "caro-demo drift check",
    "  last rendered: \(.last_rendered_at) (commit \(.last_rendered_commit))",
    "  total watched: \(.summary.total)  ok: \(.summary.ok)  drifted: \(.summary.drifted)  missing: \(.summary.missing)",
    "",
    (.files[] |
      if .status == "OK" then
        "  ✓ \(.path)"
      elif .status == "MISSING" then
        "  ✗ MISSING: \(.path)\n      └─ \(.purpose)"
      else
        "  ⚠ DRIFTED: \(.path)\n      └─ \(.purpose)"
      end
    )'
else
  echo "$OUTPUT"
fi

if [[ "$HAS_DRIFT" == "1" ]]; then
  exit 1
fi
exit 0
