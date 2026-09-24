# Scope — `caro.confinement.v1` (ADR-066)

**Date**: 2026-09-07 · **Produced by**: `caro-research--scoping-process` (autonomous run, no user present)
**Decision document**: `docs/adr/ADR-066-path-confinement-gate.md`
**Baseline**: `caro` 1.4.0, tree as of `integrator/20260711-postmerge`

This document holds the implementation detail the ADR references. It commits no code.

---

## 1. One-paragraph statement

Claude Code's `--restricted` mode (v2.1.248, 2026-08-27) fences file tools to the working
directory by removing the tools that could leave it — and re-admits them when they are named in
`--tools`. Its permission language cannot express a fence on a shell command's arguments at all:
the docs state that `Bash(command:rm *)` is ignored with a startup warning because a compound
command would bypass it, and that argument-constraining Bash patterns are "fragile". Redirect
targets are checked; every other path operand is not. `caro confine` produces the missing answer:
one subprocess call, one serialized report, one exit code, classifying each path operand of a
command as inside, outside, escaping or unverifiable against caller-supplied roots, with read/write
intent per operand — and declaring in the payload that the answer is lexical, so it can never be
mistaken for the filesystem claim that CVE-2025-59829 made and got wrong.

---

## 2. New types

All in `src/models/mod.rs` unless noted, placed beside `RiskLevel` and `SuggestedRouting`. Every
type derives at minimum `Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema`. Enums are
`#[serde(rename_all = "snake_case")]` and `#[non_exhaustive]`.

Note for the implementer: `SuggestedRouting` currently derives `Serialize, Deserialize` but **not**
`JsonSchema` (`src/models/mod.rs:187`). Since the report embeds it, the PR adds `JsonSchema` to
that derive list. That is a one-word change to an existing type and the only modification this
scope makes to anything that already ships.

### 2.1 The resolution mode — D1, the CVE-2025-59829 carve-out

```rust
/// What kind of claim this report is making. Required, serialized, never defaulted.
pub enum Resolution {
    /// Decided from the command text alone. No filesystem was consulted.
    /// A `Contained` verdict under `Lexical` means "no operand names a path outside the
    /// roots, as written" — NOT "this command cannot touch a path outside the roots."
    Lexical,
}
```

One variant in v1. It exists so that `Resolved` can be added later without a schema break, and so
that no consumer can read a lexical answer as a resolved one. This field is the ADR's D1 in the
type system.

### 2.2 Placement of a single operand

