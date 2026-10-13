# Warp fact-check for the Caro vs Warp pages (as of 2026-10-11)

> **Corrections, 2026-10-13** (PR #1583 review and owner-supplied sources). Inline text below is corrected where marked. Where this note and the corrections disagree, the corrections win.
>
> - **Routing is scoped, not universal.** Warp's custom-endpoint docs (re-read 2026-10-11) say prompts, context, the endpoint and the key transit Warp in the documented **personal custom-endpoint flow**, and that private/localhost URLs are rejected (https://docs.warp.dev/agents/inference/custom-inference-endpoint/). The BYOK page describes its own flow (https://docs.warp.dev/agents/inference/bring-your-own-api-key/). Warp's pricing page and FAQ wording differs between snapshots, and Enterprise BYOLLM has exceptions (https://www.warp.dev/pricing). Do not publish "every Warp AI request goes through Warp's servers". Rows 2 and 3 and the "Fully offline or local" note below overstate this; read them as limited to the documented personal endpoint/BYOK flows, pending a dated exact-source packet.
> - **Accounts:** the Warp app works without an account; the Agent CLI authenticates separately (login or `WARP_API_KEY`).

**Needs you:** Our Warp pages are wrong on 5 claims: open source, cross-platform, rule-based safety, existing terminal, and model-agnostic. These claims are still correct: cloud required for AI, no offline/air-gapped use, telemetry on by default (Free and individual plans).

Pages that make these claims:
- `website/src/pages/compare/warp.astro`
- `website/src/pages/compare/index.astro`
- `website/src/components/Comparison.astro`
- `website/src/i18n/locales/en/landing.json` (still says "Warp AI")

Method: every claim was checked against warp.dev, docs.warp.dev, the warpdotdev GitHub repo, or Warp press releases. I fetched each one on 2026-10-11. Most docs.warp.dev pages say "Last updated Oct 9, 2026". Third-party sources appear only where marked.

## Summary table

| # | Claim | Our current website says | Current reality (Oct 2026) | Source URL | Source date | Confidence |
|---|---|---|---|---|---|---|
| 1 | Open source | `openSource: 'no'` (Comparison.astro); "Full source code available: false" (compare/index.astro) | **WRONG.** Warp open-sourced its **client** on Apr 28, 2026. Most of the client is **AGPL v3**; the `warpui_core`/`warpui` UI crates are **MIT**. Repo: github.com/warpdotdev/warp, about 65.4k stars and 5.6k forks on 2026-10-11. **Still proprietary:** the server, the Warp Drive backend, hosted auth, Oz orchestration, and the built-in agent harness ("runs server-side"). Use "Partial (client AGPL-3.0, backend closed)". | https://www.warp.dev/blog/warp-is-now-open-source ; https://github.com/warpdotdev/warp ; https://raw.githubusercontent.com/warpdotdev/warp/master/FAQ.md | Apr 28, 2026 (blog); repo and FAQ fetched Oct 11, 2026 (FAQ undated) | High |
| 2 | Requires cloud for AI | "Warp requires cloud for AI features" | **CORRECT** (overstated; see corrections block: scope to the documented personal endpoint/BYOK flows). All AI goes through Warp's servers, including BYOK and custom endpoints. BYOK: the client sends your key to Warp's backend, which calls the provider. Custom endpoints: "Requests route through Warp's servers"; localhost and private URLs "are rejected". FAQ: "Warp's built-in agent harness runs server-side." | https://docs.warp.dev/agents/inference/custom-inference-endpoint/ ; https://docs.warp.dev/agent-platform/inference/bring-your-own-api-key.md ; FAQ.md (above) | Oct 9, 2026 (docs) | High |
| 3 | Works 100% offline / air-gapped | `worksOffline: no`, `airGapped: no` | **CORRECT.** First launch needs internet ("you will need to be online for the initial setup"). Offline, the core terminal works, but Agent, AI Autofill, Drive and Teams do not. "There is no explicit Offline Mode in Warp." | https://docs.warp.dev/support-and-community/troubleshooting-and-support/using-warp-offline/ ; https://docs.warp.dev/getting-started/quickstart/installation-and-setup/ | Oct 9, 2026 | High |
| 4 | Local model inference | `localInference: no`; "Multi-backend (MLX, Ollama, vLLM): false" | **CORRECT, with a nuance.** Warp has no native local inference. To use Ollama, LM Studio, vLLM or llama.cpp, you must expose the model at a **public HTTPS URL** (for example with ngrok). The model list names only cloud-hosted models. A community fork, OpenWarp (zerx-lab), adds local Ollama. It is not affiliated with Warp. | https://docs.warp.dev/agents/inference/custom-inference-endpoint/ ; https://docs.warp.dev/agents/inference/model-choice/ ; https://ai-tldr.dev/releases/zerx-lab-openwarp/ (third-party) | Oct 9, 2026; Apr 29, 2026 (fork) | High (Warp); Med (fork) |
| 5 | Model-agnostic / multi-provider | `multiBackend: no`; "Model-agnostic design: false" | **"Model-agnostic: false" is WRONG.** Hosted models from Anthropic, OpenAI, Google and xAI. Open-weight models (GLM, Kimi, MiniMax, Qwen, DeepSeek) run via Fireworks. Four auto-routing modes. BYOK for OpenAI, Anthropic and Google. Public OpenAI-compatible custom endpoints. Sign in with ChatGPT (Sep 2026). Enterprise BYOLLM (AWS Bedrock, Vertex AI). "Multi-backend (MLX/Ollama/vLLM)" stays "no" for **local** backends. | https://docs.warp.dev/agents/inference/model-choice/ ; https://warp.dev/blog/sign-in-to-warp-with-chatgpt ; https://docs.warp.dev/enterprise/security-and-compliance/security-overview | Oct 9, 2026; Sep 29, 2026 | High |
| 6 | Telemetry by default | "Warp collects telemetry by default (though it can be disabled)" | **CORRECT for Free and individual/team plans.** Opt-out model: turn off "Help improve Warp" and "Send crash reports" in Settings > Privacy. Paid teams: "Data collection is enabled by default." **Business/Enterprise: "disabled by default."** **On Free, "Telemetry must be enabled to use AI features."** Admins can enforce the setting team-wide. | https://docs.warp.dev/support-and-community/privacy-and-security/privacy/ ; https://docs.warp.dev/enterprise/security-and-compliance/security-overview | Oct 9, 2026 | High |
| 7 | Privacy-first design | `privacyFirst: no` | **Defensible, but add nuance.** ZDR agreements with Anthropic, OpenAI, Google and xAI; the security page says ZDR "applies across all Warp plans", while the privacy page names only Business/Enterprise. Regex secret redaction exists, but it is **off by default**. SOC 2 Type II. Prompts and context still go through Warp servers. | privacy + security-overview URLs above ; https://docs.warp.dev/support-and-community/privacy-and-security/secret-redaction/ | Oct 9, 2026 | High (facts); "privacy-first" is a judgment |
| 8 | Cross-platform | `crossPlatform: 'macOS/Linux'` (Comparison.astro); "Cross-platform (macOS, Linux, BSD): partial" | **WRONG.** Supported: macOS 10.14+ (Intel and Apple silicon), **Windows 10 1903+ / Server 2022 (x64, ARM64)**, Linux x64/ARM64 (glibc ≥ 2.31; deb, rpm, Arch, openSUSE, AppImage). Windows launched publicly on Feb 26, 2025, with no beta label then or now. **No BSD build is listed.** The Agent CLI also runs on all three OSes. | https://docs.warp.dev/getting-started/quickstart/installation-and-setup/ ; https://warp.dev/blog/launching-warp-on-windows | Oct 9, 2026; Feb 26, 2025 | High (BSD: Med, absence only) |
| 9 | Rule-based safety validation | `ruleBasedSafety: no`; "Warp relies on AI ... without systematic safety validation" | **WRONG as worded. Use "Partial".** Warp has a **regex command allowlist** (empty by default) and a **regex command denylist**. Default denylist examples: `rm`, `curl`, `wget`, `eval`, `ssh`, shells. A denylist match **requires approval**, even under "Always allow". Admin Panel denylist rules are "never bypassed". **But** auto-approve / "Run until completion" bypasses the user denylist **by default**. A denylist match triggers an approval prompt; it does not block the command. The rules govern **agent-executed** commands. | https://docs.warp.dev/agents/capabilities/agent-profiles-permissions/ ; https://docs.warp.dev/agents/cli/permissions-and-profiles/ | Oct 9, 2026 | High |
| 10 | Customizable safety rules | "Customizable safety rules: false" | **WRONG.** Users and team admins can edit the regex allowlist and denylist. The CLI has `command_allowlist` and `command_denylist` in `settings.toml`; setting `command_denylist` replaces the defaults. There is also an MCP allowlist/denylist and per-action autonomy (Agent decides / Always ask / Always allow / Never). | same as #9 | Oct 9, 2026 | High |
| 11 | Blocks destructive patterns | "Blocks destructive patterns: false" | **Mostly CORRECT.** No documented hard block or curated catastrophic-pattern library. Denylist matches prompt for approval; they are not blocked. A user can set command execution to "Never". | same as #9 | Oct 9, 2026 | Med |
| 12 | Risk level assessment | `riskLevel: no` | **CORRECT as far as the docs show.** No risk classification is documented. "Agent decides" acts "when it's confident" and prompts "when uncertain". The docs also say the Agent "always prompts ... before executing potentially risky commands", with no rules given (model judgment). | same as #9 | Oct 9, 2026 | Med |
| 13 | Explicit confirmation | `explicitConfirmation: partial` | **CORRECT.** "Always ask" exists per action type, but "Always allow" ("YOLO mode") and auto-approve also exist. | same as #9 | Oct 9, 2026 | High |
| 14 | Requires switching terminals | `existingTerminal: no`; "It requires you to switch from your current terminal" | **WRONG since Aug 4, 2026.** The **Warp Agent CLI** (`warp` command) "works in any terminal emulator, including over SSH" (Ghostty, iTerm2, VS Code, Terminal.app, Windows Terminal). It does not need the Warp app, but it **needs a Warp account login** (or `WARP_API_KEY`). The Warp terminal app is still a separate emulator. | https://www.warp.dev/blog/introducing-the-warp-agent-cli-coding-agent ; https://docs.warp.dev/agents/cli/ ; https://docs.warp.dev/agents/cli/quickstart/ ; https://warp.dev/agent-cli | Aug 4, 2026; Oct 9, 2026 | High |
| 15 | Self-hostable | `selfHostable: no` | **Partial (Enterprise only).** Enterprise can self-host cloud-agent execution (`oz-agent-worker`, MIT; Kubernetes, Docker). BYOLLM via Bedrock or Vertex. But the control plane and server are proprietary, and transcripts and inference still route through Warp. | https://docs.warp.dev/enterprise/security-and-compliance/security-overview ; https://github.com/warpdotdev ; https://sdtimes.com/ai/warp-updates-oz-to-help-enterprises-orchestrate-coding-agents-across-any-model-or-harness/ | Oct 9, 2026; May 20, 2026 | High |
| 16 | Single binary distribution | "Single binary: false" | **Probably outdated.** The Agent CLI installs with `curl -fsSL https://app.warp.dev/download/agent-cli \| bash` or `brew install --cask warp-agent-cli`. A Jul 3, 2026 changelog summary mentions statically compiled CLI binaries on Linux, but I did not get that line verbatim. | https://docs.warp.dev/agents/cli/quickstart/ ; https://docs.warp.dev/changelog/2026/ | Oct 9, 2026 | Low |
| 17 | Positioning / name | Tagline "AI-native terminal emulator"; landing.json calls it "Warp AI" | **Outdated.** The homepage hero reads "Open Infrastructure for Cloud Software Factories". The nav lists Factories, Terminal ("An open-source terminal built for AI-assisted software development") and Agent CLI. The footer says "the agentic development environment". "Warp AI" is old branding; the agent is now "Warp Agent". | https://www.warp.dev/ ; https://docs.warp.dev/changelog/2026/ (2026.08.13 rename) | Fetched Oct 11, 2026 | High |
| 18 | Login | (not stated) | The terminal works without login ("Log in to Warp (optional)"), but a logged-out user still gets an anonymous account ID. Changelog 2026.07.03: "Agent onboarding now ties AI features to having a Warp account." The CLI needs login or an API key. | https://docs.warp.dev/getting-started/quickstart/installation-and-setup/ ; using-warp-offline URL ; https://docs.warp.dev/changelog/2026/ | Oct 9, 2026 | High |
| 19 | Team collab / modern UI / blocks | competitor: yes | **CORRECT** (Drive, Teams, Session Sharing, vertical tabs). | using-warp-offline URL | Oct 9, 2026 | High |

## Notes per question

### 1. Open-source status
- **Announced:** Apr 28, 2026. Sources: blog post "Warp is now open-source" (Zach Lloyd) and press release https://www.warp.dev/newsroom/2026/4/28/warp-open-sources-its-agentic-development-environment . The docs changelog has a 2026.04.27 entry: "Warp is now open source".
- **License:** `LICENSE-AGPL` and `LICENSE-MIT` sit at the repo root. README: the "UI framework (the warpui_core and warpui crates) are licensed under the MIT license. The rest of the code in this repository is licensed under the AGPL v3."
- **What is open:** FAQ.md lists "the Warp client app, the WarpUI framework, integration tests, agent skills, and feature specs."
- **What is closed:** FAQ.md: "The server, the Warp Drive backend, and Oz (our agent orchestration layer) are not in this repository and remain proprietary today." Hosted auth is also excluded. "Warp's built-in agent harness runs server-side and isn't open in this repo today." On opening them later: "We haven't committed to a date."
- **Repo:** https://github.com/warpdotdev/warp . About 65.4k stars, 5.6k forks and 3.9k open issues, read from the repo page on 2026-10-11. The `gh` API was not available in this session.
- **Other details:** OpenAI is the "founding sponsor" of the repo. Contributors must sign a CLA. The contribution model is agent-first: Oz triages, specs and implements, and humans review.
- **Other warpdotdev repos:** `oz-agent-worker` (MIT, self-hosted Oz workers), `claude-code-warp` and `codex-warp` (MIT integrations), `docs` (MIT).
- **Do not cite these errors:**
  - everydev.ai says Apache-2.0. Wrong.
  - Wikipedia says Oz was open-sourced. FAQ.md contradicts it.
  - fosslinux.com claims an "Offline Mode" and a self-hostable Drive. Warp's docs contradict both.

### 2. AI and cloud
- **Fully offline or local:** No. (Overstated next sentence; see corrections block.) Every AI request goes through Warp's backend.
  - Custom endpoints must be public HTTPS; "localhost, 127.0.0.1, and other private or local network URLs are rejected."
  - The docs tell local-model users (Ollama, LM Studio, vLLM, llama.cpp) to tunnel to a public URL.
- **BYOK:**
  - Providers: OpenAI, Anthropic and Google.
  - Plans: Free, individual plans, and teams of ≤10 people; larger orgs need Business or Enterprise.
  - The key is stored in the OS keychain, but it is sent to Warp's backend with each request.
  - Not available for Cloud Agents.
- **Other inference options:**
  - Custom OpenAI-compatible endpoints. Since Jul 31, 2026 they also accept the OpenAI Responses and Anthropic Messages schemas.
  - Sign in with ChatGPT (changelog 2026.09.23, blog Sep 29, 2026).
  - SuperGrok login for the CLI (Aug 4, 2026 CLI post).
  - Enterprise BYOLLM through AWS Bedrock or Vertex AI.
- **Hosted models:** Anthropic, OpenAI, Google and xAI, plus open-weight models via Fireworks (GLM, Kimi, MiniMax, Qwen, DeepSeek). Auto modes: `auto`, `auto-efficient`, `auto-genius`, `auto-open`.
- **Login:**
  - The terminal does not need login.
  - AI is tied to a Warp account (changelog 2026.07.03).
  - On Free, telemetry must be on to use AI.
  - The pricing page lists no included agent usage on Free: AI is "pay-as-you-go" or "Bring your own AI inference". The footnote text was not captured (Med).

### 3. Telemetry and privacy
- **Default:**
  - Free and individual: opt-out model.
  - Paid teams: "enabled by default".
  - Business/Enterprise: "disabled by default".
  - Admins can enforce the setting.
- **What is collected:**
  - Analytics go to Rudderstack; crash reports go to Sentry.
  - A Network Log shows telemetry in real time.
  - If you have not opted out, Warp "may collect" AI interactions and console inputs that power AI features.
  - With telemetry off, "no console interactions are ever persisted on Warp's servers".
- **ZDR:** Agreements with Anthropic, OpenAI, Google and xAI. The security overview says it applies "across all Warp plans". The privacy page names only Business/Enterprise. Cite both if you use this.
- **Secret redaction:** Regex-based, off by default. It redacts before AI requests and in the display. It is not applied to Session Sharing.

### 4. Platforms
- macOS 10.14+, Windows 10 1903+ and Linux. Windows has been public since Feb 26, 2025, with PowerShell, WSL and Git Bash.
- No BSD build is listed, and I found no statement either way. This is an absence finding.
- Our "macOS/Linux" cell is wrong. If our row means "macOS, Linux, BSD", Warp is "partial" only because it has no BSD build. It does have Windows.

### 5. Safety and command approval
- **Autonomy options:** Each action type (apply diffs, read files, create plans, execute commands, interact with running commands, MCP) has Agent decides / Always ask / Always allow / Never. Applying diffs under "Agent decides" behaves like "Always ask".
- **Allowlist and denylist:** Both are regex lists. The allowlist starts empty. In normal autonomy modes, the denylist overrides the allowlist and "Agent decides", so a match needs approval (corrected 2026-10-13, thread T19; see the auto-approve bypass below).
- **Auto-approve bypass:** Auto-approve / "Run until completion" bypasses the user denylist by default. To stop that:
  - App: Settings > Agents > Warp Agent > Input.
  - CLI: `auto_approve_bypasses_command_denylist = false`.
- **Admin rules:** Denylist rules set in the Admin Panel are never bypassed.
- **Other controls:** an MCP allowlist and denylist, and directory allowlist/denylist in Oz CLI profiles.
- **What Warp does not have:** no curated destructive-pattern library, no risk levels, and no hard block. A match means "ask", not "deny".
- **Fair framing for Caro:** "Warp gates agent-run commands with a user-editable regex denylist that asks for approval. Caro checks every generated command against 52+ curated patterns with risk levels, and blocks critical ones."

### 6. Running outside the Warp terminal
- **Agent CLI:** Launched Aug 4, 2026; command `warp`. It runs in any terminal and over SSH, needs no app install, and needs login or `WARP_API_KEY`.
- **Oz CLI:** `oz` runs cloud agents from scripts and CI.
- **MCP:** Both the app and the CLI are MCP clients. A built-in Factory MCP server exists for logged-in users (2026.08.18).
- **Third-party agents inside Warp:** Universal Agent Support (Apr 14, 2026) covers 16 CLI agents, including Claude Code, Codex, Gemini CLI, OpenCode, Amp, Copilot CLI, Cursor CLI, Goose, Kiro CLI and Grok Build. It adds rich input, inline code review, vertical tabs and Remote Control. Notifications work only for Claude Code, Codex and OpenCode.
- **Cloud harnesses:** Oz and Factories run Claude Code and Codex harnesses in the cloud (May 20, 2026).

### 7. Product direction
**Timeline:**
- Jun 24, 2025: Warp 2.0 "Agentic Development Environment".
- Sep 3, 2025: Warp Code.
- Feb 10, 2026: Oz cloud agents.
- Mar 22, 2026: newsroom item "Warp reaches 1 million active users" (title seen in the listing; page not fetched).
- Apr 14, 2026: Universal Agent Support.
- Apr 27–28, 2026: open source, plus the TOML settings file.
- May 20, 2026: Oz multi-harness and Kubernetes self-hosting.
- Aug 4, 2026: Agent CLI.
- Aug 13, 2026: the "Oz agent" UI is renamed "Warp Agent".
- Aug 18, 2026: **Warp Factories** (closed beta / Early Access).
- Sep 23–29, 2026: Sign in with ChatGPT.
- Oct 7, 2026: Factories works with Microsoft Teams and Azure DevOps.

The homepage now leads with Factories.

**Pricing (Terminal and agents):**
- Free $0 (BYOK/BYO inference, pay-as-you-go).
- Build $20/mo ($18 annual), with $20 of usage at API rates.
- Max $200/mo ($180 annual), 10× Build.
- Business $50/user ($45 annual), with SSO, BYOK/custom endpoints and admin data controls.
- Enterprise: custom (BYOLLM, self-hosted cloud agents).

Factories has separate pricing, from pay-as-you-go (20% above API rates) up to Enterprise.

**User counts disagree:** the homepage says "800k+ devs"; the Apr 28 and Aug 18 press releases say "nearly one million developers". Quote one source and name it.

### 8. Other points the Caro vs Warp page must get right
1. **Same license family.** The Warp client is AGPL-3.0, like Caro. Do not say "Caro is open source and Warp is not". The difference: Caro's whole stack, inference included, is open and runs locally. Warp's AI harness and backend are closed and hosted.
2. **Strongest true differentiators:**
   - No Warp AI feature works offline or on a local model without a public tunnel.
   - Free-plan AI needs telemetry on.
   - AI is tied to a Warp account.
   - Warp's safety is approval-gating with a coarse, user-editable regex denylist. The user denylist is bypassed by auto-approve by default. Warp has no curated pattern library, no risk levels and no blocking.
3. **Retire the "you must switch terminals" argument,** or limit it to the Warp app. The `warp` Agent CLI runs in any terminal. A fairer contrast: Warp's CLI is a full coding agent that needs an account and the cloud; Caro is a single-purpose NL-to-command tool that runs offline.
4. **Warp still has NL-to-command in the terminal.** "AI Command Search" opens with `#`, and a Aug 27, 2026 setting stops `#` from opening it. This is the feature closest to Caro.
5. **Fix the Windows error** in both comparison tables.
6. **Rename "Warp AI"** in `landing.json` (and other locales) to "Warp". Consider a tagline such as "Agentic development environment (open-source client)".

## Could not verify
- Whether the Agent CLI source code is in the open-source repo. Not stated anywhere I read.
- The default autonomy value for "execute commands" in the default profile.
- Whether the command denylist applies to `#` AI Command Search suggestions, or only to agent-executed commands. The docs describe only agent execution.
- The Free-tier footnote text and whether Free includes any agent usage.
- "Statically compiled CLI binaries" (Jul 3, 2026): from a tool summary of the changelog, not a verbatim quote.
- Exact star count through the API. The 65.4k figure comes from the repo page.
- BSD support: no official statement either way. I found only that no BSD build is listed.
