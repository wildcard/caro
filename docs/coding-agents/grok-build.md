# Grok Build (xAI)

**Vendor**: xAI | **Type**: Terminal coding agent (TUI, headless `grok -p`, ACP server)
**Source**: [github.com/xai-org/grok-build](https://github.com/xai-org/grok-build) (Apache-2.0)

## Overview

Grok Build is xAI's terminal coding agent, written in Rust. The model's shell
tool is `run_terminal_command`. Before every tool call, Grok Build runs:

1. `PreToolUse` hooks
2. permission rules (deny > ask > allow)
3. remembered grants
4. read-only auto-approvals
5. the mode policy (`default`, `auto`, `bypassPermissions`, …)

See [Lessons from Grok Build](../research/2026-10-03-grok-build-lessons.md)
for an architecture review.

## Integration with Caro

### Guardian hook (`caro guard`, experimental)

Grok Build reads hook files from `~/.grok/hooks/*.json`. It also reads
`~/.claude/settings.json` unchanged, so a Claude Code `caro guard` hook
already covers Grok Build. To install Caro for Grok Build only:

```json
// ~/.grok/hooks/caro-guard.json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [{ "type": "command", "command": "caro guard --harness grok", "timeout": 10 }]
      }
    ]
  }
}
```

`Bash` is Grok's matcher alias for `run_terminal_command`. Caro accepts Grok's
camelCase payload (`toolName`, `toolInput.command`, `sessionId`) as well as
Claude's snake_case.

| Mode | Effect |
|---|---|
| `shadow` (default) | Logs Caro's verdict to `<platform data dir>/caro/guard/decisions.jsonl` (on Linux `~/.local/share/caro/guard/`) and changes nothing. Run `caro guard report` to review. |
| `enforce` (`--mode enforce` / `CARO_GUARD_MODE=enforce`) | Critical → `deny`, High → `ask`, anything else → no opinion |

### Why the guard works this way

- **Grok hooks fail open.** A timeout (5 s by default), crash or malformed
  output is recorded and the tool still runs. `caro guard` uses static rules
  only, with no model and no network, so it answers well inside the timeout.
- **Caro never returns `allow`.** It returns `deny`, `ask`, or nothing.
- **Prefer user scope.** Project hooks need folder trust and live in the
  workspace the agent edits. User scope keeps the guard out of the workspace;
  it is not a security boundary against an agent with home-directory write
  access.

## Using Grok as Caro's model

Caro can also *use* Grok to generate commands. This needs a build with
`--features remote-backends`.

```bash
export XAI_API_KEY=...
caro --backend grok "find files larger than 100MB modified this week"
caro --advisor grok "..."           # escalate hard requests to Grok
caro --backend grok --model-name grok-4.5 "..."
```

You can also route the hybrid privacy gateway's remote enhancer to Grok, so
requests are sanitized locally before they reach xAI. Neither path sends
your local execution context (cwd, directory listing, knowledge entries) to
xAI; only the request itself goes out. Select the `hybrid` backend and point
it at Grok:

```toml
# Caro config file (Linux: ~/.config/caro/config.toml)
default_model = "hybrid"          # or pass --backend hybrid per call
[backends]
hybrid_remote = "grok"
# xai_url = "https://api.x.ai/v1"   # optional override
```

## Resources

- [Grok Build hooks guide](https://github.com/xai-org/grok-build/blob/main/crates/codegen/xai-grok-pager/docs/user-guide/10-hooks.md)
- [Grok Build permissions & safety](https://github.com/xai-org/grok-build/blob/main/crates/codegen/xai-grok-pager/docs/user-guide/22-permissions-and-safety.md)
- [Spec 011: Harness Guardian](../../specs/011-harness-guardian/spec.md)
- [ADR-019](../adr/ADR-019-guardian-hook-harness-adapters.md)
