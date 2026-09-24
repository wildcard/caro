# What Caro can learn from google/ax

**Date:** 2026-09-24
**Source:** <https://github.com/google/ax> (Apache-2.0, `ax.io/v1alpha1`, pre-stable)
**Status:** Research. Nothing here is a feature commitment. Anything that grows
into a new product line must clear the gates in
[`validation-discipline.md`](../../.claude/rules/validation-discipline.md).

## What AX is

AX is Google's declarative orchestrator for running autonomous agents on
Kubernetes. Its thesis: *"Agents are a new kind of workload. They are neither
stateless microservices nor run-to-completion batch jobs."* Everything is a
manifest:

| Kind | What it declares |
|---|---|
| `Task` | image, command, env, CPU/memory limits, workspaces, gateway, `debug` |
| `Workspace` | git repos to clone, MCP servers/registries, skills path |
| `Gateway` | listeners + **egress allowlist** of hosts/ports |
| `Model` | provider, model id, **secret reference**, parameters |

A kubectl-shaped CLI drives it (`ax apply/get/watch/delete/ssh/suspend/resume`).
A separate **runner contract** (`docs/runner.md`) pins down how the
control plane talks to the process in the sandbox:

- inputs arrive as env vars (`AX_TASK_YAML`, `AX_WORKSPACES_YAML`)
- `/healthz` and `/readyz` endpoints report health and readiness
- workspace setup runs once, and a marker file makes it idempotent across restores
- shutdown is SIGTERM to the process group, then a bounded grace period, then SIGKILL
- only `/workspace` survives suspend/resume
- guest services that allow arbitrary exec are **off unless `spec.debug: true`**,
  and `ax ssh` *refuses* to connect otherwise

## Where Caro stands

Caro is a local-first, single-binary CLI, not a cluster control plane. Most of
AX's machinery doesn't transfer, but several of its design choices do:

| Area | Caro today |
|---|---|
| Execution | `src/execution/executor.rs` runs `sh -c` directly. No sandbox, rlimits or network fence. Before this PR, `with_timeout()` never killed anything. |
| Sandbox | ADR-010 (bubblewrap) is *Proposed*, not implemented |
| Task files | CaroML `.caro` / Carofile have no schema version. Only `caroml.lock` has one (`SCHEMA_VERSION = 2`). |
| Runner semantics | `src/caroml/runner.rs`: sequential, stop on first failure. The behavior is not documented as a contract. |
| Credentials | Hard-coded env var names (`backends/remote/claude.rs`, `openrouter.rs`) |
| Network egress | Only which backend URL you configure. Executed commands have unrestricted egress. |
| Budgets | `AgentLoop` has `_max_iterations` (unused) and a 15s wall clock |
| MCP | None in `src/` |
| Governance | `src/governance/` is a Phase 0 agentmesh build spike |

## Lessons, ranked

### 1. Enforce what you declare (P0, fixed in this PR)

AX's limits are real: the sandbox is killed, not just marked late. Caro's
`CommandExecutor::with_timeout` accepted a deadline but only compared elapsed
time *after* the child exited, so `sleep 3600` would hang forever. The fix in
this PR runs the command in its own process group and SIGKILLs the whole group
at the deadline. The group kill matters because `bash -c "a; b"` forks
grandchildren that keep the pipes open.
Regression guard: `execution::executor::tests::test_timeout_kills_hung_command`.

Follow-ups:
- Wire a default timeout into `caro run` / `caro do` steps (`caroml/runner.rs`,
  `caroml/jobs.rs`). Today no production caller sets one. **Blocked on signal
  forwarding.** A timed command runs in its own process group, so a Ctrl-C would
  kill `caro` but leave the step running. AX's runner forwards the signal to the
  group (`syscall.Kill(-pid, SIGTERM)`, `runner/runner.go`), and we need the same
  before a default timeout can ship.
