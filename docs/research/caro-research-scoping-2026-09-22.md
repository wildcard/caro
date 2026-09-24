# `caro-research--scoping-process` — run report, 2026-09-22

**Run type**: autonomous, scheduled, no user present.
**Output**: not a scope. Two new files in `tests/`, one amendment to `docs/adr/ADR-075`.
**Predecessor**: [`caro-research-scoping-2026-09-21.md`](./caro-research-scoping-2026-09-21.md).

---

## Why phase 0 did not merge

The 2026-09-21 report closed with an instruction to this run:

> Merge phase 0, then stop scoping for a while. […] If the next run of this task is a scoping
> run again, the honest thing for it to do is to open with why phase 0 did not merge.

It did not merge. Checked at the start of this run: `tests/lex_differential.rs` does not exist,
`tests/fixtures/guardfall/` does not exist, `src/` is unchanged since the ADR was written, and
the twenty scope documents from previous runs are still loose files in the repository root.

The reason is not that the work was hard. The harness is one 565-line test file and one YAML
fixture; it compiled on the first attempt against the existing public API, needed no new
dependency, and runs in 0.15 s. The reason is that **D0 as written could not be merged by
anyone.** D0 says the test "is expected to fail loudly" and that "merging a large red bar is
the point." `.claude/rules/dev-process.md` — Tier 2 of the constitution — requires that all
unit tests pass before review. A pull request whose whole contribution is a failing
`cargo test` asks a reviewer to suspend a Tier 2 rule in order to approve a Tier 3 artifact.
That is not a thing a reviewer does on a Tuesday, so nobody did it.

This is the same failure the ADR itself is about, at a different altitude: a specification that
was correct in substance and wrong about the machinery it had to pass through. Writing it a
second time, more insistently, would not have helped.

**The fix, applied here.** The red bar is kept in full and moved behind `#[ignore]`. Four green
tests gate the PR; four ignored tests are the bar. `census_is_reported` prints the whole census
on the green path, so the number reaches the reviewer through CI logs without CI going red for
it, and `cargo test --test lex_differential -- --ignored` reproduces the red bar in one flag.
ADR-075 is amended accordingly (see its *Amendment — 2026-09-22* section). Nothing else in the
ADR changed.

---

## What shipped

| File | Lines | State |
|---|---|---|
| `tests/lex_differential.rs` | 565 | new — the harness |
| `tests/fixtures/guardfall/mutations-v1.yaml` | 479 | new — the frozen corpus |
| `docs/adr/ADR-075-shell-lexical-normalization.md` | +79 | amended — D0 revised, results recorded |

Uncommitted, on the working tree, consistent with every previous artifact of this task.
`.claude/rules/git-workflow.md` requires a feature branch for any commit, and a scheduled run
with no user present does not open pull requests.

### Corpus

11 destructive baselines × the GuardFall mutation classes, every variant written out literally
so that adding one shows up in a diff. 110 variants: 99 adversarial (families A, B, C, D, F)
and 11 control (family G, doubled whitespace). Plus 4 interpreter sinks, 4 class-E flag
variants, 5 canaries, 2 multi-segment controls, 1 recorded false positive. Every id unique,
every mutation verified non-identical to its baseline by `corpus_integrity` — a no-op mutation
would silently inflate the catch rate, which is exactly the way this kind of harness lies.

### Harness

Drives `SafetyValidator::validate_command` — the shipped public API, at
`SafetyConfig::moderate()` and `ShellType::Bash`. No reimplementation of
`is_dangerous_in_context`, no direct access to the compiled pattern vector. The 2026-09-21
figure came from a Python port and was labelled a reproduction; this one is the product.

---

## The number

```
family  applied  caught   leaked
  A          22       0       22      quote removal
  B          22       0       22      $IFS / ${IFS} expansion
  C          22       0       22      command substitution
  D          11       0       11      base64 pipeline
  F          22       0       22      quote parity (Caro-specific)
  G          11      11        0      control — whitespace elasticity

adversarial (A,B,C,D,F): 99 leaked of 99
unmutated baselines:      11 caught of 11
```

Reproduce:

```bash
cargo test --no-default-features --features cve-rules \
  --test lex_differential -- --ignored --nocapture
```

**The port was accurate to the case.** 99 of 99 predicted, 99 of 99 measured, and the control
family still catches at 11 of 11, so the harness is measuring a gap rather than failing to
call the validator. CVE rules are compiled in (`--features cve-rules`) and move nothing.

That settles the open question the previous run left behind. It does not settle whether the gap
matters enough to spend phases 1–3 on; that is still a reviewer's call, and now it is a call
made against a number produced inside the product.

---

