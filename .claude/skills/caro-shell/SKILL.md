---
name: caro-shell
description: Use this skill when the user needs a POSIX shell command synthesized from natural language — "how do I find/grep/awk/find files modified in the last hour", "kill the process on port 3000", "tar this up excluding .git", or any other terminal-task-as-prose. Shells out to the `caro` CLI for safety-validated command inference and presents the suggestion for explicit approval. Refuses to execute the command itself.
---

# caro-shell — safe shell command inference via Caro

## When to use

Trigger this skill when the user asks for help expressing a terminal task as a shell command, especially anything where:

- The user describes the goal in prose (`"find python files larger than 1MB modified this week"`, `"rotate these logs and gzip the old ones"`).
- The command is non-trivial enough that getting it wrong has consequences (`rm`, `dd`, `chmod`, `find -exec`, anything piped to `sudo`).
- The user is on a system where you don't have full context (BSD vs GNU userland, shell flavor, locale).
- The user explicitly asks for a "safe" or "validated" version of a shell command.

Don't use this skill for:

- Programs/scripts that need to live in a file (write the file directly).
- Commands the user has already typed and just wants explained (`man` / `tldr` are better).
- Pure questions like "what does `awk` do?" (answer directly).
- Routine agent-internal shell work where the agent already knows the API surface: `git status`, `git add`, `git commit`, `gh pr view`, `bd create`, `npm install`, `cargo test`, `ls`, `cat`, `mkdir`, `find <known-pattern>`. Caro adds friction without value when the agent already has full context.
- Multi-line shell scripts (`bash -c '...'` blocks with newlines). Caro is single-line synthesis; for scripts, write the file.

## What caro provides

