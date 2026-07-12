# Evidence — Flox Use Case + Distribution Channel

**Feature**: Flox as a first-class use case, dev environment, dev-process
rule, and distribution channel for caro.
**Origin**: Async LinkedIn exchange with Ben Futoriansky (Head of Bus Ops,
Flox), 2026-05. The three-layer framing (command safety / boundary /
environment) is his.
**Branch**: `claude/unruffled-hypatia-f7b6ef`
**Verified**: 2026-05-28

This artifact is the durable record that the feature works. The live demo
below is reproducible; the regression test locks it down so future edits
cannot silently break it.

## What shipped (5 surfaces)

| # | Surface | File(s) |
|---|---------|---------|
| 1 | Dev env manifest | `.flox/env/manifest.toml`, `.flox/.gitignore` |
| 2 | Use-case landing page | `website/src/pages/use-cases/flox.astro` |
| 3 | Hub registration | `website/src/pages/use-cases/index.astro` (flox persona) |
| 4 | Dev-process rule | `.claude/rules/coder-agent-isolation.md` + `constitution.md` Tier 3 |
| 5 | CI packaging + docs | `.github/workflows/packages.yml` (flox job) + `flox/README.md` |

## Regression guard (e2e test)

`website/src/__tests__/flox-integration.test.ts` — 18 assertions across
all 5 surfaces. Read-as-text checks on the real source files, so a future
refactor that drops the flox page from the personas array, un-registers
the rule, or removes the CI job fails loudly.

Notable cross-checks the test enforces:
- The Flox manifest's `rustup default <MSRV>` must match `Cargo.toml`'s
  `rust-version` — MSRV bumps force the env to bump with it.
- The `flox` CI job must appear in the `summary` job's `needs[]` — a job
  not wired into `needs[]` is orphaned and its failure is invisible.
- No unescaped `:(){` fork-bomb text in the `.astro` template (guards
  `astro-esbuild-shell-syntax.md`).

## Verification matrix

| ID | Claim | Verifier | Result |
|----|-------|----------|--------|
| V1 | Website test suite green (baseline kept) | `npx vitest run` | ✅ 64 passed (46 baseline + 18 new) |
| V2 | Flox use-case page renders end-to-end | dev server + browser page-text capture | ✅ all sections present, 0 console errors |
| V3 | Flox persona card live in hub | JS query on `/use-cases` | ✅ `/use-cases/flox` link present, correct text |
| V4 | Astro parses the new page (no esbuild brace crash) | `npx astro sync` | ✅ types generated, no error |
| V5 | Workflow YAML valid | `python3 -c "import yaml; yaml.safe_load(...)"` | ✅ OK |
| V6 | Flox manifest valid TOML | `python3 tomllib` load | ✅ sections: version, install, vars, hook, options |
| V7 | GH Actions injection guard | `security_reminder_hook` + env-only run: blocks | ✅ version/tag flow via `env:` |
| V8 | Rust baseline unaffected | `git diff --name-only` | ✅ zero Rust files touched |
| V9 | Flox manifest actually reaches CI | `git add .flox/env/manifest.toml` after `.gitignore` fix | ✅ tracked (see below) |

### V9 detail — the silent `.gitignore` trap

The root `.gitignore` carries the standard Python `env/` / `ENV/`
virtualenv patterns. On a case-insensitive filesystem these **also match
`.flox/env/`**, so `git add .flox/` silently skipped the manifest — the
one file Surface 1 depends on. The regression test read the working tree
and passed, but a fresh CI checkout would have had no manifest and the
test would have failed there.

Fix: explicit negations in `.gitignore` (`!.flox/env/` +
`!.flox/env/manifest.toml`) that re-track the manifest without weakening
the Python-artifact ignores. This is the class of latent bug that only
surfaces on a clean clone; catching it here keeps the feature green in CI.

## Reproduce the demo

```bash
# 1. Website tests (regression guard + baseline)
cd website && npx vitest run

# 2. Live use-case page
npm run dev            # astro dev on :4321
open http://localhost:4321/use-cases/flox
open http://localhost:4321/use-cases     # flox card in the hub

# 3. Static validation
npx astro sync         # parse check (no --remote / DB needed)
python3 -c "import yaml,sys; yaml.safe_load(open('../.github/workflows/packages.yml'))"
```

Flox itself is not installed on the verification host, so Surface 1 is
validated by TOML structure + the MSRV cross-check in the test, not a live
`flox activate`. The activation commands are documented on the use-case
page and in `flox/README.md`; a follow-up on a Flox-equipped host should
run `flox activate -- cargo check --no-default-features --features embedded-cpu`
to close that gap.

## Deferred (tracked as follow-ups)

- Upstream `caro` binary to the `flox-floxpkgs` catalog (needs Flox-side
  review).
- Publish `caro-skill` bundle as a discoverable Flox component.
- Live `flox activate` smoke on a Flox-equipped CI runner.
