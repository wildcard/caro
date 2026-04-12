# Caro Threat Model

**Version**: 1.0
**Last updated**: 2026-04-12
**Status**: Living document

## Purpose

This document enumerates the security threats caro is designed to address,
the trust boundaries it enforces, and the residual risks users should be
aware of. It complements `SECURITY.md` (which covers vulnerability
disclosure policy) by describing *what* caro protects against and *how*.

The structure is inspired by the threat-model section of the
[OpenEndpointSecurity](https://github.com/5BSD/OpenEndpointSecurity)
`DESIGN.md`, which enumerates specific threats to the authorization
pipeline with explicit mitigations for each.

## Scope

Caro is a CLI tool that:

1. Accepts a natural-language prompt from the user.
2. Sends the prompt to a local or remote LLM backend for shell command
   generation.
3. Validates the generated command against a database of 54+ dangerous
   patterns plus shell-expansion detection.
4. Prompts the user for confirmation before executing the command.
5. Runs the command through the user's shell and reports output.

Caro does **not**:

- Sandbox command execution -- once approved, commands run with the
  user's full privileges.
- Protect against a user who explicitly runs caro with
  `--allow-dangerous` or passes commands through unrelated channels.
- Validate LLM model weights or training data.
- Encrypt telemetry or backend traffic beyond what the transport layer
  (HTTPS/TLS) provides.

## Trust Boundaries

```
                ┌─────────────────────────┐
                │ User (local)            │
                │ prompt input            │
                └────────────┬────────────┘
                             │
                             ▼
 ───────────────── TRUST BOUNDARY 1 ─────────────────
                             │
                ┌────────────▼────────────┐
                │ caro CLI (local)        │
                │ argument parsing        │
                │ stdin/flag resolution   │
                └────────────┬────────────┘
                             │
                             ▼
 ───────────────── TRUST BOUNDARY 2 ─────────────────
                             │
                ┌────────────▼────────────┐
                │ LLM backend             │
                │ (embedded | remote API) │  <-- untrusted output
                └────────────┬────────────┘
                             │ raw JSON
                             ▼
 ───────────────── TRUST BOUNDARY 3 ─────────────────
                             │
                ┌────────────▼────────────┐
                │ JSON parser             │
                │ CommandGenerator trait  │
                └────────────┬────────────┘
                             │ parsed command
                             ▼
 ───────────────── TRUST BOUNDARY 4 ─────────────────
                             │
                ┌────────────▼────────────┐
                │ SafetyValidator         │
                │ - 54+ regex patterns    │
                │ - expansion detection   │
                │ - timeout + fail-safe   │
                │ - recursion guard       │
                └────────────┬────────────┘
                             │ validated command
                             ▼
 ───────────────── TRUST BOUNDARY 5 ─────────────────
                             │
                ┌────────────▼────────────┐
                │ User confirmation       │
                │ (interactive dialog)    │
                └────────────┬────────────┘
                             │ confirmed command
                             ▼
 ───────────────── TRUST BOUNDARY 6 ─────────────────
                             │
                ┌────────────▼────────────┐
                │ CommandExecutor         │
                │ shell spawn w/ depth+1  │
                └─────────────────────────┘
```

Every arrow crossing a trust boundary is an enforcement point. The most
critical is **Boundary 4**, where untrusted model output is validated
before being shown to the user.

## Threat Actors

| Actor | Capabilities | Goal |
|---|---|---|
| **Adversarial prompt crafter** | Controls the natural-language input | Trick caro into generating a dangerous command the user will approve |
| **Hallucinating / buggy LLM** | Produces the model output (unintentionally wrong) | Not hostile, but can produce dangerous commands by mistake |
| **Compromised model file** | Substituted model file on disk | Produce attacker-controlled commands |
| **Network attacker (MITM)** | Can intercept/modify traffic to remote backends | Substitute responses from remote APIs |
| **Co-resident process** | Runs on the same machine without caro privileges | Read cached model files or telemetry logs |

The first two are by far the most important. Prompt crafting and LLM
hallucination are everyday risks; the others are specialized.

## Attack Surface & Mitigations

### A1. Prompt injection producing dangerous command
**Threat**: User (or an upstream tool piping text into caro) provides a
prompt that causes the LLM to emit a destructive command like `rm -rf /`
that looks innocuous in natural language.

**Mitigation**: The `SafetyValidator` runs 54+ regex patterns on every
generated command before showing it to the user, flagging known
destructive forms. `Moderate` safety level (the default) blocks
`Critical` risk commands; `Strict` additionally blocks `High` risk
commands.

**Status**: Implemented (`src/safety/patterns.rs`).

**Residual risk**: Novel patterns not yet in the database. Users can
mitigate by running in `Strict` mode and/or adding custom patterns.

---

### A2. Model output parsing bypass
**Threat**: LLM produces non-JSON or partial-JSON output that bypasses
parsing or causes the fallback parser to extract an unintended command.

**Mitigation**: `EmbeddedModelBackend` uses a multi-strategy JSON parser
with bounded retries. On parse failure it returns a structured error
rather than executing undefined behavior.

**Status**: Implemented (`src/backends/embedded/`).

**Residual risk**: A carefully crafted model output that parses but
extracts a different command than the LLM "intended" could sneak past.
This is mitigated by Mitigation A1 (pattern matching on the parsed
command).

---

### A3. Safety pattern evasion via shell metacharacters
**Threat**: A command like `echo $(rm -rf /)` or `` echo `rm -rf /` ``
hides a destructive operation inside a shell expansion that the
top-level regex patterns cannot see into.

**Mitigation**: `ExpansionDetector` (OES-inspired, see
`src/safety/expansion.rs`) detects `$(...)`, backticks, `${...}`,
process substitution, and arithmetic expansion *before* pattern
matching runs. Commands containing command-executing expansions are
raised to at least `Moderate` risk (and `High` in `Strict` mode). The
detector is quote-aware: single-quoted expansions are literal on POSIX
shells and are not flagged.

**Status**: Implemented (`src/safety/expansion.rs`).

**Residual risk**: Complex nested quoting (e.g. `"'\"$(...)\"'"`) may
not be analyzed correctly. Fish shell's unique `(...)` substitution
syntax is only partially handled.

---

### A4. TOCTOU between validation and execution
**Threat**: A command passes validation but is modified (e.g. via
environment variable expansion) before shell execution.

**Mitigation**: The validated command string is passed directly to
`shell -c` without intermediate processing. `CommandExecutor` does not
re-expand the command.

**Status**: Implemented (`src/execution/executor.rs`).

**Residual risk**: The shell itself performs variable expansion when
interpreting the command. This is why `ExpansionDetector` raises the
risk level on variable references so the user is warned.

---

### A5. Catastrophic safety validation hang
**Threat**: A crafted command causes regex catastrophic backtracking
(or a very large custom pattern list) such that `validate_command`
never returns, blocking the user indefinitely.

**Mitigation**: OES-inspired timeout: `SafetyConfig::validation_timeout_ms`
(default 500ms) bounds validation time. Unlike OES's default-ALLOW on
timeout (appropriate for kernel auth hooks that cannot hang the system),
caro uses **default-DENY** -- a CLI tool should fail closed when it
cannot confirm safety.

**Status**: Implemented (`src/safety/mod.rs::timeout_result`).

**Residual risk**: The Rust `regex` crate guarantees linear-time
execution, so catastrophic backtracking is not a realistic threat
against the built-in patterns. The timeout primarily guards against
pathological custom pattern lists or future performance regressions.

---

### A6. Recursive caro invocation / fork loop
**Threat**: Generated command invokes `caro` itself, producing an
infinite recursion that exhausts PIDs or file descriptors.

**Mitigation**: Two-layer defense inspired by OES self-mute:

1. **Pattern-based**: Regex patterns in `src/safety/patterns.rs`
   detect `caro` and `cmdai` as standalone command tokens (not
   substrings of `cargo`, `scaro`, etc.) and flag them as `High`
   risk.
2. **Runtime depth limit**: `CARO_RECURSION_DEPTH` environment
   variable is incremented by each caro process and propagated to
   child shells. On startup, caro refuses to run if the depth
   exceeds `MAX_RECURSION_DEPTH` (2).

**Status**: Implemented (`src/safety/patterns.rs`, `src/safety/mod.rs`,
`src/main.rs`, `src/execution/executor.rs`).

**Residual risk**: An attacker who can bypass both the pattern matcher
*and* the environment variable (e.g., by clearing env vars via `env -i`)
can still trigger recursion. This would still be bounded by OS limits.

---

### A7. Redundant work amplification
**Threat**: Repeatedly validating the same command (e.g. during agent
loop refinement iterations or user retry) causes unnecessary CPU load.

**Mitigation**: OES-inspired LRU+TTL decision cache
(`src/safety/cache.rs`). Keyed by `(hash(command), shell, safety_level)`,
with default 256-entry capacity and 60-second TTL. Timeout results are
not cached (they may be transient).

**Status**: Implemented (`src/safety/cache.rs`).

**Residual risk**: None -- the cache is a pure optimization and fails
open to the uncached path.

---

### A8. Silent / unobservable safety decisions
**Threat**: Safety decisions happen without any audit trail, making it
impossible to debug false positives, investigate bypass attempts, or
measure validator behavior in production.

**Mitigation**: OES-inspired structured tracing (`tracing` crate). Every
call to `validate_command` emits a `safety validation complete` event
with fields: `decision`, `risk_level`, `patterns_matched`, `duration_us`,
`cache_hit`, `shell`, `command_len`. Timeout events emit a separate
`decision="timeout"` event. Raw command text is **not** logged for
privacy.

**Status**: Implemented (`src/safety/mod.rs`).

**Residual risk**: Users must opt in by setting `RUST_LOG=caro::safety=info`.

---

### A9. Model cache poisoning
**Threat**: Attacker replaces a cached LLM model file with a malicious
variant that always produces attacker-controlled output.

**Mitigation**: SHA-256 checksum verification on model download
(`src/cache/`). Models are stored in a user-owned cache directory.

**Status**: Partially implemented.

**Residual risk**: If the attacker can write to the user's cache
directory, they can replace the model file and matching checksum. This
is beyond caro's trust boundary; the OS file permissions are the
enforcement layer.

---

### A10. Environment variable manipulation
**Threat**: Attacker sets `PATH`, `LD_PRELOAD`, or similar variables
to redirect command execution to a malicious binary.

**Mitigation**: Caro detects `export PATH=` as a moderate-risk pattern.
`CommandExecutor` does not clean the environment; it inherits the
user's.

**Status**: Partial.

**Residual risk**: A pre-compromised shell environment is outside
caro's scope.

---

## Mitigations Matrix (Summary)

| Threat | Mitigation | Module | Status |
|---|---|---|---|
| A1. Prompt injection | 54+ regex patterns | `src/safety/patterns.rs` | Implemented |
| A2. JSON parse bypass | Multi-strategy parser + retry | `src/backends/embedded/` | Implemented |
| A3. Expansion evasion | `ExpansionDetector` | `src/safety/expansion.rs` | Implemented |
| A4. TOCTOU | Pass-through to `shell -c` | `src/execution/executor.rs` | Implemented |
| A5. Validation hang | Timeout + fail-closed | `src/safety/mod.rs` | Implemented |
| A6. Recursive invocation | Pattern + env-var depth | `src/safety/mod.rs`, `src/main.rs` | Implemented |
| A7. Work amplification | LRU+TTL cache | `src/safety/cache.rs` | Implemented |
| A8. Silent decisions | `tracing` events | `src/safety/mod.rs` | Implemented |
| A9. Cache poisoning | SHA-256 checksums | `src/cache/` | Partial |
| A10. Env manipulation | Pattern detection | `src/safety/patterns.rs` | Partial |

## Residual Risks

Caro explicitly does **not** protect against:

- **User-approved destructive commands**: If the user confirms a `rm -rf /`
  prompt, caro will run it. The safety system can only warn.
- **LLM quality issues**: A model that consistently generates poor
  commands is not a caro bug -- use a better model or backend.
- **OS-level privilege escalation**: Caro does not drop privileges or
  sandbox. It runs with the user's full permissions.
- **Shell-level metacharacter exploits** after our expansion detection:
  e.g., extremely creative quoting that we do not yet parse.
- **Side-channel attacks** on local LLM inference (timing, cache, power).
- **Supply-chain attacks** on cargo dependencies (outside of the
  `cargo audit` baseline).

## Review Cadence

This document should be revisited:

- When a new dangerous pattern category is identified.
- When a new safety mitigation is added (expand the matrix).
- On every major version release.
- In response to any CVE filed against caro or a direct dependency.

## See Also

- [`SECURITY.md`](../SECURITY.md) -- Vulnerability disclosure policy
- [`docs/TESTING_STRATEGY.md`](TESTING_STRATEGY.md) -- How the threats above are tested
- [OES DESIGN.md](https://github.com/5BSD/OpenEndpointSecurity) -- The inspiration for this document's structure
