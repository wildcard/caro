# Runner contract: what `caro run` and `caro do` guarantee

This page describes how CaroML executes a task. It records what the code does
today, so that tests can pin it and a change to it is a visible decision. The
idea comes from google/ax, which treats its runner's behavior as a spec
(`docs/research/2026-09-24-google-ax-lessons.md`, lesson 3).

Code: `src/caroml/runner.rs`. Contract tests: the `contract_*` tests in that
file (`cargo test --lib caroml::runner::tests::contract`). They cover the
step-by-step path only; the runbook path has no contract tests yet (#1557).

## Two execution paths

`caro run <task>` first prints the plan and asks `Proceed? [y/N]` (skip with
`--yes`; `--dry-run` stops after the plan). Then it picks one path:

| Path | When | How |
|---|---|---|
| Runbook | `<task>.<platform>.sh` exists and its hash matches the lock | `bash <runbook>`: all steps in **one** shell, with `set -euo pipefail` |
| Step-by-step | Any other case: no runbook, an edited runbook (drift), or a lock with no runbook hash stamp | Each step's active variant runs in its **own** shell |

`caro do <task>` and `RUN <alias>` in a Carofile always use the step-by-step
path.

## Guarantees (step-by-step path)

The contract tests run on Linux and macOS (`#[cfg(unix)]`). On Windows the
same code runs each step in PowerShell, but no contract test covers it yet.

| # | Guarantee | Pinned by |
|---|---|---|
| 1 | Steps run one at a time, in file order. Results come back in the same order. | `contract_steps_run_in_order_and_return_results_in_order` |
| 2 | Each step starts in a fresh shell (`bash -c` on Linux and macOS, PowerShell on Windows). Shell variables, `export`, and `cd` do not carry into the next step. | `contract_each_step_gets_a_fresh_shell` |
| 3 | Every step runs in the directory where you started caro. | `contract_each_step_gets_a_fresh_shell` |
| 4 | The run stops at the first step that exits non-zero. Later steps do not run. The error names the step's line, its intent, its exit code and its stderr. | `contract_stops_on_first_failure_and_reports_it` |
| 5 | Steps get no stdin, even when caro's own stdin has data: a step that reads input sees end-of-file at once. | `contract_steps_get_no_stdin` |

## Facts that are not guarantees yet

- **`LET` values are text, not variables.** `LET` is substituted into the
  `DO` intent at parse time ([grammar](grammar.md)). The generated command
  gets no environment variable for it.
- **The lock.** The runner itself never writes the lock: an execution failure
  is not a generation failure. A step's own command could still change or
  delete it. No test pins this yet, because the check needs a full `caro run`
  against a fixture lock.
- **Environment.** Each step inherits caro's environment unchanged. Caro adds
  nothing and removes nothing.
- **Exit code of `caro run`.** On any failure `caro run` exits `1`. It does not
  pass the step's own exit code through. The run journal records more: on the
  step-by-step path, the failed step's exit code and line; on the runbook
  path, the exit status of the whole `bash` run (with line 0).
- **Timeouts.** Steps have no time limit. `CommandExecutor` supports one, but
  the runner does not set it yet; that is blocked on signal forwarding (below).
  At the deadline the executor kills the step at once, so cleanup handlers do
  not run. On Unix it sends SIGKILL to the step's whole process group; on
  Windows it kills only the step's own process. #1545 proposes SIGTERM, a grace period,
  then SIGKILL.
- **Ctrl-C.** Without a timeout, a step runs in caro's own process group. A
  Ctrl-C at the terminal reaches both caro and the running step.
- **Idempotency.** Caro does not make steps safe to repeat. Re-running a task
  runs every step again.

## Known difference between the paths

The two paths do not behave the same:

- In the runbook path, `cd`, `export` and shell variables **do** carry into
  later steps, because everything runs in one shell.
- `set -u` makes an unset variable a failure in the runbook path, but not in
  the step-by-step path.

So whether a clean runbook exists can change a task's result. Until the paths
agree, write steps that do not depend on state left by an earlier step.

## Changing this contract

Change this page, the matching `contract_*` test, and the code in one PR. A
change to a guarantee needs a line in the PR body that says why the old
behavior was wrong.
