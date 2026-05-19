# Dependency Upgrade Verification — CI-Green Merge Rule

> **The rule, in one sentence.**
> **No PR may be merged while any required CI check is failing — regardless of who is reviewing, who is merging, what the change is, or how trivial it appears.** This applies equally to Dependabot PRs, agent-authored PRs, and human-authored PRs. There is no exception for "trivial" version bumps, "obviously safe" lockfile changes, or "the CI is just flaky" assertions.

This is a **Tier 1 — Project Safety** rule. It sits in `.claude/rules/constitution.md` above every engineering-discipline rule. It overrides them on contention.

---

## Why this rule exists

On 2026-04-27, Dependabot opened [PR #925](https://github.com/wildcard/caro/pull/925): `deps: bump bincode from 1.3.3 to 3.0.0`.

`bincode 3.0.0` is **an intentional tombstone release**. Its entire `src/lib.rs` is one line:

```rust
compile_error!("https://xkcd.com/2347/");
```

The bincode maintainers published 3.0.0 as a commentary on software-dependency fragility (referencing [xkcd 2347 "Dependency"](https://xkcd.com/2347/)) — there is no expectation that anyone build against it.

On PR #925, the AI reviewer `cubic-dev-ai` posted, **before the merge**:

> **P0**: Pinning `bincode` to `3.0` breaks the build: `3.0.0` is a tombstone release that intentionally emits a compiler error.

PR #925 was merged anyway by the project owner on 2026-05-17 at 02:11:45 UTC. From that point until [PR #1154](https://github.com/wildcard/caro/pull/1154) landed the unblock, **every `cargo build` on `main`, every CI matrix job, every release pipeline, and every `cargo install caro` failed unconditionally** — for 24+ hours.

This was not a Rust bug, not a Dependabot bug, not a CI bug. It was a **process bug**: an unverified merge of an external dependency upgrade, despite an explicit P0 flag from a reviewer, without anyone running `cargo build` against the bumped lockfile.

The bincode maintainers, by publishing a deliberately-broken version, did us a favor: they exposed exactly this gap. The fix is mechanical (replace bincode with [postcard](https://crates.io/crates/postcard)), but the *real* fix is to close the gap so the same class of failure cannot recur — whether the next instance is another well-meaning xkcd reference, a [Shai-Hulud-class supply-chain worm](https://blog.npmjs.org/) (npm/PyPI/crates.io have all seen real-world examples), or just a normal SemVer-major bump that nobody actually compiled.

---

## What every agent and contributor MUST do before invoking `gh pr merge`

This is the checklist. If you cannot tick every box honestly, **stop**. Do not merge.

### 1. Verify CI is green

```bash
gh pr checks <PR-number>
```

- **Required**: Zero `fail` rows among required checks.
- **Required**: Zero `pending` rows among required checks (wait — do not race).
- **Required**: At least the build/test/lint suite has actually run on the head SHA. If a required check is absent, that is *not* a pass — investigate why it didn't trigger.

```bash
gh pr view <PR-number> --json mergeStateStatus
```

- **Required**: `mergeStateStatus` is `CLEAN` or `HAS_HOOKS`.
- **Forbidden**: `BLOCKED`, `BEHIND`, `DIRTY`, `UNSTABLE`, `UNKNOWN`. If you see one of these, you have *not* satisfied the rule, regardless of how the `gh pr checks` output reads.

### 2. Read every AI-reviewer comment

The reviewers run by this project (`cubic-dev-ai`, `claude-review`, `coderabbitai`, `copilot`, etc.) post structured findings. Any comment that mentions:

- `P0`, `critical`, `blocker`, `breaking`
- `compile_error`, `panic`, `unsoundness`
- `tombstone`, `yanked`, `unmaintained`, `RUSTSEC-*`

…is a **hard merge blocker** until explicitly resolved. "Resolved" means one of:
- The change has been made on the branch and re-verified.
- The reviewer is demonstrably wrong (false positive); the rationale is recorded in a reply comment with evidence.
- The finding is out of scope for the PR; a tracking issue exists, is linked in the PR, and a labeled "ack-out-of-scope" comment from a human maintainer is on the thread.

"It looked OK to me" is not a resolution.

### 3. For ANY dependency PR — verify the actual build

Dependabot, Renovate, and human-authored dep bumps are equally subject to this rule. The presence of `app/dependabot` as the PR author is **not** a green light.

For a **major version bump** (`X.Y.Z` where `X` changes) of any direct dependency:

```bash
# Either: confirm a CI build job ran on the PR's HEAD SHA
gh pr checks <PR> | grep -E "Build|build"

# Or: build locally against the bumped lockfile
git fetch origin pull/<PR>/head:pr-<PR>
git checkout pr-<PR>
cargo build --release --features embedded-cpu
cargo test --no-run
```

If neither is satisfied, **do not merge**, regardless of what the Dependabot bot wrote, what the diff looks like, or whether a "Dependency Review" workflow reported "No vulnerabilities found". Vulnerability scanners catch known advisories; they do not catch deliberately-broken releases, transitive breakage from MSRV changes, or API renames.

### 4. Check transitive impact on direct dependencies

Even a minor or patch bump can shift the resolved version of a transitive dep that *another* direct dep depends on, breaking the build. Run:

```bash
cargo tree --duplicates
```

After the bump, this should show zero new duplicate-version entries among security-critical crates (serde family, tokio, rustls, ring, sha2/sha3, ed25519-dalek, regex, syn, candle-*, safetensors, bincode → postcard, etc.). If a duplicate appears, investigate before merging; if you cannot resolve it, file the issue and do not merge.

### 5. Cross-feature build matrix

For PRs touching `Cargo.toml`, `Cargo.lock`, `build.rs`, or any code under a feature flag, verify all relevant feature gates compile:

```bash
cargo check --no-default-features
cargo check --no-default-features --features cve-rules
cargo check --no-default-features --features embedded-cpu
cargo check --no-default-features --features knowledge
cargo check          # default features
```

A green `cargo check` on default features is **not** sufficient if the change touches any optional/gated code.

---

## What this rule prohibits

- **`gh pr merge --admin`** to bypass failing checks. Admin bypass exists for repository administration emergencies (broken CI configuration, locked-out maintainer, etc.), not for "the change looks fine". Every admin-merge is reviewable in the audit log; the project owner has explicitly committed to reviewing them.
- **"CI is just flaky" merges.** If a check is known-flaky, the flake is itself a `bug` that must have an open tracking issue. The PR description must link that issue, name the specific check, and state why this PR is unaffected. Otherwise: re-run, wait, fix the flake, or do not merge.
- **"Re-run until it passes" merges** (`gh run rerun ... && gh pr merge`). Re-running checks to flush out non-determinism is **hiding a bug, not fixing one.** A check that passes on the second try is not a passing check — it is a check whose first failure was uninvestigated.
- **Dependabot auto-merge with a passing-but-incomplete check set.** If the CI matrix runs ten platforms and only three reported in by merge time, that is not "green CI". Wait.
- **Stale-base merges.** If `mergeStateStatus` is `BEHIND`, the CI ran on a state that doesn't reflect the actual merged result. Rebase, re-run CI, then re-evaluate.

---

## The "main is already broken" exception (narrow, documented)

When `main` is in a state where new PRs cannot reach green CI because of a *pre-existing* breakage, a hotfix PR may merge with failing checks **if and only if all of the following are true**:

1. **The PR title starts with `fix(...)`** and references the issue tracking the broken state.
2. **The PR body explicitly lists every failing check**, with one paragraph per check explaining why it cannot pass yet AND linking the follow-up issue that will resolve it.
3. **The failing checks are demonstrably pre-existing**, not introduced by this PR. Evidence: a CI run on the parent commit (i.e. before this PR's changes) shows the same checks failing.
4. **A second agent or contributor** has independently confirmed the analysis on the PR thread (not just "LGTM" — they must name the failing checks and confirm the pre-existence claim).
5. **A short post-mortem issue** is opened concurrently, capturing how `main` reached the broken state and the rule revision (if any) needed to prevent recurrence.

The PR #1154 hotfix that unblocked PR #925's breakage is the worked example. It used this exception narrowly: three commits (bincode pin + rusqlite cast + candle alignment) all required to reach green — none of which could be verified in isolation because the *preceding* failure short-circuited everything.

**This exception is not a license to merge red CI in the general case.** It is a tightly-scoped escape valve for the specific situation where the rule itself created a deadlock.

---

## Enforcement

### Phase 1 — Today (this PR)

- Rule added to `.claude/rules/dependency-upgrade-verification.md` (this file).
- Indexed in `.claude/rules/constitution.md` Tier 1, immediately after `git-workflow.md`.
- Quoted in `CONTRIBUTING.md` "Pull Request Process" section.
- Quoted in `SECURITY.md` "Supply Chain Security" section.
- Quoted in `.github/PULL_REQUEST_TEMPLATE.md` pre-merge checklist.
- Quoted in `.github/copilot-instructions.md` for the GitHub Copilot review pass.

### Phase 2 — Follow-up issue (not this PR)

- **GitHub branch protection on `main`**: require the full set of required checks (`Build Check (x86_64-unknown-linux-gnu)`, `Unit Tests (ubuntu-latest)`, `Unit Tests (macos-latest)`, `Lint & Format`, `MSRV Check (Rust 1.85)`, etc.) to pass before merge. No admin bypass. Block force-push.
- **`cargo-deny` policy file**: lock major-version bumps of security-critical crates behind a manual `cargo-deny check bans` allowlist. Dependabot major-bump PRs that would violate the allowlist will fail CI mechanically, not by reviewer attention.

These mechanical enforcement steps land in their own PRs with their own discussion. The rule above is the human-and-agent-facing layer; the GitHub configuration is the belt-and-suspenders.

---

## What to do if you find yourself rationalizing

Watch for these red flags. They mean **stop and re-read the rule**.

| Thought | Reality |
|---------|---------|
| "It's just a patch bump." | Patch bumps have shipped yanked-then-republished tombstones (this happens). |
| "Dependabot says no vulnerabilities." | Dependency Review catches advisories, not deliberate breakage. |
| "The diff looks safe." | bincode 3.0's diff was a one-line `lib.rs` you'd never read in a Cargo.toml PR. |
| "We have great test coverage." | Tests can't run if the crate doesn't compile. |
| "I'll fix it in a follow-up if it breaks." | The cost to revert a broken `main` is ~24 hours of every contributor's time. The cost to wait for CI is 15 minutes. |
| "CI has been flaky lately." | Then the flake is the bug to fix, not the gate to bypass. |
| "I'm an admin, I can just merge." | The admin bit is for emergencies, not convenience. |
| "This is exactly what Dependabot is for." | Dependabot proposes changes. Humans and agents are responsible for verifying them. |
| "The other reviewer LGTM'd it." | Did the other reviewer verify CI was green? Did they read every cubic-dev-ai comment? |

---

## Precedent and references

- **PR [#925](https://github.com/wildcard/caro/pull/925)** — `deps: bump bincode from 1.3.3 to 3.0.0`. Merged 2026-05-17 despite `cubic-dev-ai` P0 flag. The originating incident.
- **Issue [#1150](https://github.com/wildcard/caro/issues/1150)** — `cli: cargo build fails — bincode 3.0.0 has deliberate compile_error! (xkcd/2347)`. Filed by `caro-qa-agent` on 2026-05-18; the regression report.
- **PR [#1154](https://github.com/wildcard/caro/pull/1154)** — `fix(deps,build): unblock main — bincode pin + rusqlite cast + candle align (closes #1150)`. The hotfix that re-greened `main` after invoking the "main is already broken" exception.
- **Follow-up PR (bincode → postcard migration)** — replaces bincode entirely, closes the `audit.toml` RUSTSEC-2025-0141 suppression.
- **[xkcd 2347 "Dependency"](https://xkcd.com/2347/)** — the cartoon bincode 3.0.0 references.
- **[Shai-Hulud npm worm (2024)](https://blog.npmjs.org/)** — example of an actual supply-chain attack class. The rule's threat model includes both deliberate tombstones (this incident) and malicious supply-chain compromise.
- **[`external-sdk-integration.md`](./external-sdk-integration.md)** — sibling rule for *adding* new external SDKs; complements this rule which governs *upgrading* existing ones.

---

## Quick reference card (paste into your scratch buffer before any merge)

```
1. gh pr checks <PR>                        →  zero fail, zero pending in required
2. gh pr view <PR> --json mergeStateStatus  →  CLEAN or HAS_HOOKS only
3. Read every AI-reviewer comment           →  zero unresolved P0/blocker/compile_error
4. For dep PRs: build verified on head SHA  →  CI job OR local cargo build --release
5. cargo tree --duplicates                  →  no new dup among security-critical crates
6. Feature-gate matrix all checked          →  no-default-features + each feature
THEN AND ONLY THEN: gh pr merge
```

Five minutes. Codified once, so the grep is a checklist instead of a bug report.
