# Caro Integrations — Status Matrix

> Living matrix maintained by the **caro-integrator** nightly agent.
> Updated every nightly pass (cron `0 23 * * *`).
>
> **Last updated:** 2026-10-28 — validation-only night; backlog did not move (8 integrator-area PRs open, #1719 still waits for a human `main` merge). ✓ VERIFIED on published `caro 1.5.0`: OpenRouter loud config error (row re-dated, was 2026-10-03), Candle CPU `df -h`, `caro guard` deny, #1723 stdin hang still live (rc=142), #1732 `caro test --backend mlx` still rejected, completion roster still `… static` (fix in #1719).
>
> **Previous:** 2026-10-27 — backlog moved: PR #1689 (#1657 website fix) and PR #1658 (caro-shell skill refresh) merged, and the main-CI blockers #1699/#1701 landed. ✓ VERIFIED caro.sh/integrations/ now serves the corrected 1.5.0 copy (loud `remote-backends` error, no "Unknown backend"), and the caro-shell skill on `main` documents `--model-name` and the feature gate. New P2: `caro test --help` advertises `mlx`/`ollama`, but both fail with `Supported: static, embedded`. A merge of `origin/main` into this branch is blocked on the interactive safety-pattern pre-commit prompt (see log).
>
> **Previous:** 2026-10-26 — validated the `caro guard` rows against published `caro 1.5.0` for the first time (they were last checked on the PR #1513 branch). All five harness formats deny Critical commands in enforce mode, malformed input fails safe to `ask`, and the documented smoke commands work verbatim. #1723 stdin hang still live (fix in PR #1043, unmerged). No PR: nothing in the integrator backlog merged since 2026-10-24.
>
> **Previous:** 2026-10-25 — validated the stalest rows (Ollama/vLLM/Exo, Candle CPU, MLX, integrations page) against published `caro 1.5.0`. Remote backends now fail loudly (exit 1) in the default binary, no longer silently. New P1 #1723: top-level `caro "<prompt>"` blocks on an open stdin pipe, which is how agent harnesses spawn it (fix already in PR #1043). No PR: the fix exists, and 7 integrator PRs await review.
>
> **Earlier:** 2026-10-24 — validated the three oldest rows (caro-shell skill, README agent table, `integrations.ts`) against published `caro 1.5.0`. Shell completion for `--backend` still offered `static` (rejected) and hid 6 servable backends; fixed off `CLI_SERVABLE_BACKENDS`. README "Use Caro from your agent" table is stale (MCP/shim marked in-progress with no implementation, OpenRouter marked planned though shipped) — open PR #1689 does not touch that table.
>
> **Earlier:** 2026-10-03 — PR #1513 (`claude/nifty-brown-mtjkpu`) adds the experimental `caro guard` PreToolUse hook (Claude Code / Grok Build / Codex / OpenCode) and wires `grok`, `claude` and `openrouter` into `--backend`. Non-`remote-backends` builds get the existing loud "requires remote-backends" error for those names, the same as `mesh`/`vllm`, so the #1298 divergence stays closed.
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
| Anthropic Claude API | 🚧 in-progress | 2026-10-03 | `create_backend` arm + `CLI_SERVABLE_BACKENDS` entry in PR #1513 (`--backend claude`, `ANTHROPIC_API_KEY`) | #1081, PR #1513 | Wired on the PR branch; ✅ once #1513 merges and a published binary is validated. The shipped 1.4.0 still rejects `--backend claude`. |
| Ollama | ⚠️ partial (feature-gated) | 2026-10-25 | published 1.5.0: `caro --backend ollama --dry-run "list pdfs"` → exit 1, error names `cargo install caro --features remote-backends --locked` and the embedded alternative (loud, no silent fallback) | — | `remote-backends` is **not** in `default = ["embedded-mlx","embedded-cpu","cve-rules"]`. `cargo install caro` and the release-workflow `cargo build --release` (no `--features`) both omit it. |
| vLLM | ⚠️ partial (feature-gated) | 2026-10-25 | same as Ollama — loud error in default binary | — | Same root cause as Ollama. |
| Exo | ⚠️ partial (feature-gated) | 2026-10-25 | same as Ollama — loud error in default binary | — | Same root cause as Ollama. |
| MLX (Apple Silicon embedded) | ⚠️ partial | 2026-10-25 | published 1.5.0: `caro --backend embedded --dry-run "list pdf files"` → `ls -la` (served by static matcher "List files (simple)", drops "pdf"; 1.4.0 gave `ls *.pdf`) | #1645, #1723 | Backend loads in the default build. Static matcher pre-empts the model (#1645). Hangs when stdin is an open pipe (#1723). |
| Candle CPU (embedded) | ✅ working | 2026-10-28 | ✓ published 1.5.0: `caro --dry-run "show disk usage" </dev/null` → `df -h` | — | `embedded-cpu` is in default features; works in default build. |
| OpenRouter | ⚠️ partial (feature-gated) | 2026-10-28 | ✓ published 1.5.0: `caro --backend openrouter --dry-run -p "list files"` → `Configuration error: Backend 'openrouter' requires the 'remote-backends' feature` | #931, #1081 | CLI-wired since PR #1513; same feature-gate omission as claude/grok/ollama. |
| xAI Grok | 🚧 in-progress | 2026-10-03 | wiremock tests; live bad-key call → `Grok authentication failed` | PR #1513 | `--backend grok` / `--advisor grok` / `hybrid_remote = "grok"`; `XAI_API_KEY` |
| Gemini / Jules | 🚧 in-progress | — | — | PR #782 | Coordinate with existing PR; don't fork |
| LM Studio + FunctionGemma | 🚧 in-progress | — | — | PR #558 | Tracked, no action this night |
| quant.cpp | 🚧 in-progress | — | — | PR #838 | Tracked, no action this night |
| Azure Foundry | ⏳ not-yet | — | — | #661 (epic) | Big lift; deprioritized |
| Claude Code session-token | ⏳ not-yet | — | — | #930 | Reuse user's CC subscription/API auth; default to Haiku |

## Caro-as-a-tool (outward integrations)

| Tool | Status | Last validated | Method | GH | Notes |
|---|---|---|---|---|---|
| Claude Code skill (`caro-shell`) | ⚠️ partial | 2026-10-27 | published 1.5.0: `caro --dry-run` with the skill's large-files `find` prompt (stdin closed) → dry-run OK in 0.26 s; SKILL.md on `main` (PR #1658 merged) documents `--model-name` and the `remote-backends` gate | #1490, #1628, #1627 | Guidance fixed (#1629); remaining partial is generation quality (#1627/#1628) and marketplace install (PR #1624) |
| Harness guardian hook (`caro guard`) | 🧪 experimental | 2026-10-28 | ✓ published 1.5.0: `… \| caro guard --mode enforce --no-log` → `permissionDecision: deny` for `rm -rf /`; shadow (default) logs only; `not json` → `ask`; `caro guard report` lists verdicts by harness; e2e `tests/e2e_guard.rs` | spec 011 / ADR-020 | PreToolUse adapter for Claude Code, Grok Build, Codex; OpenCode via `integrations/opencode/caro-guard.ts`. Shadow by default |
| Claude Code MCP server (`caro mcp serve`) | ⏳ not-yet | — | `mcp-inspect` against `caro mcp serve` | #928 | Spec: `.github/first-time-issues/06-mcp-claude-code-integration.md`; tools `generate_command` / `validate_command` / `explain_safety` / `show_decision_tree` |
| OpenAI-compat HTTP shim (`caro serve --openai`) | ⏳ not-yet | — | `curl /v1/chat/completions` with a tool call | #929 | Highest leverage — unlocks Codex/Cursor/Continue/Aider/Tabby in one shot |
| Codex (OpenAI) | 🧪 guard only | 2026-10-26 | `[[hooks.PreToolUse]]` → `caro guard --harness codex` (docs/coding-agents/codex.md) | #789 (Crush MCP config PR) | Generation still awaits OpenAI shim |
| Cursor | ⏳ not-yet | — | OpenAI shim snippet copy-pasted into Cursor settings | — | Satisfied by OpenAI shim |
| Continue (continue.dev) | ⏳ not-yet | — | OpenAI shim snippet | — | Satisfied by OpenAI shim |
| Aider | ⏳ not-yet | — | OpenAI shim snippet | — | Satisfied by OpenAI shim |
| Tabby (self-hosted) | ⏳ not-yet | — | OpenAI shim snippet | — | Satisfied by OpenAI shim |
| opencode | 🧪 guard only | 2026-10-26 | Plugin `integrations/opencode/caro-guard.ts` → `caro guard --harness opencode` | TBD | sst/opencode; generation still awaits MCP/shim |
| crush | ⏳ not-yet | — | MCP or OpenAI shim | #789 (related) | Charm.sh ecosystem |
| droid | ⏳ not-yet | — | MCP or OpenAI shim | TBD | |
| Sourcegraph Amp | ⏳ not-yet | — | MCP | TBD | Enterprise coding agent |
| Letta (MemGPT) | ⏳ not-yet | — | Tool registration | TBD | Persistent-memory agents |
| Pi | ⏳ not-yet | — | TBD | TBD | |
| Aug | ⏳ not-yet | — | TBD | TBD | |
| CodePal | ⏳ not-yet | — | TBD | TBD | |
| Qwen Code | ⏳ not-yet | — | OpenAI shim or native | TBD | |
| Grok Build (xAI) | 🧪 guard only | 2026-10-26 | Reads `~/.claude/settings.json` hooks natively; or `~/.grok/hooks/caro-guard.json` (docs/coding-agents/grok-build.md) | spec 011 | Grok as a *model* via `--backend grok` |
| GitHub Copilot CLI | ⏳ not-yet | — | `preToolUse` hook → `caro guard` | #1623 | Row added 2026-10-25; issue already open |
| Kiro CLI (AWS) | ⏳ not-yet | — | MCP or hook, once available | — | Row added 2026-10-26; listed in 2026 terminal-agent comparisons. Not filed: low leverage until #928/#929 |
| Gemini CLI | ⏳ not-yet | — | Native backend or shim | PR #782 | Coordinate |
| Jules (Google) | ⏳ not-yet | — | Native backend or shim | PR #782 | Coordinate |
| Autocoder | ⏳ not-yet | — | TBD | #667 (epic) | Big lift |
| Handy.Computer | ⏳ not-yet | — | TBD | #662 (epic) | Big lift |

## Marketing & discovery surfaces

| Surface | Status | Last validated | GH | Notes |
|---|---|---|---|---|
| `website/src/data/integrations.ts` | ✅ working | 2026-10-27 | #1657 (closed) | PR #1689 merged; Claude/Ollama/vLLM/Exo entries describe the 1.5.0 loud `remote-backends` error; snippet uses `--model-name` |
| `website/src/pages/integrations/index.astro` | ✅ working | 2026-10-27 | — | caro.sh/integrations/ → 200; live copy says the default binary "no longer silently falls back"; `partial` bucket renders (PR #1689) |
| README "Use caro from your agent" section | ⚠️ partial | 2026-10-24 | — | Table drift: MCP (#928) and OpenAI shim (#929) shown 🚧 though unimplemented; OpenRouter shown ⏳ though wired in 1.5.0; Gemini row cites closed PR #782 (#1439) |
| `.github/first-time-issues/06-mcp-claude-code-integration.md` | 🚫 reference only | — | — | Spec doc; not a runtime surface |

---

## Priority queue (next nights)

Topmost unblocked row drives the next nightly PR.

1. ✅ **DONE — [PR #1298](https://github.com/wildcard/caro/pull/1298) MERGED** (2026-07-12 UTC), **[#1115](https://github.com/wildcard/caro/issues/1115) CLOSED**. Acute divergence half resolved via `backends::CLI_SERVABLE_BACKENDS` single source of truth.
2. **[#1081](https://github.com/wildcard/caro/issues/1081) — wiring half + feature-gate split** (P1; surviving home now #1115 is closed). `create_backend` (`src/cli/mod.rs:295`) has no arm for `claude`/`openrouter`; structs exist under `src/backends/remote/`. Add arms instantiating `ClaudeBackend`/`OpenRouterBackend` with config-error paths for missing `ANTHROPIC_API_KEY`/`OPENROUTER_API_KEY`. **CAUTION:** `CLI_SERVABLE_BACKENDS` is NOT feature-gated — adding those names to the unconditional slice would re-open the #1298 divergence for default (`remote-backends`-off) builds. The roster needs a feature-gated view first. ~60–80 LOC + the gate split — a single-night fit only once the gating approach is settled. **Topmost unblocked integrator row.**
3. **[#1732](https://github.com/wildcard/caro/issues/1732) — `caro test --help` roster drift** (P2). ✓ VERIFIED 2026-10-27 on published 1.5.0: help (`src/main.rs:547`) advertises `static, mlx, ollama, or embedded`; `--backend mlx`/`ollama` → `Unknown backend ... Supported: static, embedded` (`src/main.rs:3091`). One-line doc fix; quick-win slot once the PR backlog clears.
4. **Release-cadence P0 (out of integrator scope — flag to release management / caro-qa-agent):** published crates.io binary is still `1.4.0` (2026-05-09). VERIFIED 2026-07-11 that the #1115 divergence is live in the shipped 1.4.0 (`--backend claude`/`static` → `Unknown backend` despite `--backend-info` advertising them). Mesh-LLM/AI-Horde/hybrid (#1209), OpenRouter (#1097), smart-approval (#1206), the #1092 loud-error fix, and now #1298 are all stranded in main, never reaching a released binary. `cargo install caro` users get none of it. A 1.4.1/1.5.0 cut is overdue.
6. **Claude Code MCP server** (`caro mcp serve`) — spec drafted (`.github/first-time-issues/06-mcp-claude-code-integration.md`, [#928](https://github.com/wildcard/caro/issues/928)); coordinate with [#789](https://github.com/wildcard/caro/pull/789).
7. **OpenAI-compat HTTP shim** (`caro serve --openai`, [#929](https://github.com/wildcard/caro/issues/929)) — single highest-leverage surface; unlocks ~6 long-tail tools at once.
8. **Claude Code session-token reuse backend** ([#930](https://github.com/wildcard/caro/issues/930)) — let users with CC subscription get Haiku for free.
9. **Coordinate Gemini [PR #782](https://github.com/wildcard/caro/pull/782)** — review and merge or supersede. PR #782 is CLOSED; Gemini CLI row retirement (#1439) is covered by sweep PR #1722.
10. **opencode / crush / droid copy-paste snippets** — once OpenAI shim ships, document each.
11. **Sourcegraph Amp / Letta** — MCP-based; ride on top of the MCP server work.
12. **Long tail** — Pi, Aug, CodePal, qwen, Tabby — one each as polish nights.

---

*Each nightly pass updates the date column for any row it validates and may demote/promote rows in the queue based on what shipped and what regressed.*
