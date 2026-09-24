# Scope: `caro.posture.v1` — Does This Command Reconfigure the Thing That Would Stop the Next One?

> **Provenance.** Produced by the `caro-research--scoping-process` scheduled task,
> autonomous run, **2026-09-15**, no user present. The template's `[FEATURE NAME]`
> shipped unbound; target-selection rationale is §0.
>
> **This is a scope, not an implementation.** No branch, no PR, no code committed
> (`.claude/rules/git-workflow.md`). Companion ADR:
> [`docs/adr/ADR-071-guard-posture-mutation-classification.md`](docs/adr/ADR-071-guard-posture-mutation-classification.md).

---

## 0. Target selection

Three candidates surfaced in the 2026-09-09 → 2026-09-15 window. The chosen one is the third.

| Candidate | Why not |
|---|---|
| MS-Agent `Shell` tool, **CVE-2026-2256** — *"uses a regex-based blacklist to filter dangerous commands, which is unsafe"* | A direct critique of Caro's own architecture and worth a document. But the response is a **measured recall number against a public suite** (Hermes 3.11), which is ADR-060/063 territory and already scoped. Scoping it again would be a rewrite, not a scope. |
| Apollo Research **Watcher** staged monitor (regex → triage model → evaluator) | Competitor analysis, not a feature scope. Hermes 2026-09-09 §4 already recorded the correct strategic answer (*"be the stage the monitors call"*) and the correct non-answer (do not build stages two and three). |
| **DeepSeek Harness process sandbox + CVE-2026-82533** | **Chosen.** |

**Why this one.** The eight-week scan series has produced two findings that say Caro's *threat model*
is incomplete rather than its distribution: shift K (GitSpawn — the payload is in the state the
command reads, closed by ADR-069) and this one. CVE-2026-82533 is the mirror image of GitSpawn:
**the payload is in the state the command writes**, and the state it writes is the guard's own
configuration. ADR-069 covers the read side of ambient configuration. Nothing in the tree — shipped
or paper — covers the write side.

Three properties made it the target rather than the fourth interesting CVE of the month:

1. **It is a single shell command.** Not a supply-chain artifact, not a model behaviour, not an
   isolation primitive. One `curl`. That is Caro's lane, exclusively and by construction.
2. **The vendor documented the gap before the researcher found it, in its own reference doc**
   (*"Network and process visibility are outside this vocabulary"*) and in its own `SAFETY.md`
   (*"Review plugins, configuration, and proposed commands before allowing them to run"*). A product
   specification for a standalone command reviewer, published by a harness vendor, with 215,000
   stars behind it.
3. **The failure mode is one Caro cannot commit.** DeepSeek's escalation succeeded because the
   confinement of the next command was resolved from state the current command could write, and
   because the resulting log said `source: {kind: 'user'}`. Caro is a stateless subprocess with no
   attribution channel. What looks like structural poverty is, for this exact problem, the correct
   architecture — provided the *schema* is designed so it cannot be given an attribution field later.

Selection rationale in one line: **ADR-069 asked what a directory will run before anyone types a
command; this asks what a command will do to the thing that would have stopped it.**

---

## 1. Phase 1 — Feature research

### 1.1 The feature: the DeepSeek Harness process sandbox

