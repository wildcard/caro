---
description: Acceptance-verify a caro release — does it actually deliver what was advertised, what PRs claimed, what beta testers requested? Report gaps as bugs and randomly sample one feature for deep testing.
---

**Path reference rule:** When you mention directories or files, provide either the absolute path or a path relative to the project root (for example, `CHANGELOG.md`). Never refer to a folder by name alone.

## User Input

```text
$ARGUMENTS
```

You **MUST** consider the user input before proceeding (if not empty).

**If user provides version** (e.g., `v1.3.0` or `1.3.0`): use that version.
**Otherwise**: default to the latest git tag.

---

## Purpose

This command is the **reality check** between what a release *claimed to do* and
what it *actually does in the user's hands*. Installability is covered by
`/caro.release.verify` — this command covers **acceptance**:

1. Every CHANGELOG entry must map to a verifiable, working feature.
2. Every feature PR in the release must have its headline claim verified.
3. Every GitHub issue closed-by the release must be reproducibly fixed.
4. Every beta-tester request logged in `.claude/beta-testing/` must be either
   shipped or explicitly deferred with a tracking issue.
5. A randomly-sampled feature gets a deep acceptance test.

**Gaps become GH issues** with `bug` + `release-gap` labels, priority assigned
via the rubric in Phase 7. The next release cannot proceed (per
`/caro.release.prepare`) until all `P0`/`P1` release-gap issues for the
prior release are closed or explicitly waived.

---

## Workflow Context

**Before this**: A release has shipped (`/caro.release.publish` + `/caro.release.verify`).

**This command**: Validates that the release actually delivers what it promised.

**After this**: `/caro.release.prepare` reads this command's audit output to
decide whether the *next* release is clear to start.

Run this command:
- Immediately after a release ships (fresh in memory, testers still engaged).
- As the first step of the next release cycle (grooming gate).
- Whenever a user reports a regression and you need to triage scope.

---

## Outline

### 1. Determine Target Version

```bash
if [ -n "$ARG_VERSION" ]; then
  VERSION="${ARG_VERSION#v}"
else
  VERSION=$(git describe --tags --abbrev=0 | sed 's/^v//')
fi
TAG="v$VERSION"
echo "Acceptance-verifying caro $TAG"
```

If no tag exists, refuse: `ERROR: no git tags — nothing to verify`.

### 2. Gather Claims

Build the **claim set** — every assertion this release made. Claims come from
four sources; union them.

#### 2a. CHANGELOG.md

Extract the `## [X.Y.Z] - YYYY-MM-DD` section. Parse every bullet under
`### Added`, `### Changed`, `### Fixed`, `### Security`, `### Internal`. Each
bullet is one **claim**.

```bash
awk "/^## \[$VERSION\]/{p=1;next} p&&/^## \[/{p=0} p" CHANGELOG.md | grep -E '^- ' > /tmp/claims-changelog.txt
```

#### 2b. GitHub Release notes

```bash
gh release view "$TAG" --json body | jq -r .body > /tmp/claims-release-notes.md
```

Diff against CHANGELOG — they should match. Any delta is itself a gap
(advertised but not in changelog, or vice versa).

#### 2c. Feature PRs merged into this release

```bash
PREV_TAG=$(git describe --tags --abbrev=0 "${TAG}^" 2>/dev/null || echo "")
if [ -n "$PREV_TAG" ]; then
  gh pr list --state merged --search "merged:>=$(git log -1 --format=%aI "$PREV_TAG") base:main" \
    --json number,title,body,labels --limit 100 > /tmp/claims-prs.json
else
  gh pr list --state merged --base main --json number,title,body,labels --limit 100 > /tmp/claims-prs.json
fi
```

Every PR title headline is a claim. Flag PRs labeled `feature` / `feat` for
deeper verification; PRs labeled `chore` / `docs` can be spot-checked.

#### 2d. Closed-by-this-release issues

```bash
gh issue list --state closed --search "closed:>=$(git log -1 --format=%aI $PREV_TAG)" \
  --json number,title,labels,body --limit 200 > /tmp/claims-issues.json
```

For each issue: does its closing PR belong to this release? If yes, the issue's
reproduction steps are claims that must still fail on the previous version and
pass on this one.

#### 2e. Beta-tester requests

Per `CLAUDE.md`, beta-test feedback lives under `.claude/beta-testing/`. Read
any file there dated between `$PREV_TAG` and `$TAG`:

```bash
find .claude/beta-testing -type f -newer "$(git log -1 --format=%cI "$PREV_TAG" -- .)" 2>/dev/null
```

For each request: is it shipped (matched to a CHANGELOG entry), explicitly
deferred (has a tracking GH issue open), or silently dropped (neither)? The
third category is a gap.

### 3. Build the Verification Matrix

Produce a table mapping each claim to a verification method:

| Claim source | ID | Statement | Verifier |
|---|---|---|---|
| CHANGELOG Added | CHL-01 | `caro ai` once-mode subcommand | `./caro ai --once "list files"` exits 0 with a command on stdout |
| PR #861 | PR-861 | Safety validator blocks AI-generated `rm -rf /` | `./caro ai --once "rm -rf /"` exits non-zero, stderr mentions Critical |
| Issue #839 | IS-839 | `caro knowledge` respects `--limit N` | repro steps from issue body |
| Beta note 2026-04-10 | BT-01 | `?` keybinding works in zsh | manual smoke in a fresh zsh |

