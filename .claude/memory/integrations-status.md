# Caro Integrations — Status Matrix

> Living matrix maintained by the **caro-integrator** nightly agent.
> Updated every nightly pass (cron `0 23 * * *`).
>
> **Last updated:** 2026-05-12 — silent-fallback fix shipped for `--backend {ollama,exo,vllm}` in non-remote-backends builds (option (c) from #1081); upstream packaging fix still pending.

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

1. **Release binaries lack `remote-backends` feature** ([#1081](https://github.com/wildcard/caro/issues/1081)) — `default` features in `Cargo.toml` omit `remote-backends`; `cargo install caro` and the GitHub-Release workflow (`.github/workflows/release.yml:245`) both build without it. Three landing options were on the table: (a) add `"remote-backends"` to `default`; (b) pass `--features remote-backends` in release/publish workflows; (c) demote the silent fallback to a loud error. **Option (c) shipped 2026-05-12** — `--backend {ollama,exo,vllm}` in a default build now returns `CliError::ConfigurationError` with install hint instead of silently falling through. Remaining: options (a)/(b) still pending — both are maintainer policy calls about binary-size posture vs. default-on remote backends.
2. **Anthropic Claude backend not wired into CLI** (tracked under [#1081](https://github.com/wildcard/caro/issues/1081)) — `ClaudeBackend` exists at `src/backends/remote/claude.rs` but `src/cli/mod.rs:468` hardcodes the valid-backend list and `create_backend()` never instantiates it. ~30 LOC of wiring + a new arm in `validate_backend_name`. Good fit for a single-night PR once #1081 is triaged.
3. **Claude Code MCP server** (`caro mcp serve`) — spec already drafted (`.github/first-time-issues/06-mcp-claude-code-integration.md`, #928); coordinate with #789.
4. **OpenAI-compat HTTP shim** (`caro serve --openai`, #929) — single highest-leverage surface; unlocks ~6 long-tail tools at once.
5. **Claude Code session-token reuse backend** (#930) — let users with CC subscription get Haiku for free.
6. **OpenRouter backend** (#931) — clones `vllm.rs`; trivial leverage on hundreds of models (now 290+/400+ per upstream as of May 2026).
7. **Coordinate Gemini PR #782** — review and merge or supersede.
8. **opencode / crush / droid copy-paste snippets** — once OpenAI shim ships, document each.
9. **Sourcegraph Amp / Letta** — MCP-based; ride on top of the MCP server work.
10. **Long tail** — Pi, Aug, CodePal, qwen, Tabby — one each as polish nights.

---

*Each nightly pass updates the date column for any row it validates and may demote/promote rows in the queue based on what shipped and what regressed.*
