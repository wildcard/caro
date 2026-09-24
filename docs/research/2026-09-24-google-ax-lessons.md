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
  `caroml/jobs.rs`). Today no production caller sets one.
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
too. **Candidate ADR-017.**

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
| P1 | Default step timeout in `caro run` / `caro do` | `src/caroml/runner.rs`, `jobs.rs` |
| P1 | SIGTERM, grace period, then SIGKILL | `src/execution/executor.rs` |
| P1 | Wire or remove `AgentLoop._max_iterations` | `src/agent/mod.rs` |
| P1 | ADR-017: CaroML version header | `docs/adr/`, `src/caroml/parser.rs` |
| P1 | Runner contract doc + contract test | `docs/caroml/`, `tests/` |
| P2 | Network-off by default in ADR-010 sandbox, `NEED net:` opt-in | ADR-010, `src/caroml/` |
| P2 | Named model profiles with `api_key_env` | `src/config/`, `src/backends/remote/` |
| P2 | Per-task capability grants for `--approval auto` / `allow_public` | `src/main.rs`, `src/caroml/` |
| P3 | MCP server spike | new feature flag |
| P3 | Structured trajectory export | `src/agent/`, `src/governance/` |
