---
name: caro-scaffold
description: Scaffold a new CaroML task file (.caro) from a natural-language description. Use this when the user wants to create a repeatable shell task that Caro will validate, generate, and maintain over time. Triggers on requests like "make a CaroML task that...", "scaffold a runbook for...", "create a .caro file for...", or invocations of `/caro new <name>`.
---

# CaroML Task Scaffolding

This skill helps you create a new `.caro` file (a CaroML task) and an
accompanying lock + runbook by walking through the user's intent in a
small number of focused steps.

## When to invoke

The user is asking for a **repeatable shell task** that they'd otherwise
script by hand and gradually drift away from. Examples:

- "Make a CaroML task that cleans up old log files weekly"
- "Scaffold a runbook to deploy the API to staging"
- "Create a .caro for the nightly snapshot job"

Caro's value proposition is that the *intent* (the what) stays stable
while the *generated script* can evolve — across model upgrades, new CVE
patterns, platform changes, or learned team preferences. The eight-keyword
DSL (TASK / WHY / NEED / ON / LET / DO / NOTE / REM) is meant to feel like
your first programming language: line-oriented, minimal syntax, BASIC-like.

## Workflow

1. **Clarify the intent.** Ask 1–3 short questions:
   - What's the one-line title? (becomes `TASK`)
   - Why does this task exist? When does it run? (becomes `WHY`)
   - Does it need sudo / network / a specific tool? (becomes `NEED`)
   - Target platforms? macOS, Linux, both? (becomes `ON`)

2. **Sketch the steps.** Each `DO` line is one natural-language intent.
   Keep them short and imperative. Order matters — later steps may use
   earlier results.

3. **Run `caro new <name> "<description>"`** if the user is in a shell
   with caro installed. Otherwise, write the file content yourself
   following the format below.

4. **Show the user the resulting `.caro`** before generating. Let them
   refine before committing.

5. **Run `caro check`, then `caro generate <name> --backend mock`** for
   a dry-run; show the resulting commands. If anything looks off,
   refine the intent (not the output).

6. **For real generation**, point the user at `caro generate` with their
   configured backend; for now `mock` is the only one wired through.

## File format reference

```
REM Optional comments anywhere; ignored at runtime
TASK <one-line title>
WHY  <reason; when does this run?>

NEED <prerequisite>      # zero or more
ON   <platform> [PREFER <a, b>] [AVOID <c, d>]    # zero or more

LET  <name> = <value>     # zero or more; use {name} in DO lines

NOTE <hint to the LLM>    # attaches to the next DO line
DO   <natural-language intent>
DO   <next intent>
```

Recognized platforms: `macos`, `linux`, `windows`, `posix`.

## Example session

```
USER: Make a CaroML task that deletes log files older than 30 days,
      with confirmation per file.

ASSISTANT: A few quick questions:
  1. Title for the task? (e.g., "Clean up old log files")
  2. Where do the logs live? (`/var/log` is the conventional spot)
  3. Target platforms? Both macOS and Linux?

USER: Yes, both. /var/log is fine. Title is "Cleanup logs".

ASSISTANT: Drafting tasks/cleanup-logs.caro:

  TASK Cleanup logs
  WHY  Free disk space; runs weekly via cron

  NEED sudo
  ON   macos PREFER bsd-tools
  ON   linux PREFER gnu-tools

  LET  path = /var/log
  LET  days = 30

  DO   find log files in {path}
  DO   filter to those older than {days} days
  DO   delete each, asking confirmation per file
  DO   record what was deleted to /tmp/caro-cleanup.log

Want me to commit this as-is, or refine anything first?
```

## Don'ts

- Don't over-constrain — leave the LLM room to choose a sensible idiom.
  `DO find all .log files older than 30 days and delete them` is fine;
  `DO run "find /var/log -mtime +30 -delete"` defeats the point.
- Don't put shell flags in the intent — that's the lock's job.
- Don't ship `.sh` runbooks as the primary deliverable. The committed
  artifact is the `.caro` + `.caro.lock`; runbooks are derived.

## Triggers

This skill should activate when the user says any of:
- "make a CaroML task"
- "scaffold a runbook"
- "create a .caro for ..."
- "/caro new"
- "I want a repeatable shell task"
