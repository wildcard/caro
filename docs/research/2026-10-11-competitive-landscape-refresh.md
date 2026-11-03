# Competitive Landscape Refresh

**Date**: 2026-10-11 (corrected 2026-10-13 after the PR #1583 review)
**Status**: Research. Website fixes are in the same PR (not merged). Positioning
proposals are in
[`docs/strategy/cross-harness-auto-mode.md`](../strategy/cross-harness-auto-mode.md).
**Raw notes and sources**: [`2026-10-11-competitive-refresh/`](./2026-10-11-competitive-refresh/)
(one file per topic; most claims carry a URL and a date; each file opens with dated corrections)
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
| 8 | Copilot CLI has no safety layer | Copilot CLI has deterministic `--deny-tool` rules, hooks, and a local OS sandbox (GA **2026-10-07**; the announcement does not state the default). Its docs describe no curated list of dangerous commands. | [allowing tools](https://docs.github.com/en/copilot/how-tos/copilot-cli/use-copilot-cli/allowing-tools), [sandbox GA](https://github.blog/changelog/2026-10-07-local-sandboxing-for-gitHub-copilot-now-generally-available) |
| 9 | "Customizable safety rules" is Caro-only | All four competitors now let users write allow/deny rules. | raw notes |
| 10 | OpenCode asks before commands ("partial") | OpenCode allows most operations, shell commands included, by default. Exceptions: `doom_loop` and `external_directory` ask; `.env` reads are denied (`.env.example` allowed). | [config docs](https://opencode.ai/docs/config/) |
| 11 | shell-gpt is "partially" open source | shell-gpt is MIT. | [repo](https://github.com/TheR1D/shell_gpt) |

**Not changed:**

- Kiro CLI is still closed source and runs only on Amazon Bedrock.
- Warp rejects localhost and private custom endpoints.
- Where Warp AI traffic flows is **not settled**. The custom-endpoint docs say
  prompts, context, the endpoint and the key transit Warp in the personal
  custom-endpoint flow. Warp's pricing page and FAQ wording differs by
  snapshot, and Enterprise BYOLLM has exceptions. We publish no general
  routing claim until a dated exact-source packet settles it.

### New facts that change the strategy

- **Many harnesses now offer an LLM auto-approval judge; two turn it on by default.**
  - On by default: Claude Code auto mode (starting mode from v2.1.283) and Cursor Auto-review (since 3.6).
  - Opt-in: Codex Auto-review ("guardian", `approvals_reviewer = "auto_review"`), VS Code Assisted permissions (experimental; Manual is the default), and the Auto modes of Grok Build and Qwen Code (Ask is the default).
  - Claude Code also has built-in protections for critical-path removal, and a mod surface (`tool.check`) that can approve before the classifier. Hooks and mods have different authority.
  - Details: [`auto-modes.md`](./2026-10-11-competitive-refresh/auto-modes.md).
- **Some judges are tied to their vendor; not all.**
  - Claude Code auto mode needs a supported Claude model. Cursor's classifier runs on Cursor's backend.
  - Codex Auto-review failed on custom providers because of a hardcoded model name. That is an operational bug with catalog and gateway workarounds ([openai/codex#31255](https://github.com/openai/codex/issues/31255)), not lock-in.
  - goose Smart Approve uses the user's own provider.
- **Two cross-harness command guards already exist:**
  - [dcg](https://github.com/Dicklesworthstone/destructive_command_guard): Rust, about 6.1k stars, 17+ harnesses. **Source-available**: MIT text plus a rider that grants no rights to OpenAI, Anthropic, their affiliates or anyone acting for them, and that counts benchmarking, testing and analyzing as use. We compare it from public docs only, and do not execute, benchmark or import it.
  - [cc-safety-net](https://github.com/kenryu42/cc-safety-net): MIT, about 1.6k stars, 16 harnesses (README, 2026-10-13), parsed-command handling, secret protection, cloud/IaC rulebooks. Its [known limitations](https://ccsafetynet.com/docs/guides/known-limitations) name incomplete event coverage and fail-open hosts.
  - Details: [`guardrail-tools.md`](./2026-10-11-competitive-refresh/guardrail-tools.md).
- **Hooks converge on the Claude Code `PreToolUse` shape, but no formal standard exists.**
  - Copilot CLI, VS Code, Cursor and Grok Build read `.claude/settings.json` hooks.
  - Gen Digital proposed a draft engine/host standard, [AARTS v0.1](https://github.com/gendigitalinc/aarts). It has no named adopters yet.
- **Regex guards on raw text can be bypassed.** Adversa AI's "GuardFall" disclosure (2026-06-30) shows the bypass class. #1513 measured 6 such gaps in Caro's safety core. Open PR #1614 closes those six; #1639 lists the bypasses that remain. The parser work is #1302 and #1054.
- **[Atuin AI](https://docs.atuin.sh/18.17/ai/introduction/) (2026-03) does Caro's core job.**
  - The `?` key turns English into bash, and Atuin flags dangerous commands.
  - It reaches 32k GitHub stars.
  - It uses Atuin Hub by default, and [self-hosting](https://docs.atuin.sh/latest/ai/self-hosting/) supports local inference. It is a direct competitor on both jobs.

---

## 2. What still separates Caro

| Status | Claim | Notes |
|---|---|---|
| **Gone** | "The open-source option" | Warp's client, OpenCode, Codex CLI and Gemini CLI are all open source now. |
| **Gone** | "Customizable safety rules" | Every competitor has user-written rules. |
| **Gone** | "Works in your terminal" vs Warp, as a blanket claim | It holds against the Warp app, not against the Warp Agent CLI. |
| **Eroded** | "Works offline" | Copilot CLI (opt-in), OpenCode (after configuration), Codex (`--oss`) and Atuin (self-hosted server) all work offline now. Caro is still **offline by default**, with no provider setup. |
| **Holds, unmeasured** | A curated pattern library with risk levels; at the default safety level it blocks Critical commands, with no configuration | No competitor on our pages documents one. dcg and cc-safety-net ship rule sets outside our pages, and Claude Code has built-in critical-path protections. Whether Caro detects **more at the same nuisance rate** is unmeasured (test 1 in the strategy note). |
| **Holds, scoped** | Local by default: the default embedded backend with a cached model, and no account | The model downloads once on first use. Optional remote backends send prompts to the server the user picks. |
| **Holds, scoped** | The whole stack is open: code AGPL-3.0, default model (Qwen2.5-Coder 1.5B) Apache-2.0 | Warp opens only its client. Copilot CLI and Kiro CLI are proprietary. Other catalog models carry their own licenses. |
| **Holds, scoped** | A shell specialist: one command, with platform detection and GNU/BSD capability profiles (`src/prompts/capability_profile.rs`, `tests/beta_regression.rs`) | Every competitor on our pages is a full coding agent. Atuin AI is the exception off our pages. How well Caro's output follows the GNU/BSD profiles was not re-measured here, and competitors' GNU/BSD awareness is unverified (§6). |
| **Not found elsewhere** | A weekly CVE-derived rule pipeline | Not seen in this sweep. That is not proof that no one has one. |

The safe wording for safety claims is now: **"Caro ships a curated set of
52+ patterns with risk levels. They check every command Caro generates,
offline, with no configuration."** `caro guard` can check other agents'
commands, but by default it only logs (shadow mode). Do not say "only Caro validates deterministically", and
do not claim better detection until a matched-nuisance-rate measurement
exists. Note #1597: the website claims suite never calls the validator, so
its green CI result is not evidence for any safety claim.

---

## 3. Warp is open source: what we say now

The owner's proposal: move the Warp message from "open source" to "we
don't replace your terminal (TTY)".

Facts that bear on it:

- The Warp **app** is a terminal emulator. It replaces your terminal.
- The Warp **Agent CLI** runs in any terminal. It authenticates separately (Warp login or `WARP_API_KEY`) and needs the network. The app itself works without an account.
- Warp hosts 16 third-party agents (Claude Code, Codex, OpenCode and others). Its denylist covers only Warp's own agent.

| Option | Message | Holds today? |
|---|---|---|
| A | "Keep your terminal" | Partly. True against the app, false against the Agent CLI. |
| B | "Fully open, fully local", scoped: with the default embedded backend and a cached model, the code (AGPL-3.0) and default model (Apache-2.0) are open and run on your machine. Warp opens its client; its server, Drive backend, orchestration and agent harness are not in the repository. | Yes, if scoped this way. |
| C | "Your guard, in every agent": Caro checks commands from any agent in any terminal. | Not yet. `caro guard` merged in #1513 (2026-10-11) as an experimental, shadow-default hook for 4 harnesses. Verification comes first: #1597, #1537, #1623. |

**Recommendation:** lead with the scoped B, keep A as the supporting line,
and hold C until the guard's shadow evidence exists (spec 011 pilot). The
rewritten `/compare/warp` page follows scoped B + A.

---

## 4. Auto-mode landscape (summary)

| Harness | How it auto-approves | Locked to vendor models? | Open source? | Hook an external validator can use |
|---|---|---|---|---|
| Claude Code | Rules, built-in critical-path protections, then an LLM classifier (default mode from v2.1.283); a `tool.check` mod can approve before the classifier | Yes (Claude only) | No (`srt` sandbox is Apache-2.0) | `PreToolUse` (allow/deny/ask/defer); mods (different authority) |
| Codex | OS sandbox; escalations go to the Auto-review agent ("guardian"), opt-in | No by design; a custom-provider bug has workarounds (openai/codex#31255) | Yes (Apache-2.0, policy in `core/src/guardian/policy.md`) | `PreToolUse` (allow/deny only) |
| Cursor | Allowlist, then sandbox, then an LLM classifier (default since 3.6) | Yes (Cursor backend) | No | `beforeShellExecution`, `preToolUse` |
| Copilot CLI | Prompts; deny rules; optional sandbox | No (BYOK) | No | `preToolUse`, `permissionRequest`; also reads `.claude/settings.json` |
| Gemini CLI | TOML policy engine, no judge | Gemini | Yes (Apache-2.0) | `BeforeTool` (veto only) |
| Kiro CLI | Prompts plus `permissions.yaml` | Bedrock | No | `PreToolUse` (exit 2) |
| OpenCode | Wildcard rules; allows most operations by default (doom_loop/external_directory ask; .env reads denied) | No | Yes (MIT) | Plugin `tool.execute.before` |
| Factory Droid | Risk level vs autonomy level (who assigns the risk is unverified) | BYOK (unverified) | No | `PreToolUse` (Claude shape) |

The full table covers 20 harnesses: [`auto-modes.md`](./2026-10-11-competitive-refresh/auto-modes.md).

**The gap:**

- Users who bring their own model get no Claude Code or Cursor judge. Codex Auto-review is not tied to OpenAI models by design, but a custom-provider bug needs workarounds (openai/codex#31255). (goose Smart Approve uses the user's own provider, and Cline lets the model mark its own commands.)
- A policy written for one harness does not carry over to another.
- Vendor judges are not deterministic, and the vendor can change them server-side.
- They send parts of the transcript to the vendor.

**The strongest counter-argument:**

- In Claude Code and Cursor, the built-in judge is free, on by default, and sees intent.
- Every vendor says the sandbox, not the classifier, is the security boundary.

---

## 5. Recommended new comparison pages

From [`discovery.md`](./2026-10-11-competitive-refresh/discovery.md), ranked by overlap with Caro's two jobs ("I forgot the command" and "is this command safe?"), then by audience:

1. **Atuin (Atuin AI)**: the same job, a big shell-user audience; Hub by default, local inference when self-hosted.
2. **Claude Code (auto mode)**: the second-largest measured agent audience (Codex CLI is first); an LLM judge plus built-in protections vs a deterministic floor.
3. **Codex CLI**: open source, local models, a sandbox and deterministic rules. It forces an honest page.
4. **iTerm2**: an AI composer and an opt-in model-based safety check, macOS only.
5. **Shell-GPT**: the best-known NL→shell CLI, with no danger detection.
6. **AIChat**: a Rust twin with no release in 15 months.

Each new page needs the `verification` block (date + sources) that this PR
added to `ComparisonPageLayout`.

---

## 6. Not verified, or carried over without re-checking

- BSD support for Copilot CLI, Kiro CLI and OpenCode was not checked. The
  landing matrix row "Cross-platform (macOS/Linux/BSD)" shows those cells as
  "Not documented" (corrected 2026-10-13). OpenCode's risk-level cell is
  "Not documented" for the same reason.
- The "POSIX-first", "GNU vs BSD awareness" and "shell-specific
  optimizations" cells are judgment calls carried over from 2025.
- Whether Copilot CLI's `assisted` permission mode uses an LLM judge. The
  official reference confirms only the mode name. VS Code Copilot Chat's
  separate Assisted permissions mode is documented as an LLM judge; do not
  carry that over to the CLI.
- Whether a Claude Code `PreToolUse` "allow" skips the auto-mode classifier.
- dcg's license (read 2026-10-13): MIT text plus a rider. The rider grants
  no rights to OpenAI, Anthropic, their affiliates or anyone acting for
  them, and counts benchmarking, testing and analyzing as use. This project
  does not execute, benchmark or import dcg; comparisons use public docs
  only. Any test that involves dcg needs a human license decision first.
- Copilot's local sandbox default: the GA changelog does not state it.
- Warp AI routing: sources conflict; see section 1. No general claim.

## 7. Broken windows found (not fixed here)

- `website/src/pages/[lang]/compare/index.astro` never renders. Astro's
  i18n fallback redirects `/<lang>/compare` to `/compare`, and the build
  warns about the route conflict. This PR keeps the file in sync through
  `src/data/compare-pages.ts`, but the page is dead.
- The landing tooltip for "Privacy-first design" says "Minimal, anonymous
  telemetry with easy opt-out". GA builds use opt-in telemetry
  (`src/telemetry/config.rs`). The claim is about Caro, in 15 locales.
- `website/src/config/search-index.json` is regenerated by `prebuild` on
  every deploy; the committed copy lags the pages.
- Caro telemetry copy: `en/landing.json` says "Zero telemetry" twice and
  calls telemetry "easy opt-out". GA builds are opt-in
  (`src/telemetry/config.rs`). The text spans 15 locales; this PR only fixes
  the `/use-cases` table row.
- `website/src/components/Comparison.astro` renders category `term` icons
  as plain strings, so the `TermGlyph` allowlist guard does not cover them
  (values are valid today). Pre-existing on `main`.
- `website/src/pages/product/jobs-to-be-done.astro` still says "Bundled model
  works where nothing else can". The model is downloaded on first use, and
  the exclusivity claim no longer holds. Not in this PR's files.
- #1597: the website claims suite calls a nonexistent `--validate-only` flag,
  so its checks never reach the validator. #1537: the latest census
  (at `4a84d09`) finds 26 of 49 test targets run in no workflow. Fix verification before stronger safety positioning.
