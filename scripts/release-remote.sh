#!/usr/bin/env bash
# Helpers for the opt-in `release-remote` workflow (#1081).
#
# The workflow builds a second binary per platform with
# `--features remote-backends` and names it `caro-<version>-<platform>-remote`
# (`-remote.exe` on Windows) plus a `.sha256` file. Every check lives here so
# that tests/release_remote_contract.rs can run it against fixtures, offline,
# on Linux and macOS CI. Nothing here publishes anything except `upload`, which
# the workflow reaches only when `publish` is true.
#
# Written for bash 3.2 (macOS): no arrays, no mapfile, no associative arrays.
set -euo pipefail

PLATFORMS="linux-amd64 linux-arm64 macos-intel macos-silicon windows-amd64"
BACKENDS="ollama exo vllm mesh ai-horde hybrid grok claude openrouter"
# No build metadata (`+`): GitHub may rewrite it in asset names, which would defeat the
# name checks in release-state.
VERSION_RE='^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?$'

die() {
  echo "::error::$*" >&2
  exit 1
}

is_platform() {
  for p in $PLATFORMS; do
    [ "$p" = "$1" ] && return 0
  done
  return 1
}

check_version() {
  [[ "$1" =~ $VERSION_RE ]] || die "not a version: '$1'"
}

sha256_of() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{print $1}'
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$1" | awk '{print $1}'
  elif command -v certUtil >/dev/null 2>&1; then
    certUtil -hashfile "$1" SHA256 | grep -v ":" | tr -d '[:space:]'
  else
    die "no sha256 tool found (sha256sum, shasum, certUtil)"
  fi
}

remote_name() { # version platform
  check_version "$1"
  is_platform "$2" || die "unknown platform: '$2'"
  case "$2" in
    windows-*) echo "caro-$1-$2-remote.exe" ;;
    *) echo "caro-$1-$2-remote" ;;
  esac
}

cargo_version_at() { # repo commit
  git -C "$1" show "$2:Cargo.toml" |
    awk '/^\[package\]/{p=1;next} /^\[/{p=0} p && /^version[[:space:]]*=/{gsub(/^[^"]*"|".*$/,""); print; exit}'
}