```rust
pub enum Confinement {
    /// Lexically under one of the roots, with no `..` segment anywhere in it.
    Inside,
    /// Absolute, and under none of the roots. `/etc/passwd` with `--root /w`.
    Outside,
    /// Contains `..` and lexical normalisation lands outside the roots. `../other`.
    Escaping,
    /// Cannot be placed without information Caro refuses to gather.
    Unverifiable { reason: UnverifiableReason },
}

pub enum UnverifiableReason {
    /// Contains a `..` segment but lexically stays inside. NEVER `Inside` — D2, CWE-61.
    Traversal,
    /// `~/…` or `~user/…`. Requires `$HOME`, which is environment, not text.
    HomeAnchored,
    /// `$X`, `${X}`, `$(…)`, backticks.
    VariableExpanded,
    /// Contains `*`, `?`, `[`, `{` — the operand names a set, not a path.
    Glob,
    /// `-`, `/dev/stdin`, `/dev/fd/3`, a process substitution. Not a filesystem location.
    Device,
    /// Relative operand appearing after a `cd` in the same command. Unreachable in v1
    /// (a `cd`-bearing command is `Composed` first) — reserved for the post-ADR-007 path.
    CwdRelativeAfterCd,
    /// `--root` was not supplied. Advisory mode; access intent still computed.
    NoRoot,
}
```

`Outside` and `Escaping` are kept distinct because they are different stories for a human:
"you named `/etc/passwd`" versus "you wrote `../` and it left". Collapsing them loses the second
one's diagnostic value.

### 2.3 Access intent — D4

```rust
pub enum Access { Read, Write, ReadWrite, Unknown }

pub enum OperandOrigin {
    /// A bare argument in the command's operand position.
    Positional,
    /// The value of a flag: `tar -C DIR`, `git -C DIR`, `sed -i FILE`, `--output=FILE`.
    FlagValue { flag: String },
    /// `> f`, `>> f`, `2> f`, `< f`. The only class the host itself checks.
    Redirect,
}
```

`Unknown` is reachable only from an unknown head, and an unknown head makes the whole report
`Indeterminate` (§3, verdict rules), so `Unknown` never coexists with a `Contained` verdict.

### 2.4 The operand

```rust
pub struct PathOperand {
    /// The token exactly as it appeared. Never rewritten, never expanded, never normalised.
    pub literal: String,
    pub origin: OperandOrigin,
    pub access: Access,
    pub confinement: Confinement,
}
```

`literal` is echoed verbatim — same discipline as `UnattendedReport.command` in ADR-065. A payload
that rewrites its input cannot be diffed against the command the host is about to run.

### 2.5 The report

```rust
pub struct ConfinementReport {
    pub schema_version: &'static str,      // "caro.confinement.v1"
    pub command: String,                   // echoed verbatim
    pub roots: Vec<String>,                // as supplied, absolute, normalised (D9)
    pub resolution: Resolution,            // Lexical
    pub verdict: ConfinementVerdict,
    pub operands: Vec<PathOperand>,        // empty when Composed
    pub risk_level: RiskLevel,             // REUSED — from SafetyValidator
    pub matched_patterns: Vec<String>,     // REUSED — from ValidationResult
    pub routing: SuggestedRouting,         // REUSED — no new tier vocabulary
    pub rationale: String,                 // one human line
}

pub enum ConfinementVerdict {
    Contained,
    Escape,
    Indeterminate { reason: IndeterminateReason },
}

pub enum IndeterminateReason {
    /// Contains `|`, `&&`, `||`, `;`, `&`, a newline, `$( )`, backticks, or an `sh -c` wrapper.
    Composed,
    /// `argv[0]` is not in the operand table.
    UnknownHead,
    /// At least one operand is `Unverifiable`, and none is `Outside` or `Escaping`.
    UnplaceableOperand,
}
```

`schema_version` on the sub-object, surfaced as one additive field on `CliResult` —
`pub confinement: Option<ConfinementReport>` with
`#[serde(default, skip_serializing_if = "Option::is_none")]` — exactly ADR-064 §3's shape. No
envelope; ADR-024 is not pre-empted.

---

## 3. Classification rules

### 3.1 Order of operations

1. **Empty or whitespace-only input** → exit 1, stderr message, empty stdout. Not a verdict.
2. **Unsupported shell** (`--shell powershell` / `cmd`) → exit 1. v1 is POSIX-only and says so;
   Windows path semantics (drive letters, `\`, UNC) are §7.
3. **Composition scan** → any of `| && || ; & \n $( ) `` ` `` `, or head in `{sh, bash, zsh, dash}`
   with `-c` → `Indeterminate { Composed }`, `operands: []`, done. **D6.**
4. **Head lookup** → `argv[0]` after stripping the wrapper set (below). Miss →
   `Indeterminate { UnknownHead }`, `operands: []`.
5. **Operand extraction** per the head's entry, producing `(literal, origin, access)` triples.
6. **Placement** of each literal against the roots, per §3.3.
7. **Verdict** per §3.4.

Wrapper stripping mirrors the host's documented set so the two agree on what the real head is:
`timeout`, `time`, `nice`, `nohup`, `stdbuf`, `command`, `builtin`, `noglob`, and bare `xargs`
only. `xargs` **with** flags is not stripped, matching the host. `env` is not stripped in v1
(it may carry assignments); `env VAR=x cmd` is `UnknownHead`.

### 3.2 The operand table (v1 seed)

Head-keyed, hand-written, ~40 entries. Shape:

| Head | Operands | Access |
|---|---|---|
| `cat`, `less`, `head`, `tail`, `wc`, `md5sum`, `sha256sum` | all positional | `Read` |
| `ls`, `stat`, `file`, `du`, `find` (path args only) | all positional | `Read` |
| `cp`, `mv`, `install`, `rsync`, `ln` | all but last positional `Read`; last `Write` | mixed |
| `rm`, `rmdir`, `unlink`, `shred`, `truncate` | all positional | `Write` |
| `mkdir`, `touch`, `tee` | all positional | `Write` |
| `chmod`, `chown`, `chgrp` | positional after the mode/owner | `Write` |
| `sed` | `-i` present → positional files `ReadWrite`; else `Read` | mixed |
| `tar` | `-C DIR` → `Write` if `-x`, `Read` if `-c`; `-f FILE` per direction | mixed |
| `git` | `-C DIR` → `Write` (a git subcommand in another repo may write) | `Write` |
| `dd` | `if=` `Read`, `of=` `Write` | mixed |
| `docker`, `podman` | `-v HOST:CTR` → host side `ReadWrite` | `ReadWrite` |
| *(redirects, any head)* | `> f`, `>> f`, `2> f` `Write`; `< f` `Read` | per operator |

`find` is entered with `Read` **and** a guard: `find` with `-exec`, `-execdir`, `-delete` or `-ok`
is `Indeterminate { UnknownHead }`, because it is a composition operator wearing a program's
clothes. The host reaches the same conclusion by a different route — its docs say `Bash(find *)`
does not cover those forms.

`git -C DIR` classified as `Write` is deliberately pessimistic: `git -C ~/other status` is a read,
but the table keys on the head, not the subcommand, and a wrong `Write` produces a prompt while a
wrong `Read` produces a silent checkout in someone else's repository.

### 3.3 Placement of one literal

```
if literal is a device/stdio form               -> Unverifiable { Device }
if literal starts with `~`                      -> Unverifiable { HomeAnchored }
if literal contains $ or ` or $(                -> Unverifiable { VariableExpanded }
if roots is empty                               -> Unverifiable { NoRoot }
if literal contains a `..` path segment:
    normalise lexically against the root's base
    if the normalised form leaves every root    -> Escaping
    else                                        -> Unverifiable { Traversal }        // D2
if literal contains * ? [ {                     -> Unverifiable { Glob }
if literal is absolute:
    under some root (path-segment prefix)       -> Inside
    else                                        -> Outside
else (plain relative, no traversal)             -> Inside     // resolves against a root by
                                                              // construction; caller declared cwd
```

Two details that matter and will otherwise be got wrong:

- **Prefix matching is segment-wise.** `/workspace-backup/x` is *not* under `/workspace`. String
  `starts_with` is a bug; compare `Path::components`.
- **A trailing `/**`-style root is not accepted.** Roots are directories, not patterns (D9). This
  is the deliberate divergence from the host's gitignore rule language: one anchor, no globs, no
  four-way anchor ambiguity, no `/Users/alice/file`-is-not-absolute footgun.

### 3.4 Verdict and exit

| Operand set contains | Verdict | Exit at `permissive`/`moderate` | Exit at `strict` |
|---|---|---|---|
| any `Outside` or `Escaping` | `Escape` | **2** | **2** |
| any `Unverifiable`, no escape | `Indeterminate { UnplaceableOperand }` | 0 | **2** |
| only `Inside`, or empty with a known head | `Contained` | 0 | 0 |
| — (`Composed` / `UnknownHead`) | `Indeterminate { … }` | 0 | **2** |

`routing` is populated by feeding the report's own severity through the existing
`SuggestedRouting::from_risk_and_safety` shape: `Escape` behaves as `RiskLevel::High` for routing
purposes, `Indeterminate` as `RiskLevel::Moderate`, `Contained` as the `risk_level` the
`SafetyValidator` already returned. No new tier is minted (ADR-059 moratorium).

`risk_level` and `matched_patterns` come straight from `SafetyValidator::validate_command`, which
already runs and already produces them. The verb reuses the validator; it does not re-implement it.

---

## 4. Files that change

Minimal set. Two new files, three edited.

| File | Change |
|---|---|
| `src/models/mod.rs` | **+** `Resolution`, `Confinement`, `UnverifiableReason`, `Access`, `OperandOrigin`, `PathOperand`, `ConfinementVerdict`, `IndeterminateReason`, `ConfinementReport`. **~** add `JsonSchema` to `SuggestedRouting`'s derive list (`:187`) |
| `src/safety/confinement.rs` | **new.** `pub fn operands(command: &str, shell: ShellType) -> Result<Vec<(String, OperandOrigin, Access)>, Composed>` (the D7 shared primitive), `pub fn classify(command, roots, shell, safety, validation: &ValidationResult) -> ConfinementReport`, the operand table, the wrapper-strip set |
| `src/safety/mod.rs` | **~** one line: `pub mod confinement;` |
| `src/main.rs` | **+** `Commands::Confine { command: String, root: Vec<PathBuf>, format: Option<String> }`; **+** dispatch arm; **+** `pub const EXIT_CODE_CONFINEMENT_ESCAPE: i32 = 2;` beside `EXIT_CODE_EDIT` |
| `src/cli/mod.rs` | **+** `pub confinement: Option<ConfinementReport>` on `CliResult`, `#[serde(default, skip_serializing_if = "Option::is_none")]` |
| `tests/confinement_contract.rs` | **new.** §5 |

No new top-level module. No new crate — `Path`/`PathBuf` component comparison is `std`; no `glob`,
no `shellexpand`, no `shell-words`. No new feature flag. No new config key: `--root` is a CLI flag
and stays one in v1 (a `SafetySection.roots` key is §7).

Note the global-flag trap found during this survey: `Cli` is declared with
`args_conflicts_with_subcommands = true`, so the global `--dry-run` and friends are unusable
alongside a subcommand. `Commands::Confine` therefore declares its own `--safety`, `--shell` and
`-o/--output`, following `Commands::Run`'s precedent. There is no `--json` flag anywhere in caro;
JSON is `-o json` via `OutputFormat::from_str`.

---

## 5. Integration tests

`tests/confinement_contract.rs`, `assert_cmd` idiom per `tests/caroml_e2e.rs`. Every row runs with
`--root /w --shell bash -o json` and asserts the **full** `verdict` sub-document and the exit code
— not field presence, which is the weakness `tests/e2e_cli_tests.rs::e2e_json_output_format` has
and which ADR-064 §4 already called out.

Rows 1–16 at `--safety moderate`:

| # | Input | verdict | notable operand | exit |
|---|---|---|---|---|
| 1 | `cat README.md` | `contained` | `README.md` inside/read | 0 |
| 2 | `ls` | `contained` | (no operands) | 0 |
| 3 | `cat /etc/passwd` | `escape` | outside/read | 2 |
| 4 | `cp .env /tmp/x` | `escape` | `/tmp/x` outside/**write** | 2 |
| 5 | `rm -rf ../other` | `escape` | escaping/write | 2 |
| 6 | `tar -C / -xf pkg.tgz` | `escape` | flag_value{`-C`} outside/write | 2 |
| 7 | `echo hi > /etc/motd` | `escape` | redirect outside/write | 2 |
| 8 | `sed -i.bak s/a/b/ src/x.rs` | `contained` | inside/**read_write** | 0 |
| 9 | `cat ~/.ssh/id_rsa` | `indeterminate{unplaceable_operand}` | unverifiable{home_anchored} | 0 |
| 10 | `git -C ~/other status` | `indeterminate{unplaceable_operand}` | flag_value{`-C`}/**write** | 0 |
| 11 | `cat "$SECRETS"` | `indeterminate{unplaceable_operand}` | unverifiable{variable_expanded} | 0 |
| 12 | `cat /w/../w/README.md` | `indeterminate{unplaceable_operand}` | unverifiable{traversal} | 0 |
| 13 | `cat ../*.env` | `escape` | escaping (**not** glob) | 2 |
| 14 | `cd /w/src && cat ../../etc/passwd` | `indeterminate{composed}` | `operands: []` | 0 |
| 15 | `find /w -name '*.log' -delete` | `indeterminate{unknown_head}` | `operands: []` | 0 |
| 16 | `frobnicate /w/x` | `indeterminate{unknown_head}` | `operands: []` | 0 |

Rows 17–20, the boundary and mode cases:

| # | Input | Expectation |
|---|---|---|
| 17 | `cat /workspace-backup/x` with `--root /workspace` | `escape` — segment-wise prefix, not `starts_with` |
| 18 | rows 9–16 re-run at `--safety strict` | same payloads, exit **2** |
| 19 | `""` (empty) | exit 1, **empty stdout**, message on stderr |
| 20 | `--root w` (relative) / `--root ~/w` / `--root /w/../w` | exit 1 each, message names the offending root (D9) |
| 21 | `cat /etc/passwd` with **no** `--root` | verdict `indeterminate{unplaceable_operand}`, operand `unverifiable{no_root}` but `access: read` populated, exit 0 (advisory mode) |

Properties asserted separately, with names that make a regression legible in CI output:

- **P1 — `Unverifiable` is never `Contained`.** `proptest` over arbitrary UTF-8: no input produces
  a report whose `verdict == Contained` while any operand is `Unverifiable`, `Outside` or
  `Escaping`. This is the fail-open guard for the predicted bad fix (ADR §Consequences).
- **P2 — `..` is never `Inside`.** For any operand `literal` containing a `..` path segment,
  `confinement != Inside`. This is CWE-61 designed out, asserted.
- **P3 — composition precedence.** Row 14 must be `Composed`, decided before any head lookup. A
  regression here reads a compound command by its first word, which is a fail-open.
- **P4 — traversal beats glob.** Row 13 must be `Escape`. D5's precedence order, pinned.
- **P5 — determinism.** Every row runs twice; byte-identical stdout. No timestamps, no durations,
  no host paths beyond the `roots` the caller supplied, no `$HOME` leakage.
- **P6 — exit 1 is not approval.** Rows 19–20 assert stdout is empty and the message is on stderr.
- **P7 — totality.** `classify` never panics and is total over arbitrary UTF-8 input, including
  invalid paths, NUL-adjacent bytes, and 100 KB single tokens.
- **P8 — segment-wise containment.** `proptest`: for roots `r` and any operand `o`, `Inside`
  implies `Path::new(o).components()` has `Path::new(r).components()` as a prefix.

---

## 6. Gate 3 — what breaks at 100 real users

**The assumption that holds at demo scale.** Every row in §5 is a single, bare, uncomposed
invocation with literal path operands. Real usage is `rsync -a ./ ../backup/ && echo done`,
`docker compose run --rm app sh -c "cp /app/.env /host/"`, `make deploy`,
`find . -name '*.tmp' -delete`, and `cat "$CONFIG_PATH"`. Every one of those is
`Indeterminate` by design: the first is `Composed`, the second is `Composed` twice over, the third
is an unknown head, the fourth is the `find -delete` guard, the fifth is `VariableExpanded`.

**The failure mode.** Not a wrong answer — `Indeterminate` is the honest response and D2/D3 make
it deliberately common. The failure is *uselessness*: if 70% of real invocations return
`Indeterminate`, consumers stop calling the verb, and an advisory nobody calls is worth nothing.
The dangerous second-order effect is a maintainer cutting the noise by mapping `Unverifiable` to
`Inside` — a silent fail-open of exactly the class this ADR exists to catch. P1 and P2 exist to
make that change loud, and their test names should say so.

**Instrumentation.** One counter over the existing evaluation corpus: `unverifiable_rate`, broken
down by `UnverifiableReason` and `IndeterminateReason`. Reported, not gated, in v1. The breakdown
is the decision input for what to build next: if `Composed` dominates, that is the evidence for
finally landing ADR-007's AST — and it will be the third ADR's worth of that evidence. If
`Traversal` dominates, the cheaper fix is `--resolve` (§7.1) and the data says so. If
`HomeAnchored` dominates, the cheapest fix is letting the caller pass `--home` explicitly, which
costs one flag and no filesystem I/O.

**Fallback.** The verb is advisory and additive. A consumer that ignores the exit code is exactly
as safe as it is today; nothing in the existing generation path changes behaviour. Deleting the
verb is a four-file revert.

---

## 7. Explicitly out of scope

Belongs to the next version, not this one:

1. **Real path resolution.** `--resolve`: `canonicalize`, symlink following, `$HOME` expansion.
   The `Resolution` enum (§2.1) is the seam that admits it without a schema break, and when it
   lands it must report `Resolution::Resolved` — the whole lesson of CVE-2025-59829 is that the
   two claims must be distinguishable in the payload.
2. **Composition analysis.** Pipelines, `&&`/`||`, subshells, `sh -c`, and the `cd`-then-relative
   case that `CwdRelativeAfterCd` reserves a slot for. Requires ADR-007's AST. v1 answers
   `Indeterminate { Composed }` (D6). Third consecutive ADR to owe this.
3. **Wrapper unwrapping.** Resolving `make deploy`, `npm run build` or `docker compose up` to the
   underlying command. Requires reading `Makefile` / `package.json` — filesystem I/O this verb
   deliberately does not do.
4. **Glob expansion.** `Unverifiable { Glob }` is the answer, not a directory listing.
5. **Windows path semantics.** Drive letters, `\` separators, UNC shares, case-insensitive
   comparison. v1 exits 1 on `--shell powershell|cmd` rather than answering wrongly.
6. **Ingesting the host's `permissions` block.** Reimplementing gitignore rule syntax, the four
   anchors and symlink pairing to derive roots from `additionalDirectories` /
   `blockReadsOutsideWorkingDirectories`. That is ADR-064's `SandboxProfile`, and it should stay
   there; `--root` is the vendor-neutral input.
7. **A config key.** `SafetySection.roots` / `confinement_roots` in `config.toml`. v1 is
   flag-only, so there is no precedence question between file and flag to answer yet.
8. **CaroML validator angle.** Registering `ConfinementAngle` in `default_chain()` requires
   `ValidatorContext` to grow a root set and `ValidationOutcome`/`Verdict` to become `Serialize`;
   both are real but separate changes (ADR §A4).
9. **Nesting into `caro.assessment.v1`.** Committed to in D7, executed when ADR-058/059 merges.
10. **Enforcement.** No `caro run --confine`, no `chroot`, no `unshare`, no sandbox construction.
    Caro emits the confinement contract and never holds it — the same non-negotiable as ADR-065 D2.

---

## 8. Open questions for the implementation PR

- **`git -C` as `Write`.** §3.2 chooses pessimism because the table keys on heads, not subcommands.
  A one-level subcommand peek (`git -C D status|log|diff` → `Read`) is cheap and does not need an
  AST. Worth doing in v1, or does subcommand knowledge start a slide toward the wrapper-unwrapping
  §7.3 explicitly refuses?
- **Advisory mode's exit code.** Row 21 exits 0 with `no_root`, on the theory that a caller who
  omitted `--root` asked for description, not judgment. The alternative — exit 1, "you must say
  what you meant" — is more honest and less useful. Confirm which reads better from a hook.
- **`Access::ReadWrite` for `sed -i`.** The in-place edit reads and writes the same path. One
  operand with `ReadWrite`, or two operands with the same `literal`? The former is smaller; the
  latter composes better with a future per-access policy.
- **Exit-code reconciliation.** D8 votes `2` and defers to whichever of ADR-064/065/066 merges
  first minting `CaroExitCode`. That decision should be made in the *first* of these PRs to merge,
  not negotiated three times. Worth a bead now, before any of the three has code.
- **Stable pattern IDs.** Same gap ADR-065 §7.8 recorded and did not need: `matched_patterns`
  carries lowercased description strings, and on an allowlist hit it carries the raw allowlist
  regex instead — the field is overloaded. This ADR is designed not to depend on it, but a report
  that embeds `matched_patterns` inherits the ambiguity. Worth the same bead.

---

## 9. References

Same set as ADR-066 §References. Caro line numbers verified 2026-09-07 against
`integrator/20260711-postmerge` and expected to drift. External sources read 2026-09-07.
