# CaroML — A meta-language for intent-tracked shell tasks

CaroML is a small line-keyword DSL that lets you commit *intent* (what
you want a task to do, in plain English) and have Caro generate, validate,
and maintain the shell script that fulfills it. The same `.caro` file
produces different runbooks on macOS, Linux, and Windows; the lock keeps
A/B candidates and per-variant track records so the task evolves as
models, CVE feeds, and team preferences improve — without you rewriting
a single line.

## Why

A shell script committed today is wrong in three years. Flag conventions
churn (BSD ↔ GNU), CVE-aware idioms emerge, your team's preferred tools
change, your platforms evolve. The *intent* of the task is durable; the
*expression* is volatile.

CaroML inverts the persistence: you commit the intent, and Caro keeps
the expression in sync.

## Eight keywords, one file extension

A `.caro` file uses eight keywords. That's it.

| Keyword | Purpose |
|---|---|
| `TASK` | One-line title (exactly one) |
| `WHY` | Reason / motivation (optional) |
| `NEED` | A precondition (sudo, network, jq, ...) |
| `ON` | Platform-conditional pragma + optional `PREFER`/`AVOID` |
| `LET` | Authoring-time parameter, expanded as `{name}` in `DO` lines |
| `DO` | One natural-language intent — the unit of generation |
| `NOTE` | Free-form annotation; attaches to the next `DO` |
| `REM` | Comment, ignored everywhere |

## A small example

```text
TASK Clean up old log files
WHY  Free disk space; runs weekly via cron

NEED sudo
ON   macos PREFER bsd-tools
ON   linux PREFER gnu-tools

LET  path = /var/log
LET  days = 30

NOTE prefer single-pass find; avoid spawning a subshell per file
DO   find regular log files in {path}
DO   filter to those older than {days} days
DO   delete each one, asking confirmation per file
DO   record what was deleted to /tmp/caro-cleanup.log
```

## Files in a CaroML project

| File | Committed? | Purpose |
|---|---|---|
| `tasks/<name>.caro` | yes | The intent. Written by humans. |
| `tasks/<name>.caro.lock` | yes | Per-platform variants, A/B candidates, history. Written by Caro. |
| `tasks/<name>.<platform>.sh` | yes | The runbook a non-Caro user can `bash`. Written by Caro. |
| `Carofile` (or `Carofile.caro`) | optional | Project-level orchestration with `USE` / `JOB` / `RUN`. |
| `~/.caro/library/<name>.caro` | personal | Cross-machine library; project tasks shadow global by name. |

The lock and runbooks are **regenerated**, not edited by hand. Caro
detects manual edits to either via hash comparisons and offers to refresh.

## Workflow

```sh
caro new cleanup-logs                    # scaffold a starter .caro
$EDITOR tasks/cleanup-logs.caro          # write the intent

caro check tasks/cleanup-logs.caro       # parse + lint
caro generate cleanup-logs               # produce the lock + variants
caro export cleanup-logs                 # produce the .sh runbook
caro run cleanup-logs                    # plan-then-confirm execute

caro why cleanup-logs                    # explain RegenEvaluator decision
caro history cleanup-logs                # see lock lineage + run journal
caro experiment cleanup-logs             # add an A/B challenger
caro adopt cleanup-logs --variant <id>   # promote a challenger
caro render cleanup-logs                 # docs-grade Markdown
```

For Carofile-driven orchestration:

```sh
caro do ci          # JOB ci → run each RUN alias sequentially
caro do test        # USE alias → run external command directly
caro do cleanup-logs # falls back to `caro run cleanup-logs`
caro jobs            # list declared JOBs
```

## See also

- `grammar.md` — the full `.caro` grammar reference
- `lock-schema.md` — the `.caro.lock` format
- `carofile.md` — Carofile orchestration
- `validators.md` — the multi-angle validator framework
- `voice.md` — pager-era epilogue codes (143/371/...)
