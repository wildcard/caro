# caro Safety Philosophy

> *"In the kernel, every byte you read from userspace is hostile until proven
> otherwise, and every resource you acquire must be released even if the
> caller lied to you about its inputs."*

That sentence — paraphrased from the [FreeBSD Device Driver Book][fdd] — is
the mental model we apply to caro. Even though caro is a userspace CLI, the
trust boundary it sits on is the same one a device driver sits on: untrusted
text on one side, a privileged shell on the other, and a regex‑sized airlock
in between. Everything in the codebase is built around that assumption.

This document explains the *why* behind the layered defenses. For the *what*
and *how*, see [`SECURITY.md`](../SECURITY.md) and `src/safety/patterns.rs`.

---

## The four mindsets we steal from kernel engineering

### 1. Every command is untrusted, even from your own LLM

A driver does not trust the bytes the application hands it through `ioctl`,
even if the application is signed and the bytes come from kernel‑adjacent
code. Caro applies the same rule to the LLM: a model the user trusts can
still emit a command they don't trust, because the model is a **proximate**
producer of the command, not its origin.

In practice this means:

- LLM output is always parsed as plain text, never as Rust code
- Every command flows through `SafetyValidator::validate_command` regardless
  of which backend produced it (`embedded`, `static`, `ollama`, `vllm`)
- The validator is symmetric: it does not care whether the caller is `caro
  <prompt>`, `caro ai`, or a future scripted entry point

### 2. Resource lifecycles must be unforgiving

Drivers acquire and release resources in pairs because a kernel cannot afford
a leak. We hold the same line on caro's hot paths:

- **Cache manifest**: `Arc<RwLock<ManifestManager>>` with paired
  acquire/release; checksum verification before any read returns
- **Embedded backend**: model load → inference → unload, with errors
  returning `GeneratorError` rather than propagating panics through the loop
- **Agent loop**: hard `_max_iterations: 2` cap and a 15‑second timeout,
  re‑checked mid‑refinement so a slow backend cannot starve the user
- **No `unsafe`**: zero `unsafe { }` blocks in the crate; FFI/concurrency
  ride on `tokio`, `Arc`, `Mutex`, `RwLock` and the `regex` crate

When you add a new backend or cache layer, the contract is: if you grab a
lock, you give it back; if you open a model, you close it; if you start a
loop, you bound it. The audit at `~/.claude/plans/what-can-we-learn-buzzing-bubble.md`
walks the current state.

### 3. Defense in depth, not perimeter security

A driver layered against bad hardware doesn't trust any single check.
Neither do we. Every command sees, in order:

1. **Allowlist short‑circuit** — explicit user‑configured allow patterns
   skip the rest of the pipeline (`src/safety/mod.rs:223`)
2. **Built‑in dangerous patterns** — 62 pre‑compiled `Regex` rules covering
   GNU/Linux, BSD, Windows, and PowerShell destructive commands
   (`src/safety/patterns.rs`)
3. **CVE rules** — pattern set compiled from `data/cve_rules/*.yaml` and
   shipped as a `bincode` blob; failure to deserialize falls back to an
   empty set rather than panicking (`src/safety/cve_patterns.rs`)
4. **Custom user patterns** — organization‑specific rules added through
   `SafetyConfig::add_custom_pattern` (`src/safety/mod.rs:274`)
5. **User confirmation gate** — for `Moderate` and above commands, the CLI
   asks for explicit confirmation before any execution

A bypass at one layer should not produce execution. Adding a new safety
pattern that lives only in the LLM prompt is **not** defense in depth — it
must land in `patterns.rs` (or a CVE YAML rule).

### 4. Cross‑platform discipline beats `cfg!` sprawl

The FDD‑book book chapter on portability (Ch. 29) argues for runtime
detection over compile‑time gates whenever the runtime cost is
imperceptible. Caro follows that:

- `PlatformContext::detect()` runs *once* per process and feeds every
  downstream consumer
- The 4‑profile capability model (`Gnu` / `Bsd` / `Busybox` / `Hybrid`)
  describes the **userland** the user actually has installed — Homebrew GNU
  coreutils on macOS flips the profile to `Gnu` even though `uname` says
  Darwin
- The `BsdFlavor` sub‑classification (`FreeBsd` / `OpenBsd` / `NetBsd` /
  `MacOs` / `DragonFlyBsd`) describes the **kernel family** independently
  of userland — it tells the LLM "you can speak `pkg`/`jail`/`gpart` here"
  without lying about which `find` flags work

When you add a new BSD‑specific or Linux‑specific safety pattern, ask
yourself: is this protecting against the **kernel** doing something
destructive, or against the **userland** misinterpreting a command? That
answers whether your test should mock `BsdFlavor` or `UtilityType`.

---

## Why the FDD‑book lessons fit a userspace CLI

The FreeBSD Device Driver Book (`ebrandi/FDD-book`) is about kernel
engineering, but its central discipline transfers cleanly to any program
that crosses a privilege boundary on someone else's behalf. The chapters
that informed caro most directly:

| Chapter | Lesson | Where it lives in caro |
|---|---|---|
| Ch 29 — Portability | Runtime platform detection beats compile gates | `src/platform/mod.rs` capability profile + `BsdFlavor` |
| Ch 31 — Security Best Practices | Every privileged surface needs symmetric checks | `src/safety/patterns.rs` 62 patterns; BSD set added in PR #1005 |
| Ch 36 — Reverse Engineering | When docs lie, the binary is the truth | Why we ship a CVE harness and don't trust LLM output for safety |
| Ch 37 — Submitting to FreeBSD | Contributors deserve a stable, narrow API | The `safety-pattern-developer` skill + TDD workflow for new patterns |

The chapters about hardware DMA, taskqueues, and bus probing are not
applicable to caro. We're not pretending this is a driver — we're admitting
the discipline is the same.

---

## What this means for contributors

When you propose a change that touches `src/safety/`, `src/platform/`, or
any inference backend's lifecycle:

1. **Land tests first.** Use the `safety-pattern-developer` skill for new
   patterns. RED before GREEN, every time.
2. **Add to `patterns.rs`, not the prompt.** A safety rule that depends on
   the LLM "remembering" it is not a safety rule.
3. **Symmetric across backends.** If you skip the validator for one
   backend, you've broken the contract.
4. **Document the *why*.** A new pattern's docstring should answer "what
   does the user *think* they're doing, and why is the kernel about to do
   something else." See the BSD pattern block in `src/safety/patterns.rs`
   for the format.
5. **Bound the loop.** If you add an iterative refinement step, give it a
   hard cap and a timeout. See `src/agent/mod.rs:68-69`.

---

## See also

- [`SECURITY.md`](../SECURITY.md) — vulnerability disclosure policy and the
  user‑facing security contract
- [`CONTRIBUTING.md`](../CONTRIBUTING.md) — development workflow
- `src/safety/patterns.rs` — the 62‑pattern library
- `src/platform/mod.rs` — `PlatformContext`, `UtilityType`, `BsdFlavor`
- [FreeBSD Device Driver Book][fdd] — the systems‑engineering text this
  document cites

[fdd]: https://github.com/ebrandi/FDD-book
