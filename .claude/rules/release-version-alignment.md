# Release Version Alignment

**APPLIES TO**: Every release PR that bumps `Cargo.toml` version.

Codified from the caro v1.3.0 release (2026-04-20), which shipped with README
saying "1.1.1", homebrew tap example showing `VERSION=1.1.0`, and nuget
installer defaulting to `1.1.0`. A follow-up alignment PR (#864) was required —
this rule prevents that.

## Rule

**Every release PR must update all version references and downstream narrative
artifacts in a single commit.** A version bump in `Cargo.toml` without the
matching README/ROADMAP/install-script updates is incomplete.

## The 6-file Checklist

Every `chore(release): vX.Y.Z` PR must touch these files:

| # | File | What to update |
|---|------|----------------|
| 1 | `Cargo.toml` | `version = "X.Y.Z"` |
| 2 | `Cargo.lock` | Regenerate via `cargo check --no-default-features --features embedded-cpu` — commit the lockfile diff |
| 3 | `CHANGELOG.md` | New `## [X.Y.Z] - YYYY-MM-DD` entry with Added / Changed / Fixed / Security / Internal subsections (Keep a Changelog format) |
| 4 | `README.md` | `**Current Version:** X.Y.Z` banner line — plus any other version strings in the landing section |
| 5 | `ROADMAP.md` | (a) `**Last Updated**: <today>`; (b) Status table row marked ✅ RELEASED with the release date; (c) New `### 🎉 vX.Y.Z - <headline>` milestone section prepended to `## Release Milestones` |
| 6 | Install-script defaults | `homebrew-tap/README.md` checksum snippet `VERSION=X.Y.Z`; `nuget/tools/install.ps1` `[string]$Version = "X.Y.Z"`; any `scripts/install.sh` fallback version |

## After-Merge GitHub Release

The release PR merge is step 1 of 2. Step 2 is the GitHub release itself:

1. Tag the merge commit: `git tag -a vX.Y.Z -m "vX.Y.Z" <merge-sha>` and push.
   (If `tag.gpgSign = true` but no GPG key is loaded, pass `--no-sign` —
   unless the user's request was explicitly `-s`.)
2. The push triggers `.github/workflows/publish.yml`, which uploads to
   crates.io. If it fails with `403 Forbidden`, the `CARGO_REGISTRY_TOKEN`
   secret has expired — rotate it in GitHub repo settings, then
   `gh run rerun <run-id> --failed`.
3. On Publish success, `.github/workflows/release.yml` chains via
   `workflow_run` and creates the GitHub Release with binary assets. If it
   doesn't auto-trigger, dispatch manually:
   `gh workflow run release.yml --ref vX.Y.Z -f tag=vX.Y.Z`.
   Do NOT dispatch Release manually *before* crates.io has the version —
   the workflow's verification poll will fail.
4. Verify: `gh release view vX.Y.Z` shows binary assets and a body mirroring
   the CHANGELOG entry.

## Separation of Concerns

- **Feature PRs** carry the actual code changes. Merge into `main` BEFORE
  opening the release PR.
- **Release PR** (`chore(release): vX.Y.Z`) contains only the 6 files above.
  Small, mechanical, fast to review.
- **Post-release alignment PR** is only needed when drift is discovered
  *after* merging the release PR. Branch off `main`, not off the merged
  release branch. See [#864](https://github.com/wildcard/caro/pull/864) for
  precedent.

## Common Failure Modes

| Failure | Missing file | Fix |
|---|---|---|
| README landing page shows old version | #4 | Alignment PR |
| Homebrew tap example shows old `VERSION=` | #6 | Alignment PR |
| ROADMAP still marks release as "In Progress" | #5 | Alignment PR |
| crates.io has vX.Y.Z but no GitHub release | — | `gh workflow run release.yml --ref vX.Y.Z -f tag=vX.Y.Z` |
| Publish fails with 403 | — | Rotate `CARGO_REGISTRY_TOKEN`, rerun failed jobs |

## Why This Matters

The README, homebrew formula, and install scripts are a user's first
touchpoint — not `Cargo.toml`. If those show an old version while the actual
release is newer, the user's mental model is wrong on day one: they file bugs
against the wrong version, copy outdated install commands, and lose trust.

Six files is small enough to grep and large enough to forget one. The rule
exists so the grep is a checklist instead of a bug report.

## See Also

- [`~/.claude/rules/release-version-alignment.md`](../../.claude/rules/release-version-alignment.md) — identical rule, loaded globally across all projects
- `/caro.release.prepare` skill — operational wrapper that should reference this checklist