Write the matrix to `.claude/releases/v$VERSION-acceptance.md` so the audit is
reproducible later.

### 4. Execute Verifiers

For each row: run the verifier, capture result (`PASS` / `FAIL` / `SKIP`).

**Automated** (preferred):
- Unit/integration tests that reference the PR/issue number
- Smoke tests: `./caro <cmd>` with known input + assertion on stdout/exit
- Website-claims CI job — grep its last run for this release

**Manual** (when automation isn't feasible):
- Shell integration (`caro shell-init <shell>`): source into a real shell,
  press `?` on empty prompt, assert `caro ai` runs
- Interactive prompts: walk the UX, note any friction

Record each row's result. Keep failures with the full command, stdout, stderr,
and exit code.

### 5. Random Acceptance Sample

Independent of the matrix above, pick **one feature at random** for deep
acceptance testing — not just "does it run", but "would a user be happy with
it".

```bash
FEATURES=($(awk "/^## \[$VERSION\]/{p=1;next} p&&/^## \[/{p=0} p" CHANGELOG.md \
  | awk '/^### Added/{p=1;next} /^### /{p=0} p' | grep '^- ' | head -20))
SAMPLE=$(( RANDOM % ${#FEATURES[@]} ))
echo "Randomly sampled feature: ${FEATURES[$SAMPLE]}"
```

For the sampled feature, do what a new user would do:
1. Read the feature's doc (README section, `--help` output, blog post link).
2. Follow the doc literally — no prior knowledge.
3. Try the 3 most obvious use-cases.
4. Try one adversarial / edge case.
5. Record: did the doc match reality? Did it work? Were errors clear? Were
   defaults sensible? Where did you get stuck?

This sample often surfaces **polish gaps** that the claim-matrix misses
because claims are usually "does X exist", not "is X pleasant".

### 6. Compile the Gap Report

Aggregate every `FAIL` from step 4 and every friction point from step 5 into a
gap report:

```markdown
# caro v$VERSION acceptance audit — $(date +%Y-%m-%d)

## Summary
- Claims verified: N/M
- PRs spot-checked: N
- Issues re-verified: N
- Beta requests reconciled: N
- Random sample: <feature>

## Gaps
### P0 (blocking next release)
- [GAP-001] <one-line>. Source: CHL-03 (CHANGELOG claim). Repro: `...`
### P1 (ship-within-one-release)
- [GAP-002] ...
### P2 (track, no deadline)
- [GAP-003] ...

## Random-sample findings
<notes from Phase 5>
```

Write to `.claude/releases/v$VERSION-acceptance.md`.

### 7. File GH Issues for Gaps

For each gap, open a GitHub issue:

```bash
gh issue create \
  --title "Release gap (v$VERSION): <one-line>" \
  --label "bug,release-gap,v$VERSION" \
  --body "$(cat <<EOF
Source: <claim source, e.g. CHL-03 CHANGELOG 'caro ai --once respects --timeout'>
Expected: <what the claim said>
Actual: <what we observed>
Reproduction:
\`\`\`
<command>
\`\`\`
Priority rationale: <why P0/P1/P2>

Found by \`/caro.release.acceptance\` on $(date +%Y-%m-%d).
EOF
)"
```

**Priority rubric** (use this to label):

| Priority | Criteria |
|---|---|
| **P0** | Advertised feature doesn't work at all. Security regression. Crash on basic usage. Data loss. Blocks shipping next release until closed. |
| **P1** | Feature works but not as documented. Confusing error. Edge case advertised in docs fails. Must ship in next release. |
| **P2** | Polish gap. Minor friction. Advertised "nice-to-have" partially delivered. No deadline, but tracked. |
| **P3** | Defer / won't-fix — document in the issue why. Examples: platform we don't support, out-of-scope ask, obsoleted by another feature. |

### 8. Feed into Grooming

Print a one-line summary and exit with a status code:

```
caro v$VERSION acceptance: <P0-count> P0, <P1-count> P1, <P2-count> P2

Release gate: [BLOCKED|CLEAR]
```

- **BLOCKED** if any open `release-gap` issue from *any* prior release is
  labeled `P0` or `P1` and not closed. Exit code 1.
- **CLEAR** otherwise. Exit code 0.

`/caro.release.prepare` must read this status and refuse to start a new release
if BLOCKED.

### 9. Commit Audit Trail

```bash
git add .claude/releases/v$VERSION-acceptance.md
git commit -m "chore(release): acceptance audit for v$VERSION

Claims verified: N/M
Gaps filed: <list of issue numbers>

Co-Authored-By: Claude <noreply@anthropic.com>"
```

Do **not** create a PR for this — it's an audit artifact, committed straight
to a short-lived branch (`audit/v$VERSION-acceptance`) and merged fast-forward
so the history is linear.

---

## Grooming Integration (for `/caro.release.prepare`)

The prepare command should call this routine on the *current latest* release as
step 0, before touching the next version:

```bash
# In /caro.release.prepare step 0:
/caro.release.acceptance  # against latest tag
# If exit 1, refuse to start a new release and show the blocker list.
```

This enforces the user's rule:
> before releasing making sure the already releases are working as requested
> and no reported bug

---

## References

- Installability verification: `/caro.release.verify`
- Version alignment: `.claude/rules/release-version-alignment.md`
- Beta-testing directory: `.claude/beta-testing/` (per `CLAUDE.md`)
- Release process doc: `docs/RELEASE_PROCESS.md`
