# ADR-039: Sandbox-Aware Verdict Tier — Pre-Execution Sandbox Placement as a Deterministic, Orthogonal Axis of the Safety Verdict

- **Status**: Proposed (scope only — no implementation in this document)
- **Date**: 2026-07-23
- **Produced by**: `caro-research--scoping-process` scheduled task
- **Feature researched**: Anthropic **sandbox-runtime (`srt`)**
  ([anthropic-experimental/sandbox-runtime](https://github.com/anthropic-experimental/sandbox-runtime),
  research preview) and the **Claude Code sandboxed Bash tool**
  ([docs](https://code.claude.com/docs/en/sandboxing)) it powers —
  experimental, OSS, and the current reference design for OS-level
  agent command sandboxing
- **Depends on**: ADR-010 (bubblewrap sandbox execution — this ADR narrows
  and supersedes its execution-layer scope for v1), ADR-020 (tiered
  approval — `SuggestedRouting`), ADR-024 (headless JSON contract — exit
  codes 0–6, envelope)
- **Relates to**: ADR-027 (permission resolution — exit 7), ADR-029
  (structured output — exit 8), ADR-034 (degraded result — exit 9),
  ADR-037 (lifecycle event schema), ADR-038 (MCP validator server — the
  placement field travels in its assessment payload)

> **Provenance note (autonomous run).** Produced by the
> `caro-research--scoping-process` scheduled task, whose template left
> `[FEATURE NAME]` unbound and ran with no user present. Target selection
> rationale: the 2026-07-23 Hermes market scan names **sandbox-aware
> tiered verdicts** ("trusted targets") as opportunity C — "binary
> allow/block wastes the new cheap-sandbox reality" — and simultaneously
> warns **"integrate with sandboxes; don't become one"** (CreateOS,
> Alibaba, Windows Agent Framework are commoditizing isolation).
> Opportunity B (MCP server) was scoped by this task's previous run
> (ADR-038). The research subject is therefore the reference
> implementation of the layer caro must *integrate with*: Anthropic's
> experimental `srt` / Claude Code sandboxing. Treat the choice of
> `srt`-first adapter order (Decision D4) as a reviewable assumption.
>
> **Validation-discipline note.** Hermes flags opportunity C as a feature
> spec subject to `.claude/rules/validation-discipline.md` (Gate 1: 20
> transcripts). This ADR is the *architecture scope* that a gated feature
> spec would implement; it deliberately contains no PMF claim and commits
> no implementation. Gate obligations attach to the spec/implementation
> PR, not to this document.

---

## Context

### Phase 1 — what `srt` / Claude Code sandboxing is

**Problem it solves, and for whom.** Agent harnesses want to run *most*
shell commands without a per-command human approval. The sandbox replaces
the prompt with an OS-enforced boundary: commands may write only the
working directory and session temp dir, and may reach only allowlisted
network domains — enforced on the running process and all children, so it
holds even when the command does more than its string suggests.

**Architecture.**

- `srt` wraps an arbitrary command with OS primitives: **Seatbelt**
  (`sandbox-exec`) on macOS, **bubblewrap + socat** on Linux/WSL2, an
  optional **seccomp** filter for Unix-socket blocking, and (new) a
  WFP-based `srt-sandbox` user account on Windows. No container, no
  daemon.
- Network policy is enforced by a **proxy outside the sandbox**; the
  allow decision is made on the client-supplied hostname (no TLS
  termination by default; `tlsTerminate` is experimental). Credential
  `mask` entries substitute sentinel env values with real ones at the
  proxy, per `injectHosts`.
- Config: `~/.srt-settings.json` (`--settings <path>` override) with
  `filesystem: {allowWrite, denyWrite, denyRead, allowRead}` and
  `network: {allowedDomains, deniedDomains}`. Library API:
  `SandboxManager` + `SandboxRuntimeConfig` (TypeScript).
- In Claude Code, the sandbox is fused with approval policy:
  **auto-allow mode** runs sandboxed commands without prompting;
  commands that cannot run sandboxed fall back to the regular permission
  flow.

**Why it is experimental / the failure modes.**

1. **The escape hatch (the failure mode this ADR solves by design).**
   When a command fails under sandbox restrictions, Claude Code's model
   "analyzes the failure and may retry the command with the
   `dangerouslyDisableSandbox` parameter." The *de-escalation decision is
   made by the LLM, after a failure, opaquely*. Users report confusion;
   the June 2026 **git-worktree gitdir-confusion escape** (fixed in
   2.1.163) and the **`settings.json` creation injection**
   (CVE-2026-25725 class: sandbox could create the missing settings file
   at startup and inject `SessionStart` hooks that run on the host)
   both weaponized the gap between "sandboxed" and "policy". Strict mode
   (`allowUnsandboxedCommands: false`) exists but is opt-in.
2. **Fail-open by default.** If dependencies are missing or the platform
   is unsupported, Claude Code warns and runs *unsandboxed*;
   `failIfUnavailable: true` is opt-in, intended for managed fleets.
3. **Hostname-trust network filtering.** The proxy allows by requested
   hostname without inspecting TLS → domain-fronting exfiltration paths
   through broad allowlists (documented limitation).
4. **Compatibility discovered at runtime.** `docker`, `watchman`/jest,
   Go CLIs' TLS verification under Seatbelt, Apple Events, WSL2 Windows
   binaries — each fails inside the sandbox first, then needs a manual
   `excludedCommands` entry. There is no machine-readable pre-flight
   "this command class cannot be sandboxed here" verdict.
5. **No structured output contract.** `srt` passes through the child's
   exit code and prints human text; sandbox placement, denial reasons,
   and fallback decisions are not surfaced as machine-readable events.
   Scripts cannot distinguish "command failed" from "sandbox denied it"
   from "sandbox unavailable".

**Session/context lifecycle.** Dependency detection runs at harness
startup; domain approvals persist per session; the proxy lives for the
session. `srt` itself is per-invocation: read settings → build profile →
spawn → passthrough exit. Nothing requires a daemon — the design is
subprocess-pure, which caro can match.

### Phase 2 — differentiation

**What they get right (replicate).**

- Placement enforced by **OS primitives, not string analysis** — the
  boundary holds regardless of what the model generated.
- **Two independent layers** (filesystem / network) that can be enabled
  separately.
- Config **merging that only narrows**: any scope can add a deny; no
  scope can remove one.
- Settings-file self-protection (deny writes to own policy files),
  including symlink resolution.
- Subprocess-pure wrapper (`srt <cmd>`) — no daemon.

**Their gaps we avoid by designing the schema first.**

- Sandbox placement is **decided post-hoc by an LLM** (escape hatch)
  instead of pre-execution by deterministic policy. In caro, placement
  is computed by the validator *before* anything runs, from versioned
  pattern tables — the same discipline as the 52+ safety patterns.
- **Fail-open default.** caro's ADR-010 already chose fail-safe ("if
  sandbox fails to initialize, block execution"); this ADR keeps that:
  when the verdict requires a sandbox and none is available, the result
  is a typed error + dedicated exit code, never a silent unsandboxed run.
- **Compatibility as runtime surprise.** caro ships a versioned
  *incompatibility pattern table* (docker, watchman, sandbox-in-sandbox,
  Apple-Events-dependent commands…) so `placement: "incompatible"` is a
  deterministic pre-flight answer, not a failed attempt.
- **No machine contract.** caro's placement travels in the ADR-024
  envelope with a `schema_version`, stable field names, and a dedicated
  exit code.

**caro's unique positioning.** Offline, local-first, deterministic, and
agent-agnostic: any harness (or human) calls one subprocess and gets the
same verdict. The scarce layer per the market scan is not the sandbox —
it is the *decision about what runs where, with an audit trail*. caro
does not ship isolation; it ships the placement verdict and adapters to
whatever isolation is installed (`srt`, bwrap, sandbox-exec). This is
exactly what an LLM-coupled harness cannot offer: Claude Code's
placement logic is fused to its model loop; caro's is a pure function of
(command, platform, capability probe, config).

**Existing infrastructure that already covers part of this.**

| Capability | Exists today | Where |
| --- | --- | --- |
| Risk verdicts (Safe/Moderate/High/Critical) | ✅ | `src/models/mod.rs` (`RiskLevel`) |
| Tiered routing (AutoApprove/AsyncLog/HumanGate/Block) | ✅ ADR-020 | `src/models/mod.rs` (`SuggestedRouting`) |
| Pattern engine + zero-false-positive test discipline | ✅ | `src/safety/patterns.rs`, `validator.rs` |
| Platform detection (OS, arch, shell, BSD flavor) | ✅ | `src/platform/` |
| Execution with timeout + `ExecutionResult` | ✅ | `src/execution/executor.rs` |
| Headless envelope + exit codes 0–9 | Scoped (ADR-024/027/029/034) | `src/cli/mod.rs` (planned) |
| Sandbox execution design (bwrap profiles, fail-safe) | Proposed, unimplemented | ADR-010 |
| Config + serialization (`serde`, `schemars`) | ✅ | `src/config/`, `Cargo.toml` |

## Decision

Add **sandbox placement as a third, orthogonal axis of the safety
verdict** — alongside risk level and routing — decided deterministically
before execution, serialized in every assessment payload, and enforced
fail-closed by thin adapters over externally installed sandboxes.

### D1 — Placement is a new orthogonal field, not a fifth `SuggestedRouting` variant

Claude Code's core confusion is conflating "sandboxed" with "approved".
We keep the axes separate: routing answers *may it run and who approves*;
placement answers *where it must run if it runs*. Adding a variant to
`SuggestedRouting` would also break exhaustive matches in existing
consumers (ADR-038's MRTR mapping among them). New enum in
`src/models/mod.rs`:

```rust
/// Where a command must execute, decided pre-execution by the validator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SandboxPlacement {
    /// Safe to run on the host (routing still applies).
    HostOk,
    /// Runs anywhere; sandbox preferred when available (risk-reduction).
    PreferSandbox,
    /// Must run inside a sandbox; no sandbox ⇒ do not run (fail-closed).
    SandboxOnly,
    /// Known-incompatible with OS sandboxes (docker, watchman, …);
    /// host-only, so routing alone gates it — never silently relaxed.
    Incompatible,
}
```

Derivation is a pure function
`fn derive_placement(&RiskJudgment, &PlatformInfo, &SandboxConfig) -> SandboxPlacement`
in `src/safety/mod.rs`, driven by two new versioned pattern tables in
`src/safety/patterns.rs`: `SANDBOX_REQUIRED_PATTERNS` (e.g. `curl | sh`
class, unpinned installers, MEDIUM-risk write-heavy commands) and
`SANDBOX_INCOMPATIBLE_PATTERNS` (docker/podman, watchman-backed tools,
nested-sandbox, Apple-Events-dependent commands). Same TDD workflow as
all safety patterns (`skill: safety-pattern-developer`); L2/L3 pattern
quality rules apply (no `...` placeholders, no near-duplicates).

Canonical interaction table (normative):

| Routing \ Placement | `host_ok` | `prefer_sandbox` | `sandbox_only` | `incompatible` |
| --- | --- | --- | --- | --- |
| `AutoApprove` | run host | run sandboxed if capable, else host | run sandboxed, else **exit 10** | run host |
| `HumanGate` | prompt | prompt (sandbox noted) | prompt; approval runs sandboxed, else **exit 10** | prompt |
| `Block` | never runs — placement moot (exit 3) | ← | ← | ← |

`Block` always wins; placement never upgrades a blocked command. The
headline user value: a `HumanGate`-by-risk command whose placement is
`prefer_sandbox` can be *configured* (opt-in `sandbox.auto_approve_in_sandbox
= true`) to route `AutoApprove`-in-sandbox — fewer prompts, same safety,
which is opportunity C's "fewer hard blocks" without touching `Block`.

### D2 — De-escalation is never automatic (solves the Phase-1 failure mode by design)

The `srt`/Claude Code escape hatch — *model retries outside the sandbox
after a sandbox failure* — is structurally impossible here:

- A sandboxed run that fails **due to sandbox denial** returns the typed
  result below and **exit 10**. caro never re-runs it unsandboxed in the
  same invocation. There is no code path that does.
- The *caller* may re-invoke with `--sandbox=off`; that is a new process,
  a new explicit decision, attributable in logs (ADR-021), and it still
  passes full risk validation — `Block` stays blocked, and `sandbox_only`
  placements refuse `--sandbox=off` with `UsageError` (exit 2) unless
  the caller also passes `--i-understand-unsandboxed`, mirroring the
  visibility (but not the automaticity) of `dangerouslyDisableSandbox`.
- Equivalent of `failIfUnavailable`: implicit. `--sandbox=require` +
  no capability ⇒ exit 10 before spawning anything. No warn-and-continue.

### D3 — Capability probe is per-invocation and cached-stateless

New serializable type in `src/execution/sandbox.rs` (one new file inside
the existing `execution` module):

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SandboxCapability {
    pub backend: SandboxBackend,      // enum: Srt | Bubblewrap | Seatbelt | None
    pub filesystem: bool,             // fs isolation achievable
    pub network: bool,                // network isolation achievable (v1: srt only)
    pub probe_ms: u64,
    pub missing: Vec<String>,         // e.g. ["bubblewrap", "socat"]
}
```

Probe = `which` checks + platform gate (macOS ⇒ `sandbox-exec`,
Linux/WSL2 ⇒ `bwrap`; `srt` preferred on both when present). Cost is a
few ms — acceptable per invocation, honoring the "pure subprocess, no
daemon, no state" constraint. An optional probe snapshot may later reuse
ADR-025's init-snapshot cache; not in v1.

### D4 — Integrate, don't implement: adapter over installed sandboxes

`SandboxedExecutor` (also `src/execution/sandbox.rs`) wraps the existing
`CommandExecutor` and builds argv for the detected backend:

- **`srt` adapter (first)**: `srt --settings <generated> <command>` — we
  generate a minimal `srt-settings.json` from caro's own
  `SandboxConfig` (allow-write: cwd + tmp; deny-read: credential paths).
  Rationale: one adapter covers macOS + Linux + (eventually) Windows,
  and rides the reference implementation's fixes.
- **Native `bwrap` fallback (Linux)** and **`sandbox-exec` fallback
  (macOS)**: filesystem isolation only, profiles as scoped in ADR-010
  (its execution-layer content is reused; its "primary backend =
  bubblewrap" decision is narrowed to "fallback when `srt` absent").
- **Network layer in v1 = binary**: `network: {allow: bool}` (share or
  unshare network namespace / deny outbound in profile). Domain
  allowlists, proxies, TLS termination — explicitly out of scope (v2);
  that is where the analog's hardest unsolved problems live (domain
  fronting), and Hermes says don't rebuild them.

### D5 — Output contract

Envelope (ADR-024) gains one object, present whenever validation ran:

```json
"sandbox": {
  "placement": "sandbox_only",
  "backend": "srt",
  "executed_in_sandbox": true,
  "capability": { "backend": "srt", "filesystem": true, "network": true, "missing": [] },
  "denial": null
}
```

`denial` (nullable, typed) distinguishes the three states the analog
conflates: `{"kind": "sandbox_unavailable" | "sandbox_denied_fs" |
"sandbox_denied_net" | "sandbox_spawn_failed", "detail": "..."}`.

`ExitCode` gains one variant — the next free code after ADR-034's 9:

| Code | Meaning | Envelope promise |
| --- | --- | --- |
| 10 | `SandboxRequiredUnavailable` — placement demanded a sandbox and none was capable, **or** the sandboxed run was denied by the boundary | `status: "sandbox_blocked"`, `sandbox.denial.kind` set, `command` present but not executed (or executed-and-denied with captured stderr) |

Promises: exit 10 is reserved exclusively for sandbox placement/denial;
a child process's own non-zero exit under a sandbox stays exit 6
(`ExecutionFailed`) with `executed_in_sandbox: true`. Field names and
the placement enum's serialized strings are stable; breaking either
bumps `schema_version`. The same `sandbox` object travels in ADR-038's
MCP assessment payload and ADR-037 lifecycle events (`sandbox_probe`,
`sandbox_spawn`, `sandbox_denial`) — one schema, three transports.

### Files changed (minimal set)

1. `src/models/mod.rs` — `SandboxPlacement`, `SandboxBackend` enums
   (serde + schemars derives).
2. `src/safety/patterns.rs` — the two new pattern tables (TDD).
3. `src/safety/mod.rs` — `derive_placement()`; wire into validation
   result.
4. `src/execution/sandbox.rs` — **new file** (existing module):
   `SandboxCapability`, probe, `SandboxedExecutor`, srt/bwrap/seatbelt
   argv builders, settings-file generation.
5. `src/execution/mod.rs` — export; route execution through the adapter
   when placement demands it.
6. `src/cli/mod.rs` — `--sandbox=auto|require|prefer|off` flag,
   `ExitCode::SandboxRequiredUnavailable = 10`, envelope `sandbox`
   object.
7. `src/config/` — `sandbox` section (enabled, auto_approve_in_sandbox,
   extra allow_write paths — merge-only-narrows for deny lists, copied
   from the analog's best idea).

No new top-level modules. No new dependencies (adapters shell out;
`srt` is optional at runtime, so no SDK build-spike per
`.claude/rules/external-sdk-integration.md` is triggered — nothing
links).

## Integration tests (deterministic input → fixed JSON + exit code)

Backends are faked via a `CARO_SANDBOX_PROBE_OVERRIDE` test hook (JSON
`SandboxCapability` injected), so tests are deterministic on CI without
bwrap/srt installed:

1. **Placement derivation is pure** — fixture commands × platforms ⇒
   exact `placement` values (`docker ps` ⇒ `incompatible`;
   `curl https://example.com | sh` ⇒ `sandbox_only`; `ls` ⇒ `host_ok`).
2. **Fail-closed** — probe override = none + placement `sandbox_only`
   ⇒ exit 10, `denial.kind: "sandbox_unavailable"`, no process spawned
   (asserted via spawn-count hook).
3. **No silent de-escalation** — same input with `--sandbox=off` ⇒ exit
   2 (`UsageError`) without `--i-understand-unsandboxed`; with it ⇒ runs
   host, envelope `executed_in_sandbox: false`, placement still
   `sandbox_only` (audit trail preserved).
4. **Block wins** — `rm -rf /` with sandbox available ⇒ exit 3, `sandbox`
   object shows placement but `executed_in_sandbox: false`.
5. **Child failure ≠ sandbox failure** — sandboxed command exiting 1 ⇒
   caro exit 6, `executed_in_sandbox: true`, `denial: null`.
6. **Envelope schema stability** — `schemars` snapshot of the `sandbox`
   object; golden-file JSON for cases 1–5.
7. **Real-backend smoke (Linux CI only, `#[ignore]` locally)** — bwrap
   present: write outside cwd denied ⇒ exit 10 `sandbox_denied_fs`;
   write inside cwd ⇒ exit 0.

## Out of scope (next version)

- Domain allowlists, network proxy, TLS termination, credential
  masking — the analog's open wounds; revisit after their `tlsTerminate`
  stabilizes.
- Windows (WFP/srt-win) adapter.
- MicroVM adapters (CreateOS-class) — the `SandboxBackend` enum is the
  extension point.
- Probe snapshot caching (ADR-025 integration).
- Any change to the `SuggestedRouting` mapping table itself.
- Interactive UI affordances (badge "will run sandboxed" in TTY mode).
- The user-facing feature spec and its validation-discipline gates
  (20 transcripts) — required before implementation PRs open.

## Consequences

**Positive.** Fewer human gates at equal safety (opt-in); deterministic,
auditable placement that answers the CISO rubric's "allowed actions" and
"blast radius" questions natively; the escape-hatch class of bug is
excluded structurally, not by configuration; zero new linked
dependencies.

**Negative / risks.** Two new pattern tables to maintain (mitigated by
existing TDD skill); `srt` is a research preview and its CLI/config may
break (mitigated: adapter generates settings per-invocation and pins no
version; bwrap/seatbelt fallbacks exist); binary network isolation in v1
will disappoint users wanting domain allowlists (documented, deferred);
sandboxed-vs-host env differences (e.g. `$TMPDIR`) may confuse — the
envelope's `executed_in_sandbox` makes it at least diagnosable.

**Alternatives considered.**

1. *Add `AllowInSandbox` to `SuggestedRouting`* — rejected: conflates
   axes (the analog's root confusion), breaks exhaustive matches.
2. *Implement our own sandbox profiles fully (ADR-010 as written)* —
   rejected for v1: sandboxing is commoditizing; the differentiated
   layer is the verdict. ADR-010's profiles survive as the fallback
   adapter.
3. *LLM-assisted placement decision* — rejected: the week's headline
   incident (rogue-agent HF breach) is the argument for deterministic,
   non-model placement; also violates caro's zero-false-positive test
   discipline.
4. *Daemonized sandbox pool for latency* — rejected: violates the
   pure-subprocess constraint; probe cost is milliseconds.

## References

- [anthropic-experimental/sandbox-runtime](https://github.com/anthropic-experimental/sandbox-runtime) (README: architecture, `srt` CLI, `~/.srt-settings.json`, `SandboxManager` API)
- [Claude Code sandboxing docs](https://code.claude.com/docs/en/sandboxing) (modes, escape hatch, `failIfUnavailable`, `allowUnsandboxedCommands`, credential mask, limitations incl. domain fronting)
- [Metnew: Claude Code worktree sandbox escape](https://github.com/Metnew/write-ups/tree/main/claude-code-worktree-sandbox-escape) (June 2026, fixed 2.1.163)
- [CVE-2026-25725 advisory](https://advisories.gitlab.com/pkg/npm/@anthropic-ai/claude-code/CVE-2026-25725) (settings.json injection)
- `.hermes/digests/2026-07-23-agent-market-scan.md` (opportunity C; "integrate with sandboxes; don't become one")
- ADR-010, ADR-020, ADR-024, ADR-025, ADR-027, ADR-029, ADR-034, ADR-037, ADR-038
