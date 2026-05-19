# Caro Integrations — Status Matrix

> Living matrix maintained by the **caro-integrator** nightly agent.
> Updated every nightly pass (cron `0 23 * * *`).
>
> **Last updated:** 2026-05-16 — `caro skill install` (shipped in v1.4.0) bundles `caro-scaffold`, not `caro-shell`; matrix + website now reflect both rows separately.

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
| Claude Code skill (`caro-shell`) | ⚠️ partial — not in `caro skill install` | 2026-05-16 | `caro 1.4.0` `caro skill install --help` → "Install the bundled `caro-scaffold` skill" (NOT `caro-shell`); only way to use `caro-shell` is to copy `.claude/skills/caro-shell/SKILL.md` manually or rely on Claude Code auto-discovery from a caro checkout | — | Skill source still at `.claude/skills/caro-shell/SKILL.md`; the bundled CLI subcommand ships a different skill. Roadmapped: have `caro skill install` learn to install both. |
| Claude Code skill (`caro-scaffold`, bundled in v1.4.0) | ✅ working | 2026-05-16 | `caro skill install --help` → confirms target; `caro skill install` writes to `~/.claude/skills/caro-scaffold/SKILL.md`; trigger surface is CaroML task scaffolding (".caro task file", "scaffold a runbook", "/caro new <name>") | — | Distinct from `caro-shell`: this one is for creating repeatable `.caro` tasks, not for one-shot command synthesis. Shipped alongside CaroML preview in v1.4.0 (closed #901). |
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

1. **`caro skill install` only ships `caro-scaffold`, not `caro-shell`** — `caro skill install --help` in v1.4.0 confirms it bundles `caro-scaffold` (CaroML scaffolding). The integrator's marquee `caro-shell` skill still requires manual copy. Tonight's PR documents both rows; the upstream code fix (have `caro skill install` accept `--skill caro-shell` or install both) is a follow-on for a future night. Spec/scope is small; ~50 LOC in `src/cli/skill.rs`.
2. **`remote-backends` loud-error fix shipped to main but not released** ([#1081](https://github.com/wildcard/caro/issues/1081), [#1092](https://github.com/wildcard/caro/pull/1092)) — PR #1092 (option **c** from the remediation menu) merged to `main` on 2026-05-16. Published `caro 1.4.0` from crates.io still has the silent fallback. Cuts the next nightly's job in half: just confirm the loud-error behavior in the next release. Queue rank: high once v1.4.1+ ships.
3. **OpenRouter backend shipped to main but not released** ([#931](https://github.com/wildcard/caro/issues/931), [#1097](https://github.com/wildcard/caro/pull/1097)) — full backend landed via #1097 in main, NOT in any v1.x.y release yet (`caro --backend openrouter` returns `Unknown backend 'openrouter'` against v1.4.0). Add a working row once a release picks it up.
4. **Anthropic Claude backend not wired into CLI** (tracked under [#1081](https://github.com/wildcard/caro/issues/1081)) — `ClaudeBackend` exists at `src/backends/remote/claude.rs` but `src/cli/mod.rs:468` hardcodes the valid-backend list and `create_backend()` never instantiates it. ~30 LOC of wiring + a new arm in `validate_backend_name`. Good fit for a single-night PR.
5. **Claude Code MCP server** (`caro mcp serve`) — spec already drafted (`.github/first-time-issues/06-mcp-claude-code-integration.md`, #928); coordinate with #789.
6. **OpenAI-compat HTTP shim** (`caro serve --openai`, #929) — single highest-leverage surface; unlocks ~6 long-tail tools at once.
7. **Claude Code session-token reuse backend** (#930) — let users with CC subscription get Haiku for free.
8. **Coordinate Gemini PR #782** — review and merge or supersede.
9. **opencode / crush / droid copy-paste snippets** — once OpenAI shim ships, document each.
10. **Sourcegraph Amp / Letta** — MCP-based; ride on top of the MCP server work.
11. **Long tail** — Pi, Aug, CodePal, qwen, Tabby — one each as polish nights.

---

*Each nightly pass updates the date column for any row it validates and may demote/promote rows in the queue based on what shipped and what regressed.*
