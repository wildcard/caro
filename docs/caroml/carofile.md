# Carofile — project-level orchestration

A `Carofile` (or `Carofile.caro`) at the project root indexes CaroML tasks
alongside external runbook commands (Make targets, `package.json` scripts,
ad-hoc shell scripts), then composes them into higher-level **JOBs** that
`caro do <name>` runs.

Carofile is **not** a replacement for Makefile or `package.json` scripts;
it's an integrating layer that lets a contributor type `caro do ci` and
have Caro orchestrate whatever's actually there.

## Grammar

Reuses CaroML's line-keyword syntax with three new keywords (`USE`, `JOB`,
`RUN`) plus the inherited `TASK` / `WHY` / `REM`.

```text
TASK Project orchestration
WHY  Single front door for repeatable tasks; augments Makefile.

USE  tasks/cleanup-logs.caro       AS cleanup-logs
USE  "npm test"                     AS test
USE  "make build"                   AS build
USE  "cargo clippy --workspace"     AS lint

JOB ci
  RUN lint
  RUN test
  RUN build

JOB nightly
  RUN snapshot
  RUN cleanup-logs
```

## Resolution rules

`caro do <name>` resolves `<name>` in this order:

1. **JOB** — `JOB <name>` declared in the Carofile → run each `RUN` alias sequentially.
2. **External alias** — `USE "<command>" AS <name>` → execute the command via the `CommandExecutor` (safety still scans the resolved command).
3. **Native alias** — `USE <path>.caro AS <name>` → load the lock and execute via `caro run`.
4. **Fallback** — no match → treat `<name>` as a bare task name and run via `caro run <name>`.

## Body termination

A JOB body is closed by:
- the next top-level keyword (`TASK` / `WHY` / `USE` / `JOB`)
- end of file

Blank lines and `REM` comments inside a JOB body do **not** close it.
Indentation is purely cosmetic.

## v0.1 limitations

- JOBs run sequentially; no parallel composition syntax yet
- Job-level `NEED` is parsed but not enforced
- `USE … FROM <path>` clause from the design is reserved for v0.2
