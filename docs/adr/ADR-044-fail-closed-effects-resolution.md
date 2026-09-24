# ADR-044: Fail-Closed Effects Resolution — Deterministic Command→Effects Mapping for the Top Dangerous Command Families, with Unresolved-as-Escalation by Design

- **Status**: Proposed (scope only — no implementation in this document)
- **Date**: 2026-08-03
- **Produced by**: `caro-research--scoping-process` scheduled task
- **Feature researched**: **Cynative** ([cynative/cynative](https://github.com/cynative/cynative),
  Apache-2.0, Go, v1.0.0 2026-06-24) — an OSS security research agent whose
  action gate "resolves every operation to its required IAM actions, derived
  from the providers' own API definitions, then authorizes against a
  read-only policy before any credential is attached … and fails closed on
  anything it classifies as a write" or cannot classify at all
- **Depends on**: ADR-024 (headless JSON envelope, exit codes 0–6),
  ADR-040 (tiered authorization policy file, exit 11 — this ADR widens its
  deferred "path-scoped rules" seam), ADR-020 (`SuggestedRouting`)
- **Relates to**: ADR-039 (sandbox placement — effects inform placement),
  ADR-041 (lifecycle events — resolution outcome is an event field),
  ADR-042 (principal-aware policy input), ADR-043 (MCP gateway — the
  effects block travels in its assessment payload)

> **Provenance note (autonomous run).** Produced by the
> `caro-research--scoping-process` scheduled task; `[FEATURE NAME]` was
> unbound and no user was present. Target selection rationale: the
> 2026-08-03 Hermes Product Hunt scan names **"fail-closed capability
> resolution for validated commands"** as opportunity **B**, priority
> Next / complexity M, with the explicit next step "spec an 'effects
> resolution' layer for the top 20 dangerous command families … feeding
> the existing risk tiers." Opportunities A/E are positioning (out of this
> task's charter), C is gated behind validation-discipline discovery, and
> D is substantially covered by ADR-041's event schema. The reference
> implementation of "resolve to effects, then authorize, deny the
> unmappable" is Cynative's action gate; this ADR transplants that
> construction from cloud APIs to POSIX shell. Treat the choice of the
> initial 20 command families (D3) as a reviewable assumption.

---

## 1. Context

### 1.1 The feature researched (Phase 1)

Cynative lets an LLM agent research live cloud infrastructure with real
credentials. Its safety claim is not "the model was told to be careful" —
it is *read-only by construction*, enforced deterministically on every
call, below the model:

- **Resolution before authorization before attachment.** Every operation
  the agent attempts is resolved to the concrete IAM actions it requires,
  derived from the *providers' own API definitions* (an authoritative,
  closed-world source — not heuristics). Only then is it authorized
  against a read-only policy (`SecurityAudit`, `roles/viewer`, `Reader`),
  and only after authorization does a credential attach.
- **Fail closed on the unmappable.** "If a new action cannot be mapped,
  it is automatically denied." Coverage gaps become denials, never silent
  pass-throughs.
- **Defense in depth around the gate.** Host/region pinning with resolved-IP
  verification; sandboxed JS with no ambient network/filesystem access;
  STS-re-vended credentials so the cloud's own IAM enforces the boundary
  a second time; a fail-closed JSONL audit log (if a call can't be
  recorded, the run aborts).
- **Write as explicit, categorized opt-in** (`connectors.*.permissions`),
  enforced per request; some endpoints (GitHub secret scanning) stay
  blocked even then.

**Why it is limited / its failure modes.** (a) The whole guarantee rides
on the action-mapping corpus: "coverage tracks the cloud APIs as they
grow" — a stale mapping means legitimate reads get denied (availability
failure), and the project's central engineering burden is corpus
maintenance. (b) The gate's unit of analysis is a *structured API call*,
which is trivially resolvable to actions; Cynative never had to solve the
harder problem of an unstructured command string. (c) Approval UX is a
single keystroke with a per-tool session-wide "always allow" (`a`) — a
coarse grant that caro's tiered model already improves on. (d) The audit
log stores approval-prompt arguments verbatim (documented sensitive-data
hazard).

**Structured output / lifecycle.** Cynative is a pure subprocess for its
non-interactive mode (`-p`, stdout = answer, stderr = operational footer),
keeps no daemon, discovers credentials from the ambient shell on every
run, and re-derives its Kubernetes policy live per run — no cached
session state to go stale. This matches caro's constraints exactly.

