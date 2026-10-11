# Caro competitor discovery watchlist (2026-10-11)

**Needs you:** Two existing compare pages contain claims that are no longer true. See "Drift on existing pages" at the end. Fixing them is a separate task.

**Result:** I list 77 entries in three groups. The top 6 new `/compare` candidates are **Atuin (Atuin AI), iTerm2, Shell-GPT, Claude Code, Codex CLI, and AIChat**.

**Biggest finding:** Atuin added Atuin AI in March 2026 (v18.13). It does Caro's core job: the `?` key turns English into bash, and it flags dangerous commands. It has 32.0k GitHub stars and 8,964 Homebrew installs in the last 30 days.

## How I verified

- **Repo facts** (license, last commit, last tag date): from shallow `git clone` of each repo on 2026-10-11. The GitHub API was blocked in this session.
- **Release dates:** from the npm, PyPI, crates.io and Go proxy registries, or from git tag commit dates.
- **Audience signals:** Homebrew 30-day install analytics (`formulae.brew.sh/api/...`), npm and PyPI monthly downloads, and star counts from GitHub pages fetched on 2026-10-11.
- **Feature claims:** from official docs, blogs and READMEs.
- **Labels:** "(unv.)" marks a claim I could not confirm from a primary source. "(sec.)" marks a claim from a secondary source only.

**Status key:**

| Status | Meaning |
|---|---|
| Active | A release or commits in the last 3 months |
| Slow | A last release 4–12 months ago, or commits only |
| Stale | No release for more than 12 months |
| Dead | Archived, retired or discontinued |

---

## Group A — Natural-language → shell command tools