[DeepSeek Harness](https://github.com/deepseek-ai/deepseek-harness) (`dsh`, MIT, released August
2026, **215k stars / 25.3k forks** as read 2026-09-15) is an open-source, local-first agent harness
built on the [Cordis](https://github.com/cordiverse/cordis) plugin runtime — tagline *"Everything is
a Plugin."* `npx @deepseek-ai/dsh web` starts a Web UI and an HTTP control API at
`http://127.0.0.1:3080`. The repo carries an explicit developer-preview banner: *"THERE WILL BE
COMPATIBILITY-BREAKING CHANGES."*

The feature under study is the **process sandbox** (`ctx.sandbox` / `ctx.sandboxPolicy`), read live
from the vendor's generated reference at
[`/reference/subsystems/sandbox`](https://deepseek-harness.github.io/deepseek-harness/en/reference/subsystems/sandbox).

**Problem it solves, and for whom.** A coding agent holds a shell and runs with the ambient authority
of the developer who launched it. The sandbox bounds what that shell can touch when the agent is
working with material it should not trust. The user is a developer running an agent over a repository
they did not write.

**Architecture — data flow and separation of concerns.** Genuinely good, and worth reading closely:

- `dsh-sandbox` defines an abstract seam: `confine(argv, policy) -> ConfinedArgv`. It wraps an argv,
  not a shell string — *"a shell-shaped consumer passes `['bash','-c',command]`."*
- `dsh-sandbox-local` supplies the platform backends: Linux bwrap/Landlock, macOS Seatbelt, Windows
  ACL restricted-token. Containers, microVMs and remote execution are declared *sibling
  implementations of whole capability seams*, not providers of `ctx.sandbox` — an explicit refusal to
  conflate isolation levels.
- `dsh-bash-sandbox` and `dsh-pwsh-sandbox` consume it and own spawn and result attribution.
- `ctx.sandboxPolicy.resolve()` owns precedence and root fallback *"so bash and fs do not repeat it"*.

**Key types.**

```ts
type SandboxMode = 'read-only' | 'workspace-write' | 'danger-full-access'
type ConfinedSandboxMode = Exclude<SandboxMode, 'danger-full-access'>
type SandboxEnforcement = 'full' | 'partial'

interface SandboxExecutionPolicy { mode: SandboxMode; workspaceRoot: string; sessionId?: SessionId }
interface SandboxPolicy extends SandboxExecutionPolicy { mode: ConfinedSandboxMode }

interface ConfinedArgv {
  argv: string[]
  enforcement: SandboxEnforcement
  denialSignatures: readonly string[]
  runnerFailureRules: readonly RunnerFailureRule[]
}
```

Three of these are excellent and §2.1 says why: `ConfinedSandboxMode` makes "unconfined" unsendable
to a provider *at the type level*; `SandboxEnforcement` makes completeness **a reported fact**; and
`denialSignatures` vs `runnerFailureRules` separates *"confinement worked and blocked it"* from
*"the confinement itself failed"*, which is the distinction ten of eleven agents failed to make in
GuardFall.

**The structured output contract.** `ConfinedArgv` is the payload. There is no exit-code contract for
the sandbox itself — the sandbox's answer is a wrapped argv, and failure is a thrown
`SandboxUnavailableError` with code `SANDBOX_UNAVAILABLE`. Classification of what happened afterwards
is a stderr-dialect problem: a consumer checks `runnerFailureRules` (nonzero exit + optional
exit-code gate + a case-insensitive fatal signature in one remaining stderr line, after exact-line
informational exclusions) **before** `denialSignatures` (per-backend: EROFS text under bwrap's
read-only binds, EACCES under Landlock, EPERM under Seatbelt), and the doc is explicit that a
consumer must match *"exactly these rather than a cross-backend union — the union claims denials a
given backend never produces."* The fail-closed invariant is stated as an absolute: *"Silent
unconfined passthrough is never legal for a confined policy."*

**Session/context lifecycle — how it avoids redundant initialization.** Policy is resolved **per
capability call**, not cached on the provider, so concurrent sessions and one-shot escalated retries
can ask the same provider for different boundaries *"without mutating provider state."* Provider
selection, probing and caching live in the local provider; functional probes arbitrate multi-runner
chains and *"may be skipped for a sole candidate."* Precedence, verbatim:

> An approved explicit mode outranks the session's last `sandbox/mode` event, which outranks the
> deployment default.

That single sentence is both the lifecycle answer and, as §1.2 shows, the vulnerability.

### 1.2 Why it is experimental — the failure mode

`SAFETY.md`, verbatim, in full on the point:

> Sandboxing, approval prompts, and permission controls can reduce risk, but they do not guarantee
> isolation or prevent damage. Even correctly enforced restrictions cannot protect resources that the
> project is allowed to access.
> …
> Review plugins, configuration, and proposed commands before allowing them to run.

**[CVE-2026-82533](https://www.ox.security/blog/cve-2026-82533-deepseek-harness-ai-agent-sandbox-escape/)**
— CWE-807 (Reliance on Untrusted Inputs in a Security Decision), **CVSS 9.4**
(`CVSS:4.0/AV:N/AC:L/AT:N/PR:N/UI:P/VC:H/VI:H/VA:H/SC:H/SI:H/SA:H`), found by Nir Zadok and Moshe
Siman Tov Bustan (OX Research), reported to VulnCheck 2026-08-24, fixed in `0.1.2-alpha.1` on
2026-08-27, re-tested 2026-08-30, published 2026-09-08. Affected: `0.1.1-rc.2` and earlier.

Four conditions, each documented, composing into the escape:

| # | Condition | Source |
|---|---|---|
| F1 | `isTrustedApiRequest` read the client-supplied **`Host` header** and granted access on a loopback authority, never comparing it with the connection's actual peer address | OX §Technical Details |
| F2 | The sandbox confined **file writes only**. bwrap got `--unshare-pid` but **not** `--unshare-net`; Seatbelt used `(allow default)` + `(deny file-write*)`. Loopback reachable from inside | OX §The Sandbox Rewrites Itself |
| F3 | *"ordinary `bash` tool calls did not require approval. Approval governed sandbox escalation requests, not routine command execution."* | OX, same section |
| F4 | The escalation appended a `sandbox/mode` event to the session log — the input `resolve()` reads for call *n+1* | vendor reference, precedence sentence |

One `curl` from inside the sandbox set the session to `danger-full-access` with approval `never`.
OX ran a matched control on the same host — two sessions, same preset, same command, one having made
the `curl` first — and the confined session was blocked writing to `/home/ubuntu` while the escaped
one succeeded. **The sandbox was working. It was reconfigured.**

**The attribution failure, verbatim, and the reason this ADR exists:**

> No `approval/request` event appears anywhere in the log. The confined command ran without a prompt
> and lifted its own confinement. The attribution is also notable: the policy change is recorded as
> `source: {kind: 'user'}` — the harness could not distinguish the agent's own shell from the human
> at the keyboard.

**The vocabulary failure, in the vendor's own words:**

> `SandboxMode` governs filesystem effects only. … Network and process visibility are outside this
> vocabulary.

A correct sentence in a reference document, attached to a mode literally named
`danger-full-access` in a subsystem literally named "sandbox". The CVE lives in the distance between
the name and the vocabulary.

**It was discoverable by routine use.** CSA records community posts on the discussion board on
**Aug 13 and Aug 14**, ten days before the formal report. The Aug 14 one is public and still open:
[Discussion #1516, *"Make network access a separate sandbox permission"*](https://github.com/deepseek-ai/deepseek-harness/discussions/1516),
Ideas category, one reply, no label. Verbatim:

> Loopback also needs its own rule. #250 shows why a sandboxed process reaching the Harness control
> plane is different from reaching a package registry.

with acceptance criteria including *"`workspace-write` plus `deny` cannot reach public addresses or
the Harness loopback control plane"* and *"Loopback requires a separate grant."* DeepSeek's fix
replaced header trust with a startup token exchanged for a signed cookie — which closes F1 and leaves
F2 exactly where #1516 found it.

**The class, per CSA (2026-09-10):**

> a sandbox that restricts one channel (the filesystem) while leaving another (loopback networking)
> implicitly trusted — recurs across the AI coding-agent ecosystem and is not fully resolved by
> patching one product

with the note that this is the third distinct containment failure CSA has documented in this product
category this year, after the [trust-handoff flaw](https://labs.cloudsecurityalliance.org/research/csa-research-note-ai-coding-agent-sandbox-escapes-20260722-c/)
(2026-07-22) and [GuardFall](https://labs.cloudsecurityalliance.org/research/csa-research-note-guardfall-ai-agent-shell-injection-2026070/)
(2026-07-01), *"which argues against treating any single incident as an isolated bug."*

### 1.3 Restating the failure mode as one sentence, because §3.4 has to solve it

> **A control plane that resolves the next command's confinement from state the current command can
> write, and that cannot distinguish the agent's shell from the human, has no boundary at all — and
> the command that crosses it is indistinguishable, to every destructive-pattern matcher in the
> category, from a health check.**

---

## 2. Phase 2 — Competitive differentiation

### 2.1 What they get right and Caro should replicate

1. **`SandboxEnforcement` — "Enforcement is a reported fact."** The report states how completely the
   backend governed the promised effects, and callers requiring an absolute boundary *"must not treat
   `partial` as `full`."* This is the same discipline ADR-070 D4 arrived at from the Gemini CLI
   direction (an allowlist parsed at registration and never enforced). Two independent derivations of
   the same rule is a strong signal. **→ D4: `CatalogIdentity` is required and non-defaulted.**
2. **`denialSignatures` vs `runnerFailureRules`.** "Blocked because confinement worked" and "the
   confinement never ran" are separate facts with separate evidence rules, and the doc forbids
   unioning backend dialects because *"the union claims denials a given backend never produces."*
   **→ D7: `Unresolved` and `Unparsed` are separate verdicts, and neither is `Inert`.**
3. **`ConfinedSandboxMode = Exclude<SandboxMode,'danger-full-access'>`.** Unconfined is unsendable to
   a provider at the type level rather than by convention. **→ D3: `Authority::Loopback` is a named
   variant rather than a suppression, so removing it is a reviewed schema change.**
4. **Per-call resolution, no provider state.** *"without mutating provider state"* is the right
   instinct; it was defeated only because the *input* to resolution was writable. **→ D2: keep the
   per-call purity and remove the writable input entirely.**
5. **Fail-closed as an absolute.** *"Silent unconfined passthrough is never legal."* **→ D5/D7: a
   clean verdict can never be produced by an absent catalog or an unparsed input.**
6. **Argv, not a shell string.** `confine` takes `readonly string[]`. Same reason ADR-067 exists.

### 2.2 Their design gaps, avoided by designing the schema first

| Their gap | Where | Our design answer |
|---|---|---|
| A vocabulary that names a channel it does not cover (`SandboxMode` = filesystem only, in a subsystem called "sandbox" with a `danger-full-access` member) | vendor reference | `GuardSurface` enumerates the channel. A surface the catalog does not know is `Inert` **with a `surfaces_known` count in the same document** (D4), so "not modelled" is readable and never printed as "safe". |
| Loopback implicitly trusted because "local traffic poses no meaningful risk" | F2; still open as #1516 | `Authority::Loopback { literal }` is a first-class variant that carries no permissive semantics anywhere in the schema (D3). |
| Policy resolved from state the confined process can write | F4 | No state. Catalog compiled at build time, content-hashed into every report (D2). |
| Caller identity synthesized and wrong (`source: {kind:'user'}`) | OX §Technical Details | **The schema has no attribution field** (D1). Nothing to forge. |
| Approval mounted on the escalation event, while the escalation arrived as a routine command | F3 | Posture mutation is a verdict on **routine commands**. There is no privileged path that skips it. |
| Guard cannot classify commands that disable the guard | universal in the corpus | `reflexive: true`, floored at `Critical` (D6). |
| Report cannot say "I saw the surface and it was only read" | universal | `Observes` is a distinct verdict at exit 0 (D5). |

### 2.3 Evidence: what Caro does today

**Method.** Extract every `pattern: r"…"` literal from `src/safety/patterns.rs` (67 extracted, 67
compile cleanly — the same count ADR-070 §2.3 reports, which cross-validates the extractor), replay
each under a faithful re-implementation of `SafetyValidator::is_dangerous_in_context`
(`src/safety/mod.rs:432` — the odd-unescaped-quote-count suppression), against a 38-row corpus: **31
posture-mutation rows that must flag** across five families, and **7 control rows that must stay
clean**. Full corpus and script in §5.

```
patterns extracted: 67   compiled: 67   skipped: 0

must-flag rows: 31   FALSE NEGATIVES: 28/31
control rows:    7   false positives:  0
```

The three hits, and why none of them is evidence of coverage:

| Row | Command | Matched | Pattern's actual subject |
|---|---|---|---|
| C1 | `export PATH=/tmp/shim:$PATH` | `export\s+PATH=` (`patterns.rs:231`) | "Modify PATH environment variable", `Moderate`. Fires identically whether or not `/tmp/shim` shadows `caro`. |
| C3 | `chmod -x $(command -v caro)` | `chmod\s+[+\-]x\s+` (`patterns.rs:297`) | Generic executable-bit change. Fires on `chmod +x build.sh`. |
| D4 | `sudo systemctl stop apparmor` | `sudo\s+(systemctl\|service)\s+(restart\|stop\|disable)` | "Modify system services with elevated privileges". Fires on `sudo systemctl stop nginx`. |

Per family:

| Family | Rows | Missed |
|---|---|---|
| A — loopback / unix-socket control-plane calls (the CVE vector) | 7 | **7** |
| B — agent policy-file mutation (`.claude/settings.json`, `~/.codex/config.toml`, `~/.gemini/settings.json`) | 9 | **9** |
| C — guard-executable and shell-resolution shadowing (**reflexive**) | 7 | 5 |
| D — OS confinement facilities (`setenforce`, `sysctl`, `spctl`, `launchctl`, `bwrap`) | 6 | 5 |
| E — toolchain policy (`npm config set ignore-scripts false`, `git config --global core.fsmonitor`) | 2 | **2** |

**Zero of family A. Zero of family B. Zero of family E.** And a corroborating grep: the strings
`.claude/settings`, `codex/config`, `bypassPermissions`, `danger-full-access`, `dangerously-skip`
appear **nowhere** in `src/` or `data/`.

**The decisive structural control — why ADR-064 does not close this either.** ADR-064's unit is
`EgressPair { source: SourceKind, sink: SinkKind }`, and a finding *requires* a source drawn from
`{CredentialFile, CredentialEnvVar, PrivateKeyMaterial, ProjectFile, Stdin}`. The escalation

```
curl -s -X POST http://127.0.0.1:3080/api/session/config -d '{"sandbox":"danger-full-access","approvalPolicy":"never"}'
```

carries **no source**: no credential, no key, no project file, no stdin. It is inbound to a control
plane, not outbound with a secret. `EgressPair` cannot be formed, so `caro.egress.v1` returns
`Clear` — **by construction, not by oversight**. Separately, `SinkKind::NetworkHost` carries
`HostDisposition = Allowed | Denied | Unlisted { strict }`, a pure allowlist vocabulary with no
loopback concept: neither ADR-064 nor its scope document contains the strings `loopback`, `localhost`
or `127.0.0.1` anywhere. **If ADR-064 shipped tomorrow, Caro would hold the same asymmetry DeepSeek
shipped** — a model that restricts one channel and has no word for the other.

### 2.4 Caro's unique positioning

1. **Stateless is the feature.** Every harness in the corpus answers this question from session
   state, and session state is what the attacker writes. Caro answers from the command text and a
   build-time artifact. It is the only architecture in the category that the attack cannot reach.
2. **Vendor-neutral, therefore reflexive.** Claude Code will not ship a rule that flags edits to
   `.claude/settings.json`; the rule would fire on its own feature. DeepSeek will not ship a rule that
   flags `POST 127.0.0.1:3080`; that is its API. A standalone tool has no product to protect, which is
   why it can also flag **commands that disable Caro** (D6) — a claim with no vendor equivalent.
3. **Offline and air-gapped.** No network, no probe, no daemon. The CVE's own delivery context is a
   developer workstation and a CI runner; the Black Hat chain landed on a runner with no egress.
4. **A community layer with a schema.** `data/cve_rules/` already proves the loop in this repo: one
   YAML file, one linter, a build-time compile, `test_cases` that become eval rows. The posture
   catalog is the same loop for a corpus that currently exists only as five vendors' internal `if`
   statements and one open GitHub discussion.
5. **The universal claim.** `.claude/settings.json`, `~/.codex/config.toml`, `~/.gemini/settings.json`,
   `.cursor/rules`, `127.0.0.1:3080` — one catalog, every harness. No vendor can write that file.

### 2.5 Existing infrastructure that already covers part of this

| Exists today | Covers | Gap it leaves |
|---|---|---|
| `SafetyValidator` + 67 `DANGEROUS_PATTERNS` | Flat-string regex over the whole command | §2.3: 28/31 |
| `is_dangerous_in_context` (`src/safety/mod.rs:432`) | Quote suppression | Suppresses on odd quote counts; ADR-067 §3 already records the damage |
| `data/cve_rules/*.yaml` → `build.rs:127` `compile_cve_ruleset` → bincode → `src/safety/cve_patterns.rs:31` `include_bytes!` | **The entire delivery mechanism for a community-curated, build-time-compiled, offline, content-addressed ruleset — including the auto-generated eval rows** | Payload is one regex per CVE; no surface model |
| `scripts/validate-cve-yaml.ts` | Schema lint gate for contributed YAML | Schema-specific; needs a sibling |
| `cve-rules` feature, in `default` (`Cargo.toml:[features]`) | Precedent for a default-on data feature | — |
| `RiskLevel` (`src/models/mod.rs:152`, derives `Ord`), `SafetyLevel`, `ShellType`, `SuggestedRouting` (`:189`) | Risk vocabulary and routing | Reused unchanged. `SuggestedRouting` still lacks `JsonSchema` + `Ord` (D11) |
| `ValidatorContext` / `ValidationOutcome` / `Verdict` (`src/caroml/validators/mod.rs:38`) | The caroml validator chain | Additive seam for PR 3; untouched in v1 |
| ADR-067 `Span` / `Site`; ADR-070 word splitter | Structural container and tokenization | No variant for a guard surface (D10) |

**Nothing to build from scratch except the catalog, the surface resolver (~350 lines) and the CLI
arm.**

---

## 3. Phase 3 — Scope definition

### 3.1 New types

All in **`src/safety/posture.rs`** (one new file inside an existing module). Every type derives
`Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema` from day one; `RiskLevel` already
derives `Ord`, so no comparator is written.

```rust
/// The report. One per invocation. Stdout is exactly this document, newline-terminated.
pub struct PostureReport {
    pub schema_version: &'static str,   // "caro.posture.v1"
    pub verdict: PostureVerdict,
    pub risk: Option<RiskLevel>,        // None iff verdict is Inert | Observes
    pub mutations: Vec<PostureMutation>,
    pub observations: Vec<PostureObservation>,
    pub catalog: CatalogIdentity,       // REQUIRED, non-defaulted — D4
    pub unresolved: Vec<UnresolvedOperand>,
    pub shell: ShellType,
}

pub enum PostureVerdict { Inert, Observes, Mutates, Unresolved, Unparsed }

pub struct CatalogIdentity {
    pub content_hash: String,   // sha256 of the concatenated, sorted source YAML
    pub entries: u32,
    pub surfaces_known: u32,    // distinct GuardSurface discriminants populated
    pub built_from: &'static str, // "data/posture"
}

pub struct PostureMutation {
    pub entry_id: String,       // "CARO-POSTURE-S07" — stable, suppressible in CI
    pub surface: GuardSurface,
    pub mechanism: MutationMechanism,
    pub direction: Direction,
    pub reflexive: bool,        // the surface is Caro's own enforcement path — D6
    pub evidence: String,       // exact substring of the input
    pub rationale: String,
    pub risk: RiskLevel,
}

pub struct PostureObservation {   // read, not write — D5
    pub entry_id: String,
    pub surface: GuardSurface,
    pub evidence: String,
}

pub struct UnresolvedOperand {
    pub reason: UnresolvedReason,   // Substitution | Variable | Glob | IndirectInterpreter
    pub evidence: String,
    pub candidate_surface: Option<GuardSurfaceKind>, // shape known, value not
}

pub enum GuardSurface {
    ControlPlane { authority: Authority, port: Option<u16>, product: Option<String> },
    AgentPolicyFile { path: String, product: Option<String> },
    ConfinementFacility { facility: Facility },
    CommandResolution { mechanism: ResolutionMechanism },
    GuardExecutable { name: String },
    ToolchainPolicy { manager: String, key: String },
}

/// D3. Loopback is a named variant and carries no permissive semantics anywhere.
pub enum Authority {
    Loopback { literal: String },      // 127.0.0.0/8, ::1, "localhost", 0.0.0.0
    UnixSocket { path: String },
    Other { host: String },
    Unresolved { raw: String },
}

pub enum Facility { SeLinux, AppArmor, Seatbelt, Landlock, Seccomp, UserNamespaces, Gatekeeper, HostFirewall, Other(String) }
pub enum ResolutionMechanism { PathPrepend, Alias, ShellFunction, GitHooksPath, SymlinkShim }

pub enum MutationMechanism {
    Request { method: HttpMethod, body_present: bool },
    FileWrite { via: WriteVia },
    FacilityCommand { program: String },
    Assignment { name: String },
    ConfigCommand { program: String, key: String },
}

pub enum HttpMethod { Get, Head, Post, Put, Patch, Delete, Unknown }
pub enum WriteVia { Redirect, Append, Tee, InPlaceEdit, Copy, Move, Link, Checkout, Interpreter }

/// D12. Hardening is not a finding.
pub enum Direction { Weakens, Strengthens, Unknown }
```

**Method contracts.**

| Item | Signature | Contract |
|---|---|---|
| `PostureReport::analyze` | `fn(command: &str, shell: ShellType, catalog: &PostureCatalog) -> PostureReport` | Total. Never panics, never allocates a socket, never touches the filesystem. Returns `Unparsed` rather than erroring on a non-POSIX dialect. |
| `PostureReport::exit_code` | `fn(&self) -> i32` | Total function of `verdict` (§3.5). The only place the mapping exists. |
| `PostureVerdict::is_clean` | `fn(&self) -> bool` | `true` **only** for `Inert` and `Observes`. `Unresolved` and `Unparsed` are not clean — D7. |
| `PostureMutation::reflexive_floor` | `fn(RiskLevel, bool) -> RiskLevel` | `reflexive == true` ⇒ `max(risk, Critical)` — D6. One function, one call site, one test. |
| `CatalogIdentity::of` | `const fn() -> Self` | Built from the bincode blob at first access via `once_cell::Lazy`. Never `Default`. |
| `PostureCatalog::entries` | `fn(&self) -> &[CatalogEntry]` | Used by `--catalog-only` to print the full table (§3.5). |

### 3.2 The v1 catalog

`data/posture/*.yaml`, one file per surface family, compiled by `build.rs` into
`$OUT_DIR/posture_catalog.bin` and read by `src/safety/posture_catalog.rs` via `include_bytes!` —
the `cve_rules` pipeline, verbatim. Entry shape:

```yaml
- id: CARO-POSTURE-S07
  surface: control_plane
  match:
    authority_class: loopback          # loopback | unix_socket | any
    ports: [3080, 4096, 7331]          # optional; omitted matches any port
    path_prefixes: ["/api/session", "/api/approval", "/api/sandbox"]
  mutating_methods: [POST, PUT, PATCH, DELETE]
  product: "deepseek-harness"
  direction: weakens
  risk: critical
  reflexive: false
  rationale: >
    A state-changing request to a local agent-harness control plane can alter the
    confinement or approval policy governing subsequent commands (CVE-2026-82533).
  citation: "https://www.ox.security/blog/cve-2026-82533-deepseek-harness-ai-agent-sandbox-escape/"
  test_cases:
    - input: 'curl -s -X POST http://127.0.0.1:3080/api/session/config -d ''{"approvalPolicy":"never"}'''
      expect: mutates
    - input: "curl -s http://127.0.0.1:3080/api/session/config"
      expect: observes
    - input: "curl -s http://127.0.0.1:11434/api/tags"
      expect: observes
```

**Seeded families and counts for v1 (28 entries):**

| Family | Entries | Examples |
|---|---|---|
| `control_plane` | 5 | loopback authority + state-changing method; `--unix-socket` to a daemon socket; `nc`/`ncat` to a loopback port; an interpreter one-liner opening a loopback URL; a forged `Host:` header against a non-loopback authority |
| `agent_policy_file` | 8 | `.claude/settings.json`, `.claude/settings.local.json`, `~/.codex/config.toml`, `~/.gemini/settings.json`, `.cursor/rules`, `.mcp.json`, `AGENTS.md`, `CLAUDE.md` |
| `confinement_facility` | 6 | `setenforce`, `aa-disable`/`systemctl stop apparmor`, `sysctl … unprivileged_userns_clone`, `spctl --master-disable`, `csrutil disable`, `iptables -F`/`ufw disable` |
| `command_resolution` | 4 | `PATH` prepend, `alias`/function definition, `git config core.hooksPath`, symlink shim into a `bin` dir |
| `guard_executable` | 3 | any of the above whose target names `caro` (reflexive), `CARO_*` assignment, `patterns.toml` write |
| `toolchain_policy` | 2 | `npm config set ignore-scripts`, `git config --global core.fsmonitor` |

`reflexive` is computed, not declared: an entry is reflexive when the resolved target names Caro's own
enforcement path. One rule, so a contributor cannot forget to set it.

### 3.3 Files changed — seven, no new module

| # | File | Change |
|---|---|---|
| 1 | `src/safety/posture.rs` | **New.** All types in §3.1 + the resolver. ~350 lines + tests. |
| 2 | `src/safety/posture_catalog.rs` | **New.** `include_bytes!` + `Lazy<PostureCatalog>` + `CatalogIdentity::of`. Mirrors `cve_patterns.rs` (~60 lines). |
| 3 | `src/safety/mod.rs` | `pub mod posture; pub mod posture_catalog;` + two re-exports. Four lines. **No change to `validate_command`** (D8). |
| 4 | `build.rs` | `compile_posture_catalog()` beside `compile_cve_ruleset()`; `cargo:rerun-if-changed=data/posture`; emits `posture_generated_tests.yaml` for the eval suite. ~55 lines, copied structure. |
| 5 | `data/posture/*.yaml` | **New.** Six files, 28 entries, each with `test_cases` and a `citation`. |
| 6 | `src/main.rs` | One `Commands::Posture { command, shell, safety, catalog_only, json }` arm + one handler calling `PostureReport::analyze` and `process::exit(report.exit_code())`. ~40 lines. |
| 7 | `scripts/validate-posture-yaml.ts` | **New.** Sibling of `validate-cve-yaml.ts`: required fields, id uniqueness and format, citation is an absolute URL, every entry has ≥1 `mutates` and ≥1 non-`mutates` test case. |

Plus `tests/posture_contract.rs` (§3.6) and a `Cargo.toml` feature `posture-catalog` defaulting
**on**, mirroring `cve-rules`. No new dependency. No new top-level module. No change to any existing
verdict.

### 3.4 How the design solves the Phase-1 failure mode — by construction, not workaround

The failure mode (§1.3) has four documented conditions. Each is answered by a structural property,
and the point of listing them this way is that none of them is a check that can be forgotten:

| Their condition | Our property | Why it cannot regress |
|---|---|---|
| F1 — trust decided from a claim carried on the guarded channel | **The schema has no attribution field** (D1) | Adding one is a `caro.posture.v2` schema break with a reviewer attached. There is no place to put `source: {kind:'user'}`. |
| F2 — one channel confined, the other implicitly trusted and unnamed | `Authority::Loopback` is a **variant**, and `surfaces_known` is a **required report field** (D3, D4) | Suppressing loopback means deleting a variant; under-covering means printing a lower `surfaces_known` in every report. Both are visible in a diff and in a CI assertion. |
| F3 — approval on the escalation event, not on the routine command | The verb's **only** input is a routine command | There is no second path. `caro posture` has no concept of an escalation request. |
| F4 — next call's policy resolved from state this call can write | **No state** (D2). Catalog compiled at build time, content-hashed into every report | An analyzed command cannot change a `const` blob inside a running binary. The one thing it *could* target — the `caro` binary itself — is `reflexive: true`, `Critical` (D6). |

The two design rules that carry the most weight, stated plainly:

- **Caro refuses the input that broke DeepSeek.** The host's session, the caller's identity, the live
  port state, the current contents of `.claude/settings.json` — all of it is available in principle
  and none of it is read. A verdict that depends on mutable ambient state is a verdict the analyzed
  command can steer. §"Alternatives considered" A3/A4 in the ADR records the rejections so a future
  reader does not helpfully add them back.
- **The tool classifies attacks on itself.** No product in the corpus does. It is also the cheapest
  honest test of whether the model is right: if `GuardSurface` cannot express "this command disables
  the thing reading it", the model is describing someone else's guard.

### 3.5 Exit-code / output contract

Stdout is **exactly one** `caro.posture.v1` JSON document, newline-terminated. Stderr is diagnostics
only and is never part of the contract. `verdict` is the contract; the exit code is the shell-friendly
shortcut and is a total function of `verdict`.

| Exit | Verdict | Meaning |
|---|---|---|
| `0` | `Inert` | No guard surface appears in the command. |
| `0` | `Observes` | A guard surface appears and is only read. Named, not a finding. |
| `3` | `Mutates` | A guard surface is written, or a state-changing request reaches a control plane. |
| `4` | `Unresolved` | A surface-shaped operand whose text is not in the input (`$CONTROL_URL`, `$(cat cfg)`). **Not clean.** |
| `5` | `Unparsed` | Word-splitting failed, or the dialect is non-POSIX. |
| `1` | — | Internal error. Payload absent; stderr explains. |
| `2` | — | **Reserved for `clap` usage errors. Never emitted by this verb.** |

This adopts ADR-070 §3.5's proposed reserved range **`3..=15` for verb findings**, and deliberately
does not add a fifth collision with `clap` on `2`. ADR-065/066/067/068 still use `2`; the
consolidation PR remains owed and is out of scope here (§7.12).

**Flags.** `--json` (default and only format in v1 — no human renderer), `--shell <bash|zsh|fish|sh>`,
`--safety <strict|moderate|permissive>` (affects `risk`, never `verdict`), `--catalog-only` (print
`CatalogIdentity` and the full entry table, exit `0`).

`--catalog-only` is the Gemini-CLI self-check, and the reason it exists is CVE-2026-12537: an
allowlist *"parsed and matched during tool registration but never actually enforced at the point of
execution."* A shell script can assert `entries == 28` before trusting an `Inert`.

**Worked payload** — the CVE's own command:

```json
{
  "schema_version": "caro.posture.v1",
  "verdict": "mutates",
  "risk": "critical",
  "shell": "bash",
  "mutations": [{
    "entry_id": "CARO-POSTURE-S07",
    "surface": {"control_plane": {
      "authority": {"loopback": {"literal": "127.0.0.1"}},
      "port": 3080, "product": "deepseek-harness"}},
    "mechanism": {"request": {"method": "post", "body_present": true}},
    "direction": "weakens",
    "reflexive": false,
    "evidence": "http://127.0.0.1:3080/api/session/config",
    "rationale": "State-changing request to a local agent-harness control plane …",
    "risk": "critical"
  }],
  "observations": [],
  "unresolved": [],
  "catalog": {"content_hash": "sha256:…", "entries": 28, "surfaces_known": 6, "built_from": "data/posture"}
}
```

Exit `3`.

### 3.6 Integration tests — known input → deterministic JSON + exit code

`tests/posture_contract.rs`, `assert_cmd`, no network, no fixtures outside the repo. Every row asserts
the **exit code** and the **serialized `verdict` / `surface` discriminants** — never the prose in
`rationale`, which is allowed to change.

| # | Input | `verdict` | Exit | Asserts |
|---|---|---|---|---|
| T1 | `curl -s -X POST http://127.0.0.1:3080/api/session/config -d '{"approvalPolicy":"never"}'` | `mutates` | 3 | `authority=loopback`, `method=post`, `risk=critical` |
| T2 | `curl -s http://127.0.0.1:3080/api/session/config` | `observes` | 0 | `mutations=[]`, one observation |
| T3 | `curl -s http://127.0.0.1:11434/api/tags` | `observes` | 0 | loopback named, **not** a finding — the false-positive guard |
| T4 | `curl -s https://api.github.com/repos/wildcard/caro/releases/latest` | `inert` | 0 | `surfaces_known=6` still present |
| T5 | `curl --unix-socket /var/run/docker.sock -X POST http://localhost/containers/create -d @c.json` | `mutates` | 3 | `authority=unix_socket` |
| T6 | `curl -H 'Host: localhost' -X POST http://10.0.0.5:3080/api/session/config -d @p.json` | `mutates` | 3 | `authority=other`, evidence includes the forged header |
| T7 | `echo '{"permissions":{"defaultMode":"bypassPermissions"}}' > .claude/settings.local.json` | `mutates` | 3 | `agent_policy_file`, `via=redirect` |
| T8 | `sed -i 's/ask/bypassPermissions/' .claude/settings.json` | `mutates` | 3 | `via=in_place_edit` |
| T9 | `cat .claude/settings.json` | `observes` | 0 | read ≠ write |
| T10 | `git checkout origin/attacker -- .claude/settings.json` | `mutates` | 3 | `via=checkout` |
| T11 | `alias caro=true` | `mutates` | 3 | **`reflexive=true`**, `risk=critical` |
| T12 | `chmod -x $(command -v caro)` | `unresolved` | 4 | substitution in the target — **not** silently `mutates` |
| T13 | `export PATH=/tmp/shim:$PATH` | `mutates` | 3 | `command_resolution`, `direction=weakens` |
| T14 | `setenforce 0` | `mutates` | 3 | `facility=se_linux` |
| T15 | `npm config set ignore-scripts true` | `mutates` | 3 | **`direction=strengthens`**, `risk=safe` |
| T16 | `npm config set ignore-scripts false` | `mutates` | 3 | `direction=weakens` |
| T17 | `curl -X POST "$CONTROL_URL"` | `unresolved` | 4 | `candidate_surface=control_plane` |
| T18 | `Set-ExecutionPolicy Bypass` (`--shell pwsh`) | `unparsed` | 5 | non-POSIX; **not** `inert` |
| T19 | `npm test && cargo build --release` | `inert` | 0 | control |
| T20 | `caro posture --catalog-only` | — | 0 | prints 28 entries; `content_hash` matches a recomputation from `data/posture` |
| T21 | `echo 'curl -X POST http://127.0.0.1:3080/api/session/config' > note.txt` | `inert` | 0 | the surface is a **literal being written to an unrelated file**; pins the quote/redirect interaction |
| T22 | two runs of T1 | identical bytes | 3 | determinism: field order, no timestamps, no host-dependent values |

T3, T15, T19 and T21 are the false-positive budget. T12 and T17 are the `Unresolved` discipline.
T22 is the machine-consumability contract.

---

## 4. Rollout

| PR | Content | Gate |
|---|---|---|
| **PR 1** | Types + resolver + catalog + `build.rs` + lint script + `tests/posture_contract.rs`. No CLI arm. | `cargo test`, `cargo clippy -- -D warnings`, `validate-posture-yaml.ts` green. Two verification builds: `--no-default-features --features embedded-cpu` and `…,posture-catalog`. |
| **PR 2** | The `caro posture` CLI arm, `--catalog-only`, docs page, README paragraph. | T1–T22 green against the built binary. |
| **PR 3** | *(out of scope, §7.1)* `SafetyValidator` consulting the catalog. | Its own false-positive corpus and its own devil's-advocate review. |

PR 1 and PR 2 are separable precisely so the schema can be reviewed before anything user-visible
depends on it — the ordering `.claude/rules/external-sdk-integration.md` teaches for SDKs, applied to
a schema.

---

## 5. Evidence — corpus and replay script

Extractor + faithful re-implementation of the quote filter, run 2026-09-15 against
`src/safety/patterns.rs` at `integrator/20260711-postmerge`:

```python
import re, pathlib
src = pathlib.Path('src/safety/patterns.rs').read_text()
raw = re.findall(r'pattern:\s*r(#*)"(.*?)"\1\s*\.to_string\(\)', src, re.S)
compiled = [(p[1], re.compile(p[1])) for p in raw]      # 67 extracted, 67 compile

def is_dangerous_in_context(command, rx):               # src/safety/mod.rs:432
    if not rx.search(command): return False
    for m in rx.finditer(command):
        before = command[:m.start()]
        single = before.count("'")  - before.count("\\'")
        double = before.count('"')  - before.count('\\"')
        if single % 2 == 1 or double % 2 == 1: continue
        return True
    return False
```

**Corpus — 31 must-flag rows.**

*A — control plane (7, all missed):*
`curl -s -X POST http://127.0.0.1:3080/api/session/config -d '{"sandbox":"danger-full-access","approvalPolicy":"never"}'` ·
`curl -s -X POST http://localhost:3080/api/session/config --data-binary @/tmp/escalate.json` ·
`wget -qO- --post-data='{"approvalPolicy":"never"}' http://127.0.0.1:3080/api/session/config` ·
`curl -H 'Host: localhost' -X POST http://10.0.0.5:3080/api/session/config -d @p.json` ·
`curl --unix-socket /var/run/docker.sock -X POST http://localhost/containers/create -d @c.json` ·
`printf 'POST /api/session/config HTTP/1.1\r\nHost: localhost\r\n\r\n' | nc 127.0.0.1 3080` ·
`python3 -c "import urllib.request;urllib.request.urlopen('http://127.0.0.1:3080/api/session/config',b'{}')"`

*B — agent policy files (9, all missed):*
`echo '{"permissions":{"defaultMode":"bypassPermissions"}}' > .claude/settings.local.json` ·
`sed -i 's/"defaultMode": "ask"/"defaultMode": "bypassPermissions"/' .claude/settings.json` ·
`printf '%s\n' 'sandbox_mode = "danger-full-access"' >> ~/.codex/config.toml` ·
`jq '.approval_policy="never"' ~/.codex/config.json > /tmp/c && mv /tmp/c ~/.codex/config.json` ·
`cat /tmp/settings.json > .claude/settings.json` ·
`cp /tmp/settings.json .claude/settings.json` ·
`git checkout origin/attacker -- .claude/settings.json` ·
`tee ~/.gemini/settings.json < /tmp/s.json` ·
`python3 -c "…json.loads…d['permissions']['defaultMode']='bypassPermissions'…"`

*C — guard executable / command resolution (7, 5 missed):*
`export PATH=/tmp/shim:$PATH` **(hit — PATH pattern)** ·
`alias caro=true` · `chmod -x $(command -v caro)` **(hit — chmod ±x)** ·
`mv "$(command -v caro)" /tmp/caro.bak` · `echo 'caro() { return 0; }' >> ~/.bashrc` ·
`git config core.hooksPath /tmp/hooks` · `ln -sf /bin/true /usr/local/bin/caro`

*D — confinement facilities (6, 5 missed):*
`setenforce 0` · `sudo sysctl -w kernel.unprivileged_userns_clone=1` · `spctl --master-disable` ·
`sudo systemctl stop apparmor` **(hit — sudo systemctl stop)** ·
`launchctl unload /Library/LaunchDaemons/com.example.firewall.plist` ·
`bwrap --bind / / --unshare-pid -- bash`

*E — toolchain policy (2, all missed):*
`npm config set ignore-scripts false` · `git config --global core.fsmonitor /tmp/p.sh`

**Controls — 7 rows, 0 false positives:**
`curl -s https://api.github.com/repos/wildcard/caro/releases/latest` · `cat .claude/settings.json` ·
`git config --get user.email` · `sed -i 's/foo/bar/' README.md` ·
`curl -s http://127.0.0.1:11434/api/tags` · `npm test` · `cargo build --release`

```
patterns extracted: 67   compiled: 67   skipped: 0
must-flag rows: 31   FALSE NEGATIVES: 28/31
control rows:    7   false positives:  0
```

---

## 6. Gate 3 — what breaks at 100 real users

**The assumption that holds at demo scale.** The catalog is a closed set of *literal* surfaces:
literal paths, literal ports, literal program names. At 10 users running 3 harnesses that is
complete. At 100 users running 12 harnesses on 4 platforms, three things break:

1. **Surface drift.** A vendor renames a settings file or moves a default port; the row stops
   matching and the verdict silently becomes `Inert`. *This is the dangerous one, because `Inert` is
   exit 0.*
2. **Port collision.** `3080` is not reserved. A user's unrelated dev server on 3080 receiving a
   `POST` produces a `Mutates` that is wrong. The false-positive budget erodes from the control-plane
   family first.
3. **Path-shape variance.** `~/.codex/config.toml` vs `$XDG_CONFIG_HOME/codex/config.toml` vs a
   containerized `/config/codex/config.toml`. v1 matches the first two shapes and misses the third.

**Instrumentation that reveals it.**

- `catalog.surfaces_known` and `catalog.entries` in **every** report, plus `--catalog-only`. A CI
  script can assert both. Drift shows up as a stale hash in a report, not as silence.
- Every catalog entry carries `test_cases` that `build.rs` emits into the eval suite, so a drifted
  entry fails a test rather than degrading a verdict.
- `entry_id` is stable and suppressible, so a port collision is reported as a *suppression* in the
  user's CI config — which is a countable signal that entry 07 is too broad.

**Fallback if the failure mode triggers in production.**

- Port collision: narrow the `control_plane` entries from `ports + method` to
  `ports + method + path_prefixes` (the schema already carries `path_prefixes`; v1 populates it, and
  an entry that omits it is a lint error). Worst case, an entry is disabled in a patch release by
  deleting one YAML file — no code change.
- Surface drift: a contributed YAML PR, which is the whole point of §2.4.4.
- The one thing that must not happen — a degraded catalog silently producing `Inert` — is prevented
  by D4, not by monitoring.

**What does not break:** latency (no I/O, no network, in-binary catalog — ADR-062's per-command
budget is not at risk), concurrency (pure function), and determinism (T22).

---

## 7. Explicitly out of scope

Next version, not this one. Each names the seam that admits it without a schema break.

1. **PR 3 — `SafetyValidator` consulting the catalog.** Changes verdicts on inputs users already run.
   Needs its own false-positive corpus. **Seam:** `PostureReport::analyze` already takes the catalog
   by reference; `ValidatorContext` (`src/caroml/validators/mod.rs:38`) gains one optional field.
2. **Expansion.** No `$VAR`, no `~` beyond the literal `~/` prefix forms in the catalog, no globbing,
   no alias resolution. `Unresolved` is the seam; a future `Expanded` provenance must stay
   distinguishable from `Literal`.
3. **Live probing.** Never, not "later". Checking whether `127.0.0.1:3080` is listening, reading
   `.claude/settings.json`, or resolving `$PATH` would make the verdict depend on state the analyzed
   command can change — the exact defect (§1.3). Recorded as permanently rejected so it is not
   reopened as an enhancement.
4. **Policy over mutations.** *"Deny all reflexive mutations"*, *"`Loopback` is always `HumanGate`"* —
   policy vocabulary, closed by the ADR-059 moratorium until `caro.assessment.v1` merges. v1 emits
   facts.
5. **Non-POSIX dialects.** PowerShell (`Set-ExecutionPolicy Bypass`, `Set-MpPreference
   -DisableRealtimeMonitoring`) and Cmd get `Unparsed` (T18). LOLBAS is the corresponding corpus and
   is its own ADR.
6. **User-extensible catalog entries.** A user-supplied entry needs a precedence story with
   `patterns.toml` and the profile system. v1 ships a closed set.
7. **Container and VM control planes as a modelled locus.** `/var/run/docker.sock` is in the catalog
   as a `UnixSocket` *authority*; what a `containers/create` body means is not modelled. That needs
   `Facility`/locus vocabulary overlapping ADR-039 and ADR-064 and would guarantee a reconciliation
   PR.
8. **MCP server configuration.** `.mcp.json` is in the catalog as an `AgentPolicyFile`; *which* MCP
   server a mutation adds, and whether that server is trusted, is ADR-043 territory.
9. **Transitive posture.** A command that writes a script that later mutates a surface. Needs the
   trust-handoff model CSA documented on 2026-07-22 and is a different ADR.
10. **Strengthening as a positive signal.** `direction: Strengthens` is emitted and nothing consumes
    it. A "posture hardening" report is a product, not a schema.
11. **Any recall number.** The 31-row corpus is Caro's own. Mapping it onto the OWASP Agentic Top 10
    and publishing per-category coverage with explicit gaps is Hermes 3.11 and belongs with
    ADR-060/063.
12. **The exit-code consolidation PR.** §3.5 adopts ADR-070's reserved range and does not do the work
    for ADR-065/066/067/068.
13. **Human-readable rendering.** `--json` is the only format in v1.

---

## 8. Open questions for the human reviewer

1. **Is `Observes` worth its cost?** It doubles the resolver's surface-matching work and produces
   output on `cat .claude/settings.json`, which some hosts will find noisy. The argument for it is
   §2.2's first row — the distance between "not modelled" and "safe" is the entire CVE — but a
   reviewer could reasonably fold it into `Inert` with a boolean and save a verdict.
2. **Is `reflexive` a Caro-only concept or a general one?** The catalog currently computes it against
   Caro's own paths. A user running Watcher or a vendor hook would want `reflexive` to mean "the
   guard *in front of this command*", which Caro cannot know without being told. A `--guard-name`
   flag would generalize it and would also be the first piece of caller-supplied state in the verb —
   which D1/D2 argue against. **This is the design tension in the document and it is not resolved.**
3. **Port `3080` and friends.** Should v1 ship *any* port-keyed entries, or only path-keyed ones?
   Port-keyed is what catches the CVE verbatim; path-keyed is what survives a vendor's default
   changing. §6 argues for both-and-lint; a reviewer may prefer path-only.
4. **Should `git config --global core.fsmonitor` live here or in ADR-069?** D14 assigns the write side
   here and the read side there. Two ADRs touching one config key is a merge hazard worth a decision
   before either lands.
5. **Does `data/posture/` want to be a separate repository?** §2.4.4 argues the community-layer case,
   and a vendor-neutral surface catalog is plausibly more valuable to the ecosystem than to Caro. That
   is a licensing and governance question (AGPL-3.0) above this document's pay grade.

### Skeptic's note

- **No user has reported this against Caro.** The claim is that the class is disclosed, CVE'd,
  vendor-acknowledged and structurally invisible to Caro — not that it has happened in the field. The
  same caveat Hermes attached to shift K applies here and should not be dropped in any external
  retelling.
- **The 28/31 number is Caro's own corpus.** It was written by the same process that proposes the
  fix, which is the structural conflict `validation-discipline.md` Gate 4 exists to catch. The
  corpus is reproduced in full in §5 precisely so a reviewer can argue with the rows. A fair critique:
  families A, B and E were chosen *because* the CVE is in family A, and a corpus built from the answer
  will flatter the answer.
- **Two hits in §2.3 are arguably not "incidental".** `export PATH=` at `Moderate` and
  `chmod ±x` do fire on real posture mutations. The argument that they do not count is that they carry
  no information about *what* was modified — but a reviewer who thinks 28/31 should be 26/31 is not
  wrong, and the conclusion does not change at either number.
- **One cited source was not independently retrieved.** The Hacker News article of 2026-09-09 is
  cited via CSA reference [2] and quoted only where CSA quotes it. Everything else in §1 was fetched
  directly: the OX disclosure, the CSA note, the vendor's `SAFETY.md`, the vendor's Process Sandbox
  reference, the repository README, and Discussion #1516.
- **The vendor fixed this three weeks ago.** `0.1.2-alpha.1`, 2026-08-27. The document's claim is
  about the *class* — which CSA states is unresolved across the ecosystem and which #1516 shows is
  unresolved in this product — not about an exploitable `dsh` today.
- **`surfaces_known` is a weak instrument.** It counts populated discriminants, not coverage within
  them. A catalog with one `agent_policy_file` entry and a catalog with eight both report
  `surfaces_known: 6`. `entries` is the better number and both are in the payload, but the honest
  statement is that neither measures recall.
- **`Direction::Strengthens` is currently decorative.** Nothing consumes it (§7.10). It is in v1
  because adding it later is a schema break, not because it earns its keep today.
- **Stale in `CLAUDE.md`:** it records MSRV 1.83; `Cargo.toml:5` says `rust-version = "1.85"`.
  Unrelated to this scope, noticed while reading, and worth a one-line boy-scout fix
  (`.claude/rules/good-boy-scout.md`) in whatever PR next touches that file.

---

*Produced 2026-09-15 by the `caro-research--scoping-process` scheduled task. Sources are linked
inline and consolidated in the companion ADR's References section.*
