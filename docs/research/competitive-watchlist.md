# Competitive Watchlist

**Purpose**: the list of products we compare Caro against, or watch.
**Owner**: maintainers. Anyone may add a row.
**Last full refresh**: 2026-10-11
([research note](./2026-10-11-competitive-landscape-refresh.md),
[raw notes and sources](./2026-10-11-competitive-refresh/))

The field changes monthly. A claim on caro.sh about another product is only
as good as the date we last checked it. This file keeps that date.

## How to suggest a tool

Add a row to **Suggested** with the name, a link, and one line on why it
matters. The next refresh checks it and moves it to a group below.

## How to refresh

1. **Monthly: re-check every product that has a `/compare` page.**
   - Check each cell on its page, the `/compare` matrix
     (`website/src/pages/compare/index.astro`) and the landing matrix
     (`website/src/components/Comparison.astro`).
   - Use primary sources only: vendor docs, license files, changelogs.
   - Update the `verification` date and sources on each page, and
     `lastVerified` in `website/src/data/compare-pages.ts`.
   - Record what changed in a dated note in `docs/research/`.
2. **Quarterly: run a discovery sweep.** Use the five topics in
   [`2026-10-11-competitive-refresh/`](./2026-10-11-competitive-refresh/README.md)
   as the template. The README lists the question each topic answers.
3. **When a row changes status** (new entrant, open-sourced, retired),
   update this file in the same PR as the website fix.

Write sourced claims only. Do not cite a third-party summary when the
vendor's own docs say something else.

## Suggested (not yet checked)

| Product | Link | Why | Suggested by | Date |
|---|---|---|---|---|
| _empty_ | | | | |

## Has a `/compare` page

| Product | Category | Page | Last checked | Status | Key fact to re-check |
|---|---|---|---|---|---|
| Warp | AI terminal / agentic environment | `/compare/warp` | 2026-10-11 | Active | Open-source scope (client only?), local-model support, Agent CLI account requirement |
| GitHub Copilot CLI | Agentic CLI | `/compare/github-copilot-cli` | 2026-10-11 | Active | License, offline/BYOK mode, sandbox default, Free-plan scope |
| Kiro CLI | Agentic CLI | `/compare/kiro-cli` | 2026-10-11 | Active | Local models / BYOK (feature request #8939), license |
| OpenCode | Agentic CLI (MIT) | `/compare/opencode` | 2026-10-11 | Active | Default permissions, offline switches |

## Next `/compare` pages (ranked)

| Rank | Product | Category | Why | Where it beats Caro |
|---|---|---|---|---|
| 1 | Atuin (Atuin AI) | NL→shell in shell history | Same job (`?` key), 32k stars | Already installed by many; frontier models with no setup |
| 2 | Claude Code (auto mode) | Agentic CLI + LLM auto-approval | Largest agent audience; auto mode is a direct "is this safe?" product | Judges intent and context |
| 3 | Codex CLI (Auto-review) | Agentic CLI, Apache-2.0 | Open, local models (`--oss`), sandbox on by default | Sandbox contains what patterns miss |
| 4 | iTerm2 | Terminal with AI | AI composer + opt-in safety check | AI lives in the terminal itself |
| 5 | Shell-GPT | NL→shell CLI (MIT) | Best-known NL→shell CLI; no danger detection | Broader: chat, REPL, roles |
| 6 | AIChat | NL→shell CLI (Rust) | Closest architectural twin; no release in 15 months | Broader: RAG, agents, 20+ providers |

## Cross-harness command guards (closest to the `caro guard` position)

| Product | License | Harnesses | Last checked | Notes |
|---|---|---|---|---|
| dcg (Destructive Command Guard) | "MIT with OpenAI/Anthropic rider" (not OSI) | 17+ | 2026-10-11 | Rust, 50+ rule packs, ~6.1k stars |
| cc-safety-net | MIT | 14+ | 2026-10-11 | Parses wrappers; presets; ~1.6k stars |
| Sage (Gen Digital) + AARTS draft | Apache-2.0 code, DRL 1.1 rules | 5 | 2026-10-11 | 300+ YAML rules; proposes an engine/host standard |
| HOL Guard | Apache-2.0 | 16 | 2026-10-11 | Covers MCP, packages, secrets |
| Numbat (Perplexity) | Apache-2.0 | many (unverified) | 2026-10-11 | Sequence rules; monitor-only by default |
| Dippy, safe-chains | MIT, Apache-2.0 | 1–2 | 2026-10-11 | Real bash parsers for safe auto-approval |

## Vendor auto-approval modes (substitutes)

| Product | Mechanism | Last checked |
|---|---|---|
| Claude Code auto mode | Rules + LLM classifier, Claude models only | 2026-10-11 |
| Codex Auto-review ("guardian") | Sandbox + reviewer agent; policy open source | 2026-10-11 |
| Cursor Auto-review | Allowlist + sandbox + LLM classifier | 2026-10-11 |
| VS Code Assisted permissions | LLM judge (experimental) | 2026-10-11 |
| Grok Build Auto, Qwen Code auto | LLM classifier | 2026-10-11 |
| Factory Droid autonomy levels | Model-assigned risk vs level | 2026-10-11 |

## Sandboxes (partners: the second layer under Caro)

Anthropic `srt`, nono, NVIDIA OpenShell, agentsh, StrongDM Leash, Docker
Sandboxes, container-use, smolvm. Details:
[`guardrail-tools.md`](./2026-10-11-competitive-refresh/guardrail-tools.md).

## Watch (no page planned)

| Product | Category | Status on 2026-10-11 | Why watch |
|---|---|---|---|
| Wave Terminal | AI terminal (Apache-2.0) | Active | Open Warp alternative; command execution "coming soon" |
| Microsoft Intelligent Terminal | AI terminal (MIT) | Active | ACP agent pane; local models in 0.2 |
| cmux | Agent-hosting terminal | Active | Fast-growing 2026 entrant |
| Gemini CLI → Antigravity CLI | Agentic CLI | Gemini CLI retired for consumers 2026-06-18 | Displaced users; Antigravity is closed source |
| Crush, Goose, Amp, Aider, Cline, Mistral Vibe, Kimi Code, Junie, OpenClaw, Pi, Hermes Agent | Agentic CLIs | Mostly active | Harness adapters for `caro guard` |
| zsh-ai, fish-ai, ShellOracle, zev, Butterfish, `llm cmd` | NL→shell tools | Active or slow | Same job, small audiences |

## Retired (good "alternative to" pages)

`gh copilot suggest` (retired 2025-10-25), Charm Mods (archived
2026-03-09), Microsoft AI Shell (unmaintained since 2026-01), Builder.io
AI Shell (stale since 2024-10), Amazon Q Developer CLI (security fixes
only), Fig (folded into Kiro), Kimi CLI (archived).