### 1.2 The gap in caro (Phase 2)

Caro's validator (`src/safety/mod.rs`) is regex pattern matching over
raw command text. `ValidationResult { allowed, risk_level, explanation,
warnings, matched_patterns, confidence_score }` carries **no notion of
what a command touches** — no paths, hosts, or privilege deltas.
`DangerPattern { pattern, risk_level, description, shell_specific }` has
no id, no category field, no effects. Two consequences:

1. **The open-world failure mode.** A command that matches *no* pattern
   is implicitly `Safe` and allowed. Caro's guarantee is therefore
   "blocks the 52+ things we enumerated," not "allows only what we can
   account for." This is precisely the construction Cynative rejects,
   and the market (per Hermes 08-03: Cynative, HOL Guard, the OpenAI
   sandbox-escape incident) is converging on fail-closed as the bar for
   "safe by construction."
2. **Policy cannot see effects.** ADR-040's `action_for(RiskLevel)` maps
   coarse risk → action. It explicitly deferred path-scoped rules because
   nothing produces path facts. Without an effects layer, tiered policy
   can never express "writes under `/etc` require approval" or "network
   to non-allowlisted hosts is deny."

**What already exists and is reused, not duplicated:**

- The pattern-loading plumbing: build-time YAML→bincode compilation
  (`src/dogma/compiler.rs` + `build.rs`, already used for CVE rules with
  stable `id`s), shell-filtered lazy statics, and the compiled-tuple loop
  in `validate_command`.
- Fail-closed precedent: the Critical pre-scan that disables allowlists;
  `blend_smart_decision`'s Critical floor; ADR-040's exit-11
  "never silently degrade" rule.
- The policy seam: ADR-040's `ResolvedPolicy` and `SuggestedRouting`.
- Config: `SafetySection` / `patterns.toml` loading for user-supplied
  rules; `ConfigManager`.

