#!/usr/bin/env bash
# Render the caro-demo video, place outputs in website/public/, and
# rewrite the baseline manifest so the next drift check is anchored to
# this render.
#
# Intended caller: a Claude Code session that has decided to re-render
# based on drift detection. The skill at .claude/skills/caro-demo-video/
# documents when this is appropriate.
#
# Does NOT commit — the caller is responsible for staging + commit +
# review, because the trust model varies (output-equivalent → ship;
# semantic change → open PR for review).
#
# Usage:
#   demos/remotion-video/scripts/render-and-ship.sh
#   demos/remotion-video/scripts/render-and-ship.sh --skip-install
set -euo pipefail

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_DIR="$SCRIPT_DIR/.."
REPO_ROOT="$PROJECT_DIR/../.."
MANIFEST="$PROJECT_DIR/.baseline-manifest.json"

cd "$PROJECT_DIR"

SKIP_INSTALL=0
[[ "${1:-}" == "--skip-install" ]] && SKIP_INSTALL=1

if [[ "$SKIP_INSTALL" == "0" ]]; then
  echo "==> Installing Remotion deps (use --skip-install if already done)…"
  npm install --no-audit --no-fund --prefer-offline
fi

echo "==> Rendering MP4 → website/public/caro-demo.mp4 …"
npx remotion render CaroDemo \
  ../../website/public/caro-demo.mp4 \
  --codec=h264 --crf=23 --image-format=jpeg --log=info

echo "==> Rendering poster → website/public/caro-demo-poster.png …"
npx remotion still CaroDemo \
  ../../website/public/caro-demo-poster.png \
  --frame=60 --image-format=png

# Portable SHA-256 helper.
sha256() {
  if command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$1" | cut -d' ' -f1
  else
    sha256sum "$1" | cut -d' ' -f1
  fi
}

echo "==> Refreshing baseline manifest…"
NOW=$(date -u +%Y-%m-%dT%H:%M:%SZ)
HEAD_COMMIT=$(cd "$REPO_ROOT" && git rev-parse --short HEAD 2>/dev/null || echo "unknown")
CARO_VERSION=$(grep -m1 '^version' "$REPO_ROOT/Cargo.toml" | sed 's/.*"\(.*\)".*/\1/')
PATTERN_COUNT=$(grep -c 'description:' "$REPO_ROOT/src/safety/patterns.rs")

# Rewrite the tripwire fields while preserving the comment and schema.
# jq's slurp+update keeps the existing structure intact; we only update
# rendered_at, rendered_commit, caro_version_at_render, pattern_count,
# and each watched file's sha256.
TMP=$(mktemp)
jq --arg now "$NOW" \
   --arg commit "$HEAD_COMMIT" \
   --arg version "$CARO_VERSION" \
   --argjson pcount "$PATTERN_COUNT" \
   '.rendered_at = $now
    | .rendered_commit = $commit
    | .caro_version_at_render = $version
    | .claims_baked_into_video.pattern_count = $pcount' \
   "$MANIFEST" > "$TMP"

# Update each watched file's sha256 in place.
COUNT=$(jq '.watched_files | length' "$TMP")
for i in $(seq 0 $((COUNT - 1))); do
  path=$(jq -r ".watched_files[$i].path" "$TMP")
  abs="$REPO_ROOT/$path"
  if [[ -f "$abs" ]]; then
    new_hash=$(sha256 "$abs")
    jq ".watched_files[$i].sha256 = \"$new_hash\"" "$TMP" > "$TMP.next" && mv "$TMP.next" "$TMP"
  fi
done

mv "$TMP" "$MANIFEST"

echo ""
echo "==> Done. Outputs:"
ls -lh "$REPO_ROOT/website/public/caro-demo.mp4" "$REPO_ROOT/website/public/caro-demo-poster.png" 2>&1
echo ""
echo "==> Baseline manifest refreshed to commit $HEAD_COMMIT at $NOW."
echo "==> Next step (for the caller): stage and commit:"
echo ""
echo "    git add website/public/caro-demo.mp4 website/public/caro-demo-poster.png \\"
echo "            demos/remotion-video/.baseline-manifest.json"
echo "    git commit -m \"chore(demo): re-render caro-demo video\""
echo ""
echo "    If you also changed source files in demos/remotion-video/src/,"
echo "    stage those too. If the scene text changed, open a PR for"
echo "    human review (see .claude/skills/caro-demo-video/ Trust Model)."
