# Salvage note — PR #838 `quant.cpp` inference backend

**Date**: 2026-09-24
**Source PR**: [#838](https://github.com/wildcard/caro/pull/838) (left **OPEN** — this is a rebase plan, not a closure)
**Branch**: `claude/quant-cpp-integration-AOdW6` (head `6910342e`, dated 2026-04-24)
**Status**: Not rebased. Feature is **absent from `main`** — verified via
`git ls-tree -r --name-only origin/main | grep -i quant` → no matches.

## Why this one was kept while #805 / #808 / #993 were closed

It is the only one of the four stale PRs whose feature never shipped by
another route, and its conflict surface is **registration-only**. The
substantive code is a single new file that does not conflict at all.

| Metric | Value |
|---|---|
| Behind `origin/main` | 192 commits |
| Ahead | 3 commits (1 real + 2 merge commits) |
| Diff | +759 / −3 across 9 files |
| Conflicting files | 5 — **all wiring, none substantive** |

## The split that makes this cheap

**Carries over unchanged (no conflict):**

- `src/backends/remote/quantcpp.rs` — the entire backend implementation
- `docs/adr/ADR-015-quant-cpp-backend.md`
- `.github/ISSUES/quant-cpp-backend.md`
- `.github/ISSUES/tech-debt-parse-command-response.md`

**Must be re-applied by hand (all 5 conflicts):**

- `src/backends/remote/mod.rs`
- `src/lib.rs`
- `src/main.rs`
- `src/models/mod.rs`
- `docs/adr/README.md`

These conflict because the backend-registration surface was rewritten
after this branch was cut — see `2a76cf90` *"fix(cli): single source of
truth for backend roster (#1115) (#1298)"*, which is the merge-base
ancestor. The branch registers `quantcpp` against the **old** per-site
roster; `main` now has one canonical roster. So the conflicts are not
disagreements about behaviour, they are the same registration expressed
against two different APIs.

## Recommended rebase procedure

Do **not** `git rebase` the branch — 192 commits of replay for 1 real
commit is the wrong shape. Cherry-pick the content instead:

```bash
git checkout -b feat/quantcpp-backend origin/main
git checkout claude/quant-cpp-integration-AOdW6 -- \
  src/backends/remote/quantcpp.rs \
  docs/adr/ADR-015-quant-cpp-backend.md \
  .github/ISSUES/quant-cpp-backend.md \
  .github/ISSUES/tech-debt-parse-command-response.md
```

Then register `quantcpp` against the **current** canonical roster rather
than porting the old registration diff, and rebuild `docs/adr/README.md`
from the current file.

## Two things to check before opening the PR

1. **ADR-015 number collision.** This branch adds
   `docs/adr/ADR-015-quant-cpp-backend.md`, but `ADR-015-mcp-safety-server.md`
   also claims 015 (preserved in `3723206a`). Per
   `.claude/rules/adr-numbering.md`, ADRs are sequential with no gaps and
   no duplicates — one of these must be renumbered on merge, along with
   its `docs/adr/README.md` row and any cross-references.
2. **Feature-flag placement.** `quantcpp.rs` lives under
   `src/backends/remote/`, which is gated behind the `remote-backends`
   feature. Confirm the new backend is inside that gate and that
   `cargo check --no-default-features --features embedded-cpu` still
   builds without it.

## Verification owed at PR time

Per `.claude/rules/feature-evidence.md` this needs evidence, a runnable
demo, and a named regression guard before it can land. None of the three
exist on the branch today.
