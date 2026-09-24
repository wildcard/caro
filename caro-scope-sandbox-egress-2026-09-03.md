# Scope — `caro.egress.v1` (ADR-064)

**Date**: 2026-09-03 · **Produced by**: `caro-research--scoping-process` (autonomous run)
**Decision document**: `docs/adr/ADR-064-sandbox-egress-conjunction-gate.md`
**Baseline**: `caro` 1.4.0, tree as of `integrator/20260711-postmerge`

This document holds the implementation detail the ADR references. It commits no code.

---

## 1. One-paragraph statement

Claude Code's sandbox evaluates filesystem reads and network egress as two independent layers.
Read is allow-by-default across the whole machine — the docs name `~/.aws/credentials` and
`~/.ssh/` explicitly — and the network allowlist is decided from the client-supplied hostname
without body inspection. A command that reads a credential and posts it to an allowlisted host is
therefore *allowed by every layer simultaneously*, and the docs delegate the cross-check to the
operator in a sentence. `caro egress` performs that cross-check deterministically, before
execution, as a pure subprocess call, by reading the operator's own sandbox configuration.

---

## 2. New types

All in `src/models/mod.rs` unless noted. Every type derives at minimum
`Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema`. Enums use
`#[serde(rename_all = "snake_case")]` and are `#[non_exhaustive]`.

### 2.1 Profile ingestion

```rust
/// Read-only view of someone else's sandbox configuration. Caro owns no field here.
pub struct SandboxProfile {
    pub origin: ProfileOrigin,          // ClaudeCode { path } | Srt { path } | Unprotected
    pub enabled: bool,                  // default false  (Claude Code default)
    pub fail_if_unavailable: bool,      // default false
    pub allow_unsandboxed_commands: bool, // default true
    pub excluded_commands: Vec<String>,
    pub filesystem: ProfileFilesystem,
    pub network: ProfileNetwork,
    pub credentials: ProfileCredentials,
    #[serde(default)] pub unknown_keys: Vec<String>, // recorded, never fatal
}

pub struct ProfileFilesystem {
    pub allow_read: Vec<String>, pub deny_read: Vec<String>,
    pub allow_write: Vec<String>, pub deny_write: Vec<String>,
    pub disabled: bool,             // default false
}

pub struct ProfileNetwork {
    pub allowed_domains: Vec<String>, pub denied_domains: Vec<String>,
    pub strict_allowlist: bool,     // default false
    pub tls_terminate: bool,        // presence of the object, not its contents
}

pub struct ProfileCredentials {
    pub files: Vec<CredentialEntry>, pub env_vars: Vec<CredentialEntry>,
}

pub struct CredentialEntry {
    pub key: String,                       // path or env var name
    pub mode: CredentialMode,              // Deny | Mask
    pub inject_hosts: Vec<String>,
    pub on_extract_no_match: ExtractMiss,  // Warn (default) | Deny | Error
}
```

**Method contracts**

