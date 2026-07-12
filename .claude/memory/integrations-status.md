# Caro Integrations — Status Matrix

> Living matrix maintained by the **caro-integrator** nightly agent.
> Updated every nightly pass (cron `0 23 * * *`).
>
> **Last updated:** 2026-07-11 — main is buildable again (PR #1154 landed the bincode+rusqlite+candle unblock 2026-05-19). Tonight closed the acute half of the #1115 backend-roster divergence: `--backend-info` (and `available_backends()`, and the `--backend` help string) now iterate a single source of truth — `backends::CLI_SERVABLE_BACKENDS` — shared with `validate_backend_name`, so no surface advertises a backend `--backend` rejects. `static`/`claude` no longer advertised as usable. PR #1298. NOTE: published crates.io binary is still `1.4.0` (2026-05-09) — two months of backend work (mesh, ai-horde, hybrid, openrouter) has NOT reached a released binary; that's a release-management gap, out of integrator scope, flagged below. See log.

## Legend

| Status | Meaning |
|---|---|
| ✅ working | Validated end-to-end against published `caro` binary on `last-validated` date |
| ⚠️ partial | Validated but with caveats (documented in `notes`) |
| 🚧 in-progress | Implementation in flight (PR linked) |
| ❌ broken | Was working, now fails — **P0/P1 fix required** |
| ⏳ not-yet | Tracked target, no implementation yet |
| 🚫 n/a | Out of scope for caro |

---

## Native backends (in caro)

| Tool | Status | Last validated | Method | GH | Notes |
|---|---|---|---|---|---|
| Anthropic Claude API | 🚧 in-progress (CLI wiring missing) | 2026-05-11 | `caro --backend claude --dry-run "list pdfs"` → `Error: Unknown backend 'claude'` | — | `ClaudeBackend` struct + `BackendType::Claude` variant exist in source (`src/backends/remote/claude.rs`), but `validate_backend_name()` at `src/cli/mod.rs:468` hardcodes `["embedded","ollama","exo","vllm"]` and `create_backend()` never instantiates `ClaudeBackend::new()`. CLI rejects the flag regardless of features. |
| Ollama | ⚠️ partial (feature-gated) | 2026-05-11 | `caro --backend ollama --dry-run "list pdfs"` → `WARN Remote backends not compiled in. Build with --features remote-backends`, then silent fallback to embedded matcher | — | `remote-backends` is **not** in `default = ["embedded-mlx","embedded-cpu","cve-rules"]`. `cargo install caro` and the release-workflow `cargo build --release` (no `--features`) both omit it. |
| vLLM | ⚠️ partial (feature-gated) | 2026-05-11 | same as Ollama — silent fallback in default binary | — | Same root cause as Ollama. |
| Exo | ⚠️ partial (feature-gated) | 2026-05-11 | same as Ollama — silent fallback in default binary | — | Same root cause as Ollama. |
| MLX (Apple Silicon embedded) | ✅ working | 2026-05-11 | `caro --backend embedded --dry-run "list pdf files"` → `ls *.pdf` ✓ | — | `embedded-mlx` is in default features; works in default `cargo install caro` build. |
| Candle CPU (embedded) | ✅ working | 2026-05-11 | `caro --dry-run "show disk usage"` → `du -sh ... \| sort -rh \| head -10` ✓ (auto-fallback path) | — | `embedded-cpu` is in default features; works in default build. |
| OpenRouter | ⏳ not-yet | — | — | #931 | New backend; clones `vllm.rs` shape; supports `auto` model |
| Gemini / Jules | 🚧 in-progress | — | — | PR #782 | Coordinate with existing PR; don't fork |
| LM Studio + FunctionGemma | 🚧 in-progress | — | — | PR #558 | Tracked, no action this night |
| quant.cpp | 🚧 in-progress | — | — | PR #838 | Tracked, no action this night |
| Azure Foundry | ⏳ not-yet | — | — | #661 (epic) | Big lift; deprioritized |
| Claude Code session-token | ⏳ not-yet | — | — | #930 | Reuse user's CC subscription/API auth; default to Haiku |

## Caro-as-a-tool (outward integrations)

| Tool | Status | Last validated | Method | GH | Notes |
|---|---|---|---|---|---|
| Claude Code skill (`caro-shell`) | ✅ working | 2026-04-26 | Skill installed in fresh CC session; invokes published `caro` binary; validated suggestion returned | — | Shipped first night (this PR) |
| Claude Code MCP server (`caro mcp serve`) | ⏳ not-yet | — | `mcp-inspect` against `caro mcp serve` | #928 | Spec: `.github/first-time-issues/06-mcp-claude-code-integration.md`; tools `generate_command` / `validate_command` / `explain_safety` / `show_decision_tree` |
| OpenAI-compat HTTP shim (`caro serve --openai`) | ⏳ not-yet | — | `curl /v1/chat/completions` with a tool call | #929 | Highest leverage — unlocks Codex/Cursor/Continue/Aider/Tabby in one shot |
| Codex (OpenAI) | ⏳ not-yet | — | Codex config snippet pointing at OpenAI shim or direct MCP | #789 (Crush MCP config PR) | Satisfied by OpenAI shim |
| Cursor | ⏳ not-yet | — | OpenAI shim snippet copy-pasted into Cursor settings | — | Satisfied by OpenAI shim |
| Continue (continue.dev) | ⏳ not-yet | — | OpenAI shim snippet | — | Satisfied by OpenAI shim |
| Aider | ⏳ not-yet | — | OpenAI shim snippet | — | Satisfied by OpenAI shim |
| Tabby (self-hosted) | ⏳ not-yet | — | OpenAI shim snippet | — | Satisfied by OpenAI shim |
| opencode | ⏳ not-yet | — | MCP or OpenAI shim | TBD | Charm.sh ecosystem |
| crush | ⏳ not-yet | — | MCP or OpenAI shim | #789 (related) | Charm.sh ecosystem |
| droid | ⏳ not-yet | — | MCP or OpenAI shim | TBD | |
| Sourcegraph Amp | ⏳ not-yet | — | MCP | TBD | Enterprise coding agent |
| Letta (MemGPT) | ⏳ not-yet | — | Tool registration | TBD | Persistent-memory agents |
| Pi | ⏳ not-yet | — | TBD | TBD | |
| Aug | ⏳ not-yet | — | TBD | TBD | |
| CodePal | ⏳ not-yet | — | TBD | TBD | |
| Qwen Code | ⏳ not-yet | — | OpenAI shim or native | TBD | |
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

1. **Confirm [PR #1298](https://github.com/wildcard/caro/pull/1298) merged + CI green** — tonight's PR. Closed the acute half of #1115 via `backends::CLI_SERVABLE_BACKENDS` (single source of truth for `validate_backend_name` + `print_backend_info` + `available_backends()`). ✅ bincode/rusqlite/candle blockers all cleared by #1154 (merged 2026-05-19); `cargo check --no-default-features --features embedded-cpu` on main = green.
2. **[#1115](https://github.com/wildcard/caro/issues/1115) — finish the wiring half** (P0, still open after #1298). `create_backend` (`src/cli/mod.rs`) has no match arm for `claude`/`openrouter`/`mlx`; they fall through `_ => warn + auto-detect`. Add arms instantiating `ClaudeBackend`/`OpenRouterBackend` (structs exist under `src/backends/remote/`) with config-error paths for missing `ANTHROPIC_API_KEY`/`OPENROUTER_API_KEY`, then add the two names to `CLI_SERVABLE_BACKENDS` (one-line add updates every surface at once). ~60–80 LOC, single-night fit. **This is the topmost unblocked integrator row.**
3. **Reconcile the `caro test`-subcommand rosters** — `src/main.rs:437` help + `src/main.rs:2069` `valid_backends` advertise `static`/`mlx`, a genuinely-different roster from top-level `--backend`. Part of #1115's full fix; deferred from #1298 to keep it tight.
4. **Release-cadence gap (out of integrator scope — flag to release management / caro-qa-agent):** published crates.io binary is still `1.4.0` (2026-05-09). Mesh-LLM/AI-Horde/hybrid (#1209), OpenRouter (#1097), smart-approval (#1206), the #1092 loud-error fix, and now #1298 are all stranded in main, never reaching a released binary. `cargo install caro` users get none of it. A 1.4.1/1.5.0 cut is overdue.
6. **Claude Code MCP server** (`caro mcp serve`) — spec drafted (`.github/first-time-issues/06-mcp-claude-code-integration.md`, [#928](https://github.com/wildcard/caro/issues/928)); coordinate with [#789](https://github.com/wildcard/caro/pull/789).
7. **OpenAI-compat HTTP shim** (`caro serve --openai`, [#929](https://github.com/wildcard/caro/issues/929)) — single highest-leverage surface; unlocks ~6 long-tail tools at once.
8. **Claude Code session-token reuse backend** ([#930](https://github.com/wildcard/caro/issues/930)) — let users with CC subscription get Haiku for free.
9. **Coordinate Gemini [PR #782](https://github.com/wildcard/caro/pull/782)** — review and merge or supersede.
10. **opencode / crush / droid copy-paste snippets** — once OpenAI shim ships, document each.
11. **Sourcegraph Amp / Letta** — MCP-based; ride on top of the MCP server work.
12. **Long tail** — Pi, Aug, CodePal, qwen, Tabby — one each as polish nights.

---

*Each nightly pass updates the date column for any row it validates and may demote/promote rows in the queue based on what shipped and what regressed.*
