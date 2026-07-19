# Caro Integrations — Status Matrix

> Living matrix maintained by the **caro-integrator** nightly agent.
> Updated every nightly pass (cron `0 23 * * *`).
>
> **Last updated:** 2026-07-18 (nightly pass) — shipped a **fourth-call-site fix for the #1115 divergence class**: `caro config set backend` (`src/main.rs:2078`) still hardcoded `["embedded","ollama","exo","vllm"]` and rejected `mesh`/`ai-horde`/`hybrid` — names `--backend` has accepted since #1209. PR #1298 converted three surfaces to `backends::CLI_SERVABLE_BACKENDS` but missed this one (it lives in `main.rs` command dispatch, not `cli/mod.rs` backend plumbing, so a `VALID_BACKENDS` grep didn't surface it). ✓ VERIFIED on a `main` build pre-fix (`config set backend mesh` → `Invalid backend 'mesh'`) and post-fix (`✓ Set default backend to 'mesh'`). Extracted `validate_config_backend_name()` + 5 regression tests. Published binary remains **1.4.0** (crates.io, 2026-05-09) — the release-cadence gap is now **10 weeks** old and is the single highest-impact open item; every fix in this class is still stranded in `main`.

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
| Anthropic Claude API | 🚧 in-progress (CLI wiring missing) | 2026-07-11 | published 1.4.0: `caro --backend claude --dry-run "list pdfs"` → `Error: Unknown backend 'claude'` (still advertised by `--backend-info`) | #1081 | On `main` post-#1298: no longer advertised (divergence closed). `create_backend()` at `src/cli/mod.rs:295` still has no `claude` arm; `ClaudeBackend` struct exists. Wiring-half tracked in #1081 (#1115 closed). |
| Ollama | ⚠️ partial (feature-gated) | 2026-07-18 | `caro --backend ollama --dry-run "list pdfs"` → `WARN Remote backends not compiled in. Build with --features remote-backends`, then silent fallback to embedded matcher | — | `remote-backends` is **not** in `default = ["embedded-mlx","embedded-cpu","cve-rules"]`. `cargo install caro` and the release-workflow `cargo build --release` (no `--features`) both omit it. |
| vLLM | ⚠️ partial (feature-gated) | 2026-07-18 | same as Ollama — silent fallback in default binary | — | Same root cause as Ollama. |
| Exo | ⚠️ partial (feature-gated) | 2026-07-18 | same as Ollama — silent fallback in default binary | — | Same root cause as Ollama. |
| MLX (Apple Silicon embedded) | ✅ working | 2026-07-11 | published 1.4.0: `caro --backend embedded --dry-run "list pdf files in current directory"` → `ls *.pdf` ✓ | — | `embedded-mlx` is in default features; works in default `cargo install caro` build. |
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

1. ✅ **DONE — [PR #1298](https://github.com/wildcard/caro/pull/1298) MERGED** (2026-07-12 UTC), **[#1115](https://github.com/wildcard/caro/issues/1115) CLOSED**. Acute divergence half resolved via `backends::CLI_SERVABLE_BACKENDS` single source of truth.
2. **[#1081](https://github.com/wildcard/caro/issues/1081) — wiring half + feature-gate split** (P1; surviving home now #1115 is closed). `create_backend` (`src/cli/mod.rs:295`) has no arm for `claude`/`openrouter`; structs exist under `src/backends/remote/`. Add arms instantiating `ClaudeBackend`/`OpenRouterBackend` with config-error paths for missing `ANTHROPIC_API_KEY`/`OPENROUTER_API_KEY`. **CAUTION:** `CLI_SERVABLE_BACKENDS` is NOT feature-gated — adding those names to the unconditional slice would re-open the #1298 divergence for default (`remote-backends`-off) builds. The roster needs a feature-gated view first. ~60–80 LOC + the gate split — a single-night fit only once the gating approach is settled. **Topmost unblocked integrator row.**
3. ✅ **DONE tonight (2026-07-18)** — `caro config set backend` roster unified onto `CLI_SERVABLE_BACKENDS` via `validate_config_backend_name()`. Was the 4th call site missed by #1298.
4. **Reconcile the `caro test`-subcommand rosters** — `src/main.rs:437` help + `src/main.rs:2069` `valid_backends` advertise `static`/`mlx`, a genuinely-different roster from top-level `--backend`. Part of #1115's full fix; deferred from #1298 to keep it tight.
4. **Release-cadence P0 (out of integrator scope — flag to release management / caro-qa-agent):** published crates.io binary is still `1.4.0` (2026-05-09). VERIFIED 2026-07-11 that the #1115 divergence is live in the shipped 1.4.0 (`--backend claude`/`static` → `Unknown backend` despite `--backend-info` advertising them). Mesh-LLM/AI-Horde/hybrid (#1209), OpenRouter (#1097), smart-approval (#1206), the #1092 loud-error fix, and now #1298 are all stranded in main, never reaching a released binary. `cargo install caro` users get none of it. A 1.4.1/1.5.0 cut is overdue.
6. **Claude Code MCP server** (`caro mcp serve`) — spec drafted (`.github/first-time-issues/06-mcp-claude-code-integration.md`, [#928](https://github.com/wildcard/caro/issues/928)); coordinate with [#789](https://github.com/wildcard/caro/pull/789).
7. **OpenAI-compat HTTP shim** (`caro serve --openai`, [#929](https://github.com/wildcard/caro/issues/929)) — single highest-leverage surface; unlocks ~6 long-tail tools at once.
8. **Claude Code session-token reuse backend** ([#930](https://github.com/wildcard/caro/issues/930)) — let users with CC subscription get Haiku for free.
9. **Coordinate Gemini [PR #782](https://github.com/wildcard/caro/pull/782)** — review and merge or supersede.
10. **opencode / crush / droid copy-paste snippets** — once OpenAI shim ships, document each.
11. **Sourcegraph Amp / Letta** — MCP-based; ride on top of the MCP server work.
12. **Long tail** — Pi, Aug, CodePal, qwen, Tabby — one each as polish nights.

---

*Each nightly pass updates the date column for any row it validates and may demote/promote rows in the queue based on what shipped and what regressed.*
