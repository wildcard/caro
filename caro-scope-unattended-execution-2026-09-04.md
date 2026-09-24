# Scope — `caro.unattended.v1` (ADR-065)

**Date**: 2026-09-04 · **Produced by**: `caro-research--scoping-process` (autonomous run, no user present)
**Decision document**: `docs/adr/ADR-065-unattended-execution-contract.md`
**Baseline**: `caro` 1.4.0, tree as of `integrator/20260711-postmerge`

This document holds the implementation detail the ADR references. It commits no code.

---

## 1. One-paragraph statement

Claude Code's background task system decides a process's fate from its session membership alone: it
SIGTERMs everything tracked when the session ends, compacts, or exhausts context (#25188), on an
undocumented ~1800 s timer (#84981, open), and above a 5 GB output cap — while separately failing to
stop tasks from closed sessions (#58662). It does this without any information about what the command
is, because nothing in the pipeline ever produces that information. `caro unattended` produces it: one
subprocess call, one serialized report, one exit code, classifying the command by what an untimely
SIGTERM would do to it and recommending where it should therefore run. Caro emits the supervision
contract and never holds it.

---

## 2. New types

All in `src/models/mod.rs` unless noted. Every type derives at minimum
`Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema`. Enums are
`#[serde(rename_all = "snake_case")]` and `#[non_exhaustive]`.

### 2.1 The axis

```rust
/// How a command behaves when stopped at an arbitrary instant.
/// Orthogonal to `RiskLevel`: `Safe` + `Custodial` is a normal, important combination.
pub enum InterruptClass {
    Idempotent,
    Resumable,
    PartialDestructive,
    Custodial,
    Indeterminate { reason: IndeterminateReason },
}

pub enum IndeterminateReason {
    Composed,        // contains |, &&, ||, ;, &, $( ), or an sh -c wrapper  (D6)
    OpaqueWrapper,   // make / just / npm run / docker compose / xargs — real head is hidden
    UnknownHead,     // argv[0] not in the table
    EmptyInput,
}
```

### 2.2 The supervision contract

```rust
pub struct SupervisionContract {
    pub duration: DurationClass,          // Seconds | Minutes | Unbounded
    pub output_volume: OutputVolume,      // Bounded | Growing | Unbounded
    pub teardown_signal: TeardownSignal,  // Term | Int | TermThenKill
    pub grace_period_ms: u32,             // 0 for Idempotent; >= 10_000 for Custodial
    pub survives_session: bool,           // true iff the process must outlive session cleanup
    pub cleanup_paths: Vec<String>,       // artifacts a killed run would leave behind (may be empty)
}
```

No absolute host limits appear here (ADR-065 D5). `OutputVolume::Unbounded` is Caro's statement; the
5 GB comparison is the host's.

### 2.3 The report

```rust
pub struct UnattendedReport {
    pub schema: &'static str,             // "caro.unattended.v1"
    pub command: String,                  // echoed verbatim, never rewritten
    pub interrupt_class: InterruptClass,
    pub placement: Placement,             // Foreground | HostBackground | ExternalSupervisor
    pub requested_placement: Option<Placement>,  // what the caller asked for, if it said
    pub routing: SuggestedRouting,        // REUSED — no new tier vocabulary
    pub risk_level: RiskLevel,            // REUSED — straight from SafetyValidator
    pub matched_patterns: Vec<String>,    // REUSED — from ValidationResult
    pub supervision: SupervisionContract,
    pub rationale: String,                // one human line
    pub safety_level: SafetyLevel,        // the level this verdict was computed under
}
```

**Method contracts**

| Method | Signature | Contract |
|---|---|---|
| `InterruptClass::classify` | `fn(&str, ShellType) -> Self` | Pure, total, no I/O. Composition operators checked *before* head lookup, so `rm -rf / && echo ok` is `Indeterminate{Composed}`, never `Idempotent`. Never panics on empty or non-UTF8-lossy input |
| `Placement::for_class` | `fn(InterruptClass, SafetyLevel) -> Self` | `Custodial → ExternalSupervisor` at every level. `PartialDestructive → Foreground` at `Strict`/`Moderate`, `HostBackground` at `Permissive`. `Indeterminate → Foreground` at `Strict`, else `HostBackground` |
| `UnattendedReport::build` | `fn(&str, &dyn SafetyValidator, SafetyLevel, Option<Placement>) -> Self` | Calls the **existing** validator for `risk_level`/`matched_patterns`; adds nothing to it. Deterministic for a fixed input triple |
| `UnattendedReport::accepts` | `fn(&self) -> bool` | `requested_placement.is_none() \|\| requested == placement`. Drives the exit code and nothing else |

`routing` is computed by the existing `SuggestedRouting::from_risk_and_safety`, then escalated one
step (`AutoApprove → AsyncLog → HumanGate`) when `placement != requested_placement`. Escalation never
reaches `Block`: this command is advisory, and a verb that can silently become a blocker is a verb
consumers stop calling.

---

## 3. Files that change

Six files. One new source file inside an existing module; no new module.

| # | File | Change |
|---|---|---|
| 1 | `src/models/mod.rs` | `+` the six types in §2. No existing type modified |
| 2 | `src/safety/interrupt.rs` | **new file** — the head-keyed table and `classify()`. ~180 lines including the table. Does not touch `patterns.rs`, does not presuppose `assessment.rs` |
| 3 | `src/safety/mod.rs` | `+ pub mod interrupt;` and one re-export. Nothing else |
| 4 | `src/main.rs` | `+` one `Commands::Unattended { … }` variant (near `Commands::Doctor`, `:381`) and its handler. Handler is ~40 lines: parse → `build` → serialize → `process::exit` |
| 5 | `tests/unattended_contract.rs` | **new** — §5's table, driven by `assert_cmd` (already a dev-dependency, `Cargo.toml:148`) |
| 6 | `src/bin/generate-schema.rs` | `+` `UnattendedReport` to the emitted schema set |

**`Cargo.toml`: no change.** `serde`, `serde_json`, `schemars 0.8`, `clap 4.5` and `assert_cmd 2` are
all present. No new crate means `external-sdk-integration.md`'s spike checklist does not apply.

---

## 4. CLI surface and output contract

```
caro unattended [OPTIONS] <COMMAND>

  --placement <foreground|host-background|external-supervisor>
        What the caller intends to do. Sets `requested_placement` and therefore the exit code.
  --safety <strict|moderate|permissive>     [default: from config, as everywhere else]
  --shell <bash|zsh|fish|sh>                [default: detected]
  --format <json|text>                      [default: json]
```

`stdout` carries the report and nothing else in `json` mode — no banner, no colour, no progress. A
consumer may `caro unattended --placement host-background "$CMD" | jq -e .placement` and rely on it.

### Exit codes

| Code | Meaning | Consumer obligation |
|---|---|---|
| `0` | Advisory produced; requested placement acceptable | proceed |
| `2` | Advisory produced; requested placement refused | do not background; read `.placement` for where it should go |
| `1` | Caro failed; **no verdict reached** | must not be read as approval |

Exit `2` matches Claude Code's `PreToolUse` hook block convention, so the integration is
`caro unattended --placement host-background "$command"` with no translation layer.
`EXIT_CODE_EDIT = 201` (`src/main.rs:938`) is untouched.

### Worked integration (the #25188 case)

```
$ caro unattended --placement host-background "node dist/daemon/supervisor.js"
{
  "schema": "caro.unattended.v1",
  "command": "node dist/daemon/supervisor.js",
  "interrupt_class": "custodial",
  "placement": "external_supervisor",
  "requested_placement": "host_background",
  "routing": "human_gate",
  "risk_level": "safe",
  "matched_patterns": [],
  "supervision": {
    "duration": "unbounded",
    "output_volume": "growing",
    "teardown_signal": "term_then_kill",
    "grace_period_ms": 10000,
    "survives_session": true,
    "cleanup_paths": []
  },
  "rationale": "Long-lived service process; session cleanup or context compaction would stop it silently.",
  "safety_level": "moderate"
}
$ echo $?
2
```

The daemon never joins the tracked set, so compaction cannot reach it. That is the failure mode
solved by design rather than by a `persistent:` flag Caro has no way to add.

---

## 5. Integration tests — known input → deterministic JSON + exit code

`tests/unattended_contract.rs`, all at `--safety moderate --shell bash`, all with
`--placement host-background`. Every row asserts the full `interrupt_class`, `placement` and exit
code; none asserts on `rationale` text.

| # | Input | `interrupt_class` | `placement` | exit |
|---|---|---|---|---|
| 1 | `sleep 3600` | `idempotent` | `host_background` | 0 |
| 2 | `cargo build --release` | `idempotent` | `host_background` | 0 |
| 3 | `npx vitest --watch` | `custodial` | `external_supervisor` | 2 |
| 4 | `node dist/daemon/supervisor.js` | `custodial` | `external_supervisor` | 2 |
| 5 | `rsync -a src/ dst/` | `resumable` | `host_background` | 0 |
| 6 | `rsync -a --delete src/ dst/` | `partial_destructive` | `foreground` | 2 |
| 7 | `dd if=/dev/zero of=/tmp/f bs=1M count=100` | `partial_destructive` | `foreground` | 2 |
| 8 | `tar -xzf big.tgz -C /srv/app` | `partial_destructive` | `foreground` | 2 |
| 9 | `wget -c https://example.com/x.iso` | `resumable` | `host_background` | 0 |
| 10 | `npm run dev` | `indeterminate{opaque_wrapper}` | `host_background` | 0 |
| 11 | `rm -rf / && echo done` | `indeterminate{composed}` | `host_background` | 0 |
| 12 | `frobnicate --all` | `indeterminate{unknown_head}` | `host_background` | 0 |
| 13 | `""` (empty) | — | — | 1 |

Three properties are asserted separately from the table:

- **P1 — orthogonality.** Row 4 asserts `risk_level == "safe"` alongside
  `placement == "external_supervisor"`. If someone later collapses the axes, this fails.
- **P2 — composition precedence.** Row 11 must classify as `Composed`, not by its `rm` head. A
  regression here is a fail-open, and the test name says so.
- **P3 — determinism.** Each row runs twice; byte-identical stdout both times. No timestamps, no
  durations, no host paths in the payload.
- **P4 — exit 1 is not approval.** Row 13 asserts stdout is empty and the error goes to stderr.

Property tests (`proptest`, already in the tree): `classify` never panics and is total over arbitrary
UTF-8 input; `Placement::for_class` never returns `HostBackground` for `Custodial` at any safety
level.

---

## 6. Gate 3 — what breaks at 100 real users

**The assumption that holds at demo scale.** The classifier keys on `argv[0]` plus a small flag set.
Every row in §5 is a bare invocation. Real usage is `make deploy`, `just test`,
`docker compose up -d`, `pnpm run build && pnpm start`, and `bash -c "$(cat script.sh)"`.

**The failure mode.** Not a wrong answer — `Indeterminate` is the designed response and it is honest.
The failure is *uselessness*: if 70% of real invocations return `Indeterminate`, consumers stop
calling the verb, and a safety advisory nobody calls is worth nothing. The dangerous second-order
effect is a maintainer "fixing" this by defaulting `Indeterminate` to `Idempotent`, which is
fail-open and would be a silent regression of exactly the class this ADR exists to prevent — hence
property P2 exists specifically to make that change loud.

**Instrumentation.** One counter in the eval corpus: `indeterminate_rate`, broken down by
`IndeterminateReason`, over the existing evaluation command set. It is a reported number, not a gate,
in v1. If `Composed` dominates, that is the evidence for the ADR-007 AST dependency; if
`OpaqueWrapper` dominates, the cheaper fix is a wrapper-unwrapping table, and the data says which.

**Fallback.** The verb is advisory and additive. A host that finds the answer unhelpful ignores the
exit code and is exactly as safe as it is today. Nothing regresses.

---

## 7. Explicitly out of scope

Belongs to the next version, not this one:

1. **Composition analysis.** Pipelines, `&&`/`||`, subshells, `sh -c`. Requires ADR-007's AST.
   v1 answers `Indeterminate{Composed}` (D6).
2. **Wrapper unwrapping.** Resolving `make deploy` or `npm run dev` to the underlying command.
   Requires reading `Makefile`/`package.json` — filesystem I/O this verb deliberately does not do.
3. **Supervision itself.** No `caro run --supervise`, no process table, no PID file, no daemon.
   Non-negotiable (ADR-065 D2/A3).
4. **Generating supervisor units.** Emitting a `systemd` unit or `tmux` invocation for an
   `ExternalSupervisor` verdict. Attractive, and a separate decision about writing files.
5. **Runtime state.** No "how long has this been running." That is ADR-050's cumulative-ledger
   territory and it requires the state relaxation this ADR does not take.
6. **Nesting into `caro.assessment.v1`.** Committed to in D5, executed when ADR-058/059 merges.
7. **`Custodial` detection beyond the explicit list.** No learned or model-inferred classification.
   The deterministic floor is the product.
8. **Stable pattern IDs in `patterns.rs`.** A real gap found during this research
   (`DangerPattern` is `{pattern, risk_level, description, shell_specific}`; `matched_patterns`
   carries description strings). Worth a bead. This ADR is designed not to need it.

---

## 8. Open questions for the implementation PR

- Does `--placement` default to `host-background` when omitted, or does omitting it mean "advise
  only, always exit 0"? The tests above assume the latter is expressed by *omitting* the flag; the
  PR should confirm that reads well from a hook.
- `grace_period_ms` for `PartialDestructive`: the honest answer may be "no grace period makes this
  safe," which argues for `Option<u32>` rather than `0`.
- Whether `cleanup_paths` can be populated at all in v1 without filesystem I/O. If not, ship it
  empty and documented, or cut it until §7.2 lands.

---

## 9. References

Same set as ADR-065 §References. Line numbers verified 2026-09-04 against
`integrator/20260711-postmerge` and expected to drift.
