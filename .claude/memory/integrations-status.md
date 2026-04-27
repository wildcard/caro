# Caro Integrations — Status Matrix

> Living matrix maintained by the **caro-integrator** nightly agent.
> Updated every nightly pass (cron `0 23 * * *`).
>
> **Last updated:** 2026-04-26 — initial seed, first nightly pass.

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
| Anthropic Claude API | ⏳ not-yet (validation) | — | `caro --backend claude --dry-run "list pdfs"` | — | Implemented in `src/backends/remote/claude.rs`; needs nightly validation |
| Ollama | ⏳ not-yet (validation) | — | `caro --backend ollama --dry-run ...` | — | Implemented in `src/backends/remote/ollama.rs` |
| vLLM | ⏳ not-yet (validation) | — | `caro --backend vllm --dry-run ...` | — | Implemented in `src/backends/remote/vllm.rs` |
| Exo | ⏳ not-yet (validation) | — | `caro --backend exo --dry-run ...` | — | Implemented in `src/backends/remote/exo.rs` |
| MLX (Apple Silicon embedded) | ⏳ not-yet (validation) | — | `caro --backend embedded "..."` on macOS arm64 | — | `src/backends/embedded/mlx.rs` |
| Candle CPU (embedded) | ⏳ not-yet (validation) | — | `caro --backend embedded-cpu "..."` | — | `src/backends/embedded/cpu.rs` |
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

1. **Validate the 6 native backends** — none have a `last-validated` date yet. Even one nightly pass focused on running each `--dry-run` smoke test would put real evidence in this matrix.
2. **Claude Code MCP server** (`caro mcp serve`) — spec already drafted; coordinate with #789.
3. **OpenAI-compat HTTP shim** (`caro serve --openai`) — single highest-leverage surface; unlocks ~6 long-tail tools at once.
4. **Claude Code session-token reuse backend** — let users with CC subscription get Haiku for free.
5. **OpenRouter backend** — clones `vllm.rs`; trivial leverage on hundreds of models.
6. **Coordinate Gemini PR #782** — review and merge or supersede.
7. **opencode / crush / droid copy-paste snippets** — once OpenAI shim ships, document each.
8. **Sourcegraph Amp / Letta** — MCP-based; ride on top of the MCP server work.
9. **Long tail** — Pi, Aug, CodePal, qwen, Tabby — one each as polish nights.

---

*Each nightly pass updates the date column for any row it validates and may demote/promote rows in the queue based on what shipped and what regressed.*
