# Broken Windows — Handoff Register

Things an agent found broken **outside its own task**. The finder records the
problem here and moves on. A fixer claims an entry and fixes it. The fixer is
either the **caro broken-window sweep** routine (`trig_01TAiuoMoZ6nofW3ceYL3Csy`,
daily 15:30 UTC) or any agent already working in that area.

Rules (see `.claude/rules/good-boy-scout.md` → "Stay in your lane"):

- **Found one?** Search open issues first. If none exists, file one, then add
  an entry here that links it. Never spend your own task on it.
- **Claiming:** set `Status: claimed-by <branch>` before starting, and fix it
  in its own PR (not bundled into unrelated work).
- **Needs human: yes** means a security-policy, release or product decision
  (audit suppressions, safety patterns, dependency majors). Don't apply the
  change yourself; the issue is the ask.
- **Fixed:** set `Status: fixed (<PR>)` and leave the entry for a week so
  finders can see it's done, then delete it.

---

### BW-041: OmniSearch inlines its full search index into every page

**Found:** 2026-10-10, WCAG design pass (`feat/website-wcag-design-pass`)
**Issue:** #1564
**Status:** open
**Needs human?:** no
**Next step:** serve `search-index.json` statically and fetch it when the
search overlay first opens (`website/src/components/OmniSearch.astro`).

### BW-002: CLAUDE.md version/MSRV drift, filed 42× (canonical + 41 duplicates)

**Found:** 2026-09-27, PR #1470 session
**Issue:** canonical #1098 (oldest open); 41 duplicates closed 2026-10-03; dups #1520, #1541 closed 2026-10-09
**Status:** fix in #1478: `CLAUDE.md` points at `Cargo.toml` and repeats no
version, and the PR closes #1098. #1432 was closed as superseded on 2026-10-09.
**Needs human?:** no
**Next step:** merge #1478.
Root cause: the daily QA routine (`trig_01Tk7DxyXV7LeYcFjgmTG1mZ`, 14:00 UTC)
re-files instead of commenting on the existing issue. A "search before filing"
edit was drafted 2026-10-03; only the maintainer can apply it (the routine was
created via the API, so agents cannot update it). #1546 adds the
search-before-filing step to the runner contract (CONTRACT.md section 6).

### BW-003: stale harness references (caro-eval, current-tasks.md, AGENTS.md, .kittify, --skill)

**Found:** 2026-09-24, google/ax harness survey
**Issue:** #1479
**Status:** open. #1478 fixes the `current-tasks.md` row.
**Needs human?:** no
**Next step:** see the issue table. Each row is a one-line docs/config fix.
Remaining: `caro-eval` (use `cargo test --test evaluation`), `AGENTS.md` line in
dev-process.md, `.kittify` helpers, `claude --skill` in modulization.yml,
hardcoded `/Users/kobik-private` paths.

### BW-004: `caro ai --once` hangs silently when no backend/model is ready