| Method | Signature | Contract |
|---|---|---|
| `from_claude_settings` | `fn(&Path) -> Result<Self, ProfileLoadError>` | Reads the `sandbox` object. Missing → `unprotected()` with `origin: ClaudeCode`. Unknown keys → `unknown_keys`, never an error. Malformed JSON → `Err` |
| `from_srt_settings` | `fn(&Path) -> Result<Self, ProfileLoadError>` | Same leniency. `srt`'s inverted read/write precedence is normalized on ingest and the difference recorded in `unknown_keys` |
| `unprotected` | `fn() -> Self` | The documented Claude Code default posture, named — not an empty struct with permissive semantics |
| `admits_host` | `fn(&self, host: &str) -> HostDisposition` | `Allowed` \| `Denied` \| `Unlisted { strict: bool }`. Honours `*.` and bare `*` wildcards; **never widens** an ambiguous entry (mirrors the vendor's own allowlist rule) |
| `protects_path` | `fn(&self, path: &Path) -> bool` | True if covered by `deny_read` or a `credentials.files` entry. Narrower `deny` wins inside a wider `allow` |

`ProfileLoad { Ok, Missing, Failed { reason } }` is carried on the report; `Failed` forces
`Indeterminate` (ADR-064 D2/D4) and is never coerced to `unprotected()`.

### 2.2 The pair

```rust
pub struct EgressPair { pub source: SourceKind, pub sink: SinkKind, pub evidence: String }

pub enum SourceKind {
    CredentialFile { path: String, declared_in_profile: bool },
    CredentialEnvVar { name: String, declared_in_profile: bool },
    PrivateKeyMaterial { path: String },
    ProjectFile { path: String },
    Stdin,
}

pub enum SinkKind {
    NetworkHost { host: String, port: Option<u16>, disposition: HostDisposition },
    NetworkHostUnresolved { raw: String },     // variable or substitution in host position
    FileOutsideWritableRoots { path: String },
}
```

`evidence` is the exact substring of the input command that produced the pair — the vendor's
"name the path or host the sandbox denied" property, applied to both halves.

### 2.3 The report

```rust
pub struct EgressReport {
    pub schema_version: &'static str,        // "caro.egress.v1"
    pub verdict: EgressVerdict,              // Clear | Finding | Indeterminate
    pub risk: Option<RiskLevel>,             // reuses src/models RiskLevel; None when Clear
    pub findings: Vec<EgressFinding>,
    pub profile_findings: Vec<ProfileFinding>,
    pub profile_load: ProfileLoad,
    pub indeterminate_reasons: Vec<String>,  // populated iff verdict == Indeterminate
}

pub struct EgressFinding { pub pair: EgressPair, pub risk: RiskLevel, pub rationale: String }

pub struct ProfileFinding {
    pub id: String,        // "CARO-EGRESS-F1" … stable, suppressible in CI
    pub summary: String,
    pub citation: String,  // the documented behaviour, quoted
    pub risk: RiskLevel,
}
```

Report-level `risk` is `findings.iter().map(|f| f.risk).max()` — `RiskLevel` already derives `Ord`
(`src/models/mod.rs:152-159`), so no comparator is written.

### 2.4 Chain integration

`src/caroml/validators/mod.rs`, three additive edits:

- `ValidatorContext` gains `pub sandbox: Option<&'a SandboxProfile>` (`:38-53`).
- `ValidationOutcome` (`:56-64`) and `Verdict` (`:66-71`) gain `Serialize, Deserialize`. No
  behaviour change; unblocks every future payload ADR.
- `default_chain()` (`:164-171`) becomes
  `[SafetyAngle, PlatformAngle, SecretsAngle, SideEffectsAngle, EgressAngle]`.

`EgressAngle` is `must_pass() == false`; blocking is decided by the caller from
`EgressReport.verdict` × `SafetyLevel` (ADR-064 D4), so CaroML repair loops keep working unchanged.

---

## 3. Files changed

| File | Lines touched (est.) | Nature |
|---|---|---|
| `src/models/mod.rs` | +230 | new types only |
| `src/caroml/validators/egress.rs` | +380 | **new file** |
| `src/caroml/validators/mod.rs` | +6 | field, two derives, chain row |
| `src/cli/mod.rs` | +5 | `CliResult.egress`, `EXIT_CODE_EGRESS_BLOCKED` |
| `src/safety/mod.rs` | +2 | `SafetySection.sandbox_profile` |
| `src/main.rs` | +55 | `Commands::Egress` variant + dispatch arm at `:2996` |
| `tests/egress_contract.rs` | +320 | **new file** |
| `tests/fixtures/sandbox_profiles/*.json` | 6 files | fixtures |

No new module. No new crate. No new dependency — `url = "2.5"` (`Cargo.toml:74`), `serde_json`,
`regex`, `async-trait`, `futures` are all present.

---

## 4. CLI and output contract

```
caro egress [COMMAND]... [--profile <PATH>] [--output json|yaml|plain]
```

- `COMMAND` omitted → profile-lint only (`profile_findings`, no `findings`).
- `--profile` omitted → `[safety].sandbox_profile` from `~/.config/caro/config.toml`, else
  `~/.claude/settings.json` if present, else `SandboxProfile::unprotected()`.
- `--output` is the existing flag (`OutputFormat`, `src/cli/mod.rs:104-109`); `json` prints
  `CliResult` with `.egress` populated, exactly as every other verb does today.
- Pure subprocess: reads two files, writes stdout, exits. No daemon, no socket, no session, no
  cache, no network. Deterministic — identical inputs give a byte-identical document.

**Exit codes**

| Code | Meaning |
|---|---|
| 0 | `Clear`, or non-blocking under the active `SafetyLevel` |
| 1 | Internal error, unreadable input, usage error (unchanged behaviour) |
| 3 | `Finding`, or `Indeterminate` under `Strict` — `EXIT_CODE_EGRESS_BLOCKED`, chosen to equal ADR-024's `ExitCode::Blocked` |

Machine consumers needing the `Finding`/`Indeterminate` distinction read `.egress.verdict`.
CI suppression is per `ProfileFinding.id`, not per exit code.

Integration test matrix: ADR-064 §Implementation Checklist, T1–T10. T8/T9 (profile-lint only)
are the determinism anchors — pure functions of a checked-in JSON file, no command, no parsing.

---

## 5. What breaks at 100 real users (`validation-discipline.md` Gate 3)

**The assumption that holds at demo scale.** That the recognized-verb grammar (D5) covers enough
real commands that `Indeterminate` stays rare. On the ten demo commands it does. Across 100 users'
actual shell traffic it will not: pipelines, `python -c`, `make` targets, and vendor CLIs will push
`Indeterminate` well above the demo rate.

**The failure mode.** Under the default `SafetyLevel::Moderate`, `Indeterminate` prompts for
confirmation. A high `Indeterminate` rate turns Caro into a confirmation-fatigue generator —
precisely the thing Claude Code's sandbox exists to remove, reintroduced by the tool claiming to
improve on it. Users would then set `Permissive`, which silently discards the feature.

**Instrumentation that would show it.** Emit `verdict` counts per invocation into the existing
telemetry path (`src/telemetry/`, opt-in, redacted via `src/telemetry/redaction.rs`). The signal
is the `Indeterminate` share of non-`Clear` verdicts. A stable share above ~15% means the grammar
is too narrow for the trade in D4 to be honest.

**Fallback.** Two staged, neither requiring a schema change: (a) `Indeterminate` degrades from
confirm to warn under `Moderate`, keeping the block only under `Strict` — one row in D4's table;
(b) the verb grammar grows rows, which is additive by construction (D5). The escape hatch that
must *not* be taken is collapsing `Indeterminate` into `Clear`, because that is the vendor bug
this ADR exists to correct.

**Second assumption, second failure.** Ingesting an explicitly unstable third-party format
(`srt`: *"its configuration format may change"*). At scale, a format change makes every profile
`Failed` → every result `Indeterminate` → under `Strict`, everything blocks. Mitigation: the
ingest is lenient and additive (unknown keys recorded, never fatal), `ProfileLoad::Failed` names
the reason in the report, and the fixture set is versioned against a dated documentation snapshot
so a drift shows as a failing test rather than a field incident.

---

## 6. Out of scope

Verbatim from ADR-064: shell AST (ADR-007), enforcement/sandboxing (ADR-010, ADR-039), domain
fronting and SNI-vs-Host detection, `stdout` secret scanning, non-Claude-Code/non-`srt` profile
formats, the MCP `assess_egress` tool (ADR-058/059), egress cases in the benchmark corpus
(ADR-063 — needs the stable `PatternId` this scope deliberately does not mint), trusted-target
host reputation (ADR-047), and Windows.
