# /caro slash commands

Comment `/caro <command> [args]` on any issue or PR to invoke an automated
workflow. Implemented by `.github/workflows/slash-router.yml`. Inspired by
[`warpdotdev/oz-for-oss`](https://github.com/warpdotdev/oz-for-oss)'s
`/oz-verify` pattern.

## Available commands

| Command | Description | Status |
|---|---|---|
| `/caro help` | Show the command list as a comment | ✅ available |
| `/caro echo <text>` | Echo `<text>` back (proves routing works end-to-end) | ✅ available |
| `/caro version` | Show caro version from Cargo.toml | ✅ available |
| `/caro review` | Trigger multi-agent code review | 🛠 planned |
| `/caro qa` | Run QA investigation against this PR | 🛠 planned |
| `/caro spec` | Generate a spec from the linked issue | 🛠 planned |
| `/caro tune-prompt` | Run prompt-tuner against test failures | 🛠 planned |
| `/caro release-acceptance` | Audit release readiness | 🛠 planned |

## Feedback signals

Every recognised command gets immediate feedback:
- 👀 **eyes** reaction on the original comment when the router accepts the
  command
- 👍 **+1** reaction when the handler succeeds
- A reply comment with the result (or an error if the handler failed)

Unknown commands fall through to `help` with an "Unknown command" hint.

## Adding a new command

1. **Whitelist** the command name in `slash-router.yml::parse::Parse command`
   (the bash `case` statement). Without this it falls through to
   `help-unknown`.
2. **Add a handler job** to `slash-router.yml`:
   ```yaml
   <name>:
     name: /caro <name>
     needs: parse
     if: needs.parse.outputs.command == '<name>'
     runs-on: ubuntu-latest
     steps:
       - # … your handler …
   ```
3. **Update this file's table** to document the command.
4. **Reuse existing skills**: prefer mapping the slash command to an existing
   `.claude/skills/<name>/SKILL.md` rather than implementing logic in the
   workflow. The handler should `gh workflow run`, `repository_dispatch`, or
   shell out to a runner script — keep YAML thin.

## Permissions

The router runs with `issues: write`, `pull-requests: write`, `contents: read`.
Handlers that need more (e.g. `actions: write` to dispatch other workflows)
should set `permissions:` at the **job** level, not workflow level — keep the
default surface small.

## Future: agent dispatch

Long-running handlers (review, QA, spec generation) should:
1. Acknowledge the command synchronously (post a comment "starting…")
2. Dispatch the actual work via `repository_dispatch` to a dedicated workflow
3. Report back when the dispatched run completes

This pattern keeps the slash router cheap (~30s per invocation) and makes
each handler independently retriable. See
[oz-for-oss `respond-to-pr-comment-local.yml`](https://github.com/warpdotdev/oz-for-oss/tree/main/.github/workflows)
for a reference implementation we can mirror.

## Auth

Currently uses `GITHUB_TOKEN` (the default workflow token). When we land
Pattern 5 (GitHub App auth), this will switch to `CARO_BOT_APP_KEY` so
comments and reactions are attributed to "caro-bot" instead of
"github-actions[bot]".
