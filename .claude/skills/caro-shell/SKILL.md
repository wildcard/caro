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

## What caro provides

[`caro`](https://crates.io/crates/caro) is a Rust CLI that converts natural language → POSIX shell commands using local LLMs (MLX on Apple Silicon, Candle CPU elsewhere) or remote providers (Anthropic, Ollama, vLLM, Exo). Every generated command goes through 52+ dangerous-pattern safety regexes before being shown to the user. Risk levels: `CRITICAL`, `HIGH`, `MEDIUM`, `LOW`.

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
   | `--backend embedded` | Force the local model (privacy, no network) |
   | `--backend ollama --model qwen2.5-coder:7b` | Use a local Ollama server |
   | `--backend claude` | Use Anthropic API (`ANTHROPIC_API_KEY` must be set) |
   | `--explain` | Include a one-line rationale |
   | `--shell zsh` (or `bash`/`fish`) | Target a specific shell |

3. **Parse caro's output** — it prints the suggested command + a safety classification. If the classification is `CRITICAL` or `HIGH`, surface that prominently to the user before they decide.

4. **Present, do not execute** — show the user the command verbatim, the safety level, and any caveats. Let *them* decide whether to run it. If they ask you to run it, use the regular Bash tool with explicit confirmation per the standard destructive-command rules — do not bypass that just because caro classified it as `LOW`.

## Reply shape

Keep it tight:

```
**Command:**
```bash
<the command from caro>
```
**Safety:** <LOW|MEDIUM|HIGH|CRITICAL> — <one-line rationale if HIGH+>
**Notes:** <only if non-obvious caveats: BSD-vs-GNU, requires sudo, irreversible, etc.>
```

If `CRITICAL` or `HIGH`, lead with the safety line and ask the user explicitly to confirm before running.

If caro fails or returns empty, fall back to your own command synthesis — but say so explicitly: *"caro errored, here's a hand-written suggestion — please double-check."*

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
- **Never strip caro's safety classification.** If caro says `HIGH`, the user sees `HIGH`.
- **No `--execute` flag**, ever, from this skill.
- **One command per call.** If the user wants a multi-step pipeline that doesn't fit on one line, iterate — call caro once per step.

## Why this skill exists

Coder agents are great at writing code; shell command synthesis is its own discipline (BSD vs GNU, shell quoting, the difference between `find -delete` and `rm`). Caro is purpose-built for it, with safety regexes that catch fork bombs, recursive `chmod`, partition-targeted `dd`, and friends. Wiring it in as a skill means every Claude Code session that needs a shell command gets caro's safety net for free.

## See also

- Caro on crates.io: https://crates.io/crates/caro
- Caro homepage: https://caro.sh
- Source: https://github.com/wildcard/caro
- Sister surfaces (planned): `caro mcp serve` (Claude Code MCP server), `caro serve --openai` (OpenAI-compat HTTP shim for Codex/Cursor/Continue/Aider/Tabby)
