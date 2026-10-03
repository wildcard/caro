# Lessons from Grok Build for Caro

**Date**: 2026-10-03
**Source**: [xai-org/grok-build](https://github.com/xai-org/grok-build), HEAD 2026-09-29
("Synced from monorepo", `SOURCE_REV` 559751fd), Apache-2.0
**Status**: Research. The decisions it leads to are in
[ADR-018](../adr/ADR-018-guardian-hook-harness-adapters.md) and
[spec 011](../../specs/011-harness-guardian/spec.md).

Grok Build is xAI's terminal coding agent. It ships as a TUI binary called
`grok`, runs headless with `grok -p`, and can run as an Agent Client Protocol
server with `grok agent stdio|serve`. It is a Rust workspace of roughly 110
crates. The project is open source but does not accept contributions.

We read it to answer two questions:

1. What do we learn about how a modern agent harness decides whether a shell
   command may run?
2. What must Caro do to sit inside that harness, and inside Claude Code, Codex
   and OpenCode, as a general-purpose execution guardian?

Unless noted, the paths below are relative to the grok-build root, and `PG`
means `crates/codegen/xai-grok-pager/docs/user-guide/`.

---

## 1. Architecture in one paragraph

These crates matter to us:

| Crate | Role |
|---|---|
| `xai-grok-shell` | Agent runtime and entry points |
| `xai-grok-tools` | Tool implementations. Includes ports of OpenAI Codex and sst/opencode tools under `implementations/codex/` and `implementations/opencode/` |
| `xai-grok-workspace` | Files, VCS, execution, and the permission manager |
| `xai-grok-permission-rules` | Rule parsing and bash command splitting |
| `xai-grok-sandbox` | Landlock/bwrap/seccomp on Linux, Seatbelt on macOS |
| `xai-grok-hooks` | Lifecycle hooks |
| `xai-grok-mcp` | MCP client |
| `xai-grok-sampler` | LLM client: chat-completions, responses and messages wire formats |
| `xai-grok-compaction` | Context compaction |
| `xai-grok-telemetry` | Telemetry and OpenTelemetry export |

The model is shown a toolset *flavor*: `grok_build`, `codex` or `opencode`.
The same runtime can therefore present Codex-shaped or OpenCode-shaped tools
to a model trained on them.

**Lesson:** the harness is a commodity layer. The model, the tools and the
permission policy can each be swapped. Caro should plug into the permission
policy layer, the one place every harness converges on.

## 2. How Grok Build decides whether a command runs

### 2.1 The tool

The shell tool is called `run_terminal_command` (internal id
`grok_build.Bash`). Its input is:

- `command`
- `description`: a required one-sentence reason
- `timeout`: in ms, default 120 000, max 300 000
- `is_background`
- `block_until_ms`

Output is capped at 20 000 characters. Long foreground commands are moved to
the background automatically
(`crates/codegen/xai-grok-tools/src/implementations/grok_build/bash/mod.rs`).

**Lesson:** the required `description` field is a cheap, high-value signal. It
records the agent's stated intent next to the command. Caro's guard should log
it when a harness provides one; Claude Code sends `tool_input.description`.

### 2.2 The authorization pipeline

`PG/22-permissions-and-safety.md` defines a strict order:

1. `PreToolUse` hooks.
2. Rules, with severity `deny` > `ask` > `allow`.
3. Remembered per-project grants (`~/.grok/sessions/<scope>/permission.toml`).
4. Built-in read-only auto-approvals.
5. The prompt policy of the current mode: `default`, `acceptEdits`, `plan`,
   `auto`, `dontAsk` or `bypassPermissions`.

Admins can lock out always-approve with `disable_bypass_permissions_mode` in
`/etc/grok/requirements.toml`.

**Lesson:** hooks run **first**, before rules or grants. A Caro hook therefore
sees every shell call, even ones the user has pre-approved, which is the right
place for a guardian.

### 2.3 Rules over a parsed command, not over a string

The rule engine parses commands with **tree-sitter-bash**
(`crates/codegen/xai-grok-permission-rules/src/bash_command_splitting.rs`).

- It splits on `&&`, `||`, `;`, `|` and newlines.
- `deny` and `ask` rules match if **any** segment matches.
- `allow` is conjunctive: **every** segment must match.
- Environment assignments and wrappers (`timeout nice ionice chrt stdbuf env`)
  are stripped before matching. Rules also apply inside `bash -c '…'`.
- Subshells, `$(…)` and backgrounding are treated as an opaque unit that always
  prompts.

Some commands, listed in `crates/codegen/xai-grok-workspace/src/permission/grants.rs`,
never honor a remembered prefix grant: `rm chmod chown chgrp chattr pkill kill
killall` and `git push`.

**Caro gap:** `SafetyValidator::validate_command` (`src/safety/mod.rs`) scans
the whole string with regexes and a quote-parity heuristic, plus a
shell-unescaped re-scan for destructive commands. It does not split by
segment, so `allow` semantics cannot be conjunctive.

**Adopt later:** segment-aware validation, with deny on any segment and the
floor on any segment. This touches the safety core, so it goes through the
`safety-pattern-developer` TDD flow and needs a human safety owner (see the
follow-ups in spec 011).

### 2.4 Auto mode: heuristics first, then an LLM judge, and escalate on failure

`crates/codegen/xai-grok-workspace/src/permission/auto_mode/mod.rs` is about
4 000 lines. Its header reads "LLM transcript classifier with safe
fast-paths". The layers run in order:

1. **Fast path.** Allowlisted read-only tools pass directly, and file edits are
   accepted in this mode.
2. **Heuristic pre-pass.**
   - A substring denylist: `rm -rf /`, `curl|sh`, `/dev/tcp/`, `base64 -d`, …
   - "Hostile intent" phrases found in the transcript.
   - A fail-closed tree-sitter `classify_bash` with `ROUTINE_PREFIXES`.
   - Safe-subcommand allowlists per package manager (`NPM_SAFE_SUBCOMMANDS`, …).
   - Launcher unwrapping (`npm exec`, `uv run`).
   - Environment-variable risk checks (`LD_PRELOAD`, `DYLD_*`, `GIT_CONFIG*`).
3. **Typed findings.** `security_findings.rs` defines
   `ClassifierSecurityFinding` tokens: `OpaqueShell`, `EnvInjection`,
   `FileWrite`, `DangerousCommand`, … These are fixed enums that **never carry
   command text**, so a finding cannot be used as a prompt-injection channel.
   Any finding forces the LLM path.
4. **LLM judge.**
   - System prompt: `crates/codegen/xai-grok-workspace/templates/auto_mode_classifier_system_prompt.md`.
   - Output schema is strict JSON: `{thinking, shouldBlock, reason}`.
   - The amount of context it sees is configurable: `full`,
     `no_user_tool_prefix`, `bare_instructions` or `just_command`.
   - Only user turns recorded by the harness count as user intent. Untrusted
     transcript turns go last and are truncated to 400 characters.
5. **Failure handling.** A timeout, a transport error, or too many consecutive
   denials **escalates to a human prompt**. In headless mode the model is told
   "Auto mode blocked this action".

**Caro comparison:** this is structurally the same as `--approval smart`: a
static verdict, then `classify_risk` from the backend, then
`blend_smart_decision`, with a confidence floor and a hard Critical floor.

**What Caro already does better:** a deterministic catastrophic floor that the
judge can never relax (ADR-017).

**What Grok does better:**
- Typed findings instead of free text.
- Explicit escalation to a human when the judge fails.
- A configurable amount of context sent to the judge.

**Adopt:** typed, text-free reasons in guard output (now). An optional judge
for guard that escalates to `ask` on failure (later).

### 2.5 Sandbox

`PG/18-sandbox.md` and `crates/codegen/xai-grok-sandbox` describe it:

- The sandbox is process-wide and irreversible.
- Profiles: `off`, `workspace`, `devbox`, `read-only`, `strict`, plus custom
  profiles in `sandbox.toml`.
- Hook, config and trust files are write-denied at the kernel level.
- `[shell_environment_policy]` can strip `*KEY*`, `*SECRET*` and `*TOKEN*`
  variables from child processes.

**Lesson:** the guardian config must not be writable by the agent it guards.
For Caro guard this means:

- The decision log lives outside the workspace, under the user data dir.
- The docs tell users to install the hook at **user scope**
  (`~/.claude/settings.json`, `~/.grok/hooks/`, `~/.codex/hooks.json`), not at
  project scope, where the agent itself can edit it.

ADR-010, on a bubblewrap sandbox, stays the place for actual containment.

## 3. Hooks: the integration surface every harness now shares

`PG/10-hooks.md` and `crates/codegen/xai-grok-pager/docs/custom-hooks.md` are
the references.

| Property | Grok Build |
|---|---|
| Sources | `~/.grok/hooks/*.json`, `<proj>/.grok/hooks/*.json` (needs folder trust), `[[hooks.PreToolUse]]` in `config.toml`, managed and requirements TOML, plugins, **and `~/.claude/settings.json` / `<proj>/.claude/settings.json` read unchanged** |
| Events | SessionStart/End, UserPromptSubmit, PreToolUse, PostToolUse(+Failure), PermissionDenied, Stop(+Failure/Cancelled), Notification, SubagentStart/Stop, Pre/PostCompact |
| Handler | `{"type":"command"\|"http", "command"\|"url", "timeout", "env"}`; `matcher` is a regex over the tool name, and `Bash` is an alias for `run_terminal_command` |
| stdin (PreToolUse) | `{"hookEventName":"pre_tool_use","hook_event_name":"PreToolUse","sessionId","cwd","workspaceRoot","permissionMode","toolName":"run_terminal_command","toolInput":{"command":"…"},"toolUseId","toolInputTruncated","timestamp","promptId"}` |
| Output | `{"decision":"allow\|deny\|ask\|defer","reason"}` **or** `hookSpecificOutput.{permissionDecision, permissionDecisionReason}`; `permissionDecision` wins |
| Exit 2 | Deny, with stderr as the reason |
| Failure mode | **Fail open.** A timeout (5 s default), crash or malformed output is recorded but does not block. Only an explicit `deny` blocks. |
| `allow` | Means only "not blocked"; it does **not** auto-approve a call the user would otherwise be asked about |

### The cross-harness matrix

| | Claude Code | Grok Build | Codex CLI | OpenCode |
|---|---|---|---|---|
| Hook mechanism | `hooks.PreToolUse` in `settings.json` | Own JSON/TOML **plus** Claude's `settings.json` | `hooks.json` or `[[hooks.PreToolUse]]` in `config.toml` | JS/TS plugin, `tool.execute.before` |
| Shell tool name | `Bash` | `run_terminal_command` (matcher alias `Bash`) | `Bash` | `bash` |
| Command field | `tool_input.command` | `toolInput.command` (camelCase; `hook_event_name` is also sent) | `tool_input.command` | `output.args.command` |
| Block | `hookSpecificOutput.permissionDecision:"deny"` or exit 2 | Same, or `decision:"deny"` | Same | `throw new Error(reason)` |
| Ask the human | `permissionDecision:"ask"` | `ask` | `ask` | (no ask; we throw) |
| **`allow` semantics** | **Skips the permission prompt** | Only "not blocked" | Claude-compatible | n/a |
| `additionalContext` on PreToolUse | Allowed | Allowed | **Treated as an error, so the hook fails open** | n/a |
| Failure | Non-blocking error | Fail open | Fail open | Thrown error blocks |

Codex details are from the Codex advanced-configuration docs
(developers.openai.com/codex/config-advanced) as of 2026-10. OpenCode details
are from its plugin docs. Both should be re-checked when we add adapter tests
against real binaries.

### Four rules for Caro that follow from the matrix

1. **Never emit `allow`.** In Claude Code it would *bypass* the user's own
   permission prompt, so a guardian would lower safety. Caro emits `deny`,
   `ask`, or nothing.
2. **Hooks fail open, so Caro must be fast and explicit.** If Caro is slow and
   times out, the command runs anyway. The guard hot path is static rules only
   (no model load, no network), and a block is always an explicit JSON
   decision.
3. **One Claude-format adapter covers three harnesses.** It must accept both
   camelCase and snake_case keys and the `run_terminal_command` tool name. It
   must never put `additionalContext` on PreToolUse output, because Codex
   would fail open.
4. **The guard must not echo command text into its reason.** The reason goes
   back to the model, so only text Caro wrote (pattern descriptions, risk
   level) belongs there. This follows the typed-findings lesson in §2.4.

## 4. Provider layer: how Grok is reached

`PG/02-authentication.md`, `PG/11-custom-models.md` and
`crates/codegen/xai-grok-sampler/` describe it:

- **Base URL:** `https://api.x.ai/v1` by default. Override with
  `GROK_XAI_API_BASE_URL` or `endpoints.xai_api_base_url`.
- **Auth:** browser or device OAuth (`accounts.x.ai`), falling back to the
  `XAI_API_KEY` environment variable.
- **Wire formats:** `api_backend` can be `chat_completions` (the default),
  OpenAI `responses`, or Anthropic `messages`. Streaming uses SSE decoders in
  `xai-grok-sampler/src/stream/`.
- **Models:** the default is `grok-4.5`. The source also references grok-4,
  grok-4.3, grok-4.6, grok-4.7, grok-4.6-build and grok-build-plan.
- **Doom loops:** the server reports looping in the stream, and the client
  handles recovery in `doom_loop*.rs`.

**Caro gap:** there is no shared OpenAI-compatible client. `vllm.rs`,
`mesh.rs` and the uncompiled `openrouter.rs` each copy the same structs.

**Adopt now:** a shared `openai_compat` client. On top of it, `grok` becomes a
thin backend (`XAI_API_KEY`, `https://api.x.ai/v1`), and `openrouter` gets
compiled and CLI-wired at the same time.

## 5. Engineering practices worth copying

| Practice | Where | Caro status | Verdict |
|---|---|---|---|
| Session dir with an authoritative `updates.jsonl`, the rendered `system_prompt.txt`, `tool_definitions.json` and compaction checkpoints | `PG/17-sessions.md` | `caro ai` has no per-session transcript | Later: helps reproduce agent-loop bugs |
| Prompt templates as files rendered with minijinja | `templates/*.md` | Prompts live in Rust string constants (`src/prompts/`) | Later: makes prompt-tuner diffs reviewable |
| `PermissionClassifier` trait with `FixedClassifier` / `HeuristicPermissionClassifier` for tests | `auto_mode/` | `CommandGenerator::classify_risk` exists, but there is no fixed test double | Later, together with the guard judge |
| PTY harness: real screen via alacritty_terminal, a mock inference server, YAML scenarios, frame-time baselines | `crates/codegen/xai-grok-pager-pty-harness` | e2e tests use `assert_cmd` without a TTY | Later: the interactive confirm/edit flows are untested today |
| Scripted mock inference | `crates/codegen/xai-grok-test-support/src/inference_override.rs` | wiremock per backend | Already equivalent |
| A `wire_enum!` single source for telemetry vocabularies, plus a drift test | telemetry crates | Telemetry enums are hand-kept | Later |
| Recorded classifier provenance (`llm` / `heuristic` / `timeout` / `fast_path`) | auto mode | `SafetyDecision` has no source field | **Now:** guard records `source` |
| Adversarial goal verifier: "your job is to refute … default to `refuted: true` if uncertain" | `crates/codegen/xai-grok-shell/src/session/templates/goal_verifier_prompt.md` | The `devils-advocate` agent plays this role for specs | Already equivalent in process |
| Stop hooks that keep the agent working until a check passes, capped at 8 continuations | `PG/10-hooks.md` | n/a | Note for harness docs |

## 6. What we deliberately do not copy

- **A full TUI and harness.** Caro is the safety layer *inside* harnesses
  (see `docs/GUARDIAN_AGENT.md`), not another harness.
- **A process-wide sandbox inside Caro.** ADR-010 tracks containment
  separately. A guardian that also sandboxes would duplicate what every harness
  now ships.
- **An LLM judge on the default hook path.** It doesn't fit the latency
  budget, and since hooks fail open a slow judge is worse than none. It will be
  opt-in only.

## 7. Resulting work

- **Commit 1 (this PR):** this document, spec 011, ADR-018, and docs that
  match real CLI surfaces.
- **Commit 2 (this PR):** `caro guard`, a PreToolUse guardian for Claude Code,
  Grok Build, Codex and OpenCode. Shadow mode is the default. In enforce mode,
  Critical → deny, High → ask, and anything else → no opinion. A JSONL
  decision log plus `caro guard report`.
- **Commit 3 (this PR):** a shared OpenAI-compatible client, the `grok`
  backend (generator, advisor and hybrid remote), and `openrouter` and `claude`
  wired into `--backend`.
- **Follow-ups:** listed in spec 011 §"Follow-ups".