| Product | What | NL→shell | Local models | License | Existing terminal | Built-in safety | Status | Source URL | Source date |
|---|---|---|---|---|---|---|---|---|---|
| **Atuin AI** (in Atuin) | Shell-history tool (Rust). Press `?` on an empty prompt and describe the task. Enter runs the command, Tab edits it, `f` asks follow-up questions. Opt-in. | Yes | Partial. The default uses Atuin Hub with cloud "frontier models". It needs an account and is "currently free". Since July 2026 you can self-host `atuin-ai-server` with Ollama, vLLM, LM Studio, llama.cpp or LiteLLM. | MIT (client); Apache-2.0 (atuin-ai-server) | Yes (zsh, bash, fish) | **Hybrid.** The LLM scores danger and confidence. The blog also names static checks, and the docs say Hub checks for patterns the LLM missed. A flagged command needs Enter twice. | Active: v18.23.0, 2026-09-21 | https://docs.atuin.sh/latest/ai/introduction/ · https://blog.atuin.sh/atuin-v18-13 · https://blog.atuin.sh/atuin-ai-oss/ | 2026-03-18 (launch); 2026-07-14 (OSS server) |
| **Shell-GPT (sgpt)** | Python LLM CLI. `--shell` makes an OS- and shell-aware command. A Ctrl+L hotkey works in bash and zsh. | Yes | Partial. Ollama via LiteLLM. README: "not optimized for local models". | MIT | Yes | A prompt only: `[E]xecute, [D]escribe, [A]bort`. `--no-interaction` skips it. No danger detection. | Active (slow): 1.5.1, 2026-05-06; last commit 2026-07-02 | https://github.com/TheR1D/shell_gpt | 2026-05-06 |
| **AIChat** | All-in-one Rust LLM CLI with a "Shell Assistant" (`-e`), REPL, RAG and agents | Yes | Yes (Ollama and OpenAI-compatible) | MIT OR Apache-2.0 | Yes | Execute menu exists; no danger detection found (unv.) | Stale: v0.30.0, 2025-07-07; last commit 2026-02-23 | https://github.com/sigoden/aichat | 2025-07-07 |
| **AI Shell** (Builder.io) | Node CLI that explains the script, then asks to run it | Yes | Custom OpenAI endpoint only; local not documented | MIT | Yes | Run / Revise / Cancel prompt. `-s` skips the explanation. | Stale: 1.0.12, 2024-10-01; npm 279 downloads per 30 days | https://github.com/BuilderIO/ai-shell | 2024-10-01 |
| **Butterfish** | Go wrapper around your shell. Capital-letter prompts, autosuggest, `!` agent mode, `@` single command. | Yes | Any OpenAI-compatible endpoint (LM Studio named). Prompts are tuned for OpenAI. | MIT | Yes (wraps the shell) | You review each command. The `!!` and `@@` modes skip review ("USE WITH CAUTION"). | Active (slow): v0.4.3, 2026-06-03 | https://github.com/bakks/butterfish | 2026-06-03 |
| **`llm cmd`** (Simon Willison's `llm` + `llm-cmd` plugin) | Puts a generated command in the prompt. You edit it or press Enter. | Yes | Yes, through `llm` model plugins (unv. for llm-cmd) | Apache-2.0 | Yes | Edit before Enter. README: "This command could be **very dangerous**". | `llm` Active (0.36, 2026-09-22; 487,919 PyPI downloads per month). `llm-cmd` Stale (0.2a0, 2024-09-09). | https://github.com/simonw/llm-cmd | 2024-09-09 |
| **zsh-ai** | zsh plugin. Type `# description` and the command appears in the buffer. | Yes | Yes: Ollama, and Apfel (Apple on-device model) | MIT | Yes (zsh) | No auto-run. No danger checks mentioned. | Active: v0.11.4, 2026-09-30 (223 stars) | https://github.com/matheusml/zsh-ai | 2026-09-30 |
| **fish-ai** | fish plugin. `#` comment + Ctrl+P gives a command. Also explains commands and fixes failed ones. | Yes | Yes: self-hosted OpenAI-compatible; Ollama recommended | MIT | Yes (fish) | Redacts passwords, API keys, PEM keys and bearer tokens before sending. No danger checks mentioned. | Active: v2.17.0, 2026-09-24 (562 stars) | https://github.com/Realiserad/fish-ai | 2026-09-24 |
| **ShellOracle** | Ctrl+F widget for bash, zsh and fish | Yes | Yes: Ollama, LocalAI, LM Studio, llmman | GPL-3.0 | Yes | None mentioned | Active: v1.12.0, 2026-09-12 (353 stars) | https://github.com/djcopley/ShellOracle | 2026-09-12 |
| **tlm** | "Local CLI Copilot, powered by Ollama". Suggest and explain. | Yes | Yes (Ollama only; no API key) | Apache-2.0 | Yes | None mentioned | Stale: last tag 1.2-pre, 2025-01-31; last commit 2026-02-22 (1.5k stars) | https://github.com/yusufcanb/tlm | 2026-02-22 |
| **zev** | Natural language → a short list of commands to choose from | Yes | Yes (Ollama) | MIT | Yes | **Model-based:** "attempts to flag dangerous commands" | Slow: v0.11.1, 2026-04-21 | https://github.com/dtnewman/zev | 2026-04-21 |
| **nlsh** (abakermi) | NL→command | Yes | Yes (OpenAI-compatible `base_url`, e.g. Ollama) | MIT | Yes | **Deterministic:** a confirm step plus allow and deny wildcard lists (`rm -rf /*`, `mkfs.*`) | Slow: v1.5.0, 2026-01-31 (18 stars) | https://github.com/abakermi/nlsh | 2026-01-31 |
| **Gnix** | "Natural language to shell commands. Local. Private. Free." | Yes | Yes (Ollama only) | Apache-2.0 | Yes | **Pattern-based** safety layer. Dangerous commands need an explicit confirm. | New and very small: 0.1.1, 2026-05-02 | https://pypi.org/project/gnix/ | 2026-05-02 |
| **kubectl-ai** (Google) | Natural language → kubectl, as a Kubernetes agent | Partial (k8s only) | Yes: Ollama, llama.cpp | Apache-2.0 | Yes | Asks before resource-changing commands (`skipPermissions: false` by default) | Slow: v0.0.31, 2026-03-24; commit 2026-07-15 (7.6k stars) | https://github.com/GoogleCloudPlatform/kubectl-ai | 2026-03-24 |
| **TmuxAI** | tmux "pair programmer". Reads your panes and runs commands in a separate pane. | Partial | unv. | Apache-2.0 | Yes (tmux) | Runs commands "with your permission" | Active: v2.4.0, 2026-09-28 | https://github.com/alvinunreal/tmuxai | 2026-09-28 |
| **ShellSage** (Answer.AI) | tmux-aware helper that reads terminal history | Partial | Yes (Ollama) | Apache-2.0 | Yes (tmux) | None mentioned | Active: 1.1.3, 2026-09-09 | https://github.com/AnswerDotAI/shell_sage | 2026-09-09 |
| **Kiro CLI `translate`** (was `q translate`) | NL→bash subcommand. Kiro CLI already has a Caro compare page. | Yes | No (unv.) | Proprietary | Yes | See the existing page | Active (Homebrew cask 2.29.0) | https://docs.aws.amazon.com/cloudshell/latest/userguide/q-cli-features-in-cloudshell.html | accessed 2026-10-11 |
| **Amazon Q Developer CLI** | Open-source predecessor of Kiro CLI | Yes (`q translate`) | No | Apache-2.0 / MIT | Yes | — | **Dead (maintenance only).** README: "no longer being actively maintained and will only receive critical security fixes". v1.19.7, 2025-11-15. | https://github.com/aws/amazon-q-developer-cli | 2025-11-15 |
| **Fig** | Terminal autocomplete with AI; acquired by AWS | Was yes | No | MIT (autocomplete specs) | Yes | — | **Dead.** Folded into Amazon Q, then Kiro CLI. Spec repo last commit 2025-05-05. One secondary source says full shutdown on 2025-02-19 (sec.). | https://github.com/withfig/autocomplete | 2025-05-05 |
| **`gh copilot suggest`** | GitHub CLI extension | Was yes | No | Proprietary | Yes | — | **Dead.** Retired on 2025-10-25 and replaced by Copilot CLI, which has no direct `suggest` equivalent. | https://github.com/github/gh-copilot · https://github.blog/changelog/2025-09-25-upcoming-deprecation-of-gh-copilot-cli-extension/ | 2025-09-25 |
| **Mods** (Charm) | Pipe-friendly LLM CLI | Partial | Yes (LocalAI) | MIT | Yes | — | **Dead.** Archived 2026-03-09; README points to `crush run`. Last release v1.8.1, 2025-07-10. | https://github.com/charmbracelet/mods | 2026-03-09 |
| **Microsoft AI Shell (`aish`)** | AI side pane for PowerShell | Partial (PowerShell) | No (unv.) | MIT | Yes (Windows Terminal split pane) | — | **Dead.** README: "As of January 2026, the AI Shell project is no longer being actively maintained". Last preview.7, 2025-09-05 (sec.). | https://github.com/PowerShell/AIShell | 2026-01 |
| **Gorilla CLI** | Candidate commands from the hosted Gorilla API | Yes | No (hosted) | Apache-2.0 | Yes | "Commands are executed solely with your explicit approval" | Dead: 0.0.10, 2023-08-30 | https://github.com/gorilla-llm/gorilla-cli | 2023-08-30 |
| **howdoi** | Answers scraped from Stack Overflow (not an LLM) | No | n/a | MIT | Yes | — | Stale: 2.0.20, 2022-10-03 | https://github.com/gleitz/howdoi | 2022-10-03 |
| **tldr (tlrc client)** | Community cheat sheets | No (no AI found) | n/a | MIT | Yes | — | Active: v1.13.1, 2026-05-22 | https://github.com/tldr-pages/tlrc | 2026-05-22 |
| **Termium** (Codeium/Windsurf) | Prototype terminal autocomplete | Partial | No | Proprietary | Yes | — | **Unverified.** No 2025–2026 activity found; probably abandoned. | https://devin.ai/blog/termium-codeium-in-terminal-launch | undated |
| **uwu** | NL→command; edit before Enter | Yes | unv. | MIT | Yes | Edit before Enter | Stale: last commit 2025-08-23, no tags | https://github.com/context-labs/uwu | 2025-08-23 |
| **shell-ai (`shai`)** | NL→command | Yes | Yes (Ollama) | MIT | Yes | Confirm step; `SHAI_SKIP_CONFIRM` turns it off | Stale: 0.4.4, 2025-08-27 | https://github.com/ricklamers/shell-ai | 2025-08-27 |
| **please-cli** (TNG) | NL→command bash script | Yes | unv. | Apache-2.0 | Yes | unv. | Stale: v0.4.3, 2025-05-05 | https://github.com/TNG/please-cli | 2025-05-05 |
| **yai** | NL→command TUI | Yes | No | MIT | Yes | — | Dead: 0.6.0, 2023-04-25 | https://github.com/ekkinox/yai | 2023-04-25 |
| **Windows Terminal Chat** | Experimental chat pane in Windows Terminal Canary (GitHub Copilot, Azure OpenAI, OpenAI) | Partial | No | MIT (Windows Terminal) | Yes (Windows Terminal) | — | **Experimental.** One secondary report says it is deprecated in Canary in favor of Intelligent Terminal (unv.). | https://learn.microsoft.com/windows/terminal/terminal-chat | accessed 2026-10-11 |
| **macOS Terminal.app** | — | **No built-in AI found.** Only an unsourced third-party guide claims it. | — | — | — | — | Not applicable | — (no reliable source) | — |

## Group B — AI-native terminals and terminal emulators with AI

| Product | What | NL→shell | Local models | License | Existing terminal | Built-in safety | Status | Source URL | Source date |
|---|---|---|---|---|---|---|---|---|---|
| **Warp** (existing page) | Rust AI terminal / "agentic development environment" with Oz cloud agents | Yes | No in mainline (unv.). The community fork OpenWarp adds OpenAI-compatible endpoints and Ollama (sec.). | **AGPL-3.0 client + MIT UI crates since ~2026-04-28.** The server stays closed. | No (it replaces your terminal) | See the existing page | Active: cask 0.2026.09.30 stable; commit 2026-10-10 | https://www.warp.dev/blog/warp-is-now-open-source · https://github.com/warpdotdev/Warp | ~2026-04-28 (sources differ: 27–30 Apr) |
| **iTerm2** | macOS terminal. AI command composer ("Engage Artificial Intelligence", since 3.5). AI Chat can act on sessions (3.6). 3.7.0 adds orchestration chats. | Yes | Yes. 3.7.0 adds presets for Ollama and OpenRouter, and LAN-hosted HTTP models. | GPL-2.0-or-later | It is the terminal (macOS only) | **Model-based, opt-in** command safety check that uses your configured model and can send commands to a cloud provider. It holds risky commands for approval. Never / Ask / Always permissions. | Active: 3.7.4, built 2026-10-08 (12,667 Homebrew installs per 30 days) | https://iterm2.com/downloads.html · https://iterm2.com/documentation-ai-chat.html | 2026-10-08 |
| **Wave Terminal** | Open-source AI-native terminal. Wave AI reads scrollback and edits files; `wsh ai` pipes input to it. | Partial (command execution is "Coming Soon") | Yes: Ollama, LM Studio, OpenAI-compatible, BYOK | Apache-2.0 | No (it replaces your terminal) | File operations need approval and make automatic backups | Active: v0.14.5, 2026-04-16; commit 2026-10-05 (22.5k stars) | https://github.com/wavetermdev/waveterm | 2026-04-16 |
| **Ghostty** | Fast native terminal (Zig) with the reusable libghostty | No built-in AI found | n/a | MIT | It is the terminal | — | Active: v1.3.1, 2026-03-13; commit 2026-10-10 (19,658 Homebrew installs per 30 days) | https://github.com/ghostty-org/ghostty | 2026-03-13 |
| **cmux** (2026 entrant) | Ghostty-based macOS terminal built to run many coding agents: vertical tabs, notifications, browser API | No (it hosts agents) | n/a (agents bring their own) | GPL-3.0-or-later client; BUSL-1.1 server | It is the terminal | Resume commands auto-run only if trusted. Secret env keys are dropped. | Active: commit 2026-10-10; cask 0.65.0 (28.1k stars; 3,625 Homebrew installs per 30 days) | https://github.com/manaflow-ai/cmux | 2026-10-10 |
| **Microsoft Intelligent Terminal** (2026 entrant) | Experimental open-source fork of Windows Terminal. It has an agent pane over ACP (Copilot CLI by default; OpenCode added in 0.2) and detects command errors automatically. Announced at Build 2026. | Partial (through the agent) | Yes: 0.2 adds bring-your-own-model with base URL + model ID | MIT | It is the terminal (Windows) | unv. (the agent's own approvals apply) | Active: v0.2.2572, 2026-09-14 | https://devblogs.microsoft.com/commandline/intelligent-terminal-0-2-is-here-with-local-model-support/ · https://github.com/microsoft/intelligent-terminal | 2026-08-10 |
| **Windows Terminal** | Classic Windows Terminal. AI stays only in Canary (Terminal Chat). | Partial (Canary only) | No | MIT | — | — | Active: v1.26.2734.0, 2026-09-30 | https://github.com/microsoft/terminal | 2026-09-30 |
| **Zed** (terminal panel) | Editor with a built-in terminal. The Inline Assistant opens in the terminal panel with ctrl-enter. | Yes | Yes ("local models" via Zed providers) | GPL-3.0 + Apache-2.0 (multi-license repo) | No (inside Zed) | No confirm step documented | Active: v1.24.1-pre, 2026-10-07 (5,792 Homebrew installs per 30 days) | https://zed.dev/docs/ai/inline-assistant | accessed 2026-10-11 |
| **Termius** | Cross-platform SSH client. AI command generation inside autocomplete (beta). AI Agent (closed beta). | Yes | **No:** "the conversion ... happens in the cloud" | Proprietary | No (SSH client) | Agent: "will not execute any command on your hosts without your confirmation" | Active app (cask 10.1.3). The 2026 state of the AI features is unverified. | https://termius.com/blog/boost-your-terminal-experience-with-ai · https://termius.com/blog/ai-agent | 2023-06-26 · 2024-11-12 |
| **Chaterm** | Open-source AI terminal for cloud and DevOps work (Electron; agent derived from Cline) | Yes | "Custom models" (local unv.) | GPL-3.0 (+ Apache-2.0 Cline parts) | No | Operations are "auditable and traceable"; approval flow unv. | Active: v0.12.5, 2026-09-24 (3.1k stars) | https://github.com/chaterm/Chaterm | 2026-09-24 |
| **Tabby** | Terminal, SSH and serial client | No built-in AI. A community Tabby-MCP plugin exists (sec.). | n/a | MIT | It is the terminal | — | Active: v1.0.238, 2026-10-06 | https://github.com/Eugeny/tabby | 2026-10-06 |
| **Hyper** (Vercel) | Electron terminal | No | n/a | MIT | — | — | **Stale:** last release v3.4.1, 2023-01-08 (74 Homebrew installs per 30 days) | https://github.com/vercel/hyper | 2023-01-08 |
| **Kitty** | GPU terminal | No built-in AI found | n/a | GPL-3.0 | — | — | Active: v0.49.2, 2026-10-01 | https://github.com/kovidgoyal/kitty | 2026-10-01 |
| **WezTerm** | GPU terminal with Lua config | No built-in AI found | n/a | MIT | — | — | Slow: last stable 20240203; commit 2026-10-10 | https://github.com/wezterm/wezterm | 2026-10-10 |
| **Tabnine** | IDE code assistant, not a terminal | Out of scope | — | — | — | — | — | — | — |

## Group C — Agentic coding CLIs that run shell commands (watchlist)

| Product | What | NL→shell | Local models | License | Existing terminal | Built-in safety | Status | Source URL | Source date |
|---|---|---|---|---|---|---|---|---|---|
| **Claude Code** | Anthropic's agentic coding CLI | Partial (agent) | Unofficial: Ollama ≥ 0.14 has an Anthropic-compatible API (2026-01-16) | Proprietary | Yes | Permission rules plus **auto mode**. Auto mode is a **model-based classifier** that checks each tool call. Launched 2026-03-24; default for Pro, Max and Team since 2026-08-14 (sec.). | Active: v2.1.296, 2026-10-09 (58.1M npm downloads per month) | https://github.com/anthropics/claude-code · https://registry.ollama.ai/blog/claude · https://letsdatascience.com/news/anthropic-makes-auto-mode-default-in-claude-code-60dcee63 | 2026-10-09 |
| **Codex CLI** | OpenAI's agentic coding CLI (Rust) | Partial | Yes: `--oss` with Ollama or LM Studio | Apache-2.0 | Yes | OS sandbox plus **deterministic Starlark `prefix_rule` rules** (allow / prompt / forbidden; the strictest match wins; compound commands are split). The docs describe **no shipped dangerous-command set**. | Active: 0.162.1, 2026-10-09 (98.5M npm downloads per month; 117,822 Homebrew installs per 30 days) | https://github.com/openai/codex · https://learn.chatgpt.com/docs/agent-configuration/rules | 2026-10-09 |
| **Gemini CLI** | Google's agentic CLI | Partial | No official support | Apache-2.0 | Yes | Approvals and sandbox (unv.) | **Retired for consumers.** It stopped serving AI Pro/Ultra and free users on 2026-06-18. Still served for paid API keys and enterprise. v0.63.0, 2026-10-06 (107.3k stars; npm fell to 1.77M per month). | https://developers.googleblog.com/an-important-update-transitioning-gemini-cli-to-antigravity-cli/ | 2026-05-19 |
| **Antigravity CLI** (2026) | Google's Go-based successor to Gemini CLI | Partial | unv. | Closed (the public repo is a feedback forum with no license) | Yes | unv. | Active: tag 1.3.3, 2026-10-10 | https://github.com/google-antigravity/antigravity-cli | 2026-10-10 |
| **GitHub Copilot CLI** (existing page) | Agentic CLI that replaced `gh copilot` | Partial | **Yes since April 2026:** BYOK and local models (Ollama, vLLM, Foundry Local); `COPILOT_OFFLINE=true` | Proprietary ("GitHub Copilot CLI License") | Yes | Tool approval prompts (unv. detail) | Active: 1.0.95, 2026-10-09 (6.9M npm downloads per month) | https://github.blog/changelog/2026-04-07-copilot-cli-now-supports-byok-and-local-models/ | 2026-04-07 |
| **Kiro CLI** (existing page) | AWS agentic CLI (formerly Amazon Q Developer CLI) plus `translate` | Yes (`translate`) | No (unv.) | Proprietary | Yes | 2.26 asks before file writes in untrusted workspaces (sec.) | Active: cask 2.29.0 | https://formulae.brew.sh/api/cask/kiro-cli.json | accessed 2026-10-11 |
| **OpenCode** (existing page) | Open-source TUI agent | Partial | Yes | MIT | Yes | Permissions config (unv.) | Active: 1.19.0, 2026-10-11 (11.7M npm downloads per month; 22,040 Homebrew installs per 30 days) | https://github.com/anomalyco/opencode | 2026-10-11 |
| **Crush** (Charm) | Charm's TUI agent; `crush run` replaces Mods | Partial | Yes (Ollama and others) | FSL-1.1-MIT | Yes | Permission prompts (unv.) | Active: v0.98.1, 2026-10-09 | https://github.com/charmbracelet/crush | 2026-10-09 |
| **Goose** | General-purpose agent. Block donated it to the Linux Foundation's AAIF (Dec 2025). | Partial | Yes (Ollama) | Apache-2.0 | Yes | unv. | Active: v1.54.0, 2026-10-08 | https://github.com/aaif-goose/goose | 2026-10-08 |
| **Amp** | Agentic CLI; the rebuilt "Neo" CLI launched in May 2026 (sec.) | Partial | No (third-party listing) | Proprietary | Yes | Permissions exist (sec.) | Active: npm build 2026-10-11 | https://terminaltrove.com/compare/ai-coding-agents/amp-vs-codex-cli/ | accessed 2026-10-11 |
| **Aider** | Git-centric AI pair programmer | Partial | Yes (unv. here) | Apache-2.0 | Yes | Asks before shell commands (unv.) | **Slow:** 0.86.2, 2026-02-12; commit 2026-05-22 | https://github.com/Aider-AI/aider | 2026-02-12 |
| **Cline CLI** | The Cline agent in the terminal | Partial | Yes (Ollama, LM Studio) | Apache-2.0 | Yes | Human-in-the-loop approval | Active: npm `cline` 3.0.70, 2026-10-08 | https://github.com/cline/cline | 2026-10-08 |
| **Qwen Code** | Alibaba's agentic CLI | Partial | Yes (Ollama, vLLM) | Apache-2.0 | Yes | Auto mode and sandbox (README table) | Active: v0.25.0, 2026-10-05 | https://github.com/QwenLM/qwen-code | 2026-10-05 |
| **Factory Droid** | Enterprise agentic CLI | Partial | unv. | Proprietary (npm "UNLICENSED") | Yes | **Model-assigned risk** (low / medium / high) checked against your autonomy level, plus allow and deny lists | Active: @factory/cli 0.238.0, 2026-10-10 | https://docs.factory.ai/cli/user-guides/auto-run | accessed 2026-10-11 |
| **Cursor CLI** (`agent`) | Cursor's agentic CLI | Partial | No (unv.) | Proprietary | Yes | unv. | Active: builds until at least 2026-08-11 (sec. tracker) | https://cursor.com/docs/cli/changelog.md | 2026-08-11 |
| **Warp agents (Oz)** | Warp's cloud agent platform | Partial | No (unv.) | Client AGPL-3.0; Oz closed | No | — | Active | https://www.warp.dev/blog/warp-is-now-open-source | ~2026-04-28 |
| **Grok Build** (xAI, 2026) | xAI's official terminal coding agent | Partial | No (xAI API) | Proprietary (unv.) | Yes | Plan review before changes (sec.) | Beta May 2026; 1.0 in Aug 2026 (sec.) | https://runtimewire.com/article/xai-grok-build-agentic-cli-early-beta | 2026-05 |
| **grok-cli** (Superagent; community) | Community agent for the Grok API ("not affiliated with ... xAI") | Partial | No | MIT | Yes | Sandbox for `--verify` | Slow: grok-dev@1.1.7, 2026-05-15 | https://github.com/superagent-ai/grok-cli | 2026-05-15 |
| **Mistral Vibe** | Mistral's agentic CLI (Devstral) | Partial | Yes (any OpenAI-compatible local server; vLLM example) | Apache-2.0 | Yes | Tool-execution approval; `--yolo` skips it | Active: v2.26.1, 2026-10-09 | https://github.com/mistralai/mistral-vibe · https://docs.mistral.ai/vibe/code/cli/offline-models | 2026-10-09 |
| **Kimi CLI → Kimi Code CLI** | Moonshot's agent | Partial | unv. | Kimi CLI Apache-2.0; Kimi Code MIT | Yes | unv. | **Kimi CLI is archived** ("no longer maintained", installs "will stop working"). Kimi Code is active: 2.1.1, 2026-09-24. | https://github.com/MoonshotAI/kimi-cli · https://github.com/MoonshotAI/kimi-code | 2026-09-24 |
| **Kilo CLI** | Kilo Code's CLI | Partial | unv. | MIT | Yes | unv. | Active: 7.8.8, 2026-10-07 | https://github.com/Kilo-Org/kilocode | 2026-10-07 |
| **Junie CLI** (JetBrains, 2026) | LLM-agnostic standalone agent | Partial | BYOK (local unv.) | Proprietary | Yes | unv. | Beta in Mar 2026; GA in Jun 2026 (sec.) | https://blog.jetbrains.com/junie/2026/03/junie-cli-the-llm-agnostic-coding-agent-is-now-in-beta/ | 2026-03 |
| **OpenClaw** (2026 viral) | Self-hosted personal agent that you drive from chat apps; runs shell commands | Partial | unv. | MIT | No (chat apps) | Limited built-in restrictions; sandboxing is manual (sec.) | Active: v2026.10.1, 2026-10-10 | https://github.com/openclaw/openclaw · https://www.digitalocean.com/resources/articles/openclaw-security-challenges | 2026-10-10 |
| **Pi** (Mario Zechner) | Minimal coding agent | Partial | unv. | MIT | Yes | **None:** "does not include a built-in permission system" | Active: v1.1.0, 2026-10-07 | https://github.com/badlogic/pi-mono | 2026-10-07 |
| **Hermes Agent** (Nous Research) | Agent with local, Docker and SSH terminal backends | Partial | unv. | MIT | Yes | Command approval, allowlist patterns, container isolation | Active: v2026.9.24 | https://github.com/NousResearch/hermes-agent | 2026-09-24 |
| **Letta Code** | Memory-first coding agent | Partial | unv. | Apache-2.0 | Yes | unv. | Active: v0.34.10, 2026-10-09 | https://github.com/letta-ai/letta-code | 2026-10-09 |
| **OpenHands CLI** | OpenHands agent | Partial | unv. | MIT | Yes | unv. | Active: v1.26.0, 2026-10-08 | https://github.com/OpenHands/OpenHands | 2026-10-08 |
| **Auggie** (Augment) | Augment's CLI | Partial | unv. | Proprietary | Yes | unv. | Slow: 0.36.0, 2026-08-21 | https://github.com/augmentcode/auggie | 2026-08-21 |
| **Continue `cn`** | Continue's CLI | Partial | unv. | Apache-2.0 | Yes | unv. | Slow: 1.5.47, 2026-06-18 | https://github.com/continuedev/continue | 2026-06-18 |
| **Plandex** | Agent for large planned tasks | Partial | unv. | MIT | Yes | — | **Stale:** last commit 2025-10-03 | https://github.com/plandex-ai/plandex | 2025-10-03 |

---

## Audience signals used for ranking

| Product | Homebrew installs (30 days) | Monthly registry downloads | GitHub stars (page, 2026-10-11) |
|---|---|---|---|
| Codex CLI | 117,822 (cask) | 98.5M npm | — |
| Claude Code | 40,236 (cask) | 58.1M npm | — |
| OpenCode | 22,040 | 11.7M npm | — |
| Ghostty | 19,658 | — | — |
| iTerm2 | 12,667 | — | — (mirror repo) |
| **Atuin** | **8,964** | — | **32.0k** |
| Copilot CLI | — | 6.9M npm | — |
| Zed | 5,792 | — | — |
| Warp | 3,741 | — | — |
| cmux | 3,625 | — | 28.1k |
| Gemini CLI | 3,538 (formula still at 0.46.0) | 1.77M npm | 107.3k |
| `llm` | 1,307 | 487,919 PyPI | — |
| Kiro CLI | 1,059 | — | — |
| Wave | 337 | — | 22.5k |
| AIChat | 141 | — | 10.5k |
| Shell-GPT | — (no formula) | 10,053 PyPI | 12.3k |
| AI Shell (Builder.io) | — | 279 npm | 5.3k |

Sources: `https://formulae.brew.sh/api/{formula,cask}/<name>.json`, `https://api.npmjs.org/downloads/point/last-month/<pkg>`, `https://pypistats.org/api/packages/<pkg>/recent`. All were fetched on 2026-10-11.

**Caveat:** Homebrew counts only Homebrew installs. npm counts include CI traffic. Use them for relative size only.

---

## Ranked top 6: new `/compare` pages

Criteria, in order: overlap with Caro's two jobs ("I forgot the command" and "is this command safe?"), audience size, and how sharp the differentiation is.

### 1. Atuin (Atuin AI): `/compare/atuin`
- **Why:** It is the closest match to Caro's job: `?` gives English→bash in zsh, bash and fish, and dangerous commands need Enter twice. It reaches the largest shell-power-user base in Group A (32.0k stars, 8,964 Homebrew installs in 30 days).
- **Differentiation:** Caro runs a local model by default with no account, and checks commands against 52+ deterministic patterns on the device. Atuin's default needs an Atuin Hub login and cloud models, and its danger flag is mainly an LLM score. Self-hosting needs a separate server.
- **Where Atuin wins:** Many users already have it installed, with history search and sync. Zero-setup frontier models may write better commands. The hybrid LLM + static flag can catch risks that depend on intent, which fixed patterns cannot express.

### 2. iTerm2 (AI Chat / "Engage AI"): `/compare/iterm2`
- **Why:** It covers both jobs: a command composer, and (since 3.7.0) a safety check before AI-run commands. It has a large macOS install base (12,667 Homebrew installs in 30 days; 308,971 in a year).
- **Differentiation:** iTerm2's safety check is model-based and opt-in, and it can send commands to a cloud provider. Caro's check is deterministic and offline, and it works in any terminal, including Linux.
- **Where iTerm2 wins:** The AI lives in the terminal itself. Session-aware chats can watch, annotate and operate panes, with no extra binary. It also supports local models through Ollama presets.

### 3. Shell-GPT (sgpt): `/compare/shell-gpt`
- **Why:** It is the best-known NL→shell CLI (12.3k stars, about 10k PyPI downloads per month) and still releases (1.5.1, May 2026). Its only gate is an Execute / Describe / Abort prompt, and the README itself warns that the LLM "might execute destructive commands".
- **Differentiation:** Caro checks every command against 52+ patterns and ships an embedded local model. sgpt has no danger detection and says it is "not optimized for local models".
- **Where sgpt wins:** It does far more than commands: chat, REPL, roles, function calling, caching and `--describe-shell`. It reaches more providers through LiteLLM and has years of community guides.

### 4. Claude Code: `/compare/claude-code`
- **Why:** It is the largest agent audience after Codex (58.1M npm downloads per month), and its auto mode is a direct "is this command safe?" product. Many users also ask it for one-off shell commands.
- **Differentiation:** Auto mode uses a model-based classifier. Caro is deterministic, offline and auditable, and it can act as the pre-exec guard for an agent (Caro already ships the `caro-shell` skill).
- **Where Claude Code wins:** It does whole multi-step tasks. Its classifier judges intent and context ("the user asked to delete this directory"), which pattern matching cannot. Its plugin, skill and MCP ecosystem is much larger.

### 5. Codex CLI: `/compare/codex-cli`
- **Why:** It has the largest audience on any registry (98.5M npm downloads per month; 117,822 Homebrew installs in 30 days). It pairs an OS sandbox with **deterministic**, user-written command rules (Starlark allow / prompt / forbidden), so the safety comparison is concrete.
- **Differentiation:** Codex ships no curated dangerous-command set, so you write the rules yourself. Caro ships 52+ patterns ready to use, in a single-purpose NL→command UX for your own shell.
- **Where Codex wins:** Its OS-level sandbox contains dangerous commands that no pattern list predicts. It is also open source (Apache-2.0) with local models through `--oss`, so Caro cannot claim an open or local advantage here.

### 6. AIChat: `/compare/aichat`
- **Why:** It is Caro's closest architectural twin: a Rust single binary with a Shell Assistant, Ollama support and 10.5k stars. There has been no release for 15 months (v0.30.0, 2025-07-07), so its users have a real reason to look for a maintained alternative.
- **Differentiation:** Caro adds deterministic safety checks, an embedded model that needs no Ollama, and active releases. AIChat has no documented danger detection.
- **Where AIChat wins:** It is broader: more than 20 providers, RAG, agents, roles, and a built-in HTTP server and playground.

**Next candidates (not in the top 6):**
- **Wave Terminal:** an open-source Warp alternative (22.5k stars, local models). Overlap is weak until it ships command execution.
- **Microsoft Intelligent Terminal:** active, MIT, local models. Windows-first, so it fits Caro's POSIX focus less well.
- **"Gemini CLI alternative" page:** catches consumer users who lost access on 2026-06-18. The successor, Antigravity CLI, is closed source.
- **One "orphaned tools" roundup page:** for users of `gh copilot suggest`, Mods, Microsoft AI Shell, Builder.io AI Shell, Amazon Q CLI and Kimi CLI. All are retired or archived, and each user base is looking for a replacement.
- **zsh-ai + fish-ai:** exact job overlap with local models, but small audiences (223 and 562 stars).

---

## Drift on existing pages (separate fix)

1. **`website/src/pages/compare/github-copilot-cli.astro`:**
   - Lines 15 and 22 say Copilot CLI cannot work offline. It has supported BYOK and local models (Ollama, vLLM, Foundry Local) since April 2026, plus `COPILOT_OFFLINE=true`.
   - Line 61 says "GPT-4 class", which is stale.
   - The "Open source: partial" row is generous. The repo license is the proprietary "GitHub Copilot CLI License".
   - The page could also target "`gh copilot suggest` alternative" searches, because that command retired on 2025-10-25.
2. **`website/src/pages/compare/warp.astro`:** The Warp client has been AGPL-3.0 since about 2026-04-28. The page does not say Warp is closed, but it should now say that Warp is also copyleft open source. The "requires cloud for AI" claim seems still true for mainline Warp (unv.).
3. **Safety wording across all pages:** "Only Caro validates deterministically" is no longer safe to say.
   - Codex (Starlark rules) and Claude Code (permission rules) let users write deterministic allow and deny rules.
   - Atuin Hub runs static checks.
   - nlsh and Gnix ship small pattern lists.
   - A safe claim is: "Caro ships a curated, tested 52+ pattern set that runs offline before every command."

## Could not verify

- Termius AI status in 2026.
- Termium status.
- Warp local-model support.
- Local-model support for Cursor CLI, Amp, Kiro CLI, Factory Droid, Junie, Antigravity CLI, OpenClaw, Pi and Hermes Agent.
- Safety details for Gemini CLI, OpenCode and Crush.
- The deprecation of Windows Terminal Chat (secondary source only).
- Grok Build dates (secondary sources only).
- Any built-in AI in macOS Terminal.app: I found none.
