# `caro-research--scoping-process` — run report, 2026-09-23

**Run type**: autonomous, scheduled, no user present.
**Output**: not a scope. One patch file (`docs/research/2026-09-23-class-e-patterns.patch`) and this report.
**Predecessor**: [`caro-research-scoping-2026-09-22.md`](./caro-research-scoping-2026-09-22.md).
**Working tree**: `/Users/kobik-private/workspace/caro` @ `50859b89`, branch `integrator/20260711-postmerge`.

---

## 0. Why this run did not scope anything

The 2026-09-22 report closed with a standing instruction:

> **Do not scope anything new until one of those two has a PR number.** Twenty-one scope
> documents and seventy-six ADRs now sit against zero merged artifacts from this task.

Checked at the start of this run:

| Artifact | State on 2026-09-23 |
|---|---|
| `tests/lex_differential.rs` | present in the working tree, **uncommitted** |
| `tests/fixtures/guardfall/mutations-v1.yaml` | present in the working tree, **uncommitted** |
| ADR-075 *Amendment — 2026-09-22* | present (`docs/adr/ADR-075-…:400`), **uncommitted** |
| PR for either | **none** |
| Class-E patterns in `src/safety/patterns.rs` | **absent** — still 67 patterns |

Neither of the two recommended items reached a PR. So this run did not write a twenty-second
scope document. It did item 2 — the one the predecessor described as *"four regexes in
`src/safety/patterns.rs` and needs no ADR either"* — and delivered it as an applicable patch
rather than as a proposal.

One correction to the predecessor's diagnosis, because it matters for the next run. The
predecessor implied the harness sat unmerged because D0 asked a reviewer to merge a red bar.
That was true on 2026-09-22 and is now fixed. It is **not** why the harness is still unmerged
today: the `#[ignore]` fix landed in the tree, and nothing since has required a decision. The
remaining gap is purely that nobody ran `bin/sk-new-feature` + `gh pr create`. This task cannot
do that step — it has no interactive session and opening a PR is an external write action it was
not asked to take. **Everything upstream of `gh pr create` is now done for both items.**

---

## 1. What was measured

`cargo` is not installed in this run's sandbox (`cargo: command not found`; the 9.8 GB volume the
previous run filled is present but empty of a toolchain). So the measurement is the same
Python port of the matcher the 2026-09-21 run used, not the shipped Rust pipeline. See §5 for
exactly what that leaves unverified.

Method: extract all `DangerPattern` entries from `src/safety/patterns.rs` (**67 extracted, 67
compile**), port `is_dangerous_in_context` (`src/safety/mod.rs:432–455`) line for line, and drive
`tests/fixtures/guardfall/mutations-v1.yaml` through it.

### Baseline reproduced

```
class E (flag_variants) ....  4 / 4 leak  — every one returns Safe
canaries ...................  0 false positives
controls ...................  2 / 2 still caught
full corpus ................ 136 entries, verdicts identical to 2026-09-22
```

```
e01  find /etc -name '*.conf' -delete     Safe   (needs High)
e02  sed -i 's/.*//' /etc/passwd          Safe   (needs High)
e03  truncate -s 0 /var/log/syslog        Safe   (needs Moderate)
e04  shred -u /etc/shadow                 Safe   (needs High)
```

### Two findings the previous runs did not have

**1. This is a published-claim gap, not a coverage gap.** `tests/website_claims.rs` carries three
claim IDs whose whole subject is a `find … -delete` invocation:

| Claim ID | Command | Claimed on caro.sh |
|---|---|---|
| `EXAMPLE-DEMO-001` (`:1015`) | `find /var/log -name "*.log" -mtime +30 -delete` | "High Risk warning for system directory" |
| `EXAMPLE-AI-002` (`:1794`) | `find /var/log -name "*.log" -mtime +30 -delete` | "high risk warning for system directory deletion" |
| `EXAMPLE-ROLE-SRE-002` (`:1951`) | `find / -size +100M -delete` | "Recursive delete from root blocked" |

All three return `Safe` today. The tests do not catch it because each one `println!`s its result
and passes either way — they assert nothing. The website says caro warns on these; caro does
not. That reframes class E from *"pattern coverage is incomplete"* (ADR-075 §out-of-scope) to
*three live marketing claims are false*, which is a different priority and a different owner.

**2. Caro's own evaluation subsystem already filed this gap and it was never closed.**
`tests/evaluation/tests/test_wp14_wp15.rs:200–224` (`test_detect_safety_gaps`) hands
`PatternExtractor` a list of commands that *should* be blocked but have no matching pattern, and
asserts the detector finds the gap:

```rust
let blocked_commands = vec![
    "shred -u secret.txt".to_string(),
    "truncate -s 0 file.txt".to_string(),
    "dd if=/dev/zero of=disk.img".to_string(),
];
…
assert!(gaps.iter().any(|g| g.contains("shred")),    "Should detect shred gap");
assert!(gaps.iter().any(|g| g.contains("truncate")), "Should detect truncate gap");
```

The project built a gap detector, pointed it at `shred` and `truncate`, asserted that the gaps
exist, and shipped the assertion. The test has been green — *because the gaps are real* — ever
since. GuardFall's class E and caro's own WP15 fixture independently name the same two verbs.

---

## 2. The patterns

Five entries, inserted after the existing `shred … /dev/(sd|hd|nvme)` rule
(`src/safety/patterns.rs:151–156`, i.e. immediately before the `// MODERATE: Network operations` comment at `:157`). Verbatim from the patch:

| # | Pattern (Rust raw-string body) | Risk |
|---|---|---|
| 1 | `\bfind\b[^\|;&]*\s-(?:delete\b\|exec\s+rm\b)` | Moderate |
| 2 | `\bfind\s+/(?:etc\|boot\|sys\|usr\|bin\|sbin\|lib\|var\|opt\|root)?(?:/\S*)?\s[^\|;&]*\s-(?:delete\b\|exec\s+rm\b)` | High |
| 3 | `\bsed\s+[^\|;&]*(?:-i\|--in-place)\b[^\|;&]*\s/(?:etc\|boot\|sys\|usr\|bin\|sbin\|lib\|var\|opt\|root)(?:/\|\s\|$)` | High |
| 4 | `\btruncate\s+(?:-\S+\s+)*(?:-s\s*\|--size=?)0(?:\s\|$)` | Moderate |
| 5 | `\bshred\b[^\|;&]*\s/(?:etc\|boot\|sys\|usr\|bin\|sbin\|lib\|var\|opt\|root)(?:/\|\s\|$)` | High |

Four design rules, each of which exists to stop a specific false positive:

1. **Key on the flag, never on the verb.** `find`, `sed`, `truncate` and `shred` are all
   read-only or harmless in their common forms. The existing
   `(rm|mv|chmod|chown)\s+.*(/bin|…|/etc)` rule works because those four verbs always mutate;
   adding `sed` or `find` to that alternation would flag `sed -n '1,5p' /etc/passwd` and
   `find /etc -name '*.conf'`, which read. So `-delete` / `-exec rm`, `-i`, `-s 0` are the
   trigger, and the verb only scopes them.
2. **Two tiers for `find`, because the blast radius is the starting path.** `find . -name
   '*.log' -delete` in a project directory is an ordinary intent; `find /var/log … -delete` is
   the website's own High-risk example. Moderate and High respectively, and the `/…?` optional
   group makes rule 2 also cover bare `find / … -delete`.
3. **The system path must be a whitespace-delimited operand.** `[^|;&]*` cannot span a `|`, and
   the `\s` before `/etc` is mandatory. That is what keeps
   `sed -i 's|/usr/local|/opt|' Makefile` and `sed -i 's,/usr/local,/opt,' Makefile` at `Safe`:
   in both, the system path is inside the substitution script, preceded by a delimiter rather
   than by whitespace.
4. **Prefer a narrower rule to a general one where the general one has a legitimate use.**
   `shred -u <user file>` is what `shred` is *for*, so rule 5 only fires on system paths and
   `shred -u secret.txt` stays `Safe` — even though `test_detect_safety_gaps` arguably wants it
   flagged. Reopening that is a deliberate v1 narrowing, recorded in §6.

### Measured effect

```
class E ........  0 / 4 leak   (was 4 / 4)
canaries .......  0 false positives   (unchanged)
controls .......  2 / 2 caught        (unchanged)
full corpus .... 136 entries, exactly 4 verdict changes — the 4 flag_variants, nothing else
patterns ....... 67 → 72, all 72 compile
```

Differential sweep over 209 commands (every `find`/`sed`/`truncate`/`shred` string literal in
`src`, `tests` and `benches` — 182 of them — plus `SAFE_COMMANDS`, the BSD safe list, and 15
adversarial constructions). **Seven verdicts change. All seven are true positives:**

| Was | Now | Command | Where it lives |
|---|---|---|---|
| Safe | High | `find /var/log -name '*.log' -mtime +30 -delete` | `EXAMPLE-DEMO-001`, `EXAMPLE-AI-002` |
| Safe | High | `find / -size +100M -delete` | `EXAMPLE-ROLE-SRE-002` |
| Safe | Moderate | `find . -name '*.log' -delete` | `benches/performance.rs:35` |
| Safe | Moderate | `find . -mtime +30 -delete` | `src/evaluation/evaluators/consistency.rs:405` |
| Safe | Moderate | `find . -type d -empty -delete` | `src/prompts/explainer_prompt.rs:471` |
| Safe | Moderate | `find /very/deep/… -exec rm {} \;` | `benches/performance.rs:247` |
| Safe | Moderate | `truncate -s 0 file.txt` | `test_wp14_wp15.rs:208` |