[`caro`](https://crates.io/crates/caro) is a Rust CLI that converts natural language → POSIX shell commands using local LLMs (MLX on Apple Silicon, Candle CPU elsewhere). The default `cargo install caro` build ships with the embedded backend only; remote providers (Ollama, vLLM, Exo) require a custom build with `--features remote-backends`. Every generated command goes through 52+ dangerous-pattern safety regexes before being shown to the user. Risk levels: `CRITICAL`, `HIGH`, `MEDIUM`, `LOW`.

## How to invoke

1. **Check it's installed**:

   ```bash
   command -v caro >/dev/null && caro --version
   ```

   If absent, surface the install snippet to the user and stop:

   ```bash
   cargo install caro
   # or, if cargo isn't available:
   curl -fsSL https://caro.sh/install.sh | sh
   ```

2. **Generate (always `--dry-run`)**:

   ```bash
   caro --dry-run "<the user's natural-language prompt>"
   ```

   Optional flags worth knowing:

   | Flag | When |
   |---|---|
   | (none — let caro pick default) | **Default for agents.** caro picks `embedded` (local model, zero-cost, deterministic). Right choice for autonomous loops. |
   | `--backend embedded` | Be explicit about the local-model path |
   | `--backend ollama --model qwen2.5-coder:7b` | Only if the user explicitly asks for a local Ollama server |
   | `--explain` | Include a rationale — useful when the user asked WHY, not just WHAT |
   | `--shell zsh` (or `bash`/`fish`) | Target a specific shell |

   **Do not** pass `--backend claude` or `--backend static` in this version: both are advertised by `caro --backend-info` but rejected by the `--backend` validator at v1.4.0. Tracking issue: caro-zh41. Do not pass `--quiet` either — it suppresses the safety information you need to surface.

3. **Detect synthesis failures before presenting** — caro's output today has three known failure modes that look like success ([audit 2026-05-16](../../audits/2026-05-16-caro-dogfood/audit.md), tracking issue caro-bnr6). Before presenting any command from caro, run these tripwires:

   - **Empty `Command:` block** → caro produced nothing. Treat as failure.
   - **`echo 'Unable to generate command'`** → caro's refusal path wrapped in `echo`. Treat as failure.
   - **Unsubstituted placeholders** (`PID`, `directory/`, `output.tar.gz`, `archive.tar.gz`, or any all-caps token that looks like a variable name) → caro returned a template, not a synthesized command. Treat as failure.

   On any tripwire: tell the user *"caro could not fully synthesize this; here's a hand-written suggestion, please double-check"* and fall back to your own command synthesis with the BSD-vs-GNU caveats explicit.

4. **Parse caro's output** — when caro successfully synthesizes, it prints the suggested command. **Risk tier is only printed when the safety gate fires (block / CRITICAL).** For commands that pass the gate, caro prints no tier today (tracking issue caro-b45s). Do not invent a tier. If the user asked for safety context, re-run with `--explain` or pattern-match the command yourself (`rm -rf`, `chmod -R`, `dd if=`, `curl ... | sh`).

5. **Present, do not execute** — show the user the command verbatim, the safety status from caro (or "no block; tier not surfaced" if caro stayed silent), and any caveats. Let *them* decide whether to run it. If they ask you to run it, use the regular Bash tool with explicit confirmation per the standard destructive-command rules — do not bypass that just because caro returned a non-error.

## Reply shape

Keep it tight:

```
**Command:**
```bash
<the command from caro>
```
**Safety:** <CRITICAL|HIGH|MEDIUM|LOW|"caro did not surface a tier — agent assessment: <X>">
**Notes:** <only if non-obvious caveats: BSD-vs-GNU, requires sudo, irreversible, etc.>
```

If `CRITICAL` or `HIGH`, lead with the safety line and ask the user explicitly to confirm before running.

If any tripwire from step 3 fires (empty / `echo 'Unable...'` / placeholder), do **not** print the command. Instead say: *"caro could not fully synthesize this; here's a hand-written suggestion — please double-check."* and write the command yourself.

If caro errors (exit non-zero, e.g. CRITICAL block fired), surface the error verbatim — the safety-gate text is the most useful thing in the whole pipeline.

## Examples

### Example 1 — low-risk

User: *"find all python files in src that haven't been touched in 6 months"*

```bash
caro --dry-run "find all python files in src that haven't been touched in 6 months"
```

Caro returns something like `find src -name '*.py' -type f -mtime +180` with safety `LOW`. Present it directly.

### Example 2 — high-risk needs confirmation

User: *"clean up the docker volumes I'm not using"*

```bash
caro --dry-run "clean up unused docker volumes"
```

Caro returns `docker volume prune -f` with safety `MEDIUM`. Present it, then explicitly ask: *"This will delete every volume not currently mounted by a running container. Run it?"*

### Example 3 — caro suggests a safer alternative

User: *"delete everything in /tmp"*

Caro will likely return either a refusal or a heavily caveated `find /tmp -mindepth 1 -maxdepth 1 ... -exec rm -rf {} +` with safety `HIGH`. Present caro's output as-is including the safety warning; do not "soften" it. Let the user choose.

## Constraints

- **Always `--dry-run`.** This skill never lets caro execute. The user (or you, via the regular Bash tool with confirmation) executes if they choose to.
- **Never strip caro's safety classification.** If caro says `CRITICAL`, the user sees `CRITICAL`.
- **No `--execute` flag**, ever, from this skill.
- **No `--quiet` flag**, ever — it suppresses the safety information.
- **No paid backends by default.** Do not pass `--backend claude` or recommend paid remote backends unless the user explicitly asks. The default (no flag) gives caro's embedded local model, which is free and deterministic — the right choice for an autonomous agent loop.
- **One command per call.** If the user wants a multi-step pipeline that doesn't fit on one line, iterate — call caro once per step.
- **Tripwire before trust.** Per step 3 of "How to invoke", scan caro's output for placeholder / empty / echo-refusal failure modes before presenting. These are known v1.4.0 bugs (caro-bnr6); the binary reports success even when synthesis failed, so the skill is the safety net.

## Agent contract by risk tier

When caro *does* return a tier (block path) or you assess one yourself:

| Tier | Agent action |
|---|---|
| `LOW` | Surface command + tier to user. May proceed via Bash with normal confirmation. |
| `MEDIUM` | Surface tier prominently. Recommend the user run the command themselves or via Bash with explicit confirmation. Suggest `--dry-run` (caro's, not the agent's) for any file-touching variant. |
| `HIGH` | Lead with the tier. Refuse to execute via Bash unless the user types verbatim approval. Suggest a safer alternative if one is obvious (`-i` interactive flag, `-n` non-clobber). |
| `CRITICAL` | **Hard refusal.** Show caro's block message. Do not execute even with user approval — ask the user to run it themselves in a separate terminal if they truly want to. |

## Why this skill exists

Coder agents are great at writing code; shell command synthesis is its own discipline (BSD vs GNU, shell quoting, the difference between `find -delete` and `rm`). Caro is purpose-built for it, with safety regexes that catch fork bombs, recursive `chmod`, partition-targeted `dd`, and friends. Wiring it in as a skill means every Claude Code session that needs a shell command gets caro's safety net for free.

## See also

- Caro on crates.io: https://crates.io/crates/caro
- Caro homepage: https://caro.sh
- Source: https://github.com/wildcard/caro
- Sister surfaces (planned): `caro mcp serve` (Claude Code MCP server), `caro serve --openai` (OpenAI-compat HTTP shim for Codex/Cursor/Continue/Aider/Tabby)
