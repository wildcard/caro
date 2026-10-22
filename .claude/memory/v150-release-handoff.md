# v1.5.0 Release — Handoff (final step blocked on credential scope)

**Created**: 2026-07-12 by the autonomous ops session.
**Status**: v1.5.0 fully staged and merged to `main`; only the tag
creation remains, blocked by this environment's credential scope.

## What is DONE (all merged to main, all green)

| PR | What | Merge commit |
|----|------|--------------|
| #1246 | Safety allowlist regression fix + catastrophic-floor hardening (5 adversarial review rounds, all test-pinned) + dependency drift repair (ethnum/rust-1.97, RUSTSEC-2026-0204/0185, quick-xml suppressions) + rust-1.97 clippy | 934776b |
| #1301 | Decision record D1-D5, first weekly demo report, Tier-2 feature-evidence rule, COMPANY.md decision log | 0d18e34 |
| #1304 | v1.5.0 release commit (6-file checklist) + NuGet installer versioned-asset 404 fix | ff320a6 |
| #1306 | tag-release.yml + publish.yml dispatch trigger (release automation for token-restricted envs) | c36d129 |

- `Cargo.toml` on `main` is `version = "1.5.0"`.
- `CHANGELOG.md` has the `## [1.5.0] - 2026-07-12` entry (becomes the GH Release body).
- Filed #1302 (shell-lexer rewrite) follow-up; milestone #3 alignment fixed (#1151/#1152/#1075).

## What is BLOCKED and why

Creating/pushing the `v1.5.0` tag. This session's credentials fail BOTH paths:

1. `git push origin v1.5.0` -> HTTP 403 on all tag refs (branch pushes
   work; verified with a non-v* test tag, so it's tag-ref creation as a
   whole, not a v* ruleset).
2. `gh workflow run tag-release.yml` (the automation added in #1306) ->
   403 "Resource not accessible by integration" — the GitHub App
   integration lacks `actions: write`.

Both are session-credential scopes, not repo config. Per decision D5's
documented hard-limits list, tag/publish is an owner-gated step.

## How to finish (any ONE of these)

**A. Plain git (owner laptop / any push-capable credential):**
```
git fetch origin main
git tag -a v1.5.0 -m "v1.5.0" ff320a6342728a301a533ca346d6dfb74a408341
git push origin v1.5.0        # fires publish.yml -> crates.io -> release.yml
```

**B. The automation workflow (any credential/UI with actions:write):**
```
gh workflow run tag-release.yml --ref main \
  -f tag=v1.5.0 -f sha=ff320a6342728a301a533ca346d6dfb74a408341
# then (a GITHUB_TOKEN-pushed tag can't fire publish's push trigger):
gh workflow run publish.yml --ref main -f tag=v1.5.0
```

**C. GitHub UI:** create a Release named `v1.5.0` targeting commit
`ff320a6` — that creates the tag and (via the push trigger) starts publish.

## After the tag lands (fully automatic)

publish.yml -> crates.io, then release.yml chains via workflow_run and
creates the GitHub Release with binaries. If Publish fails with 403, the
`CARGO_REGISTRY_TOKEN` secret has expired — rotate it and
`gh run rerun <run-id> --failed` (see release-version-alignment.md).

## Verify
```
gh release view v1.5.0     # binary assets + CHANGELOG body
```