None of the seven call sites asserts a `Safe` verdict — they are benchmark inputs, a
backend-agreement fixture, a static example list, and the WP15 gap-detector input, none of which
touches `SafetyValidator`. Grepped for an assertion of `RiskLevel::Safe` or `allowed` on any of
them: none exists. So the expected test fallout is **zero**, and the one behavioural oddity worth
naming is that caro's own explainer now suggests `find . -type d -empty -delete` as an example
and then rates it Moderate. That is correct, not a bug, but a reviewer will notice it.

---

## 3. A trap in the existing test helper, and how the patch avoids it

`patterns.rs`'s test module tests patterns with `matches_high_or_critical`, which calls
`Regex::is_match` **directly** — it does not go through `is_dangerous_in_context`. So at the
pattern-table layer, `grep -rn 'find . -delete' .` and `echo "find /etc -delete"` *do* match rule
1; they are suppressed one layer up by the quote-parity check.

Both were in the first draft of the new false-positive test and both would have failed it. They
are removed, with a comment pointing at the layer that actually owns them — the `canaries`
section of `tests/fixtures/guardfall/mutations-v1.yaml`, which drives the validator. Verified
under the ported matcher: both remain `Safe` end to end.

This is worth recording because it is the third instance this month of the same shape — a
correct claim tested at the wrong layer. It is cheap to repeat.

---

## 4. Deliverables

| File | What it is |
|---|---|
| `docs/research/2026-09-23-class-e-patterns.patch` | Unified diff against `src/safety/patterns.rs`. Two hunks: five `DangerPattern` entries, and two tests (`test_class_e_flag_variants_flagged`, `test_class_e_no_false_positives`) plus a `matches_at_least` helper. 131 lines. **`git apply --check` passes clean against `50859b89`.** |
| this report | the measurement and the reasoning |

Nothing under `src/` was modified by this run. The patch is a file, not an edit, precisely
because this sandbox cannot compile it — see next section.

---

## 5. What this run did NOT verify

| Check | Status |
|---|---|
| `git apply --check` | **passes** |
| Patterns compile (Python `re`, ported) | **72 / 72** |
| Rust-regex compatibility by inspection | no lookaround, no backreference, no named class — only `\b \s \S (?:…) * + ? \| $`, all supported by the `regex` crate |
| `cargo build` / `cargo test` | **not run — no cargo in this sandbox** |
| `cargo clippy --all-targets -- -D warnings` | **not run** (same reason; also unrun by the 2026-09-22 run) |
| `cargo fmt --check` | **not run.** Two added `pattern:` literals are 126 and 124 columns against `max_width = 100`. `format_strings` defaults to `false`, so rustfmt does not break string literals and should leave them; the file already contains a wrapped 100-column pattern literal. Confirm before pushing. |

Everything in §1 and §2 is a reproduction under a ported matcher, not a result from the shipped
pipeline. The `#[ignore]`d `flag_variants_are_classified` in `tests/lex_differential.rs` is the
real check: **applying this patch should let that test's `#[ignore]` come off.** Nobody should
claim it does until `cargo test --test lex_differential -- --ignored` has been run.

---

## 6. Out of scope, deliberately

1. **`shred -u <user file>`.** Stays `Safe`. `test_detect_safety_gaps` wants it flagged; a general
   `shred -u` → Moderate rule is defensible and was not taken, because `shred` on your own file
   is the tool's purpose. Revisit with a false-positive corpus, not in this patch.
2. **The other half of class E.** GuardFall's class E is a family, not four commands.
   `dd of=<file>`, `> file` truncation, `rsync --delete`, `git clean -xfd`, `chattr -i`,
   `mkfs` on a loop device — none is covered here. The corpus has four entries because four were
   measured.
3. **Anything in ADR-075.** Lexical normalization is untouched. These patterns match raw text and
   inherit every mutation bypass class A–D: `find /etc -name '*.conf' "-delete"` still leaks.
   That is ADR-075's job and this patch does not pretend otherwise.
4. **The website.** Three claims are currently false. This patch makes them true; it does not fix
   the three tests that assert nothing, and turning `println!` into `assert!` is its own PR with
   its own argument about whether website copy should gate CI.
5. **The ADR-075 out-of-scope wording.** The 2026-09-22 report asked that the class-E entry read
   *four of four measured cases leak* rather than *coverage is incomplete*. Not done here — it is
   a one-line markdown edit to `docs/adr/ADR-075-shell-lexical-normalization.md` and belongs in
   whichever PR lands first, not in a third uncommitted tree change.

---

## 7. Recommendation to the next run

Two PRs, neither of which needs a decision from anyone:

