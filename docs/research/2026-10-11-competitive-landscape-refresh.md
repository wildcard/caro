# Competitive Landscape Refresh

**Date**: 2026-10-11
**Status**: Research. Website fixes landed in the same PR. Positioning
proposals are in
[`docs/strategy/cross-harness-auto-mode.md`](../strategy/cross-harness-auto-mode.md).
**Raw notes and sources**: [`2026-10-11-competitive-refresh/`](./2026-10-11-competitive-refresh/)
(one file per topic, every claim with a URL and a date)
**Living list**: [`competitive-watchlist.md`](./competitive-watchlist.md)

Our comparison pages were written in 2025 and early 2026. The field moved
faster than the pages. This note records what changed, what still
separates Caro from the rest, and what is now false.

Method: five research agents checked vendor docs, repos, changelogs and
license files on 2026-10-11. The parent session spot-checked the claims
that change our messaging most:

- Warp's open-source post
- Copilot CLI BYOK and offline mode
- Copilot's local sandbox
- Claude Code auto mode
- Codex Auto-review

Confidence labels in the raw notes:

- **High**: from a primary source fetched that day.
- **Med**: from a primary source seen only in search results.
- **Low**: from secondary sources only.

---

## 1. What changed

| # | Claim on our site (before) | Reality on 2026-10-11 | Source |
|---|---|---|---|
| 1 | Warp is not open source | Warp's **client** has been AGPL-3.0 since **2026-04-28**; its UI crates are MIT. The server, Warp Drive, the Oz orchestration layer and the agent harness stay closed. | [warp.dev blog](https://www.warp.dev/blog/warp-is-now-open-source), [repo FAQ](https://github.com/warpdotdev/warp) |
| 2 | Warp makes you switch terminals | The Warp **app** does. The **Warp Agent CLI** (2026-08-04) runs in any terminal, but needs a Warp account. | [Agent CLI post](https://www.warp.dev/blog/introducing-the-warp-agent-cli-coding-agent) |
| 3 | Warp is macOS/Linux only | Warp has run on Windows since 2025-02-26. It has no BSD build. | [install docs](https://docs.warp.dev/getting-started/quickstart/installation-and-setup/) |
| 4 | Warp has no rule-based safety | Warp has regex allow/deny lists for agent commands. A match asks for approval; it never blocks. By default, auto-approve skips the user's denylist. | [permissions docs](https://docs.warp.dev/agents/capabilities/agent-profiles-permissions/) |
| 5 | Copilot CLI cannot work offline | Since **2026-04-07**, Copilot CLI supports BYOK and local models (Ollama, vLLM, Foundry Local). `COPILOT_OFFLINE=true` stops all calls to GitHub. | [changelog](https://github.blog/changelog/2026-04-07-copilot-cli-now-supports-byok-and-local-models/) |
| 6 | Copilot CLI is "partially" open source | The license is proprietary ("GitHub Copilot CLI License"). The repo holds no source code. | [LICENSE.md](https://github.com/github/copilot-cli/blob/main/LICENSE.md) |
| 7 | Copilot CLI costs $10–19/month | Copilot Free includes the CLI. Paid plans: Pro $10, Pro+ $39, Max $100, Business $19/seat, Enterprise $39/seat. Billing has been in AI Credits since 2026-06-01. | [plans](https://docs.github.com/en/copilot/get-started/plans) |
| 8 | Copilot CLI has no safety layer | Copilot CLI has deterministic `--deny-tool` rules, hooks, and a local OS sandbox (GA **2026-10-07**, off by default). It ships no curated list of dangerous commands. | [allowing tools](https://docs.github.com/en/copilot/how-tos/copilot-cli/use-copilot-cli/allowing-tools), [sandbox GA](https://github.blog/changelog/2026-10-07-local-sandboxing-for-gitHub-copilot-now-generally-available) |
| 9 | "Customizable safety rules" is Caro-only | All four competitors now let users write allow/deny rules. | raw notes |
| 10 | OpenCode asks before commands ("partial") | OpenCode "allows all operations without requiring explicit approval" by default. | [config docs](https://opencode.ai/docs/config/) |
| 11 | shell-gpt is "partially" open source | shell-gpt is MIT. | [repo](https://github.com/TheR1D/shell_gpt) |

**Not changed:**

- Kiro CLI is still closed source and runs only on Amazon Bedrock.
- Warp AI still routes through Warp's servers, even with BYOK.
- Warp rejects localhost endpoints.

### New facts that change the strategy

- **Every frontier harness now ships an LLM auto-approval judge.** Some vendors made theirs the default:
  - Claude Code auto mode is the starting mode from v2.1.283.
  - Codex has Auto-review ("guardian").
  - Cursor Auto-review is the default since 3.6.
  - VS Code has Assisted permissions.
  - Grok Build and Qwen Code each have an Auto mode.
  - Details: [`auto-modes.md`](./2026-10-11-competitive-refresh/auto-modes.md).
- **Each vendor's judge works only with that vendor's models or backend.**
  - Claude Code auto mode needs a supported Claude model.
  - Codex Auto-review fails on custom providers, because the `codex-auto-review` model name is hardcoded.
  - Cursor's classifier runs on Cursor's backend.
- **Two free cross-harness command guards already exist:**
  - [dcg](https://github.com/Dicklesworthstone/destructive_command_guard): Rust, about 6.1k stars, 17+ harnesses.
  - [cc-safety-net](https://github.com/kenryu42/cc-safety-net): MIT, about 1.6k stars, 14+ harnesses.
  - Details: [`guardrail-tools.md`](./2026-10-11-competitive-refresh/guardrail-tools.md).
- **Hooks converge on the Claude Code `PreToolUse` shape, but no formal standard exists.**
  - Copilot CLI, VS Code, Cursor and Grok Build read `.claude/settings.json` hooks.
  - Gen Digital proposed a draft engine/host standard, [AARTS v0.1](https://github.com/gendigitalinc/aarts). It has no named adopters yet.
- **Regex guards on raw text can be bypassed.** Adversa AI's "GuardFall" disclosure (2026-06-30) shows the bypass class. PR #1513 measured 6 such gaps in Caro's safety core (tracked in #1054).
- **[Atuin AI](https://docs.atuin.sh/latest/ai/introduction/) (2026-03) does Caro's core job.**
  - The `?` key turns English into bash, and Atuin flags dangerous commands.
  - It reaches 32k GitHub stars.
  - By default it uses cloud models through Atuin Hub. A self-hosted server with local models has existed since 2026-07.

---

## 2. What still separates Caro

| Status | Claim | Notes |
|---|---|---|
| **Gone** | "The open-source option" | Warp's client, OpenCode, Codex CLI and Gemini CLI are all open source now. |
| **Gone** | "Customizable safety rules" | Every competitor has user-written rules. |
| **Gone** | "Works in your terminal" vs Warp, as a blanket claim | It holds against the Warp app, not against the Warp Agent CLI. |
| **Eroded** | "Works offline" | Copilot CLI (opt-in), OpenCode (after configuration), Codex (`--oss`) and Atuin (self-hosted server) all work offline now. Caro is still **offline by default**, with no provider setup. |
| **Durable** | A curated pattern library with risk levels, which blocks critical commands, with no configuration | No competitor on our pages ships one. dcg and cc-safety-net do, outside our pages. |
| **Durable** | Local by default: an embedded model and no account | The model downloads once on first use. After that Caro needs no network. |
| **Durable** | The whole stack is open, inference included | Warp opens only its client. Copilot CLI and Kiro CLI are proprietary. |
| **Durable** | A shell specialist: one command, platform-aware, GNU/BSD-aware | Every competitor on our pages is a full coding agent. |
| **Durable** | A weekly CVE-derived rule pipeline | No competitor found has one. |

The safe wording for safety claims is now: **"Caro ships a curated, tested
set of 52+ patterns with risk levels. They run offline before every
command, with no configuration."** Do not say "only Caro validates
deterministically".

---

## 3. Warp is open source: what we say now

The owner's proposal: move the Warp message from "open source" to "we
don't replace your terminal (TTY)".

Facts that bear on it:

- The Warp **app** is a terminal emulator. It replaces your terminal.
- The Warp **Agent CLI** runs in any terminal, but needs a Warp account and the cloud.
- Warp hosts 16 third-party agents (Claude Code, Codex, OpenCode and others). Its denylist covers only Warp's own agent.

| Option | Message | Holds today? |
|---|---|---|
| A | "Keep your terminal" | Partly. True against the app, false against the Agent CLI. |
| B | "Fully open, fully local": the whole stack, inference included, is open and runs on your machine. Warp opens its client; its agent harness runs on Warp's servers. | Yes, with sources. |
| C | "Your guard, in every agent": Caro checks commands from any agent in any terminal. | Only after `caro guard` (#1513) merges. |

**Recommendation:** lead with B, keep A as the supporting line, and add
C when #1513 ships. The rewritten `/compare/warp` page follows B + A.

---

## 4. Auto-mode landscape (summary)

| Harness | How it auto-approves | Locked to vendor models? | Open source? | Hook an external validator can use |
|---|---|---|---|---|
| Claude Code | Rules, then an LLM classifier (default mode from v2.1.283) | Yes (Claude only) | No (`srt` sandbox is Apache-2.0) | `PreToolUse` (allow/deny/ask/defer) |
| Codex | OS sandbox; escalations go to the Auto-review agent ("guardian") | In practice yes (it breaks on custom providers) | Yes (Apache-2.0, policy in `core/src/guardian/policy.md`) | `PreToolUse` (allow/deny only) |
| Cursor | Allowlist, then sandbox, then an LLM classifier (default since 3.6) | Yes (Cursor backend) | No | `beforeShellExecution`, `preToolUse` |
| Copilot CLI | Prompts; deny rules; optional sandbox | No (BYOK) | No | `preToolUse`, `permissionRequest`; also reads `.claude/settings.json` |
| Gemini CLI | TOML policy engine, no judge | Gemini | Yes (Apache-2.0) | `BeforeTool` (veto only) |
| Kiro CLI | Prompts plus `permissions.yaml` | Bedrock | No | `PreToolUse` (exit 2) |
| OpenCode | Wildcard rules; allows all by default | No | Yes (MIT) | Plugin `tool.execute.before` |
| Factory Droid | Model-assigned risk vs autonomy level | BYOK (unverified) | No | `PreToolUse` (Claude shape) |

The full table covers 20 harnesses: [`auto-modes.md`](./2026-10-11-competitive-refresh/auto-modes.md).

**The gap:**

- Users who bring their own model get no Claude, Codex or Cursor judge. (goose Smart Approve uses the user's own provider, and Cline lets the model mark its own commands.)
- A policy written for one harness does not carry over to another.
- Vendor judges are not deterministic, and the vendor can change them server-side.
- They send parts of the transcript to the vendor.

**The strongest counter-argument:**

- The built-in judge is free, on by default, and sees intent.
- Every vendor says the sandbox, not the classifier, is the security boundary.

---

## 5. Recommended new comparison pages

From [`discovery.md`](./2026-10-11-competitive-refresh/discovery.md), ranked by overlap with Caro's two jobs ("I forgot the command" and "is this command safe?"), then by audience:

1. **Atuin (Atuin AI)**: the same job, a big shell-user audience, and cloud by default.
2. **Claude Code (auto mode)**: the largest agent audience; an LLM judge vs a deterministic floor.
3. **Codex CLI**: open source, local models, a sandbox and deterministic rules. It forces an honest page.
4. **iTerm2**: an AI composer and an opt-in model-based safety check, macOS only.
5. **Shell-GPT**: the best-known NL→shell CLI, with no danger detection.
6. **AIChat**: a Rust twin with no release in 15 months.

Each new page needs the `verification` block (date + sources) that this PR
added to `ComparisonPageLayout`.

---

## 6. Not verified, or carried over without re-checking

- The landing matrix row "Cross-platform (macOS/Linux/BSD)" still says
  "yes" for Copilot CLI, Kiro CLI and OpenCode. Nobody checked BSD support
  for them.
- The "POSIX-first", "GNU vs BSD awareness" and "shell-specific
  optimizations" cells are judgment calls carried over from 2025.
- Whether Copilot's "assisted" permission mode is an LLM judge (secondary source only).
- Whether a Claude Code `PreToolUse` "allow" skips the auto-mode classifier.
- dcg's license: its README says "MIT with an OpenAI/Anthropic rider". That
  is not OSI open source, so do not import its rules into AGPL code.
  This needs legal review.

## 7. Broken windows found (not fixed here)

- `website/src/pages/[lang]/compare/index.astro` never renders. Astro's
  i18n fallback redirects `/<lang>/compare` to `/compare`, and the build
  warns about the route conflict. This PR keeps the file in sync through
  `src/data/compare-pages.ts`, but the page is dead.
- The landing tooltip for "Privacy-first design" says "Minimal, anonymous
  telemetry with easy opt-out". GA builds use opt-in telemetry
  (`src/telemetry/config.rs`). The claim is about Caro, in 15 locales.
- `website/src/config/search-index.json` still has the old Copilot
  pricing. The `prebuild` step regenerates it on every deploy.