- Use SIGTERM, then a grace period, then SIGKILL (AX's shutdown order) instead of
  an immediate SIGKILL, so commands can clean up.
- Either wire `AgentLoop._max_iterations` or delete it. A declared but unused
  budget is worse than none.

### 2. Version the declarative contract (P1)

Every AX resource carries `apiVersion: ax.io/v1alpha1`, and the project warns up front
that breaking changes will come before stable. CaroML files carry no version,
so any grammar change silently changes behavior for everyone's saved task
library. Adopt an optional header such as `CAROML 1` (or `REM caroml:v1`). Its
absence means v1, and the parser rejects unknown major versions with a clear
error. The lock file already does this, so the task files that feed it should
too. This needs the next free ADR number (ADR-017 is taken by open PR #1438).

Note that AX *declares* `apiVersion` but never checks it: any string is
accepted (`pkg/apis/v1alpha1/types.go`, `internal/store/redis/store.go`). A
version header only helps if the parser rejects versions it doesn't know.

### 3. Write the runner contract down (P1)

AX treats the runner's semantics as a spec (`runner.md`), not an
implementation detail. Caro should add `docs/caroml/runner-contract.md` covering
what `caro run` / `caro do` guarantee:
- the cwd and env each step sees
- what happens to `LET` vars
- exit-code propagation
- stop-on-first-failure
- signal handling on Ctrl-C
- timeouts (lesson 1)
- whether setup steps are idempotent

This is cheap, and it turns today's behavior into something tests can pin
(`tests/*_contract.rs`, per `feature-evidence.md`).

### 4. Egress allowlist as a first-class policy (P2)

AX's `Gateway` makes the network fence explicit and declarative. Caro has
two places for this:
- **Command execution:** when ADR-010 lands, run with `bwrap --unshare-net` by
  default. A `.caro` task could declare `NEED net: github.com` to opt a step back in.
- **Model traffic:** `BackendsConfig.allow_public` and the hybrid privacy
  gateway (`backends/hybrid/`) already decide *whether* data leaves. An explicit
  `allowed_hosts` list would make *where* it goes auditable too.

### 5. Models as named resources with secret references (P2)

AX's `Model` separates *which model* from *where the key lives*
(`secretKey` ref). Caro reads fixed env var names in each backend. Named
profiles in `config.toml` would let users switch models without changing the
code path. For example:

```toml
[models.fast]
provider = "anthropic"
model = "claude-haiku-4-5-20251001"
api_key_env = "ANTHROPIC_API_KEY"
```

An optional keyring backend could come later.

### 6. Dangerous capabilities are opt-in per task, and refused otherwise (P2)

`spec.debug` is off by default, and `ax ssh` refuses to connect rather than
warning. The same posture fits Caro's riskiest switches:
- `--approval auto`
- `allow_public`
- a future "skip safety" escape hatch

Each should be declared per task (in the `.caro` file or per invocation), not
set globally and forgotten. Where Caro can't verify that a capability was granted, it should
refuse, not degrade silently.

### 7. Workspace = repos + tools + skills (P3)

AX's `Workspace` bundles what an agent needs before it starts. Caro's
nearest equivalents are the Carofile `USE` directive and the skills system
(ADR-004, `src/caroml/skill.rs`). MCP is absent. An MCP *server* exposing
`caro`'s safety-validated generation would let other agents use Caro as a
tool. It would go through `external-sdk-integration.md` (build spike first).

### 8. Observability as trajectories (P3)

AX's roadmap adds OpenTelemetry and "structured agent trajectory export".
For Caro, a trajectory is short:

> query → static match? → backend → safety verdict → approval → exit code

Emitting it as structured JSON would feed the eval suite, ADR-003's audit
trail and the governance spike together, without new infrastructure.

## What not to copy

- **The control plane.** Kubernetes, CRDs, a controller and multi-cluster
  contexts solve fleet problems. Caro runs one process for one user on one
  machine. Adopting them would be gold-plating
  (`good-boy-scout.md`, ponytail reviewer).
- **Suspend/resume.** Checkpointing matters for hour-long agents. Caro
  commands last seconds. The only plausible use is a long multi-step Carofile
  `JOB` resuming from its last completed step, and even that can wait for real
  user evidence.
- **The kubectl verb set.** `caro` has one verb that matters (turn this
  sentence into a safe command). `apply/get/watch` would make the CLI harder
  to use.

## Follow-up backlog

| Pri | Item | Touches |
|---|---|---|
| P0 | ✅ Real timeout enforcement (this PR) | `src/execution/executor.rs` |
| P1 | SIGINT forwarding to the process group, then a default step timeout in `caro run` / `caro do` | `src/execution/executor.rs`, `src/caroml/runner.rs`, `jobs.rs` |
| P1 | SIGTERM, grace period, then SIGKILL | `src/execution/executor.rs` |
| P1 | Wire or remove `AgentLoop._max_iterations` | `src/agent/mod.rs` |
| P1 | ADR: CaroML version header (next free number) | `docs/adr/`, `src/caroml/parser.rs` |
| P1 | Runner contract doc + contract test | `docs/caroml/`, `tests/` |
| P2 | Network-off by default in ADR-010 sandbox, `NEED net:` opt-in | ADR-010, `src/caroml/` |
| P2 | Named model profiles with `api_key_env` | `src/config/`, `src/backends/remote/` |
| P2 | Per-task capability grants for `--approval auto` / `allow_public` | `src/main.rs`, `src/caroml/` |
| P3 | MCP server spike | new feature flag |
| P3 | Structured trajectory export | `src/agent/`, `src/governance/` |

---

## Part 2: What AX's implementation teaches

We read the full source (about 7.5k lines of Go at `ace0360`: controller,
Redis store, runner, workspace setup, CLI, tests). Several things the docs
promise are not what the code does. Those gaps are the most useful lessons,
because our own harness has the same kind.

### Patterns worth copying

- **Supervise the process group, not just the process.** The runner starts
  the command with `Setpgid: true`. On shutdown it sends SIGTERM to `-pid`,
  waits 10s, then SIGKILLs the group (`runner/runner.go`). This PR copies
  the process-group supervision into `CommandExecutor`, but not the shutdown
  sequence: on timeout it sends SIGKILL immediately. Adding SIGTERM and a
  grace period is a P1 follow-up.
- **Stay inspectable after the command exits.** The runner remains PID 1 and
  keeps its metadata server up once the agent finishes, so you can still look
  at what happened. For our harness, a routine should leave its worktree and
  logs behind for inspection, not clean up on exit.
- **Idempotent, retried setup.** Workspace setup is `git init` → `remote add`,
  falling back to `set-url` → `fetch` → `checkout FETCH_HEAD`, retried 5× with
  git invoked directly, never through a shell (`internal/workspace/setup.go`).
  A marker file is written only when every clone succeeded, so a partial
  failure is retried on the next start.
- **Strict decoding.** Manifests go YAML → JSON → protojson with unknown fields
  rejected, so a misspelled key fails loudly instead of being ignored
  (`pkg/apis/v1alpha1/types.go`).
- **Status writes don't trigger reconciles.** `UpdateTaskStatus` deliberately
  emits no event, which avoids self-triggering loops. The worker also acks
  failed events "so a bad task cannot wedge the queue".
- **Two-phase delete.** `Terminating`, then tear down the actor and templates,
  and only then delete the record. `ax delete` blocks until it's gone.
- **Secrets stay off argv.** The bootstrap reads the API key from the
  environment "so it does not show up in process listings".

### Anti-patterns: declared, not enforced

| AX declares | What the code does |
|---|---|
| `resources` CPU/memory limits | Never read by the controller |
| Gateway egress `port` | Ignored; `*` becomes allow-all on every port |
| MCP servers and skill registries in a Workspace | Never materialized by the runner |
| `apiVersion: ax.io/v1alpha1` | Never validated |
| `ax ssh` refuses unless `debug: true` | Only the CLI checks this. The guest gRPC service itself has no auth, and toggling `debug` never reaches an existing actor |
| Workspace marker "survives resume" | Written to `/ax`, outside the durable `/workspace` volume |
| Goal-driven setup "prepares the environment" | Result never verified; the marker is written even when the agent failed |
| Model client plans a workspace | On 401/403/429/503 or no key, it returns a **fabricated** canned plan with fake token counts |
| Reconciler drives tasks to Ready | Edge-triggered only, no resync or requeue. A 15s readiness poll against a 10-minute bootstrap means goal-driven tasks can stay not-Ready forever |

Two principles follow for Caro:
1. **An unenforced field is worse than no field.** It tells readers a
   guarantee exists when it doesn't.
2. **Never fabricate success on a failure path.** A harness that turns an
   error into a plausible-looking result can't be trusted.

---

## Part 3: Applying this to Caro's agent harness

"Harness" here means the machinery that runs agents on this repo:
`.claude/` hooks, agents, skills and routines, plus `loop.sh`, the worktree
workflow, beads and Hermes, and the CI workflows. The map below comes from a
full survey on 2026-09-24.

### Fixed in this PR

1. **Guard hooks that never enforced anything.**
   - The three PreToolUse Bash hooks read `CLAUDE_TOOL_NAME` /
     `CLAUDE_TOOL_PARAMS_COMMAND` env vars, which Claude Code never sets. Hook
     input arrives as JSON on stdin, so every hook exited 0.
   - Their block path used `exit 1`. That is a non-blocking error; only
     `exit 2` blocks.
   - So "never commit to main", which `constitution.md` Tier 1 says is
     *"Enforced by a PreToolUse hook"*, was not enforced. Neither were
     forced-worktree-removal protection or budget-leak scanning.
   - The fix adds `.claude/hooks/lib/hook-input.sh` and blocks with exit 2.
   - Two false positives that would have started firing once the hooks worked:
     - `cd .worktrees/x && git commit` was judged by the session cwd's branch.
     - Any path containing `-f` counted as the force flag.
   - The budget scanner had its own bug: `grep -v '^\+\+\+'` means "one or
     more" in GNU BRE, which dropped every diff line.
   - Regression guard: `.claude/hooks/tests/guard-hooks.test.sh` (12 cases),
     run in the ShellCheck workflow.
   - This is the AX `ax ssh` lesson turned inward: a check the caller skips
     isn't a check.
2. **Read-only agents that could write.** `devils-advocate` and
   `ponytail-reviewer` call themselves read-only but had no `tools:` line, so
   they inherited Edit, Write and Bash. They now have explicit allowlists.
   This matches AX's "minimally privileged policies" roadmap item: privileges
   per role, enforced by the platform.

### Recommended next (ranked)

| Pri | AX concept | Harness gap | Concrete change |
|---|---|---|---|
| P1 | Budgets are part of the Task spec | `loop.sh` defaults to `RALPH_MAX_ITERATIONS=0` (unlimited). `schedule.yaml` has `timeout_minutes`, but nothing in the repo enforces it | Default `loop.sh` to a finite cap (e.g. 20) and wrap each iteration in `timeout`. Make a missing cap an error, not a default |
| P1 | Status conditions written back | `.claude/automation/state/last_run.json` and `metrics.json` have been null since 2026-01-11 | Each routine appends one JSONL record per run: `{routine, started, finished, phase: Succeeded/Failed/TimedOut, reason, pr, issues}`, much like AX's phase plus conditions |
| P1 | Never fabricate success | `evaluation.yml` runs the eval suite with `\|\| true` and then applies regex pass-rate gates | Keep the real exit code. Fail the job when the harness itself fails, as distinct from a low pass rate |
| P2 | Workspace prepared once, with a marker | No SessionStart `startup` hook, so cloud sessions start cold (no `cargo fetch`, no `bd`, no node for the continuity hooks) | Add a `startup` hook that runs idempotent setup, keeps a marker in `target/` (durable), and skips on resume. The existing `session-start-continuity.sh` only handles `resume\|compact\|clear` |
| P2 | Least privilege per role | 31 of 34 agents inherit every tool | Give every agent an explicit `tools:` line. The ones that file issues or run caro need Bash; reviewers and researchers don't |
| P2 | Gateway egress allowlist | No CI job restricts egress. Agents run the real `caro` binary with full Bash | Add `step-security/harden-runner` in audit mode to CI, then `block` for jobs with a known host set. Pair with open PR #1438 (sandboxed execution for verification) |
| P2 | Runner contract document | Routines are prose. Their inputs, outputs and exit semantics aren't written down | A short `.claude/automation/CONTRACT.md` covering what a routine reads (env, schedule entry), what it must write (the status record above), how it signals failure, and a max runtime |
| P2 | Model as named resource | `model: sonnet` is hard-coded in 33 agent files | One place to change it: prefer `inherit`, or a single profile for the default |
| P3 | Two-phase delete | Worktree cleanup relies on the `caro.prune` audit plus a hook | Keep it. AX confirms the "mark, tear down, then delete the record" order. Also untrack the 8 empty `.worktrees/` stubs that are in git even though the directory is gitignored |
| P3 | Declarative, versioned specs | `schedule.yaml` has no version field, and the `ml_fine_tune_loop` entry is malformed (no `enabled` or `timeout`) | Add `version: 1` and a schema check in CI (strict decoding, per AX's `types.go`) |

### Stale harness references

Found during the survey. Each one is a small documentation or config fix:

- `CLAUDE.md` says `cargo run --bin caro-eval`, but no such binary exists; the
  eval runs as `caro test`.
- `CLAUDE.md` points to `.claude/memory/current-tasks.md`, which doesn't exist.
- `bin/sk-new-feature` and `bin/sk-merge` call `.kittify/scripts/bash/*.sh`,
  which don't exist, yet `git-workflow.md` tells agents to use them.
- `.github/workflows/modulization.yml` runs `claude --skill modulization`, and
  `--skill` is not a CLI flag.
- The hooks and the prune skill hardcode `/Users/kobik-private/...` paths
  (one removed in this PR).
