# Caro Integrator — Nightly Log

> One-line-per-night journal maintained by the **caro-integrator** agent.
> Newest entries on top. Append, don't overwrite.

---

## 2026-05-13 (23:00 PT fire) — option 2 loud-error code fix for #1081

- **Validated:** re-confirmed [#1081](https://github.com/wildcard/caro/issues/1081) reproduces on the newly-published `caro 1.4.0` from crates.io (the prior validation was on 1.3.0). `caro --backend ollama --dry-run -p "list pdf files"` still emits `WARN  caro::cli: Remote backends not compiled in` and silently falls through to `ls -la` via the static matcher. Identical reproduction for `vllm` and `exo`. `--backend claude` still returns `Error: Invalid argument: Unknown backend 'claude'`. `--backend embedded` works (control). Evidence: claim-verification step before touching code, per the project rule that says "code premised on a stale evidence base is a wasted PR."

- **Shipped:** code fix at `src/cli/mod.rs` — replaces the `tracing::warn!` silent-fallback arm (lines 395–400) with `return Err(Self::remote_backend_unavailable_error(model))`. Extracted a tiny `remote_backend_unavailable_error(backend: &str) -> CliError` helper (gated `#[cfg_attr(feature = "remote-backends", allow(dead_code))]`) so the error message can be unit-tested in isolation. Message names the requested backend, the missing feature, the exact `cargo install caro --features remote-backends --locked` invocation that fixes it, and the no-setup-required embedded alternative. Added `test_remote_backend_unavailable_error_message` in `cli::tests` covering all four message assertions. Smoke-tested against the branch binary: `caro --backend ollama` → exit 1 with the new error; `caro --backend vllm` and `caro --backend exo` produce equivalent errors; `caro --backend embedded` exit 0 (no regression). Both feature configurations (`embedded-cpu` and `embedded-cpu remote-backends`) clippy-clean with `-D warnings`; `cargo fmt --all -- --check` clean.

- **Filed:** none. Tonight's PR closes the loop on the existing P1 ([#1081](https://github.com/wildcard/caro/issues/1081)) for option **2** of its remediation menu; the issue itself stays open until the related options ship (option 3: default-feature flip, or release-workflow `--features` injection; option 5: Claude CLI wiring). Dedup-checked `gh issue list --label integration --state all --search "remote-backends OR loud-error OR silent fallback"` — no other matches.

- **Discovered:**
  - **PR #1082** (last night's integrator's docs re-statusing PR) is still **open**, with `Lint & Format` and `MSRV Check (Rust 1.85)` CI failures. Both failures originate in pre-existing main code (`src/evaluation/evaluators/utils.rs` fmt — already removed on main by [#1084](https://github.com/wildcard/caro/pull/1084), and `backends::static_matcher::tests::test_website_example_2` — also fixed by #1084). A rebase of PR #1082 onto current `main` (HEAD `fcfa844a`) will clear both. Out of scope for tonight's one-PR-per-night budget — flagged for next pass or for a maintainer's hand on the original branch.
  - `caro 1.4.0` shipped to crates.io between the last fire and tonight (`max: 1.4.0 newest: 1.4.0` from `crates.io/api/v1/crates/caro`). New top-level subcommands include `caro skill`, `caro do`, `caro run`, `caro generate`, `caro adopt`, `caro experiment`, `caro history`, `caro why`, `caro check`, `caro list`, `caro jobs`, `caro new`, `caro export`, `caro render`, `caro ai`, `caro suggest` — the CaroML task language has clearly landed. `caro mcp serve` and `caro serve --openai` are **not** yet in the help output, so the matrix queue ordering is unchanged. No new `--backend` options either.
  - `caro skill` subcommand exists — useful next-pass surface for surfacing the bundled `caro-shell` skill from the CLI itself. File this as a research item rather than a regression.

- **Next pass should:**
  - (a) Rebase PR #1082 onto current `main` if it's still open and unmerged, so the previous integrator's matrix-row re-statusing can finally land. Trivial rebase; the only conflict will be on the date-banner line in `integrations-status.md` which intentionally was not edited tonight.
  - (b) Pursue [#1081](https://github.com/wildcard/caro/issues/1081) option **5** (Claude CLI wiring) — `validate_backend_name` arm for `"claude"`, a new `create_backend` match arm wiring `ClaudeBackend::new(api_key)`, and a config-error path for the missing `ANTHROPIC_API_KEY`. ~30 LOC, fits a single-night PR. The existing `src/backends/remote/claude.rs` already does the heavy lifting; only CLI glue is missing.
  - (c) If both (a) and (b) are unblocked, prioritize (a) — keeping last night's PR alive is higher value than starting a new code fix.

---

## 2026-05-11 (23:00 PT fire) — first real validation pass; remote-backend packaging gap surfaced

- **Validated:** native backends top-three priority row.
  - **MLX / Candle CPU (embedded)** — ✓ VERIFIED PASS. `caro --backend embedded --dry-run -p "list pdf files in current directory"` → `ls *.pdf`. Default `caro --dry-run -p "show disk usage"` → `du -sh /Users/kobik-private/workspace/caro/.worktrees/integrator-20260511 | sort -rh | head -10`. Both work in the default `cargo install caro` build.
  - **Ollama / vLLM / Exo** — ⚠️ PARTIAL. CLI accepts the flag but the published v1.3.0 binary logs `WARN  caro::cli: Remote backends not compiled in. Build with --features remote-backends` and silently falls back to the static matcher. Reproduced for all three flags identically. Root cause: `default = ["embedded-mlx", "embedded-cpu", "cve-rules"]` in `Cargo.toml`; neither `cargo install caro` nor the release workflow (`.github/workflows/release.yml:245`) passes `--features remote-backends`.
  - **Anthropic Claude API** — ❌ FAIL. `caro --backend claude --dry-run -p "list pdf files"` → `Error: Invalid argument: Unknown backend 'claude'`. Root cause: `validate_backend_name()` at `src/cli/mod.rs:468` hardcodes `VALID_BACKENDS = ["embedded","ollama","exo","vllm"]`; the `BackendType::Claude` variant + `ClaudeBackend` struct exist but `create_backend()` never instantiates them. Broken in every build, not feature-gated.

- **Shipped:** doc-accuracy PR — `website/src/data/integrations.ts` re-statuses the 6 backend rows from `'working'` with `lastValidated: null` to honest `'working' / 'partial' / 'in-progress'` with `lastValidated: '2026-05-11'`; `.claude/skills/caro-shell/SKILL.md` qualifies the "remote providers" claim; `.claude/memory/integrations-status.md` rows + priority queue refreshed.

- **Filed:** [#1081](https://github.com/wildcard/caro/issues/1081) `integration: published caro binary omits remote-backends feature; Claude backend unreachable` — P1, labels `integration` + `backend` + `bug` + `P1` + `regression` + `nightly-discovery`. Documented 5 remediation options ranging from doc-only (this PR) to a 1-LOC default-feature flip to Claude CLI wiring; recommendation noted on the issue is "loud error + release-workflow `--features remote-backends` + separate Claude wiring PR". Deduped against #790 (broad v1.1.3 gap analysis), #791 (website gap analysis), #792 (credibility-gap epic), #809 (website copy fixes), #843 (provider-neutral messages, blocked on this being reachable first) — none of those were the right home for a v1.3.0 packaging-shaped finding.

- **Discovered:**
  - Two adjacent rules-of-the-road are worth noting for future passes. (a) The skill description `caro-shell/SKILL.md` referenced "remote providers (Anthropic, Ollama, vLLM, Exo)" but none of those are reachable from a default install — corrected in tonight's PR. (b) The website matrix entry `IntegrationStatus = 'working'` is defined as "✅ Validated end-to-end against published caro binary" yet had 6 backend rows marked `'working'` with `lastValidated: null` — a contradiction that the first-night seed didn't catch. The next nightly should add a `null`-lastValidated invariant check to either the matrix linter or the integrator playbook itself.
  - OpenRouter is now 290+/400+ models (May 2026 — `openrouter.ai/docs/guides/routing/routers/auto-router`) so issue #931 becomes higher leverage than the initial seed estimated.
  - Anthropic-skills marketplace published a "delegates coding tasks to Codex, Claude Code, or Pi agents" skill on 2026-05-11 — Pi shows up as a downstream target again. Adds weight to keeping Pi on the long-tail integration list.

- **Next pass should:** pick up [#1081](https://github.com/wildcard/caro/issues/1081) directly. The most defensible single-PR scope from the remediation menu is option **2 (loud error instead of silent fallback)** — ~10 LOC at `src/cli/mod.rs:402`, no policy ambiguity, restores the contract. Defer option 3 (default-feature flip) until the user weighs in on binary-size posture. Defer option 5 (Claude CLI wiring) to a separate night so the two changes don't bundle.

---

## 2026-05-10 (23:00 PT fire) — third bootstrap-blocked night, awaiting merge action

- **Validated:** none — bootstrap-check still says PLAYBOOK_NOT_MERGED. PR #939 remains OPEN. Per the scheduled-task header protocol, no integration-row validation runs until the playbook lands on `main`.
- **Shipped:** this log entry. Status delta vs the 2026-05-09 23:00 fire: ✅ codespell now green (last fire's `preserv` ignore landed); ✅ MSRV Check (Rust 1.83) now green (PR #1056 bumped MSRV to 1.85 on `main` — committed 4dc902a9, 2026-05-09 14:00 UTC). Remaining failing checks on PR #939 are `ChromaDB Integration Tests` (pre-existing flake — `test_chromadb_multiple_operations` fails with `left: 14, right: 10`; `test_chromadb_record_success` similarly fails; same failure repro'd on `main` push run [25636983573](https://github.com/wildcard/caro/actions/runs/25636983573); root cause looks like `ChromaDbBackend::clear()` only wipes `caro_commands` while `stats()` sums entries across all five sub-collections — `caro_commands`, `caro_corrections`, `caro_command_docs`, `caro_user_preferences`, `caro_project_context` — so prior test residue is counted by later assertions; this is a different bug than the closed #537 which was about parallel-test isolation and got fixed by the unique-collection-name swap) and `Vercel – cmdai` (vestigial pre-rename project deploy — out of scope per fire #2's note). Neither failing check touches a file PR #939 modifies (only `.claude/memory/`, `.claude/skills/caro-shell/`, `.codespellignore`, `README.md`, `website/src/data/integrations.ts`, `website/src/pages/integrations/index.astro`).
- **Filed:** none. ChromaDB `clear()` multi-collection bug is internal infra (not integration scope per the integrator charter); deferring to QA / coder-loop. No new integration-labeled issues to file. Dedup-checked the existing tracker: `gh issue list --label integration` has no overlap with tonight's findings.
- **Discovered:** PR #939's `mergeStateStatus` is `UNSTABLE` but `mergeable` is `MERGEABLE`. Verified `main` is unprotected (`gh api repos/wildcard/caro/branches/main/protection` → 404 Branch not protected) so no required-check policy blocks merge. The PR can be merged manually right now without overriding any policy. The pre-merge bootstrap pattern flagged on fire #1 ("auto-skip on subsequent nights") is biting now — third no-op night in a row. If a fourth fire is also blocked, the next agent should consider proposing a tiny enhancement to the scheduled-task header to `exit 0` immediately when `PLAYBOOK_NOT_MERGED && tonight's PR is unchanged from the last fire`.
- **Next pass should:** if PR #939 has merged by the 2026-05-11 23:00 fire, run the full Step 1–9 loop against the topmost queue row ("validate the 6 native backends" — none have a real `last-validated` date). If still unmerged, follow the auto-skip suggestion above OR escalate via `**Needs user input:**` again with even tighter framing.

---

## 2026-05-09 (23:00 PT fire) — codespell unblock on PR #939

- **Validated:** none — bootstrap-check still says PLAYBOOK_NOT_MERGED, so no integration-row validation runs. Per the protocol, this fire's job is to make PR #939 mergeable.
- **Shipped:** `.codespellignore` now lists `preserv` so codespell stops flagging the deliberate regex word-stem at `src/backends/static_matcher.rs:1406` (`(preserv|maintain|keep)` matches `preserve`/`preserves`/`preserved`/`preserving`). Verified locally with `uvx codespell` — exit 65 → exit 0 after the one-line addition. The `preserv` token is intentionally never spelled out as the full word in this regex; mangling it to `preserve` would silently break the safety pattern. Pre-existing condition on `main` (codespell job there only passes by virtue of GHA docker-image cache divergence; locally on 2.4.2 it fails identically). Defensive — fixes both surfaces.
- **Filed:** none. The other PR #939 CI failures — `ChromaDB Integration Tests` (known shared-collection flake), `MSRV Check (Rust 1.83)` (pre-existing on `main`, not introduced by #939), `Vercel – cmdai` (vestigial pre-rename project) — remain out of scope per the one-PR-per-night budget. They will re-roll on the fresh push.
- **Discovered:** `MSRV Check` failing on PR #939 is interesting because PR #939 adds zero Rust code. That means MSRV on `main` is also red — file as a follow-up next pass after dedup against `gh issue list --label ci --search "MSRV"`.
- **Next pass should:** if PR #939 has merged, run the full Step 1–9 loop against the topmost queue row (likely "validate the 6 native backends"). If still unmerged, re-check CI and triage the next gating failure — but do NOT bundle multiple fixes into one PR; one tight commit per night.

---

## 2026-05-09 (00:00 PT fire) — first scheduled cron pass (bootstrap-night fix)

- **Validated:** none — first scheduled fire on a not-yet-merged scaffolding PR. Per the bootstrap-check protocol in the scheduled-task header, no integration-row validation runs until PR #939 is on `main`.
- **Shipped:** this log entry. Diagnosis: PR #939's CI history shows 2 failures (ChromaDB Integration Tests + Security Audit) from the 2026-04-27 run, but the branch was rebased onto current `main` (`f8028edb`) on 2026-05-06. The rebase already includes the RUSTSEC-{0098,0099,0104} fix from PR #1026 (`rustls-webpki 0.103.13` in `Cargo.lock`, legacy 0.101.7 path covered by `.cargo/audit.toml` ignores). ChromaDB failures (`expected 10, got 14`; `expected 1, got 15`) are pre-existing flake from shared-collection state pollution — observed intermittently on `main` too. Pushing this commit triggers a fresh CI run on the rebased HEAD, which should clear Security Audit and re-roll the ChromaDB flake. Vercel `cmdai` failure is a vestigial pre-rename project deploy — out of scope.
- **Filed:** none. ChromaDB flake is real but already a known issue; not filing a duplicate without a `gh issue list --label integration --search chromadb` dedup pass next night when the bootstrap completes.
- **Discovered:** the pre-merge bootstrap loop has a long-tail risk — if the scaffolding PR sits unmerged for many nights, every nightly fire produces a near-duplicate "fix CI / wait for merge" entry. Mitigations: (a) keep this PR small enough to merge fast (it is), (b) auto-skip the loop on subsequent nights if state hasn't changed (future enhancement, file later if it bites).
- **Next pass should:** if PR #939 has merged by tomorrow's 23:00 fire, run the full Step 1–9 loop and pick up the topmost queue row from `integrations-status.md` — most likely "validate the 6 native backends" since none have a real `last-validated` date. If still unmerged, this entry tells the next agent the situation is identical and the right move is `**Needs user input:** still waiting on PR #939 merge` rather than another no-op commit.

---

## 2026-04-26 — first night (manual, pre-cron)

- **Shipped:** caro-shell Claude Code skill (`.claude/skills/caro-shell/SKILL.md`); website integrations data + landing page (`website/src/data/integrations.ts`, `website/src/pages/integrations/index.astro`); README "Use caro from your agent" section; integrator persona + playbook + status matrix + this log; nightly cron `caro-integrator-nightly` at `0 23 * * *`.
- **Validated:** Skill installs in a fresh Claude Code session and invokes the published `caro` binary end-to-end (verification step #4 from the plan). Other rows in the status matrix are seeded with `not-yet` validation dates — to be picked up over the next several nights.
- **Filed:** companion GH issues — deduped against #661/#667/#662/#504/#449/#789/#782:
  - #928 — caro mcp serve (Claude Code MCP server)
  - #929 — caro serve --openai (OpenAI-compat HTTP shim)
  - #930 — Claude Code session-token reuse backend
  - #931 — OpenRouter backend
  - #932 — native-backend nightly validation harness
  - #933 — long-tail tool copy-paste snippets (umbrella)
- **Next:** first scheduled cron firing at 23:00 local. The agent will pick up the topmost unblocked queue row — likely "validate the 6 native backends" since none have a real `last-validated` date.
