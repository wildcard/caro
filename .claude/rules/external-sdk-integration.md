# External SDK Integration

**APPLIES TO**: Adding any non-trivial external SDK dependency that will touch
multiple modules (e.g. agentmesh/AGT, MCP client SDK, observability framework,
identity provider SDK).

Codified from the agent-governance-toolkit integration ([PR #1103](https://github.com/wildcard/caro/pull/1103),
2026-05-16), where a 50-LOC spike PR de-risked the full 5-phase architectural
plan in under an hour by catching license/MSRV/transitive-dep concerns before
any wrapping design effort.

## Rule

When integrating a non-trivial external SDK, the **first PR must be a
build-spike** that proves the dep compiles in caro's workspace — nothing
else. No wrapping, no design commitments, no default-on feature flips,
no architectural decisions baked into code.

Real integration design starts only **after the spike merges green**.

## The 5-step spike checklist

| # | Check | How |
|---|-------|-----|
| 1 | License direction is permissible | The SDK's license must absorb cleanly into caro's AGPL-3.0. MIT/Apache-2.0/BSD/ISC OK (one-way absorption). GPL/AGPL needs case-by-case review. LGPL is borderline. Proprietary is blocked. |
| 2 | MSRV is satisfied | SDK's `rust-version` ≤ caro's `rust-version` (check `Cargo.toml:rust-version`). Fail loud if not — bumping caro's MSRV is its own decision, never bundled into a spike. |
| 3 | Optional dep + new feature flag | `Cargo.toml`: `<crate> = { version, optional = true }` and `<feature> = ["dep:<crate>"]`. **Never in `default` for the spike PR.** Default-on flips in the next phase, once the wrapping code justifies the binary-size cost. |
| 4 | One code reference forces compile | Cargo resolves optional deps but does NOT compile them unless the code graph references them. Add a single `pub fn smoke()` that touches the SDK's public API (e.g. construct its top-level client). Without this, `cargo check --features <new>` would silently succeed even if the SDK had a build error against caro's MSRV. |
| 5 | Two verification builds + smoke test | `cargo check --no-default-features --features <baseline>` AND `…--features <baseline>,<new>` must both pass. `cargo test --features <new> --lib <module>::` must run the smoke. Record both results in the commit body. |

## Delivery shape

- **Spike PR**: ≤100 LOC, only the 5 checks above. **No** architectural
  wrapping. Title pattern: `feat(<area>): phase 0 - <sdk> build spike`.
- **Beads epic** filed in the same PR as a separate `chore(beads):` commit,
  with child issues for each subsequent integration phase and dep edges
  so `bd ready` surfaces only Phase 1 after merge.
- **Architectural plan** referenced from the PR body (e.g.
  `.claude/plans/<name>.md`). The plan does NOT block the spike — the
  spike de-risks the plan.
- **NOTICE/attribution** deferred to the phase that flips the feature
  into `default`. The spike PR does not need it because no shipped binary
  links the SDK yet.

## Why this matters

Skipping the spike means the architectural PR has to undo design work if
the SDK doesn't resolve cleanly — wasted hours of wrapping code against
a dep that was never going to land.

The spike costs <1 hour and:
- Catches license direction problems before architecture is written.
- Catches MSRV gaps that would force a separate caro-wide MSRV bump.
- Catches transitive-dep version conflicts (`cedar-policy` clashing with
  an existing version, an `ed25519-dalek` major-bump, etc.).
- Establishes the feature-flag boundary, so subsequent phases can land
  incrementally without forcing all-users opt-in.

Run it once per major SDK. Re-skipping for "small" SDKs is how
medium-sized SDKs get integrated without the full checklist.

## Common failure modes

| Failure | Cause | Fix |
|---|---|---|
| `cargo check --features <new>` passes but the SDK is broken | No code in the graph references the optional dep — cargo never compiled it | Add the `pub fn smoke()` from step 4 |
| Cargo.lock conflict on rebase | Spike branch resolved one transitive version; main has resolved another | `cargo update -p <conflicted-crate>`, recommit lockfile |
| MIT-licensed SDK appears OK, but pulls a GPL transitive | License check looked only at the top-level crate | `cargo deny check licenses` — also covers transitives |
| `optional = true` dep is on a `[target.'cfg(...)']` line | Feature gate only fires on the targeted platform | Move to the top-level `[dependencies]` block unless platform-specific is intended |

## See also

- `~/.claude/rules/release-version-alignment.md` — the 6-file release
  checklist; spike checklist runs the same template-of-template pattern
  (codify a lesson as a checklist so the grep is the bug report)
- `~/.claude/rules/dev-process.md` — branch / PR / CI workflow; spike
  PRs follow normal branch+PR rules with no exceptions
- caro precedent: [PR #1103](https://github.com/wildcard/caro/pull/1103)
  (agentmesh build spike) + plan
  `.claude/plans/intgrate-https-github-com-microsoft-agen-witty-scroll.md`
