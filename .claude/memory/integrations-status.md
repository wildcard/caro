# Caro Integrations — Status Matrix

> Living matrix maintained by the **caro-integrator** nightly agent.
> Updated every nightly pass (cron `0 23 * * *`).
>
> **Last updated:** 2026-11-26 — ✓ VERIFIED published `caro 1.5.0` (still newest on crates.io). Re-validated the oldest rows (2026-11-22): `caro-shell` skill, `integrations.ts` and README. Results are the same as 2026-11-22 (#1627, #1628, #1116/#1662). First check of the README basic-usage flags: `--shell zsh`, `--output json`, `--safety permissive`, `--confirm`, `--force-llm` and `--knowledge-backend chromadb` all parse and exit 0. `remove old log files` → `mv source.txt /path/to/dest.txt` (already in #1662). Nothing merged to `main` since 2026-11-02; no new PR.
>
> **Previous:** 2026-11-25 — ✓ VERIFIED published `caro 1.5.0` (still newest on crates.io). Re-validated the oldest rows (2026-11-19): MLX embedded (`list pdf files…` → `ls -la`, exit 0, still #1645); claude/ollama/vllm/exo/openrouter/grok each → `Configuration error: … requires the 'remote-backends' feature`, exit 1; `--backend-info` lists only `embedded` as available. #1723 hang still reproduces on the published binary (exit 142). Nothing merged to `main` since 2026-11-02; no new PR (rule 6a, eighth night).
>
> **Previous:** 2026-11-24 — ✓ VERIFIED published `caro 1.5.0` (still newest on crates.io). Re-validated the three oldest rows (2026-11-18): guard adapters for Codex (`rm -rf /` → `permissionDecision: deny`; `ls -la` → no output), Grok Build (camelCase payload → deny) and opencode (`rm -rf /` → `verdict: deny`, exit 2; `ls` → `verdict: none`, exit 0). #1723 stdin hang still reproduces (exit 142). No new tools confirmed. Nothing merged since 2026-11-02; no new PR.
>
> **Previous:** 2026-11-23 — ✓ VERIFIED published `caro 1.5.0` (still newest on crates.io). Re-validated the three oldest rows (2026-11-17): Candle CPU (`show disk usage` → `df -h`), `caro guard` (`rm -rf /` → deny, `ls -la` → allow), live `caro.sh/integrations` (200, 1.5.0 config-error copy present). #1723 stdin hang still reproduces (exit 142). Added a ⏳ Kiro row (single secondary source). Nothing merged since 2026-11-02; no new PR.
>
> **Previous:** 2026-11-22 — ✓ VERIFIED published `caro 1.5.0` (still newest on crates.io; installed binary `caro 1.5.0 ( crates.io)`). Re-validated the three oldest rows: `website/src/data/integrations.ts` and README (2026-10-13; #1657 fix from #1689 is on `main`) and the `caro-shell` skill (2026-11-09). Skill Examples 1 and 2 still do not match 1.5.0 output (#1628, #1627). The MLX snippet on the website (`kill the process on port 3000`) returns a literal `kill PID` (#1116, #1662). Nothing merged since 2026-11-02; no new PR.
>
> **Previous:** 2026-11-19 — ✓ VERIFIED published `caro 1.5.0` (still newest on crates.io). PASS: MLX embedded (`list pdf files…` → `ls -la`, still #1645); claude/ollama/vllm/exo/openrouter/grok each → `Configuration error: … requires the 'remote-backends' feature`, exit 1; `--backend-info` lists only `embedded` as available. #1723 stdin hang still reproduces (exit 142 at 8 s; `</dev/null` control passes). No new PR: rule 6a, backlog unchanged.
>
> **Previous:** 2026-11-18 — ✓ VERIFIED published `caro 1.5.0` (still newest on crates.io). PASS: `caro guard` adapters for Codex, Grok Build and OpenCode (enforce mode, `rm -rf /` → deny, safe → allow). #1723 stdin hang still reproduces; fix PR #1754 still open. No new PR: 6a merge-backlog rule applies.
>
> **Previous:** 2026-11-09 — ✓ VERIFIED published `caro 1.5.0` (still newest on crates.io). Re-validated Candle CPU, `caro guard`, and the `caro-shell` blocking path. New finding: agent-subprocess stdin hang (#1723) reproduces on 1.5.0; fix is open as PR #1754. No code PR from the integrator tonight.
>
> **Previous:** 2026-10-19 — ✓ VERIFIED against published `caro 1.5.0` (crates.io + GH release, tagged 2026-10-13 — the release-cadence gap flagged on 2026-07-11 is closed). `--backend-info`, `--backend claude/openrouter/static` and `--backend embedded` all re-validated: claude/grok/openrouter now give a clean `Configuration error: ... requires the 'remote-backends' feature` (no more `Unknown backend` for advertised names, no more silent WARN-fallback for ollama/vllm/exo). Shipped `docs(integrations): fix stale claude/ollama backend descriptions + --model-name snippet for 1.5.0` — no code PR needed, the P1 was a docs/website staleness gap (#1657), confirmed scoped and non-conflicting with open PR #1504 (different lines).
>
> **Previous:** 2026-10-13 — **caro 1.5.0 is published on crates.io** (2026-10-13 01:27Z). Re-validated the `caro-shell` skill + marketing rows against `cargo install caro` 1.5.0: the skill's backend guidance and `--model` flag were stale (fixed for #1629 in tonight's PR); the website `integrations.ts` has the same drift (filed #1657). The default 1.5.0 binary still omits `remote-backends` (#1081).
>
> **Previous:** 2026-10-03 — PR #1513 (`claude/nifty-brown-mtjkpu`) adds the experimental `caro guard` PreToolUse hook (Claude Code / Grok Build / Codex / OpenCode) and wires `grok`, `claude` and `openrouter` into `--backend`. Non-`remote-backends` builds get the existing loud "requires remote-backends" error for those names, the same as `mesh`/`vllm`, so the #1298 divergence stays closed.
>
> **Previous:** 2026-07-11 (post-merge pass) — **PR #1298 is MERGED** (2026-07-12 UTC) and **issue #1115 is CLOSED**: the acute half of the backend-roster divergence is resolved on `main` (`--backend-info` / `available_backends()` / `--backend` help all iterate the single source of truth `backends::CLI_SERVABLE_BACKENDS`, shared with `validate_backend_name`). No code PR tonight — the fix already exists on `main`, and re-touching the roster would regress it (see below). Instead: re-validated the **still-published `caro 1.4.0`** (crates.io, 2026-05-09) and confirmed the #1115 P0 is **live in the shipped artifact** — `--backend-info` advertises `static`+`claude`, `--backend claude`/`--backend static` → `Unknown backend`. This is now purely a **release-cadence gap**: `main` exposes 7 CLI-servable backends (adds `mesh`/`ai-horde`/`hybrid` from #1209), the shipped binary knows 4. Posted a post-#1298 status update to **#1081** (the surviving tracking home now that #1115 is closed) covering the remaining wiring-half + release gap. **The wiring half (claude/openrouter arms in `create_backend`) is NOT trivial**: `CLI_SERVABLE_BACKENDS` is not feature-gated, so adding those names without a `#[cfg(feature="remote-backends")]` split would re-open the divergence for default builds. See log.

## Legend

| Status | Meaning |
|---|---|
| ✅ working | Validated end-to-end against published `caro` binary on `last-validated` date |
| ⚠️ partial | Validated but with caveats (documented in `notes`) |
| 🚧 in-progress | Implementation in flight (PR linked) |
| ❌ broken | Was working, now fails — **P0/P1 fix required** |
| 🧪 experimental | Shipped behind an experimental label (shadow/opt-in); not yet validated against a published binary |
| ⏳ not-yet | Tracked target, no implementation yet |
| 🚫 n/a | Out of scope for caro |

---

## Native backends (in caro)

| Tool | Status | Last validated | Method | GH | Notes |
|---|---|---|---|---|---|
| Anthropic Claude API | ⚠️ partial (feature-gated) | 2026-11-25 | ✓ VERIFIED published `caro 1.5.0`: `caro --backend claude --dry-run -p "..."` → `Configuration error: Backend 'claude' requires the 'remote-backends' feature, which is not compiled into this build.` + fix instructions. `--backend-info` lists it as `not compiled` (consistent with the error). | #1081 (tracking home; wiring is done, feature-gate omission from default build remains) | #1115/#1298 divergence is CLOSED for claude: no more `Unknown backend`, no silent fallback — a clean, consistent config error. Needs `--features remote-backends` + `ANTHROPIC_API_KEY` to actually route. |
| Ollama | ⚠️ partial (feature-gated) | 2026-11-25 | ✓ VERIFIED published `caro 1.5.0`: `caro --backend ollama --dry-run -p "..."` → `Configuration error: ... requires the 'remote-backends' feature`, exit 1. No more silent WARN-fallback. | — | Behavior improved since 2026-05-11: was silent fallback, now a loud, actionable error. `remote-backends` still not in default features. |
| vLLM | ⚠️ partial (feature-gated) | 2026-11-25 | ✓ VERIFIED published `caro 1.5.0`: same loud config error as Ollama, no silent fallback | — | Same root cause/fix as Ollama; same improvement since 2026-05-11. |
| Exo | ⚠️ partial (feature-gated) | 2026-11-25 | ✓ VERIFIED published `caro 1.5.0`: same loud config error as Ollama, no silent fallback | — | Same root cause/fix as Ollama; same improvement since 2026-05-11. |
| MLX (Apple Silicon embedded) | ✅ working | 2026-11-25 | ✓ VERIFIED published `caro 1.5.0`: `caro --backend embedded --dry-run -p "list pdf files in current directory"` → `ls -la` (file-type qualifier dropped — see #1645, separate from this row) | — | `embedded-mlx` is in default features; works in default `cargo install caro` build. |
| Candle CPU (embedded) | ✅ working | 2026-11-23 | ✓ VERIFIED published `caro 1.5.0`: `caro --dry-run -p "show disk usage" </dev/null` → `df -h` ✓ (auto-fallback path) | — | `embedded-cpu` is in default features; works in default build. |
| OpenRouter | ⚠️ partial (feature-gated) | 2026-11-25 | ✓ VERIFIED published `caro 1.5.0`: `caro --backend openrouter --dry-run -p "..."` → same loud `Configuration error: ... requires the 'remote-backends' feature` | #931, #1081 | `openrouter.rs` is `openai_compat.rs`; CLI-wired since PR #1513, same feature-gate omission as the others. |
| xAI Grok | ⚠️ partial (feature-gated) | 2026-11-25 | ✓ VERIFIED published `caro 1.5.0`: `caro --backend grok --dry-run -p "..."` → same loud `Configuration error: ... requires the 'remote-backends' feature` | PR #1513 | `--backend grok` / `--advisor grok` / `hybrid_remote = "grok"`; `XAI_API_KEY`. Consistent with claude/openrouter/ollama/vllm/exo. |
| Gemini / Jules | 🚧 in-progress | — | — | PR #782 | Coordinate with existing PR; don't fork |
| LM Studio + FunctionGemma | 🚧 in-progress | — | — | PR #558 | Tracked, no action this night |
| quant.cpp | 🚧 in-progress | — | — | PR #838 | Tracked, no action this night |
| Azure Foundry | ⏳ not-yet | — | — | #661 (epic) | Big lift; deprioritized |
| Claude Code session-token | ⏳ not-yet | — | — | #930 | Reuse user's CC subscription/API auth; default to Haiku |

## Caro-as-a-tool (outward integrations)

| Tool | Status | Last validated | Method | GH | Notes |
|---|---|---|---|---|---|
| Claude Code skill (`caro-shell`) | ⚠️ partial | 2026-11-26 | 2026-11-26: same results as 2026-11-22. 2026-11-22 (published 1.5.0, `</dev/null`): `caro --backend-info` lists claude/grok/openrouter as `not compiled` ✓; Example 1 → `find . -name "*.py" -type f` (drops `src` and `-mtime +180`, #1628); Example 2 → `docker system prune -f && docker volume prune -f` with no risk note (skill says `docker volume prune -f`, MEDIUM; #1627); `--model foo` → clap usage error with `--model-name` tip ✓ (skill already says `--model-name`). 2026-11-09: `caro --dry-run "delete everything in /tmp"` → Critical block ✓; **agent-subprocess hang (#1723, PR #1754)**: with an open stdin pipe, `caro --dry-run -p "list files"` printed nothing in 8s on 1.5.0. Earlier (2026-10-13): published 1.5.0: skill's `caro --dry-run "<prompt>"` path runs; Example 3 (`delete everything in /tmp`) → CRITICAL block ✓; Example 1 drops `src`+age (#1628); Example 2 → `docker system prune -f && …` (#1627); `--model` flag in skill → clap usage error (fixed for #1629) | #1629, #1490 | Distributed via plugin marketplace (#1624); not by `caro skill install` (that ships `caro-scaffold`) |
| Harness guardian hook (`caro guard`) | 🧪 experimental | 2026-11-23 | ✓ VERIFIED published `caro 1.5.0`: `rm -rf /` → `permissionDecision: deny` (critical risk); `ls -la` → no output (allow). Also `tests/e2e_guard.rs` | spec 011 / ADR-020 | PreToolUse adapter for Claude Code, Grok Build, Codex; OpenCode via `integrations/opencode/caro-guard.ts`. Shadow by default |
| Claude Code MCP server (`caro mcp serve`) | ⏳ not-yet | — | `mcp-inspect` against `caro mcp serve` | #928 | Spec: `.github/first-time-issues/06-mcp-claude-code-integration.md`; tools `generate_command` / `validate_command` / `explain_safety` / `show_decision_tree` |
| OpenAI-compat HTTP shim (`caro serve --openai`) | ⏳ not-yet | — | `curl /v1/chat/completions` with a tool call | #929 | Highest leverage — unlocks Codex/Cursor/Continue/Aider/Tabby in one shot |
| Codex (OpenAI) | 🧪 guard only | 2026-11-24 | ✓ VERIFIED published 1.5.0: Claude-shaped payload `rm -rf /` → `caro guard --harness codex --mode enforce` emits `permissionDecision: deny`; `ls -la` → no output. Config per docs/coding-agents/codex.md | #789 (Crush MCP config PR) | Generation still awaits OpenAI shim |
| Cursor | ⏳ not-yet | — | OpenAI shim snippet copy-pasted into Cursor settings | — | Satisfied by OpenAI shim |
| Continue (continue.dev) | ⏳ not-yet | — | OpenAI shim snippet | — | Satisfied by OpenAI shim |
| Aider | ⏳ not-yet | — | OpenAI shim snippet | — | Satisfied by OpenAI shim |
| Tabby (self-hosted) | ⏳ not-yet | — | OpenAI shim snippet | — | Satisfied by OpenAI shim |
| opencode | 🧪 guard only | 2026-11-24 | ✓ VERIFIED published 1.5.0: `{"command":"rm -rf /"}` → `caro guard --harness opencode --mode enforce` → `verdict: deny`, exit 2; `ls` → `verdict: none`, exit 0. Plugin `integrations/opencode/caro-guard.ts` | TBD | sst/opencode; generation still awaits MCP/shim |
| crush | ⏳ not-yet | — | MCP or OpenAI shim | #789 (related) | Charm.sh ecosystem |
| droid | ⏳ not-yet | — | MCP or OpenAI shim | TBD | |
| Kiro (AWS) | ⏳ not-yet | — | MCP or hooks | — | Added 2026-11-23 from one secondary source (agentic-coding landscape post, 2026-11-03); confirm CLI surface before filing |
| Sourcegraph Amp | ⏳ not-yet | — | MCP | TBD | Enterprise coding agent |
| Letta (MemGPT) | ⏳ not-yet | — | Tool registration | TBD | Persistent-memory agents |
| Pi | ⏳ not-yet | — | TBD | TBD | Earendil Pi 1.0 (MIT, terminal agent) reported stable 2026-10-01; unverified secondary source (2026-11-17 research) Pi 1.5.0 reported 2026-11-02 (single source, unverified). |
| Aug | ⏳ not-yet | — | TBD | TBD | |
| CodePal | ⏳ not-yet | — | TBD | TBD | |
| Kilo CLI | ⏳ not-yet | — | MCP or OpenAI shim | — | Kilo CLI 1.0, open-source terminal agent (discovered 2026-11-17) |
| Cline CLI | ⏳ not-yet | — | MCP or OpenAI shim | — | Cline CLI 2.0 (discovered 2026-11-17) |
| Kimi Code CLI | ⏳ not-yet | — | OpenAI shim | — | Moonshot AI terminal agent (discovered 2026-11-17) |
| Qwen Code | ⏳ not-yet | — | OpenAI shim or native | TBD | |
| Grok Build (xAI) | 🧪 guard only | 2026-11-24 | ✓ VERIFIED published 1.5.0: camelCase payload `rm -rf /` → `caro guard --harness grok --mode enforce` emits `permissionDecision: deny`. Reads `~/.claude/settings.json` hooks natively; or `~/.grok/hooks/caro-guard.json` | spec 011 | Grok as a *model* via `--backend grok` |
| Gemini CLI | ⏳ not-yet | — | Native backend or shim | #1439, PR #1722 | Retired for consumer accounts 2026-06-18; successor Antigravity CLI (secondary sources, 2026-11-18 research). Retirement tracked in #1439 / PR #1722 |
| Jules (Google) | ⏳ not-yet | — | Native backend or shim | #1439 | Reported read-only since 2026-11-11, wind-down 2026-12-31 (single secondary source citing the Jules changelog; unverified) |
| Autocoder | ⏳ not-yet | — | TBD | #667 (epic) | Big lift |
| Handy.Computer | ⏳ not-yet | — | TBD | #662 (epic) | Big lift |

## Marketing & discovery surfaces

| Surface | Status | Last validated | GH | Notes |
|---|---|---|---|---|
| `website/src/data/integrations.ts` | ⚠️ partial | 2026-11-26 | #1116, #1662 | #1657 fixed on `main` by #1689: Claude/Ollama entries describe the 1.5.0 `requires the 'remote-backends' feature` error, Ollama snippet uses `--model-name` ✓ (`caro --backend ollama --model-name qwen2.5-coder:7b --dry-run …` → config error, exit 1, as described). Open gap: MLX row snippet `caro --backend embedded "kill the process on port 3000"` → literal `kill PID` (tracked in #1116 / #1662). Open PR #1738 also edits this file |
| `website/src/pages/integrations/index.astro` | ✅ working | 2026-11-23 | — | `curl https://caro.sh/integrations` → 200; Ollama copy shows the 1.5.0 `requires the 'remote-backends' feature` text (#1657 fix is live) |
| README "Use caro from your agent" section | ✅ working | 2026-11-26 | — | 2026-11-26: basic-usage flags `--shell zsh`, `--output json`, `--safety permissive`, `--confirm`, `--force-llm`, `--knowledge-backend chromadb` all parse with `--dry-run`, exit 0; `remove old log files` output is wrong (#1662). `README.md:273` now states `--backend ollama` needs `cargo install caro --features remote-backends` ✓ (#1689). Open PR #1738 further aligns the matrix with 1.5.0 |
| `.github/first-time-issues/06-mcp-claude-code-integration.md` | 🚫 reference only | — | — | Spec doc; not a runtime surface |

---

## Priority queue (next nights)

Topmost unblocked row drives the next nightly PR.

1. ✅ **DONE — [PR #1298](https://github.com/wildcard/caro/pull/1298) MERGED** (2026-07-12 UTC), **[#1115](https://github.com/wildcard/caro/issues/1115) CLOSED**. Acute divergence half resolved via `backends::CLI_SERVABLE_BACKENDS` single source of truth.
2. ✅ **DONE — Release-cadence P0 CLOSED.** `caro 1.5.0` tagged + published to crates.io and GH releases on 2026-10-13 (confirmed 2026-10-19: `cargo install caro --force --locked` → `caro 1.5.0`). [#1419](https://github.com/wildcard/caro/issues/1419) tracks this — worth a comment/close pass by release management, not integrator scope, but the underlying gap this queue flagged since 05-18 is resolved.
3. ✅ **DONE (this PR) — [#1657](https://github.com/wildcard/caro/issues/1657) website/README staleness** for claude/ollama/vllm/exo entries after 1.5.0 (wrong `Unknown backend`/silent-fallback claims, wrong `--model` flag). Docs-only PR, no conflict with open PR #1504 (different lines).
4. **[#1081](https://github.com/wildcard/caro/issues/1081) — feature-gate the default build, or flip `remote-backends` into `default`** (P1; wiring itself is done — claude/openrouter/grok all give the same clean config error as ollama/vllm/exo, verified tonight). The remaining ask is a product decision: ship remote backends in the default binary, or keep them opt-in and this stays a docs-only story. **Topmost unblocked *code* row, but needs a maintainer call on the default-feature question before a PR makes sense.**
5. **Reconcile the `caro test`-subcommand rosters** — `src/main.rs:437` help + `src/main.rs:2069` `valid_backends` advertise `static`/`mlx`, a genuinely-different roster from top-level `--backend`. Not re-checked tonight (budget); carry forward.
6. **[#1645](https://github.com/wildcard/caro/issues/1645) — static matcher Pattern 43 drops file-type qualifiers.** Reproduced tonight as a side-effect of validation: `caro --backend embedded --dry-run -p "list pdf files in current directory"` → `ls -la`, not `ls *.pdf`. Already filed (P1); not a tonight-sized fix (matcher logic, not docs).
7. **[#1439](https://github.com/wildcard/caro/issues/1439) — retire Gemini CLI row, track Antigravity CLI instead.** PR #782 confirmed CLOSED (not merged). Small docs fix; good candidate for next pass's quick-win slot.
8. **Claude Code MCP server** (`caro mcp serve`) — spec drafted (`.github/first-time-issues/06-mcp-claude-code-integration.md`, [#928](https://github.com/wildcard/caro/issues/928)); coordinate with [#789](https://github.com/wildcard/caro/pull/789).
9. **OpenAI-compat HTTP shim** (`caro serve --openai`, [#929](https://github.com/wildcard/caro/issues/929)) — single highest-leverage surface; unlocks ~6 long-tail tools at once.
10. **Claude Code session-token reuse backend** ([#930](https://github.com/wildcard/caro/issues/930)) — let users with CC subscription get Haiku for free.
11. **opencode / crush / droid copy-paste snippets** — once OpenAI shim ships, document each.
12. **Sourcegraph Amp / Letta** — MCP-based; ride on top of the MCP server work.
13. **Long tail** — Pi, Aug, CodePal, qwen, Tabby — one each as polish nights.

---

*Each nightly pass updates the date column for any row it validates and may demote/promote rows in the queue based on what shipped and what regressed.*