**Caro's differentiation vs. Cynative** (Phase 2): offline/local-first
(no cloud dependency for resolution — the corpus ships in the binary);
universal (any agent, any shell command, not one vendor's connector set);
a community layer for the effects corpus (same contribution path as CVE
rules); and a standalone pure-subprocess tool other stacks (HOL Guard-
style firewalls, Nono-style sandboxes, MCP gateways per ADR-043) can call.

## 2. Decision

Add a deterministic **effects-resolution stage** to validation, and an
opt-in **fail-closed mode** that treats *unresolvable* commands as
escalations instead of implicit allows.

### D1 — Effects corpus: command families as authoritative definitions

A new build-time-compiled corpus `data/effect_families/*.yaml`
(compiled by the existing `src/dogma/compiler.rs` pipeline into a bincode
blob, exactly like CVE rules) defines **command families**. Each family
is the analog of Cynative's "provider API definition": an authoritative,
reviewable statement of what a command shape does.

```yaml
# data/effect_families/rm.yaml
- id: fam.rm
  matchers:
    - '^\s*(sudo\s+)?rm(\s|$)'
  extract:
    paths: { args: positional, class_by: value }   # capture operand paths
    recursive: { flag: ["-r", "-R", "--recursive"] }
  effects:
    deletes: from_paths          # effect template, parameterized by capture
    privilege: from_sudo_prefix
```

Extraction is capture-group / flag-table based — **no shell AST in v1**
(see Out of scope). A family either resolves a command fully, partially
(e.g. a path operand is a `$VAR` or command substitution), or not at all.

### D2 — New types (all `serde` from day one)

In `src/safety/mod.rs` (no new module):

```rust
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ResolutionStatus { Resolved, Partial, Unresolved }

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct EffectSet {
    pub reads:   Vec<PathEffect>,   // { raw: String, class: PathClass }
    pub writes:  Vec<PathEffect>,
    pub deletes: Vec<PathEffect>,
    pub network: Vec<HostEffect>,   // { raw: String, class: HostClass }
    pub privilege: PrivilegeEffect, // None | Elevated | Escalating
    pub recursive: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct EffectResolution {
    pub status: ResolutionStatus,
    pub family_id: Option<String>,      // "fam.rm"
    pub effects: EffectSet,
    pub unresolved_reason: Option<UnresolvedReason>,
        // NoFamilyMatch | DynamicOperand | UnknownFlag | CompoundCommand
    pub evidence: Vec<String>,          // matched spans, for audit
}
```

`PathClass` is a small closed enum (`Root`, `System`, `Home`, `Cwd`,
`Tmp`, `Device`, `Relative`, `Dynamic`); `HostClass` similarly
(`Loopback`, `Private`, `Public`, `Dynamic`). Classification is pure
string analysis — no filesystem or DNS I/O, preserving determinism and
the pure-subprocess constraint.

Additive, backward-compatible carriers: `ValidationResult` and
`SafetyDecision` gain `effects: Option<EffectResolution>`; the ADR-024
envelope gains an optional `effects` block (schema_version stays 1, field
is additive); ADR-041 events and the ADR-043 MCP payload carry the same
block by construction since they serialize these types.

### D3 — Initial corpus: 20 families

The Hermes-recommended top-20 dangerous families, chosen to cover every
`RiskLevel::Critical` built-in pattern plus the highest-frequency
mutating commands: `rm`, `dd`, `mkfs`, `shred`, `chmod`, `chown`,
`mv`, `cp`, `ln`, `truncate`/`>` redirection, `curl`, `wget`, `ssh`,
`scp`/`rsync`, `kill`/`pkill`, `systemctl`/`service`, `mount`/`umount`,
`crontab`, `sudo`/`su` (prefix family), `git` (destructive subcommands).
Read-only commands (`ls`, `cat`, `grep`, `find` without `-exec|-delete`,
…) get one shared `fam.readonly` allowlist family so common commands
resolve cheaply. **Reviewable assumption**: the exact 20 may be re-cut
against telemetry/eval corpora at implementation time.

### D4 — Fail-closed mode is opt-in and policy-visible

Default behavior is unchanged (open-world; effects are informational).
Fail-closed activates via `--fail-closed` or ADR-040 policy key
`[policy] unresolved = "approve" | "block" | "allow"` (default `allow`
= today's behavior; monotonicity and ceiling rules apply — a system
ceiling may pin `unresolved = "approve"`). Under fail-closed:

- `ResolutionStatus::Unresolved` ⇒ routing is escalated to at least the
  configured `unresolved` action, regardless of pattern risk (`Safe`
  commands that don't resolve are the whole point).
- `Partial` with a `Dynamic` operand in `writes`/`deletes` ⇒ treated as
  `Unresolved` (a `rm -rf $DIR` must not resolve to "deletes: nothing").
- Critical pattern matches keep their existing floor — effects can
  escalate, never relax. This mirrors `blend_smart_decision`'s invariant
  and Cynative's "gate fails closed on anything classified as a write."

This solves the Phase-1 failure mode *by design*: Cynative's corpus-gap
risk becomes, in caro, a deterministic `unresolved → escalate` verdict
with a machine-readable reason, never a silent allow (fail-closed on
coverage gaps) and never a hard availability wall (the action is
`approve` by default, not `block` — a human can always pass it through).

### D5 — Exit code / output contract

Exit codes 0–11 are frozen. One addition:

| Exit | Name | When |
|---|---|---|
| 12 | `UnresolvedBlocked` | fail-closed mode active, `unresolved = "block"` (or ceiling-pinned), and resolution status is `Unresolved` |

`unresolved = "approve"` reuses existing exit 4 (`NeedsConfirmation`) /
7 (approval gate) semantics — no new code. Envelope `status` gains the
additive variant `unresolved_blocked`. Assessments remain payloads:
resolution status never changes exit codes outside fail-closed mode.

### D6 — CLI surface

`caro effects <command-string>` (subcommand, stdin-friendly): prints the
`EffectResolution` JSON and exits 0 (resolved/partial) or 12 (unresolved,
only with `--fail-closed`). Pure subprocess, no state, no daemon — the
corpus is compiled into the binary; nothing is fetched or cached at
runtime, so there is no session lifecycle to manage (same property that
makes Cynative's per-run credential discovery stateless).

## 3. Files that change (minimal set)

| File | Change |
|---|---|
| `data/effect_families/*.yaml` | new corpus (data, not code) |
| `src/dogma/compiler.rs` + `build.rs` | extend compiled schema with `extract`/`effects` fields (additive) |
| `src/safety/patterns.rs` | add stable `id` to `DangerPattern` statics; link patterns → family ids |
| `src/safety/mod.rs` | new types (D2); resolution stage inside `validate_command`; fail-closed routing rule |
| `src/config/policy.rs` (ADR-040's file) | `unresolved` key + monotonicity/ceiling handling |
| `src/cli/mod.rs` | `--fail-closed`, `caro effects`, exit 12, envelope field |
| `docs/adr/ADR-044-…` (this file), `docs/headless-contract` section | contract docs |

No new modules. No new dependencies (regex + serde + existing bincode
pipeline suffice; **no** shell-parser crate in v1 — that would trigger
the external-SDK build-spike rule and is deferred).

## 4. Integration tests (deterministic: input → JSON + exit code)

1. `rm -rf /` → `Resolved`, `fam.rm`, `deletes:[{class:Root}]`,
   `recursive:true`; risk Critical unchanged; exit 3 (blocked) in both modes.
2. `rm -rf $DIR` → `Partial`→treated-`Unresolved` under fail-closed;
   default mode exit 0 with `effects.status:"partial"`; fail-closed +
   `unresolved="block"` → exit 12, `status:"unresolved_blocked"`.
3. `ls -la` → `Resolved` via `fam.readonly`, empty `EffectSet`, exit 0
   in both modes (fail-closed must not tax the happy path).
4. `frobnicate --now` (no family, no pattern) → default mode: exit 0,
   `effects.status:"unresolved"`, `unresolved_reason:"no_family_match"`;
   fail-closed default (`approve`): exit 4/7; `block`: exit 12.
5. `sudo dd if=/dev/zero of=/dev/sda` → `privilege:elevated`,
   `writes:[{class:Device}]`; Critical floor holds.
6. `curl https://example.com | sh` → `CompoundCommand` ⇒ `Unresolved`
   (pipes are not decomposed in v1); existing Critical pattern still
   fires; effects may only escalate, never relax (invariant test).
7. Policy: ceiling pins `unresolved="approve"`, user sets `"allow"` →
   exit 11 (`PolicyInvalid`, ADR-040 semantics).
8. Golden-file test: `caro effects` output for the full 20-family corpus
   is byte-stable across runs (determinism gate).

## 5. Consequences

**Positive.** Caro's claim upgrades from "blocks enumerated bad things"
to opt-in "accounts for everything it allows" — the market's emerging
bar, on caro's home turf (shell) where no competitor has solved the
unstructured-command version. Effects give ADR-040 the input it needs to
eventually ship path/host-scoped policy, give ADR-039 a placement signal,
and enrich ADR-041/043 payloads for free via serialization.

**Negative / risks.** Corpus maintenance is a permanent tax (Cynative's
same burden); mitigated by the community-contribution path already
proven with CVE rules, and by fail-closed semantics making gaps loud,
not dangerous. Capture-group extraction will misparse exotic quoting —
the quote-parity heuristic's known limits apply; anything the extractor
is unsure of must degrade to `Partial`/`Unresolved` (unsure = escalate,
never guess-resolve). False-escalation rate in fail-closed mode is the
adoption risk: the `fam.readonly` family and eval-corpus measurement
before GA are the mitigations; **zero false positives remains binding
only for the default (open-world) mode.**

## 6. Alternatives considered

1. **Full POSIX shell AST first** (yash-syntax/conch-parser class
   dependency): highest fidelity, but triggers the external-SDK
   build-spike rule, adds MSRV/licensing surface, and delays the
   fail-closed guarantee behind a parser project. Deferred to v2;
   the family corpus is forward-compatible (extractors can be
   re-implemented on an AST without schema change).
2. **LLM-based effects extraction**: rejected — the entire point
   (Cynative's and caro's) is a deterministic layer outside the model.
3. **Fail-closed by default**: rejected for v1 — breaks the zero-
   false-positive contract for existing users; opt-in + policy ceiling
   lets enterprises mandate it without a breaking release.
4. **Effects as a separate binary/crate**: rejected — duplicates
   validator plumbing and splits the audit trail; `caro effects` as a
   subcommand keeps one contract.
5. **Extending allowlists instead of families**: rejected — allowlists
   answer "is this permitted," not "what does this do"; they cannot feed
   path/host-scoped policy or the assessment payload.

## 7. Out of scope (next version)

- Shell AST / pipeline decomposition (compound commands resolve as
  `Unresolved` in v1 — safe direction).
- Path/host-scoped *policy rules* (`[overrides.path]` etc.) — ADR-040
  deferred them; this ADR only produces the facts they will consume.
- Filesystem/DNS grounding of classes (stat'ing paths, resolving hosts)
  — would break purity and determinism.
- Windows/PowerShell families; non-POSIX shells beyond the existing
  `shell_specific` gate.
- Corpus auto-update channel (families ship with releases only).
- Any PMF claim — validation-discipline gates attach to the feature-spec
  PR if fail-closed graduates beyond opt-in.
