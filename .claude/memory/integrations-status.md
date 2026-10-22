# Caro Integrations — Status Matrix

> Living matrix maintained by the **caro-integrator** nightly agent.
> Updated every nightly pass (cron `0 23 * * *`).
>
> **Last updated:** 2026-10-19 — ✓ VERIFIED against published `caro 1.5.0` (crates.io + GH release, tagged 2026-10-13 — the release-cadence gap flagged on 2026-07-11 is closed). `--backend-info`, `--backend claude/openrouter/static` and `--backend embedded` all re-validated: claude/grok/openrouter now give a clean `Configuration error: ... requires the 'remote-backends' feature` (no more `Unknown backend` for advertised names, no more silent WARN-fallback for ollama/vllm/exo). Shipped `docs(integrations): fix stale claude/ollama backend descriptions + --model-name snippet for 1.5.0` — no code PR needed, the P1 was a docs/website staleness gap (#1657), confirmed scoped and non-conflicting with open PR #1504 (different lines).
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
| Anthropic Claude API | ⚠️ partial (feature-gated) | 2026-10-19 | ✓ VERIFIED published `caro 1.5.0`: `caro --backend claude --dry-run -p "..."` → `Configuration error: Backend 'claude' requires the 'remote-backends' feature, which is not compiled into this build.` + fix instructions. `--backend-info` lists it as `not compiled` (consistent with the error). | #1081 (tracking home; wiring is done, feature-gate omission from default build remains) | #1115/#1298 divergence is CLOSED for claude: no more `Unknown backend`, no silent fallback — a clean, consistent config error. Needs `--features remote-backends` + `ANTHROPIC_API_KEY` to actually route. |
| Ollama | ⚠️ partial (feature-gated) | 2026-10-19 | ✓ VERIFIED published `caro 1.5.0`: `caro --backend ollama --dry-run -p "..."` → `Configuration error: ... requires the 'remote-backends' feature`, exit 1. No more silent WARN-fallback. | — | Behavior improved since 2026-05-11: was silent fallback, now a loud, actionable error. `remote-backends` still not in default features. |
| vLLM | ⚠️ partial (feature-gated) | 2026-10-19 | ✓ VERIFIED published `caro 1.5.0`: same loud config error as Ollama, no silent fallback | — | Same root cause/fix as Ollama; same improvement since 2026-05-11. |
| Exo | ⚠️ partial (feature-gated) | 2026-10-19 | ✓ VERIFIED published `caro 1.5.0`: same loud config error as Ollama, no silent fallback | — | Same root cause/fix as Ollama; same improvement since 2026-05-11. |
| MLX (Apple Silicon embedded) | ✅ working | 2026-10-19 | ✓ VERIFIED published `caro 1.5.0`: `caro --backend embedded --dry-run -p "list pdf files in current directory"` → `ls -la` (file-type qualifier dropped — see #1645, separate from this row) | — | `embedded-mlx` is in default features; works in default `cargo install caro` build. |
| Candle CPU (embedded) | ✅ working | 2026-05-11 | `caro --dry-run "show disk usage"` → `du -sh ... \| sort -rh \| head -10` ✓ (auto-fallback path) | — | `embedded-cpu` is in default features; works in default build. |
| OpenRouter | ⚠️ partial (feature-gated) | 2026-10-19 | ✓ VERIFIED published `caro 1.5.0`: `caro --backend openrouter --dry-run -p "..."` → same loud `Configuration error: ... requires the 'remote-backends' feature` | #931, #1081 | `openrouter.rs` is `openai_compat.rs`; CLI-wired since PR #1513, same feature-gate omission as the others. |
| xAI Grok | ⚠️ partial (feature-gated) | 2026-10-19 | ✓ VERIFIED published `caro 1.5.0`: `caro --backend grok --dry-run -p "..."` → same loud `Configuration error: ... requires the 'remote-backends' feature` | PR #1513 | `--backend grok` / `--advisor grok` / `hybrid_remote = "grok"`; `XAI_API_KEY`. Consistent with claude/openrouter/ollama/vllm/exo. |
| Gemini / Jules | 🚧 in-progress | — | — | PR #782 | Coordinate with existing PR; don't fork |
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