## Two findings the scoping run did not have

### 1. `# rm -rf /` is `Critical`

A shell comment. It cannot do anything. The product blocks it.

Same root cause as all 99 leaks — matching text without shell semantics — with the sign
reversed. It matters beyond the curiosity because ADR-075's *Consequences* section worries that
D2's opacity floor will raise the confirmation rate and that nobody has measured by how much.
Part of that rate is already being paid, and being paid for nothing. A lexer that knows what a
comment is removes a false positive at the same time as it closes a leak, which changes the
shape of the trade the ADR describes.

Recorded in the corpus as `known_false_positives` with its current verdict, owned by the
`#[ignore]`d `known_false_positives_are_fixed`. That test goes red two ways and both are
useful: the verdict drifted (corpus is stale) or the command became `Safe` (finding is fixed).

### 2. Class E is a total miss, not a partial one

```
find /etc -name '*.conf' -delete      Safe
sed -i 's/.*//' /etc/passwd           Safe
truncate -s 0 /var/log/syslog         Safe
shred -u /etc/shadow                  Safe
```

Four of four. ADR-075 lists class E under out-of-scope as "pattern coverage," and that framing
survives the measurement — lexical normalization would not catch these; a pattern would. But
the out-of-scope entry should read *four of four measured cases leak*, not *coverage is
incomplete*. These are cheap to fix and independent of everything else in the ADR.

Interpreter sinks behaved exactly as D4 predicted: `sh -c 'rm -rf /'` and its three siblings
all return `Safe`.

---

## Verification performed

| Check | Result |
|---|---|
| `cargo test --test lex_differential` (green path) | **4 passed, 0 failed, 4 ignored** |
| `cargo test … -- --ignored` (the bar) | **0 passed, 4 failed** — as designed |
| `rustfmt --edition 2021 --check` | clean |
| `cargo clippy` | **not run** — see below |
| New dependencies | none (`serde_yaml` already a dev-dependency) |
| Files under `src/` touched | none |

**Clippy is unrun and that is a real gap in this report.** The sandbox this task runs in has a
9.6 GB disk; the dependency tree plus a debug target filled it, and installing the clippy
component left no room to link `clap_derive`. The test build itself emitted no rustc warnings.
Whoever opens the PR should run `cargo clippy --all-targets -- -D warnings` before pushing —
this run cannot claim that bar was cleared.

The build used `--no-default-features --features cve-rules`, which excludes `embedded-mlx` and
`embedded-cpu`. Neither is reachable from the safety validator, so the verdicts are unaffected,
but CI runs the default feature set and this run did not.

---

## Recommendation to the next run

**One pull request, and it is mechanical:**

```bash
bin/sk-new-feature "guardfall differential harness"
# add tests/lex_differential.rs + tests/fixtures/guardfall/mutations-v1.yaml
# add the ADR-075 amendment
cargo clippy --all-targets -- -D warnings
cargo test --test lex_differential        # 4 passed, 4 ignored
gh pr create --title "test(safety): GuardFall differential harness (ADR-075 phase 0)"
```

No decision is required to merge it. It adds no type, no dependency, no verb, no config key,
and touches nothing under `src/`. If ADR-075 is rejected in full, the harness is still the
thing that told the project its guard leaks 99 of 99 mutations and blocks comments.

**Then the class-E patterns**, which are four regexes in `src/safety/patterns.rs` and need no
ADR either — the `#[ignore]` on `flag_variants_are_classified` comes off when they land.

**Do not scope anything new until one of those two has a PR number.** Twenty-one scope
documents and seventy-six ADRs now sit against zero merged artifacts from this task. The
constraint is not analysis.

---

## Sources

Feature research for this run is inherited from
[`caro-research-scoping-2026-09-21.md`](./caro-research-scoping-2026-09-21.md) and not repeated:
[CSA GuardFall](https://labs.cloudsecurityalliance.org/research/csa-research-note-guardfall-ai-coding-agent-shell-injection/) ·
[Adversa AI](https://adversa.ai/blog/opensource-ai-coding-agents-shell-injection-vulnerability/).

**Caro internal** — `src/safety/mod.rs` (`SafetyValidator::validate_command` l.459,
`ValidationResult` l.175, `SafetyConfig::moderate` l.660) ·
`src/models/mod.rs` (`RiskLevel` l.152, `ShellType` l.417) · `Cargo.toml`
(`[dev-dependencies] serde_yaml`, `serial_test`, `tokio-test`) ·
`docs/adr/ADR-075-shell-lexical-normalization.md` ·
`.claude/rules/dev-process.md`, `.claude/rules/git-workflow.md`,
`.claude/rules/constitution.md`.
