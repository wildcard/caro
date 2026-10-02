# Routine Runner Contract

Every scheduled routine (Claude Code trigger) that works on this repo follows
this contract. It is modeled on google/ax's task status: a phase plus the
reason, written back by the runner rather than inferred from the platform.
See `docs/research/2026-09-24-google-ax-lessons.md`, Part 3.

**Why:** a trigger's run status (`SUCCEEDED`) only means its session ended.
The broken-window sweep's first two runs "succeeded" while doing nothing,
because the session had no repo or GitHub access. Status that nothing writes
is a declared guarantee nothing enforces.

## 1. Preflight first

Before any work, check that the run can do its job: a wildcard/caro checkout,
the GitHub tools it needs, and any required binaries. If preflight fails,
record `Blocked` with the missing piece as the reason, and stop.

If the failure is the very thing the recorder needs (no checkout, so no
`bin/routine-status`, or no push access), the record can't be written. Then:

- make the run's final message start with `ROUTINE BLOCKED: <reason>`, which
  shows in the session and in its push notification; and
- rely on staleness: no record arrives, so `show` marks the routine overdue
  once its `--max-age-hours` passes. A missing record is itself the signal,
  which is why every routine must set an honest max age.

## 2. Record exactly one status per run

The last thing a run does is append one record:

```bash
bin/routine-status record <routine> <phase> "<reason>" \
  [--started <iso8601>] [--pr <n>] [--issues <n,n,...>] \
  [--max-age-hours <n>]
```

| Phase | Meaning |
|---|---|
| `Succeeded` | The run did its job. The reason says what changed (issues filed, PR opened, "nothing to do") |
| `Failed` | The run tried and failed. The reason names the failing step |
| `TimedOut` | The run hit its max runtime before finishing |
| `Blocked` | Preflight failed; nothing was attempted |

- `<routine>` is a stable slug, such as `broken-window-sweep` or `qa-agent`.
  Keep it the same across runs; `show` groups by it.
- Pass `--started` with the time the run began, so the record shows duration.
- Pass `--max-age-hours` matching the routine's cadence, with some slack:
  `26` for daily, `170` for weekly, `5` for every four hours. `show` uses it
  to decide when the routine is overdue, so one command covers routines on
  different schedules.
- **Never record `Succeeded` on a failure path.** "Nothing to do" is a
  success only if the run checked and found nothing.

Records go to `runs.jsonl` on the `automation/routine-status` branch, never to
`main`, so writing one needs no PR.

## 3. Respect the max runtime

A routine's max runtime is its `timeout_minutes` in
`.claude/automation/config/schedule.yaml` when it has an entry there;
otherwise its prompt states one (default 30 minutes). When the run is near it,
it stops starting new work, records `TimedOut` with what is unfinished, and
exits.

## 4. Reading status

```bash
bin/routine-status show           # latest run per routine
bin/routine-status show --check   # exit 1 if any is not Succeeded or overdue;
                                  # exit 2 if the status branch can't be read
```

A `!` marks a routine whose latest run did not succeed or is overdue (older
than the `--max-age-hours` it recorded; 26 h if it recorded none). Hermes and
the sweep read this instead of the platform status.

## Regression guard

`tests/harness/routine-status.test.sh`, run in the ShellCheck workflow.
