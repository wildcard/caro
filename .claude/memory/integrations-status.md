# Caro Integrations — Status Matrix

> Living matrix maintained by the **caro-integrator** nightly agent.
> Updated every nightly pass (cron `0 23 * * *`).
>
> **Last updated:** 2026-10-21 — **v1.5.0 is now the published binary** (crates.io + GitHub release, tagged `d705666`, verified 2026-10-13 on issue #1419; closed that issue tonight as resolved). Re-validated the full native-backend roster against the published `caro 1.5.0`: `claude`/`openrouter`/`grok` are now recognized names (wired by PR #1513) that fail loudly with a feature-gate config error in the default build — no more `Unknown backend`. `ollama`/`vllm`/`exo` get the same loud error instead of the old silent fallback. `embedded` still ✅ working. Filed no new issues; folded the already-filed #1657 website-staleness fix into this pass (PR #1504, rebased onto current `main` — its prior CI failure was pre-existing `main` breakage already fixed upstream, confirmed via local `cargo clippy` green).
>
> **Previous:** 2026-10-03 — PR #1513 (`claude/nifty-brown-mtjkpu`) adds the experimental `caro guard` PreToolUse hook (Claude Code / Grok Build / Codex / OpenCode) and wires `grok`, `claude` and `openrouter` into `--backend`. Non-`remote-backends` builds get the existing loud "requires remote-backends" error for those names, the same as `mesh`/`vllm`, so the #1298 divergence stays closed.
>
> **Earlier:** 2026-07-11 (post-merge pass) — **PR #1298 is MERGED** (2026-07-12 UTC) and **issue #1115 is CLOSED**: the acute half of the backend-roster divergence is resolved on `main` (`--backend-info` / `available_backends()` / `--backend` help all iterate the single source of truth `backends::CLI_SERVABLE_BACKENDS`, shared with `validate_backend_name`). No code PR tonight — the fix already exists on `main`, and re-touching the roster would regress it (see below). Instead: re-validated the **still-published `caro 1.4.0`** (crates.io, 2026-05-09) and confirmed the #1115 P0 is **live in the shipped artifact** — `--backend-info` advertises `static`+`claude`, `--backend claude`/`--backend static` → `Unknown backend`. This is now purely a **release-cadence gap**: `main` exposes 7 CLI-servable backends (adds `mesh`/`ai-horde`/`hybrid` from #1209), the shipped binary knows 4. Posted a post-#1298 status update to **#1081** (the surviving tracking home now that #1115 is closed) covering the remaining wiring-half + release gap. **The wiring half (claude/openrouter arms in `create_backend`) is NOT trivial**: `CLI_SERVABLE_BACKENDS` is not feature-gated, so adding those names without a `#[cfg(feature="remote-backends")]` split would re-open the divergence for default builds. See log.

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
| Anthropic Claude API | ⚠️ partial (feature-gated) | 2026-10-21 | ✓ VERIFIED on published `caro 1.5.0`: `--backend claude --dry-run` → `Configuration error: Backend 'claude' requires the 'remote-backends' feature...` (loud, names the fix + `ANTHROPIC_API_KEY`), not `Unknown backend` | #1081, PR #1513 | Wired by #1513 (merged). Default `cargo install caro` build omits `remote-backends`; needs `cargo install caro --features remote-backends` + `ANTHROPIC_API_KEY`. |
| Ollama | ⚠️ partial (feature-gated) | 2026-10-21 | ✓ VERIFIED on published `caro 1.5.0`: `--backend ollama --dry-run` → `Configuration error: ... requires the 'remote-backends' feature`, exit 1 — **no more silent fallback** | #1081 | `remote-backends` is **not** in `default = ["embedded-mlx","embedded-cpu","cve-rules"]`. `cargo install caro` and the release-workflow `cargo build --release` (no `--features`) both omit it. Loud-error fix (from the 2026-05-13 pass) confirmed still live on 1.5.0. |
| vLLM | ⚠️ partial (feature-gated) | 2026-10-21 | same as Ollama — loud config error on published 1.5.0 | #1081 | Same root cause as Ollama. |
| Exo | ⚠️ partial (feature-gated) | 2026-10-21 | same as Ollama — loud config error on published 1.5.0 | #1081 | Same root cause as Ollama. |
| MLX (Apple Silicon embedded) | ✅ working | 2026-10-21 | published 1.5.0: `caro --backend embedded --dry-run "list pdf files in current directory"` → `ls -la` (not `ls *.pdf`) | — | `embedded-mlx` is in default features; backend itself works (no error, no crash) but reproduces the already-filed **#1645** (static matcher Pattern 43 drops file-type qualifiers on 1.5.0) — not a new finding, no action tonight. |
| Candle CPU (embedded) | ✅ working | 2026-07-11 | `caro --dry-run "show disk usage"` → `du -sh ... \| sort -rh \| head -10` ✓ (auto-fallback path) | — | `embedded-cpu` is in default features; works in default build. Not re-run tonight (embedded row above covers the default-build smoke); stale date flagged for a future pass. |
| OpenRouter | ⚠️ partial (feature-gated) | 2026-10-21 | ✓ VERIFIED on published `caro 1.5.0`: `--backend openrouter --dry-run` → same loud feature-gate error, `OPENROUTER_API_KEY` named | #931, PR #1513 | `openrouter.rs` → `openai_compat.rs`; CLI-wired by #1513 (merged). |
| xAI Grok | ⚠️ partial (feature-gated) | 2026-10-21 | ✓ VERIFIED on published `caro 1.5.0`: `--backend grok --dry-run` → same loud feature-gate error, `XAI_API_KEY` named | PR #1513 | `--backend grok` / `--advisor grok` / `hybrid_remote = "grok"`. |
| Gemini / Jules | ⏳ not-yet | — | — | #1439 | PR #782 **closed unmerged**; Gemini CLI product shut down 2026-06-18. #1439 (open) proposes retiring this row in favor of Antigravity CLI — not yet actioned. |
| LM Studio + FunctionGemma | 🚧 in-progress | — | — | PR #558 | Tracked, no action this night |
| quant.cpp | 🚧 in-progress | — | — | PR #838 | Tracked, no action this night |
| Azure Foundry | ⏳ not-yet | — | — | #661 (epic) | Big lift; deprioritized |
| Claude Code session-token | ⏳ not-yet | — | — | #930 | Reuse user's CC subscription/API auth; default to Haiku |

## Caro-as-a-tool (outward integrations)

| Tool | Status | Last validated | Method | GH | Notes |
|---|---|---|---|---|---|
| Claude Code skill (`caro-shell`) | ✅ working | 2026-04-26 | Skill installed in fresh CC session; invokes published `caro` binary; validated suggestion returned | — | Shipped first night (this PR) |
| Harness guardian hook (`caro guard`) | 🧪 experimental | 2026-10-03 | `echo '{"tool_name":"Bash","tool_input":{"command":"rm -rf /"}}' \| caro guard --mode enforce` → deny; e2e `tests/e2e_guard.rs` | spec 011 / ADR-020 | PreToolUse adapter for Claude Code, Grok Build, Codex; OpenCode via `integrations/opencode/caro-guard.ts`. Shadow by default |
| Claude Code MCP server (`caro mcp serve`) | ⏳ not-yet | — | `mcp-inspect` against `caro mcp serve` | #928 | Spec: `.github/first-time-issues/06-mcp-claude-code-integration.md`; tools `generate_command` / `validate_command` / `explain_safety` / `show_decision_tree` |
| OpenAI-compat HTTP shim (`caro serve --openai`) | ⏳ not-yet | — | `curl /v1/chat/completions` with a tool call | #929 | Highest leverage — unlocks Codex/Cursor/Continue/Aider/Tabby in one shot |
| Codex (OpenAI) | 🧪 guard only | 2026-10-03 | `[[hooks.PreToolUse]]` → `caro guard --harness codex` (docs/coding-agents/codex.md) | #789 (Crush MCP config PR) | Generation still awaits OpenAI shim |
| Cursor | ⏳ not-yet | — | OpenAI shim snippet copy-pasted into Cursor settings | — | Satisfied by OpenAI shim |
| Continue (continue.dev) | ⏳ not-yet | — | OpenAI shim snippet | — | Satisfied by OpenAI shim |
| Aider | ⏳ not-yet | — | OpenAI shim snippet | — | Satisfied by OpenAI shim |
| Tabby (self-hosted) | ⏳ not-yet | — | OpenAI shim snippet | — | Satisfied by OpenAI shim |
| opencode | 🧪 guard only | 2026-10-03 | Plugin `integrations/opencode/caro-guard.ts` → `caro guard --harness opencode` | TBD | sst/opencode; generation still awaits MCP/shim |
| crush | ⏳ not-yet | — | MCP or OpenAI shim | #789 (related) | Charm.sh ecosystem |
| droid | ⏳ not-yet | — | MCP or OpenAI shim | TBD | |
| Sourcegraph Amp | ⏳ not-yet | — | MCP | TBD | Enterprise coding agent |
| Letta (MemGPT) | ⏳ not-yet | — | Tool registration | TBD | Persistent-memory agents |
| Pi | ⏳ not-yet | — | TBD | TBD | |
| Aug | ⏳ not-yet | — | TBD | TBD | |
| CodePal | ⏳ not-yet | — | TBD | TBD | |
| Qwen Code | ⏳ not-yet | — | OpenAI shim or native | TBD | |
| Grok Build (xAI) | 🧪 guard only | 2026-10-03 | Reads `~/.claude/settings.json` hooks natively; or `~/.grok/hooks/caro-guard.json` (docs/coding-agents/grok-build.md) | spec 011 | Grok as a *model* via `--backend grok` |
| Gemini CLI | ⏳ not-yet | — | Native backend or shim | PR #782 | Coordinate |
| Jules (Google) | ⏳ not-yet | — | Native backend or shim | PR #782 | Coordinate |
| Autocoder | ⏳ not-yet | — | TBD | #667 (epic) | Big lift |
| Handy.Computer | ⏳ not-yet | — | TBD | #662 (epic) | Big lift |

## Marketing & discovery surfaces

| Surface | Status | Last validated | GH | Notes |
|---|---|---|---|---|
| `website/src/data/integrations.ts` | ✅ working | 2026-04-26 | — | Shipped first night |
| `website/src/pages/integrations/index.astro` | ✅ working | 2026-04-26 | — | Shipped first night |
| README "Use caro from your agent" section | ✅ working | 2026-04-26 | — | Shipped first night |
| `.github/first-time-issues/06-mcp-claude-code-integration.md` | 🚫 reference only | — | — | Spec doc; not a runtime surface |

---

## Priority queue (next nights)

Topmost unblocked row drives the next nightly PR.

1. ✅ **DONE — [PR #1298](https://github.com/wildcard/caro/pull/1298) MERGED** (2026-07-12 UTC), **[#1115](https://github.com/wildcard/caro/issues/1115) CLOSED**. Acute divergence half resolved via `backends::CLI_SERVABLE_BACKENDS` single source of truth.
2. ✅ **DONE — [#1081](https://github.com/wildcard/caro/issues/1081) wiring half**. [PR #1513](https://github.com/wildcard/caro/pull/1513) (merged 2026-10-11) wired `claude`/`openrouter`/`grok` into `create_backend` under `#[cfg(feature = "remote-backends")]`, so the roster stays honest for default builds. ✓ VERIFIED on published `caro 1.5.0` tonight (2026-10-21) — see native-backend rows above. #1081 itself is still open tracking the "default build omits remote-backends" half; not integrator-actionable (binary-size/feature-default policy call).
3. ✅ **DONE — release-cadence P0**. **v1.5.0 shipped 2026-10-13** (tag `d705666`, crates.io `max_version=1.5.0`, GitHub release has all 5 platform assets). Closed [#1419](https://github.com/wildcard/caro/issues/1419) tonight with this verification.
4. **Reconcile the `caro test`-subcommand rosters** — `src/main.rs` help + `valid_backends` advertise `static`/`mlx`, a genuinely-different roster from top-level `--backend`. Part of #1115's full fix; deferred from #1298 to keep it tight. Not re-checked against 1.5.0 line numbers tonight.
5. **[#1645](https://github.com/wildcard/caro/issues/1645) — static matcher Pattern 43 drops file-type qualifiers** (P1, already filed 2026-10-13). Reproduced again tonight: `"list pdf files in current directory"` → `ls -la`, not `ls *.pdf`, on published 1.5.0.
6. **Claude Code MCP server** (`caro mcp serve`) — spec drafted (`.github/first-time-issues/06-mcp-claude-code-integration.md`, [#928](https://github.com/wildcard/caro/issues/928)); coordinate with [#789](https://github.com/wildcard/caro/pull/789). **Topmost unblocked net-new integrator row** now that the roster/release gaps are closed.
7. **OpenAI-compat HTTP shim** (`caro serve --openai`, [#929](https://github.com/wildcard/caro/issues/929)) — single highest-leverage surface; unlocks ~6 long-tail tools at once.
8. **Claude Code session-token reuse backend** ([#930](https://github.com/wildcard/caro/issues/930)) — let users with CC subscription get Haiku for free.
9. **[#1439](https://github.com/wildcard/caro/issues/1439) — retire Gemini CLI row**, track Antigravity CLI instead (PR #782 closed unmerged, Gemini CLI shut down 2026-06-18). Already filed, not yet actioned; small polish night.
10. **opencode / crush / droid copy-paste snippets** — once OpenAI shim ships, document each.
11. **Sourcegraph Amp / Letta** — MCP-based; ride on top of the MCP server work.
12. **Long tail** — Pi, Aug, CodePal, qwen, Tabby — one each as polish nights.

---

*Each nightly pass updates the date column for any row it validates and may demote/promote rows in the queue based on what shipped and what regressed.*