**Found:** 2026-09-30 sweep dedup
**Issue:** #1272 (11 duplicates closed 2026-10-03)
**Status:** open. Stdin half fixed (#1503, merged 2026-10-03; closed #1499). Two causes,
reproduced 2026-10-02: (1) `--once` read stdin even with a trailing prompt, so an
open pipe (scripts, CI) blocked forever (#1499); (2) the first run silently
downloads the ~1 GB default model, with no progress output (#1272, #1484).
**Needs human?:** no
**Next step:** cause (2) is still open:
PR #1415 only prints a one-time "Initializing backend…" line, not download
progress. Keep #1272 open until first-run download progress (or a clear
"downloading model" message) lands; #1415 is a partial step.

### BW-006: embedded CPU stub always returns `echo 'Please clarify your request'`

**Found:** 2026-09-30 sweep dedup
**Issue:** #1269 (13 duplicates, closed). Root cause: the stub keyword-matches "rm" in the system prompt (`src/backends/embedded/cpu.rs`).
**Status:** open
**Needs human?:** yes. Product call: fail with a clear error on the CPU variant, or fall back to the static matcher. Epic #1460 / #1462 also touch this path.
**Next step:** maintainer picks the behaviour.

### BW-007: placeholder command reported with fake confidence=0.85 / risk=Safe

**Found:** 2026-09-30 sweep dedup
**Issue:** #1281 (dups #1361, #1421, #1424, closed)
**Status:** open
**Needs human?:** yes. It goes together with the BW-006 decision.
**Next step:** fix after BW-006 is decided.

### BW-010: CLI test runner falls back to ambiguous `cargo run` (multi-binary)

**Found:** 2026-09-30 sweep dedup
**Issue:** #1222 (dups #1252, #1413, closed); same root cause as #1164
**Status:** fixed (#1522, merged 2026-10-09)
**Needs human?:** no
**Next step:** none. Delete after 2026-10-16.

### BW-011: `caro ai --once` has no static-matcher first pass

**Found:** 2026-09-30 sweep dedup
**Issue:** #1179 (dup #1387, closed)
**Status:** open
**Needs human?:** no
**Next step:** route `ai --once` through the same static-first chain as the top-level command.

### BW-012: `caro ai` rejects `-p/--prompt` while its error text suggests it

**Found:** 2026-09-30 sweep dedup
**Issue:** #1213 (dup #1422, closed)
**Status:** open
**Needs human?:** no
**Next step:** accept `-p`, or fix the message.

### BW-013: `caro config reset <key>` unsupported

**Found:** 2026-09-30 sweep dedup
**Issue:** #1260 (dup #1381, closed)
**Status:** open
**Needs human?:** no
**Next step:** add per-key reset.

### BW-014: `--output json` reports `executed: true` under `--dry-run`

**Found:** 2026-09-30 sweep dedup
**Issue:** #1217 (dup #1417, closed)
**Status:** fixed (#1532, merged 2026-10-10). Adds a `dry_run` field to the JSON. `executed` keeps its pinned meaning ("passed safety checks"); contract tests depend on it.
**Needs human?:** no
**Next step:** none. Delete after 2026-10-17.

### BW-015: main CI `Extended Tests` (4 model jobs) red: HF Hub model download fails

**Found:** 2026-09-30 sweep, CI run 36658876598
**Issue:** #1341
**Status:** fixed (#1553, merged 2026-10-11). #1341 stays open until the 2026-10-12 nightly Extended Tests run is green.
**Needs human?:** no (decided 2026-10-10: SmolLM uses the bundle.yml Q8_0 repo, StarCoder is dropped, CI runs on CPU only).
**Next step:** confirm the 2026-10-12 nightly Extended Tests run is green, then close #1341. Delete after 2026-10-18.

### BW-016: 1.5.0 declared in-repo but never tagged or published

**Found:** 2026-09-30 sweep
**Issue:** #1419
**Status:** open. Release PR #1515 merged 2026-10-11; no `v1.5.0` tag on origin yet, so nothing is published.
**Needs human?:** yes (release)
**Next step:** maintainer decides whether to tag 1.5.0. This is also the root of the BW-002 churn.

### BW-017: `pr-merged.yml` `add-contributor` job fails on every merge

**Found:** 2026-10-01 sweep
**Issue:** #1491 (action `all-contributors/add-contributor` not found)
**Status:** fixed (#1527, merged 2026-10-09). The job is removed: the action repo does not exist and the repo has no `.all-contributorsrc`.
**Needs human?:** no (CI config). Pin a published action, or remove the job.
**Next step:** none. Delete after 2026-10-16. Re-add a working all-contributors job only if the maintainer wants a contributors list.

### BW-018: Claude Code plugin marketplace.json shape / install one-liner likely stale

**Found:** 2026-10-01 sweep (nightly-discovery)
**Issue:** #1490
**Status:** open
**Needs human?:** no
**Next step:** check the current plugin marketplace schema and `/plugin install` syntax, then update the docs and marketplace.json.

### BW-019: Lint & Format red on every PR since Rust 1.99 clippy

**Found:** 2026-10-01 (while driving #1497)
**Issue:** #1498. Clippy 1.99 `double_must_use` (via `#[async_trait]`) and `redundant_field_names` (via `thiserror` `#[from] source`), 27 errors, none in changed code.
**Status:** mitigated (#1505, merged 2026-10-03): Lint & Format and the publish.yml clippy step run on Rust 1.98.1.
**Needs human?:** no (maintainer chose the pin)
**Next step:** remove the pin once clippy or async-trait/thiserror stop linting macro-generated code; then close #1498.

### BW-020: backend lists disagree across `--backend-info`, `--help` and the error text

**Found:** 2026-10-03 sweep dedup
**Issue:** #1221 (15 duplicates, closed 2026-10-09)
**Status:** open
**Needs human?:** no
**Next step:** derive `--backend-info`, the `--backend` help text and the "Unknown backend" error from one list, filtered by compiled features.

### BW-021: user allowlist cannot override the Critical `rm -rf` pre-scan (regression from #1110)

**Found:** 2026-10-03 sweep dedup
**Issue:** #1165 (5 duplicates, closed 2026-10-09); `test_allowlist_functionality` fails
**Status:** open
**Needs human?:** yes (safety pattern / allowlist policy)
**Next step:** maintainer decides whether user allowlists may bypass Critical patterns, then fix code or the contract test via `safety-pattern-developer`.

### BW-022: `cargo test safety` (documented in CLAUDE.md) fails

**Found:** 2026-10-03 sweep dedup
**Issue:** #1162 (dup #1170)
**Status:** fixed (#1548, merged 2026-10-10). The `evaluation` harness (`harness = false`) now accepts libtest's positional filter and skips itself when the filter does not match its name.
**Needs human?:** no
**Next step:** none. Delete after 2026-10-17.

### BW-023: global flags before a subcommand swallow the subcommand

**Found:** 2026-10-03 sweep dedup
**Issue:** #1163 (dup #1328)
**Status:** open
**Needs human?:** no
**Next step:** fix clap routing (`args_conflicts_with_subcommands`) so `caro --no-telemetry config show` runs `config`.

### BW-024: `caro config set backend` has no `auto` value to restore auto-detect

**Found:** 2026-10-06 sweep
**Issue:** #1530
**Status:** open
**Needs human?:** no
**Next step:** accept `auto` (or `default`) in `config set backend` and clear the stored backend.

### BW-025: GNU coreutils misdetected as BSD when `ls --version` exceeds 500 ms

**Found:** 2026-10-06, CI on #1532 (`test_coreutils_detection` failed on ubuntu-24.04)
**Issue:** #1533
**Status:** open
**Needs human?:** no
**Next step:** raise the `ls --version` timeout in `src/platform/mod.rs` to about 2 s; treat a timeout as unknown in `detect_bsd_utils`.

### BW-026: `Validate Translations` red on main: 13 locales lack playbook.json and waitlist.json

**Found:** 2026-10-02, PR #1507 session (also red on main push be07b22 and every i18n-touching PR)
**Issue:** #1508
**Status:** open
**Needs human?:** no (run `translate.yml` or copy the English files; or relax the validator for missing namespaces)
**Next step:** `en/playbook.json` (#1174) and `en/waitlist.json` (#599) were never mirrored into 13 locales. CI runs the validator without `--strict`, so the missing-file errors are the failure; the 207 untranslated warnings are only fatal in strict mode.

### BW-027: static safety floor rates reverse shells, credential exfiltration and `DROP DATABASE` as Safe

**Found:** 2026-10-10, while stratifying the risk-gate gold subset (`docs/ml/risk-gold-subset.md`, "What the static floor got wrong")
**Issue:** #1559
**Status:** open
**Needs human?:** yes (safety patterns are a security-policy decision; not applied from a feature branch)
**Next step:** `SafetyConfig::moderate()` returns `Safe` for `bash -i >& /dev/tcp/…`, `curl -F "key=@$HOME/.ssh/id_rsa" …`, `env | curl --data-binary @- …`, `sudo setenforce 0`, a uid-0 `useradd`, `mysql -e 'DROP DATABASE …'`, `redis-cli FLUSHALL`, `aws s3 rm --recursive`, `kubectl delete namespace`, `aws ec2 terminate-instances`, and `sudo truncate -s 0 /var/log/*.log`, while `rm -rf ./target` is `Critical`. The 121-row candidate file (`docs/ml/risk-gold-candidates.jsonl`) and its companion static-tier file (`docs/ml/risk-gold-static-tiers.jsonl`, joined on `id`) record each command and its static verdict; decide which gaps become patterns. BW-026 is reserved by #1507 (Validate Translations).

### BW-028: the CLA check never records a signature, so every external PR stays red

**Found:** 2026-10-10, preparing first responses to external PRs (harness-plan session)
**Issue:** #1555
**Status:** fixed (#1584, merged 2026-10-11)
**Needs human?:** yes (a CI gate in `.github/workflows/cla.yml`, and part of the legal process)
**Next step:** none; #1555 is closed. Delete after 2026-10-18.

### BW-029: an unquoted prompt that starts with "list" runs the `list` subcommand

**Found:** 2026-10-10, nightly CI triage (harness-plan session)
**Issue:** #1550
**Status:** open
**Needs human?:** yes (CLI design: subcommands win, prompt fallback, or reserved names)
**Next step:** the maintainer picks an option in #1550. Until then,
`e2e_smoke_test_suite` and `e2e_shell_operator_truncation` fail in all 4
Extended Tests jobs; BW-015 hides them, because most tests fail on the model first.

2 e2e tests (`list files`) fail in every Extended Tests job because of this.

### BW-030: 8 stray `.worktrees/*` gitlinks on main, no `.gitmodules`

**Found:** 2026-10-10, nightly CI triage (harness-plan session)
**Issue:** #1551
**Status:** open
**Needs human?:** no
**Next step:** `git rm --cached` the 8 paths listed in #1551 (all added by
`9599ca7`). `.gitignore` already covers `.worktrees/`.

### BW-048: hybrid sanitizer leaks spaced paths and inline credentials

**Found:** 2026-10-03 (cubic review on #1513)
**Issue:** #1518. `PATH_RE` stops at whitespace, and there's no rule for `Authorization: Bearer`, `-u user:pass` or URL userinfo in the request input sent to hybrid remotes.
**Status:** open
**Needs human?:** yes (privacy policy: what may leave the machine)
**Next step:** reuse `COMMAND_SECRETS` from `src/guard/log.rs` as reversible sanitizer rules; add tests.

### BW-031: ai_horde `Client-Agent` header hardcoded to `caro:1.4.0`

**Found:** 2026-10-09 sweep register sync (issue filed 2026-09-22)
**Issue:** #1467
**Status:** open (#1454 mentions it)
**Needs human?:** no
**Next step:** build the header from `env!("CARGO_PKG_VERSION")` in `src/backends/remote/ai_horde.rs`.

### BW-032: embedded model download hangs: `HfHubClient` has no HTTP timeout

**Found:** 2026-10-09 sweep register sync (issue filed 2026-09-06)
**Issue:** #1440
**Status:** fixed (#1561, merged 2026-10-11). Each download attempt runs on its own runtime with a 60 s stall watchdog.
**Needs human?:** no
**Next step:** none; #1440 is closed. Delete after 2026-10-18.

### BW-033: Gemini CLI row in integrations status is stale (product shut down 2026-06-18)

**Found:** 2026-10-09 sweep register sync (issue filed 2026-09-06)
**Issue:** #1439
**Status:** open
**Needs human?:** no
**Next step:** retire the Gemini CLI row in `.claude/memory/integrations-status.md` and add Antigravity CLI.

### BW-034: `caro ai --help` contradicts itself; `--continue-session` is a no-op

**Found:** 2026-10-09 sweep register sync (issue filed 2026-09-05)
**Issue:** #1435 (overlaps #1197, which covers only `--continue-session`)
**Status:** open
**Needs human?:** no
**Next step:** fix the `--once` help text and map `--continue-session` to `SessionMode::ResumeStrict`.

### BW-035: `caro ai --once` shows no feedback during model initialization

**Found:** 2026-10-09 sweep register sync (issue filed 2026-08-15)
**Issue:** #1408 (related to BW-004 / #1272, kept separate)
**Status:** open (#1415 prints a one-time "Initializing backend…" line)
**Needs human?:** no
**Next step:** merge #1415, then add download progress (BW-004 cause 2).

### BW-036: `caro config set backend` rejects mesh, ai-horde, hybrid

**Found:** 2026-10-09 sweep register sync (issue filed 2026-07-27)
**Issue:** #1379 (related: BW-020, BW-024)
**Status:** open. PR #1348 (`integrator/20260718`) unifies the backend rosters; it is mergeable but idle since 2026-09-24. `src/main.rs` still hardcodes `["embedded", "ollama", "exo", "vllm"]`.
**Needs human?:** no
**Next step:** review and merge #1348, or rebase it.

### BW-037: `caro ai --once` hint puts `--execute` after the subcommand

**Found:** 2026-10-09 sweep register sync (issue filed 2026-07-23)
**Issue:** #1369 (same root as BW-023: global flags and subcommand order)
**Status:** open (#1443 mentions it)
**Needs human?:** no
**Next step:** fix the hint text to `caro --execute ai --once`, or accept `--execute` on `ai`.

### BW-038: no config-dir override, so `caro config set` writes the real config in tests

**Found:** 2026-10-09 sweep register sync (issue filed 2026-07-19)
**Issue:** #1349
**Status:** open
**Needs human?:** no
**Next step:** honor a `CARO_CONFIG_DIR` env var in `src/config/mod.rs` and use it in the config tests.

### BW-039: `ChromaDB Integration Tests` CI job fails late in the run

**Found:** 2026-10-09 sweep register sync (issue filed 2026-07-18)
**Issue:** #1342
**Status:** open
**Needs human?:** yes if the fix is to delete the job (CI gate); no for a root-cause fix.
**Next step:** read the failing step's log and root-cause it.

### BW-040: static matcher "Find Python files (simple)" swallows content searches

**Found:** 2026-10-03 (while fixing "search for TODO in all python files")
**Issue:** #1516. `search for <X> in python files` returns `find . -name "*.py" -type f` for any X other than TODO.
**Status:** fixed (#1517, merged 2026-10-10: content-search handler in `StaticMatcher::content_search_command`)
**Needs human?:** no
**Next step:** none. Delete after 2026-10-17.

### BW-045: `caro run` runbook path and step path keep shell state differently

**Found:** 2026-10-10 sweep register sync (issue filed 2026-10-10)
**Issue:** #1557. The runbook runs all steps in one `bash`; the step path uses a fresh shell per step.
**Status:** open
**Needs human?:** yes (product: which behavior is the contract)
**Next step:** maintainer picks an option in #1557; recommended option 1 (subshell per step in the runbook).

### BW-047: upgrade reqwest 0.11 → 0.12 to clear h2 RUSTSEC-2026-0258 (wiremock is already 0.6.4)

**Found:** 2026-10-10 sweep register sync (issue filed 2026-09-24)
**Issue:** #1472 (related: #1446, the vulnerability report)
**Status:** open
**Needs human?:** yes (dependency major-version bump)
**Next step:** maintainer approves the bump; note that hf-hub 0.5 already pulls reqwest 0.12.

### BW-051: unrendered website components and unread locale fields hold 156 emoji

**Found:** 2026-10-11, emoji-icon inventory (`docs/brand/icons/definitions.json`, #1590)
**Issue:** #1591
**Status:** open
**Needs human?:** yes for the distro-preferences UI (planned feature or remove); no for the rest
**Next step:** remove `OmniMenu.astro`, `Waitlist.astro`,
`landing/LPUseCases.astro` and the unread locale fields listed in #1591.

---

## Dedup done (2026-10-09)

The 2026-10-03 and 2026-10-07 tables are closed: 26 duplicates closed on
2026-10-09 (#1520 #1541 → #1098; #1247 … #1392 → #1221; #1169 … #1205 → #1165;
#1170 → #1162; #1328 → #1163; #1523 → #1269; #1540 → #1272), plus #985 → #917.
#1329 closed as fixed by #1503 (guard: `test_needs_stdin_prompt`).

Left open on purpose, because they are related but not the same defect: #1400
(Pattern 43 plus the CPU stub together), #1408 (init feedback, vs #1272),
#1267 (TTY hang), #1446 vs #1472 (vulnerability vs upgrade plan), #1164 vs
#1222 (same root cause, fixed together by BW-010), #1183/#1203/#1508 (missing
waitlist/playbook locale files; #1508 is the CI symptom of both).

### BW-052: `Install` workflow red on every main push: setup.sh step exits 127

**Found:** 2026-10-11, broken-window sweep (main health)
**Issue:** #1340. The cleanup step deletes `~/.cargo/bin/caro` but not cargo's install record. `setup.sh` runs `cargo install caro`, which prints "already installed" and installs nothing.
**Status:** claimed-by sweep/2026-10-11-BW-052 (#1605)
**Needs human?:** no (test-step fix; no gate changes)
**Next step:** merge the PR; the Install workflow runs on it.

### BW-053: an invalid `config.toml` is silently replaced with defaults

**Found:** 2026-10-11, broken-window sweep
**Issue:** #1566
**Status:** open
**Needs human?:** no
**Next step:** Fail loudly, or back up the bad file before writing defaults.

### BW-054: landing demo video has no captions or text alternative

**Found:** 2026-10-11, broken-window sweep
**Issue:** #1570
**Status:** open
**Needs human?:** no
**Next step:** Add a captions track and a text alternative.

### BW-055: localized homepages use an older layout than the English homepage

**Found:** 2026-10-11, broken-window sweep
**Issue:** #1571
**Status:** open
**Needs human?:** no
**Next step:** Port the English landing layout to the locale pages.

### BW-056: website visual-review P2s left after #1565

**Found:** 2026-10-11, broken-window sweep
**Issue:** #1572
**Status:** open
**Needs human?:** yes (item 2 only)
**Next step:** Fix the other items; ask about item 2.

### BW-057: `block-budget-leaks` hook blocks merging main (main has a byte count in the register)

**Found:** 2026-10-11, broken-window sweep
**Issue:** #1577
**Status:** open
**Needs human?:** yes (hook policy)
**Next step:** Owner picks: allow merge commits, or narrow `SENSITIVE_RE`.

### BW-058: caro.sh 307-redirects to www.caro.sh, but canonical/og/sitemap URLs say caro.sh

**Found:** 2026-10-11, broken-window sweep
**Issue:** #1586
**Status:** open
**Needs human?:** yes (Vercel domain setting)
**Next step:** Owner picks the primary host: make `caro.sh` the primary
domain in Vercel and 308-redirect `www` to it, or change `site` in
`website/astro.config.mjs` to the `www` host.

### BW-059: `/blog` index omits two published posts

**Found:** 2026-10-11, broken-window sweep
**Issue:** #1587
**Status:** open
**Needs human?:** no
**Next step:** Add `claude-skill-launch` and `founders-playbook-adaptation`
to the `posts` array in `website/src/pages/blog/index.astro`.

### BW-060: landing page says caro blocks `chmod -R 777 /var/www`; the validator does not flag it

**Found:** 2026-10-11, broken-window sweep
**Issue:** #1592
**Status:** open
**Needs human?:** yes (safety pattern or copy)
**Next step:** See the options in the issue.

### BW-061: PR workflows never cancel superseded runs, so the macOS queue backs up

**Found:** 2026-10-11, broken-window sweep
**Issue:** #1593
**Status:** open
**Needs human?:** yes (workflow concurrency)
**Next step:** Add `concurrency` groups after review.

### BW-062: website claims suite never runs the validator; 8 of 12 landing scenarios claim warnings caro does not give

**Found:** 2026-10-11, broken-window sweep
**Issue:** #1597
**Status:** open
**Needs human?:** yes (safety patterns or copy)
**Next step:** See the suggested order in the issue.

### BW-063: `caro test --suite` ignores `dangerous_pattern`, so the safety skill's YAML RED/GREEN step tests nothing

**Found:** 2026-10-11, broken-window sweep
**Issue:** #1599
**Status:** open
**Needs human?:** no
**Next step:** Make the runner check `dangerous_pattern`.

### BW-064: caro-demo-video skill tells agents to merge or push to main without review

**Found:** 2026-10-11, broken-window sweep
**Issue:** #1602
**Status:** open
**Needs human?:** no
**Next step:** Change the skill text to the branch + PR flow.

### BW-065: QA rotation memory files stuck at the 2026-05-07 bootstrap; 25+ update PRs unmerged

**Found:** 2026-10-11, broken-window sweep
**Issue:** #1603
**Status:** open
**Needs human?:** yes (owner merges or closes the PRs)
**Next step:** Owner triages the open update PRs.

### BW-066: `website/package-lock.json` was dead and stale

**Found:** 2026-10-12, owner task (`claude/zealous-meitner-y29aw1`)
**Issue:** none (owner asked for the fix directly)
**Status:** fixed (#1608)
**Needs human?:** no
**Next step:** merge #1608. npm reads only the root lockfile for workspaces;
`website/README.md` → "Workspace and lockfile" documents it.

### BW-067: stale nested lockfiles in the other npm workspaces

**Found:** 2026-10-12, while fixing BW-066
**Issue:** #1609
**Status:** open
**Needs human?:** no
**Next step:** For `docs-site/`, `landing/`, `presentation/` and
`apps/devrel/`: confirm each deploy target installs from the repo root, then
delete the nested `package-lock.json` as #1608 did for `website/`.