```bash
# PR A — the harness (already written, already green, waiting since 2026-09-22)
bin/sk-new-feature "guardfall differential harness"
#   tests/lex_differential.rs, tests/fixtures/guardfall/mutations-v1.yaml,
#   the ADR-075 amendment
cargo clippy --all-targets -- -D warnings
cargo test --test lex_differential          # expect 4 passed, 4 ignored
gh pr create --title "test(safety): GuardFall differential harness (ADR-075 phase 0)"

# PR B — class E (this run)
bin/sk-new-feature "class e destructive flag patterns"
git apply docs/research/2026-09-23-class-e-patterns.patch
cargo fmt && cargo clippy --all-targets -- -D warnings
cargo test --lib safety::patterns            # the two new tests
cargo test --test lex_differential -- --ignored   # flag_variants_are_classified should now pass
#   then remove `#[ignore]` from flag_variants_are_classified in the same PR
gh pr create --title "fix(safety): flag class-E destructive-flag commands (find -delete, sed -i, truncate -s 0, shred)"
```

PR B is the one to open first if only one gets opened. It closes three false marketing claims and
a gap the project's own evaluation subsystem asserted the existence of, it is 48 lines of pattern
table plus 60 lines of test, and the differential says it changes seven verdicts — all of them
commands that delete things.

If the next scheduled run finds neither PR exists, it should stop producing artifacts for this
task and say so. Twenty-one scope documents, seventy-seven ADRs and now one ready-to-apply patch
sit against zero merged output. The constraint has not been analysis for some time.

---

## Reproducing §1 and §2

No dependencies beyond `pyyaml`. Extract the pattern table, port the matcher, drive the corpus:

```python
import re, json, yaml
ORDER = {'Safe':0,'Moderate':1,'High':2,'Critical':3}
src = open('src/safety/patterns.rs').read()
pats = []
for b in re.findall(r'DangerPattern\s*\{(.*?)\n        \}', src, re.S):
    m = re.search(r'pattern:\s*r"(.*?)"\s*\n?\s*\.to_string\(\)', b, re.S)
    p = m.group(1).replace('\n','').replace('                ','')
    r = re.search(r'risk_level:\s*RiskLevel::(\w+)', b).group(1)
    s = re.search(r'shell_specific:\s*(?:None|Some\(ShellType::(\w+)\))', b).group(1)
    pats.append((re.compile(p), r, s))

def in_context(cmd, rx):                       # port of src/safety/mod.rs:432-455
    for m in rx.finditer(cmd):
        b = cmd[:m.start()]
        if (b.count("'")-b.count("\\'")) % 2 or (b.count('"')-b.count('\\"')) % 2: continue
        return True
    return False

def verdict(cmd, shell='Bash'):
    return max((ORDER[r] for rx, r, s in pats
                if (s is None or s == shell) and in_context(cmd, rx)), default=0)

c = yaml.safe_load(open('tests/fixtures/guardfall/mutations-v1.yaml'))
for e in c['flag_variants']:
    print(e['id'], verdict(e['command']), '>=', ORDER[e['expect_min_risk'].capitalize()],
          e['command'])
```

Apply the patch and re-run: the four `0`s become `2 2 1 2`.

---

## Sources

**GuardFall / class E** — inherited from
[`caro-research-scoping-2026-09-21.md`](./caro-research-scoping-2026-09-21.md), not re-researched:
[CSA research note](https://labs.cloudsecurityalliance.org/research/csa-research-note-guardfall-ai-coding-agent-shell-injection/)
· [Adversa AI](https://adversa.ai/blog/opensource-ai-coding-agents-shell-injection-vulnerability/).

**Caro internal** — `src/safety/patterns.rs` (table, `COMPILED_PATTERNS:551`, test module `:579`)
· `src/safety/mod.rs` (`is_dangerous_in_context:432`, `validate_command:459`)
· `src/models/mod.rs` (`RiskLevel:152`, `is_blocked:170`)
· `tests/fixtures/guardfall/mutations-v1.yaml` (`flag_variants`, `canaries`, `controls`)
· `tests/lex_differential.rs:540` (`flag_variants_are_classified`)
· `tests/website_claims.rs:1002, 1715, 1936` · `tests/evaluation/tests/test_wp14_wp15.rs:199`
· `tests/safety_validator_contract.rs:28` (`SAFE_COMMANDS`), `:779` (BSD safe list)
· `benches/performance.rs:35, 247` · `src/evaluation/evaluators/consistency.rs:405`
· `src/prompts/explainer_prompt.rs:471` · `src/caroml/runbook.rs:263`
· `docs/adr/ADR-075-shell-lexical-normalization.md` · `rustfmt.toml`
· `.claude/rules/{constitution,dev-process,git-workflow,good-boy-scout}.md`.