# pin <ref_type> <ref_name> <sha> <input_tag> <repo>
# Runs before any build, in the single checkout of the dispatched commit. The
# workflow must have been started from the release tag itself, the operator's
# `tag` input must repeat that tag, and the checked-out commit must be the
# dispatched commit, the commit the tag points to now, and a source tree whose
# Cargo.toml version equals the tag. Nothing is fetched or checked out from an
# input. Prints tag=, version= and commit= lines.
cmd_pin() {
  local ref_type=$1 ref_name=$2 sha=$3 input_tag=$4 repo=$5 version head tagged cargo
  [ "$ref_type" = "tag" ] ||
    die "start this workflow from the release tag, not from a $ref_type ('$ref_name')"
  [[ "$ref_name" == v* ]] || die "tag must start with 'v': '$ref_name'"
  [ "$input_tag" = "$ref_name" ] ||
    die "the tag input does not match the tag this run was started from ('$ref_name')"
  version=${ref_name#v}
  check_version "$version"
  [[ "$sha" =~ ^[0-9a-f]{40}$ ]] || die "dispatched commit is not a full SHA: '$sha'"
  head=$(git -C "$repo" rev-parse --verify HEAD 2>/dev/null) || die "no checked-out commit in $repo"
  [ "$head" = "$sha" ] || die "checked-out commit $head is not the dispatched commit $sha"
  tagged=$(git -C "$repo" rev-parse --verify "refs/tags/${ref_name}^{commit}" 2>/dev/null) ||
    die "tag '$ref_name' does not exist in $repo"
  [ "$tagged" = "$sha" ] || die "tag $ref_name moved: it points at $tagged, this run is $sha"
  cargo=$(cargo_version_at "$repo" "$head")
  [ "$cargo" = "$version" ] ||
    die "tag $ref_name is version $version but Cargo.toml at $head says '$cargo'"
  echo "tag=$ref_name"
  echo "version=$version"
  echo "commit=$head"
}

# cve_sources <rules_dir>: the source URL of every CVE rule the build compiles
# in: data/cve_rules/CVE-*.yaml, minus the draft sentinel (the compiler skips
# those, and also EXAMPLE-TEMPLATE.yaml, which does not start with CVE-).
cve_sources() {
  local dir=$1 f
  for f in "$dir"/CVE-*.yaml; do
    [ -f "$f" ] || continue
    if grep -q 'TODO_NIMBLE_AUTHORED' "$f"; then continue; fi
    sed -n 's/^source:[[:space:]]*\([^[:space:]#]*\).*$/\1/p' "$f" | head -n 1
  done
}

# preflight <rules_dir> <target_dir>
# Runs before any build (#1652). build.rs reads data/cve_rules through a
# compile-time path and writes an empty ruleset, without a warning, when that
# directory is missing; a reused build directory can also hold a build script
# compiled in another checkout. So: there must be rules to compile, and the
# build directory must not exist yet.
cmd_preflight() {
  local rules=$1 target=$2 n
  [ -d "$rules" ] || die "no CVE rules: $rules does not exist, so the build would compile an empty ruleset"
  n=$(cve_sources "$rules" | grep -c . || true)
  [ "$n" -ge 1 ] || die "no CVE rules to compile in $rules (only drafts or the template?)"
  [ ! -e "$target" ] ||
    die "build directory '$target' is not fresh; a cached build script could point at another checkout"
  echo "preflight ok: $n compiled CVE rule(s) in $rules; build directory '$target' is fresh"
}

# cve_static <asset>: every compiled rule's source URL must be in the binary.
# The URLs occur in the compiled ruleset only, so a build that compiled fewer
# rules than data/cve_rules holds fails. This is the only CVE check the
# cross-built linux-arm64 binary gets: it shows the ruleset is in the file, not
# that the validator behaves on the target.
cve_static() {
  local asset=$1 rules=${CARO_CVE_RULES_DIR:-data/cve_rules} srcs u
  srcs=$(cve_sources "$rules")
  [ -n "$srcs" ] || die "no CVE rules to check the binary against in $rules"
  while IFS= read -r u; do
    [ -n "$u" ] || continue
    grep -aqF -- "$u" "$asset" ||
      die "CVE ruleset: source $u not found in the binary; it compiled fewer rules than $rules holds (stale build script or missing rules directory?)"
  done <<<"$srcs"
}

# cve_guard <asset>: run the compiled binary's validator, never the command.
# `caro guard` reads a hook payload on stdin and prints a verdict; the string
# below is data for the validator and is never executed. A benign control keeps
# a guard that denies everything from passing.
cve_guard() {
  local asset=$1 home out rc=0
  home=$(mktemp -d "${TMPDIR:-/tmp}/caro-smoke.XXXXXX")
  guard_run() {
    printf '%s' "$1" |
      env -i PATH="$PATH" HOME="$home" XDG_CONFIG_HOME="$home" XDG_DATA_HOME="$home" \
        XDG_CACHE_HOME="$home" HF_HUB_OFFLINE=1 \
        "$asset" guard --harness generic --mode enforce --no-log 2>&1
  }
  out=$(guard_run '{"command":"xz --lzma1=preset=9 file.txt"}') || rc=$?
  { [ "$rc" = 2 ] && grep -q '"verdict":"deny"' <<<"$out"; } ||
    die "CVE-2024-3094 trigger was not blocked by 'caro guard' (exit $rc): $out"
  grep -qi 'cve-2024-3094' <<<"$out" ||
    die "guard verdict does not cite cve-2024-3094 (update this check if the rule moved): $out"
  rc=0
  out=$(guard_run '{"command":"ls -la"}') || rc=$?
  { [ "$rc" = 0 ] && grep -q '"verdict":"none"' <<<"$out"; } ||
    die "benign control was denied or errored (exit $rc): $out"
}

# name <version> <platform>
cmd_name() {
  remote_name "$1" "$2"
}

# prepare <version> <platform> <binary> <outdir> <tag> <commit>
# Copies the binary under its -remote name, writes `<name>.sha256` and a
# `<name>.manifest` (tag, commit and checksum, kept in the workflow artifact
# and never uploaded to the release). Prints asset=<name>.
cmd_prepare() {
  local version=$1 platform=$2 bin=$3 out=$4 tag=$5 commit=$6 name hash
  name=$(remote_name "$version" "$platform")
  [ -f "$bin" ] || die "binary not found: $bin"
  [ -s "$bin" ] || die "binary is empty: $bin"
  mkdir -p "$out"
  cp "$bin" "$out/$name"
  chmod +x "$out/$name"
  hash=$(sha256_of "$out/$name")
  echo "$hash  $name" >"$out/$name.sha256"
  {
    echo "tag=$tag"
    echo "version=$version"
    echo "commit=$commit"
    echo "platform=$platform"
    echo "asset=$name"
    echo "sha256=$hash"
  } >"$out/${name%.exe}.manifest"
  echo "asset=$name"
}

# smoke <asset> <target>
# Native targets: run `--backend-info`; it must list all nine remote backends
# and report none as "not compiled". The cross-built linux-arm64 binary cannot
# run on the runner, so it gets a static check on two strings that differ
# between the builds.
cmd_smoke() {
  local asset=$1 target=$2 out b
  [ -f "$asset" ] || die "asset not found: $asset"
  if [ "$target" = "aarch64-unknown-linux-gnu" ]; then
    grep -aq 'ANTHROPIC_API_KEY environment variable not set' "$asset" ||
      die "arm64 binary lacks the remote-backends missing-key message"
    if grep -aq 'which is not compiled into this build' "$asset"; then
      die "arm64 binary carries the default build's 'not compiled' message"
    fi
    cve_static "$asset"
    echo "static check passed (not run: cross-built; CVE ruleset checked in the file only)"
    return 0
  fi
  case "$asset" in */*) ;; *) asset=./$asset ;; esac
  out=$("$asset" --backend-info </dev/null 2>&1) || {
    echo "$out"
    die "--backend-info exited non-zero"
  }
  echo "$out"
  for b in $BACKENDS; do
    grep -Eq "^[[:space:]]*${b}[[:space:]]" <<<"$out" ||
      die "--backend-info printed no $b row"
  done
  if grep -q "not compiled" <<<"$out"; then
    die "remote-backends asset reports a backend as 'not compiled'"
  fi
  cve_guard "$asset"
  cve_static "$asset"
  echo "CVE ruleset ok: trigger denied and cited, benign control allowed, rule sources present"
  echo "smoke passed"
}

platform_of() { # file name (binary or .sha256) + version -> platform, or empty
  local f=$1 version=$2 p
  for p in $PLATFORMS; do
    case "$f" in
      "caro-$version-$p-remote" | "caro-$version-$p-remote.exe" | \
        "caro-$version-$p-remote.sha256" | "caro-$version-$p-remote.exe.sha256")
        echo "$p"
        return 0
        ;;
    esac
  done
  return 0
}

# verify <dir> <version> [platform...]
# Every file must be a -remote asset or manifest; every expected platform must
# have both the binary and its .sha256; each checksum must match the binary.
# Default platforms: all five.
cmd_verify() {
  local dir=$1 version=$2 expected f base p name sums line got hash n=0
  shift 2
  expected=${*:-$PLATFORMS}
  check_version "$version"
  [ -d "$dir" ] || die "not a directory: $dir"
  for f in "$dir"/*; do
    [ -e "$f" ] || die "no files in $dir"
    base=$(basename "$f")
    case "$base" in *.manifest) continue ;; esac
    p=$(platform_of "$base" "$version")
    [ -n "$p" ] || die "unexpected file in $dir: $base (not a $version -remote asset)"
    case " $expected " in *" $p "*) ;; *) die "unexpected platform $p in $dir: $base" ;; esac
  done
  for p in $expected; do
    is_platform "$p" || die "unknown platform: '$p'"
    name=$(remote_name "$version" "$p")
    sums=$dir/$name.sha256
    if [ -f "$dir/$name" ] && [ ! -f "$sums" ]; then
      die "partial pair for $p: $name has no .sha256"
    fi
    if [ ! -f "$dir/$name" ] && [ -f "$sums" ]; then
      die "partial pair for $p: .sha256 without $name"
    fi
    [ -f "$dir/$name" ] || die "missing asset for $p: $name"
    [ -s "$dir/$name" ] || die "empty asset for $p: $name"
    [ "$(wc -l <"$sums" | tr -d ' ')" = "1" ] || die "$name.sha256 must have exactly one line"
    line=$(cat "$sums")
    got=${line%% *}
    { [[ "$got" =~ ^[0-9a-f]{64}$ ]] && [ "$line" = "$got  $name" ]; } ||
      die "$name.sha256 is not '<sha256>  $name': $line"
    hash=$(sha256_of "$dir/$name")
    [ "$got" = "$hash" ] ||
      die "checksum mismatch for $name: file is $hash, .sha256 says $got"
    n=$((n + 1))
  done
  echo "verified $n pair(s)"
}

# identity <dir> <tag> <commit> <version> [platform...]
# Every manifest must carry the resolved tag, commit and version, and the
# checksum it recorded.
cmd_identity() {
  local dir=$1 tag=$2 commit=$3 version=$4 expected p name m got n=0
  shift 4
  expected=${*:-$PLATFORMS}
  for p in $expected; do
    name=$(remote_name "$version" "$p")
    m=$dir/${name%.exe}.manifest
    [ -f "$m" ] || die "no manifest for $p"
    for kv in "tag=$tag" "version=$version" "commit=$commit" "platform=$p" "asset=$name"; do
      grep -qxF "$kv" "$m" || die "manifest for $p lacks '$kv'"
    done
    got=$(sed -n 's/^sha256=//p' "$m")
    [ "$got" = "$(sha256_of "$dir/$name")" ] || die "manifest checksum differs from $name"
    n=$((n + 1))
  done
  echo "identity ok for $n platform(s): $tag $commit"
}

# release-assets <tag>: asset names of the published release (never a draft).
cmd_release_assets() {
  gh api "repos/${GITHUB_REPOSITORY:?GITHUB_REPOSITORY is not set}/releases/tags/$1" \
    --jq '.assets[].name'
}

# release-state <version> [--expect absent|complete] < asset names
# Prints one `<platform> absent|partial|complete` line per platform. Exits
# non-zero if any platform is partial, or differs from --expect (default absent).
cmd_release_state() {
  local version=$1 expect=absent names p name bin sum state bad=0 partial=0
  shift
  if [ "${1:-}" = "--expect" ]; then expect=${2:?--expect needs a value}; fi
  case "$expect" in absent | complete) ;; *) die "--expect must be absent or complete" ;; esac
  names=$(cat)
  for p in $PLATFORMS; do
    name=$(remote_name "$version" "$p")
    bin=0
    sum=0
    grep -qxF "$name" <<<"$names" && bin=1
    grep -qxF "$name.sha256" <<<"$names" && sum=1
    if [ "$bin" = 1 ] && [ "$sum" = 1 ]; then
      state=complete
    elif [ "$bin" = 0 ] && [ "$sum" = 0 ]; then
      state=absent
    else
      state=partial
      partial=1
    fi
    echo "$p $state"
    [ "$state" = "$expect" ] || bad=1
  done
  local repair="remove every caro-$version-*-remote* asset from the release by hand, then re-run"
  [ "$partial" = 0 ] || die "partial binary/checksum pair on the release; $repair"
  [ "$bad" = 0 ] || die "release assets are not all '$expect'; if -remote assets are left over from an interrupted publish, $repair"
}

# upload <dir> <tag> <version>
# Never overwrites (no --clobber) and never creates or deletes a release.
cmd_upload() {
  local dir=$1 tag=$2 version=$3 p name
  for p in $PLATFORMS; do
    name=$(remote_name "$version" "$p")
    gh release upload "$tag" "$dir/$name" "$dir/$name.sha256"
  done
}

usage() {
  echo "usage: $0 <pin|preflight|name|prepare|smoke|verify|identity|release-assets|release-state|upload> ..." >&2
  exit 2
}

[ $# -ge 1 ] || usage
sub=$1
shift
case "$sub" in
  pin) [ $# -eq 5 ] || usage; cmd_pin "$@" ;;
  preflight) [ $# -eq 2 ] || usage; cmd_preflight "$@" ;;
  name) [ $# -eq 2 ] || usage; cmd_name "$@" ;;
  prepare) [ $# -eq 6 ] || usage; cmd_prepare "$@" ;;
  smoke) [ $# -eq 2 ] || usage; cmd_smoke "$@" ;;
  verify) [ $# -ge 2 ] || usage; cmd_verify "$@" ;;
  identity) [ $# -ge 4 ] || usage; cmd_identity "$@" ;;
  release-assets) [ $# -eq 1 ] || usage; cmd_release_assets "$@" ;;
  release-state) [ $# -ge 1 ] || usage; cmd_release_state "$@" ;;
  upload) [ $# -eq 3 ] || usage; cmd_upload "$@" ;;
  *) usage ;;
esac
