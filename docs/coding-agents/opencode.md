# OpenCode

**Vendor**: sst / anomalyco | **Type**: Terminal coding agent
**Source**: [github.com/sst/opencode](https://github.com/sst/opencode)

## Overview

OpenCode is extended with JS/TS plugins. A plugin's `tool.execute.before`
hook runs before every tool call. It can block the call by throwing an Error.

## Integration with Caro

### Guardian plugin (`caro guard`, experimental)

Copy the plugin shipped with Caro into OpenCode's global plugin directory:

```bash
mkdir -p ~/.config/opencode/plugin
cp integrations/opencode/caro-guard.ts ~/.config/opencode/plugin/caro-guard.ts
```

The plugin sends `{"command": "...", "cwd": "..."}` to
`caro guard --harness opencode` without blocking OpenCode's event loop. The
plugin follows `CARO_GUARD_MODE`, and **shadow** is the default:

| Mode | Effect |
|---|---|
| `shadow` | Caro logs its verdict and the command runs. Review with `caro guard report`. |
| `enforce` | Caro exit 2 (Critical → deny) or exit 3 (High → needs approval) makes the plugin throw, which blocks the call with Caro's reason. |

Plugins can't open an interactive approval prompt. OpenCode is the
exception to "High → ask": in enforce mode a High command is blocked with
an explanation. Approve it by running it yourself, or by adding it to
`[safety] allowlist_patterns`.

If `caro` is not on `PATH`, the plugin lets the command through and prints a
warning, matching how hooks fail open in the other harnesses. If `caro guard`
does not finish within 10 seconds, enforce mode blocks the call ("command not
checked"); shadow mode only warns.

## Resources

- [OpenCode plugins](https://opencode.ai/docs/plugins/)
- [Spec 011: Harness Guardian](../../specs/011-harness-guardian/spec.md)
