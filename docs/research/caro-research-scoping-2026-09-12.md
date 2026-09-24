# caro-research--scoping-process — Run Report

**Date**: 2026-09-12
**Agent**: Automated scheduled task (`caro-research--scoping-process`)
**Status**: ✅ Completed — ADR-069 produced, adversarially reviewed, and **rescoped**
**Headline**: the review found a **live arbitrary-binary spawn in `src/agent/mod.rs:790`** that
nobody was looking for. See "The Actual Defect" below — it outranks the ADR.

---

## What Happened This Run

The task's SKILL.md still carries the unfilled `[FEATURE NAME]` placeholder
(outstanding since 2026-06-02). Resolved autonomously by the now-conventional
route: take the highest-priority uncovered engineering item from the most
recent Hermes scan.

1. **Selection signal.** The 2026-09-09 weekly memo names **3.10 — ambient-state
   preflight** as its §4 "build or test next: one thing", priority **Now**,
   complexity **S**, and defends displacing the standing session-state spike on
   evidence quality (a disclosed defect class with assigned CVEs, versus an
   inferred market pattern). Of the memo's four new opportunities, 3.11
   (published recall) and 3.13 (sandbox boundary paragraph) are documentation,
   3.12 (hook adapter) is a spike the memo explicitly couples *behind* 3.10
   ("do not ship a gate whose config the gated process can rewrite"). 3.10 is
   the only one that adds user-facing capability.
2. **Coverage check.** No ADR covers it. ADR-044 resolves a *command's* path
   effects; ADR-066 states outright that it "touches no filesystem"; ADR-047
   allowlists *targets*. Grep confirms nothing in `src/` reads `.git/config`,
   `.claude/settings.json`, `.vscode/tasks.json`, `.envrc` or `.git/hooks/`.
3. **Phase 1 analogs researched** (all read live, 2026-09-12):
   - **VS Code Workspace Trust** as the mature prior art, verified against
     `microsoft/vscode` @ 1.139.0 source rather than docs.
   - **GitSpawn** (Manifold Security, 2026-09-01) and **ChainDrop** (Pillar
     Security, 2026-08-04) as the failure corpus, plus the five-year CVE
     lineage back to CVE-2021-43891.
   - **direnv / Neovim `:trust` / Emacs `safe-local-variable-values`** as the
     content-addressed counter-designs, and Vim's `securemodelines` as the
     allowlist-beats-blocklist proof.
   - **guardskill, repotricks, codex-preflight, deepsafe-scan,
     agentsec-scanner, snyk/agent-scan, TruffleHog, gitleaks** for output
     contracts and their defects.
4. **Phase 2/3 output**: `docs/adr/ADR-069-ambient-configuration-preflight.md` —
   `caro.preflight.v1`: one verb, one new module (five files), no new dependency,
   **no new exit code**, 10 new serializable types, an 18-row rule table, 16
   integration tests, 11 out-of-scope items, 7 alternatives considered, and 5
   named preconditions on the implementation PR.

5. **Phase 4 (added this run): Gate 4 adversarial review.** `validation-discipline.md` requires
   the [`devils-advocate`](../../.claude/agents/devils-advocate.md) agent to review any AI-drafted
   proposal. Since this run had no human present, the review was run in-session against the first
   draft. **Verdict: Revise — and the revision is a rescope, not an edit pass.** Seven objections;
   five falsified specific claims against the tree. All are answered in ADR-069 §0. This is the
   first run of this scheduled task to run its own Gate 4, and on this evidence it should become
   standard: the review changed the scope, the CLI surface, the source set, two tests and the
   exemption claim, and it found the defect below.

---

## The Actual Defect — `src/agent/mod.rs:790` ★

**This is the most valuable output of the run and it is not the ADR.**

While pressure-testing ADR-069's claim to a "defect-class response" exemption, the review asked
the obvious question the ADR had not: *is Caro itself exposed to this class?* Two answers, and
the second one matters.

**Caro is not exposed to GitSpawn.** There is no `git` subprocess anywhere in `src/`. Git-repo
detection is a filesystem stat (`src/context/directory.rs:77`, `path.join(".git").exists()`); the
only git spawns are in `build.rs:14,23,32`, at compile time, in the developer's own checkout.

**Caro has its own instance of the same class, live on `main`.** Verified this run:

```rust
// src/agent/mod.rs:790-812  — get_command_info()
let version = Command::new(command).arg("--version").output()...
let help_text = Command::new(command).arg("--help").output()...
```

`command` arrives from `extract_commands()` (`src/agent/mod.rs:749-774`), which splits a
**model-generated** command string on `| ; &` and takes the first whitespace token of each part.
The only filter is a nine-keyword `matches!` against shell builtins
(`if then else fi while do done > < >>`). There is:

- **no allowlist** of permitted binaries;
- **no path-shape check** — a token of `./gradlew` or `/tmp/x` is executed as given;
- **no `.current_dir()`** — it inherits the process cwd, i.e. the untrusted directory;
- **no `env_clear()`** — it inherits the full environment.

Reached from `src/agent/mod.rs:353` via `get_command_context()`, on the agent loop's refinement
iteration (triggered by `confidence_score < 0.8` / `should_refine()` at `:726-746`) — i.e.
**before the user sees the suggestion, with no prompt.** If the model proposes `./gradlew build`,
Caro runs the repository's script. If it proposes `npm install`, Caro runs `npm --version` in a
directory where `./.npmrc` containing `node-options=--require ./pwn.js` is arbitrary code
execution — the identical ambient-config sink as `core.fsmonitor`, in Caro's own process.

**Fix is ~20 lines**: reject any token containing `/`, resolve the rest against an explicit
allowlist or `PATH` lookup, `env_clear()` with a minimal env, and `current_dir` to a neutral
directory. This *is* covered by `validation-discipline.md`'s exemption (a bug fix in a thing Caro
already has), needs no transcripts, and should land before ADR-069 is implemented — shipping a
verb that audits other people's config-execution sinks while carrying one is not a defensible
position.

Not filed as a bead (this task is read-mostly and the beads state is not mine to write); it is
follow-up #1 below.

---

## Two Corrections to the 2026-09-09 Hermes Memo

Both were found by primary-source verification and both change what the ADR can
cite. Worth feeding back into the scan process, which the memo's own skeptic's
note already flagged as trusting secondary aggregators.

1. **The GitSpawn CVE attribution is wrong in the memo, in both directions.**
   Only **two** CVEs are Manifold findings: CVE-2026-72718 (Goose, CVSS 4.0 =
   7.0, fixed 1.44.0) and CVE-2026-71963 (Hermes Agent, assigned by **VulnCheck
   as a third-party CNA** because Nous Research never triaged the private GHSA
   after six contacts across five channels; still unpatched). CVE-2026-19592
   (Codex CLI) is real and is the same `core.fsmonitor` mechanism, but traces to
   an **earlier, different researcher's** report — Manifold's Codex submission
   was closed as its duplicate. CVE-2026-55607 (Claude Code) is *"Sandbox Escape
   via Git Worktree Path Confusion"* (2.1.38 → fixed 2.1.163), requires
   prompt-injection content in the repo to steer the agent, and is **not** the
   zero-interaction startup path. **Neither of Manifold's two Claude Code
   findings received a CVE** — both were closed as duplicates.
2. **`git.path` is not a workspace-trust-gated setting.** The memo lists it as a
   check target. It is `ConfigurationScope.MACHINE` in VS Code and can never be
   set from a workspace, trusted or not. The memo should distinguish
   `scope: MACHINE` (never workspace-settable, strictly stronger) from
   `restricted: true` (workspace-settable, applied only when trusted). The
   better citation for the same argument is **CVE-2026-33068 / GHSA-mmgp-wc2j-qcv7**
   (CWE-807, CVSS 7.7), which the memo does not mention: Claude Code resolved
   `permissions.defaultMode` from the repo-controlled `.claude/settings.json`
   *before* deciding whether to show the trust dialog, so a repository shipping
   `bypassPermissions` skipped it. Fixed 2.1.53. That is the cleanest statement
   of the problem the ADR exists to solve, and it is now ADR-069's §D7.

One thing the memo got exactly right and understated: it says GitSpawn is the
first item in eight weeks that says Caro's *threat model* is incomplete. The
research supports that and adds that the class is **five years old** — CVE-2021-43891
was `.git/config` `core.fsmonitor` executed by VS Code's git extension before the
trust prompt, fixed in 1.63.1 for a $30k bounty. Cross-vendor twin: CVE-2022-24346
(JetBrains). Nobody solved it; everyone patched their own call site.

---

## What the Gate 4 Review Changed

Recorded per `validation-discipline.md` Gate 4 ("the spec author addresses each objection … either
by revising the spec or by recording the deliberate decision to ignore the objection"). Full
response is ADR-069 §0.

| Objection | Verified? | Outcome |
|---|---|---|
| The defect-class exemption from Gate 1 is claimed for the wrong artifact — the evidence licenses a *fix*, not a *feature* | ✅ | **Accepted.** Work split: `get_command_info` hardening is ungated; the verb goes through Gate 1. ADR-069 §0.2, §0.4 |
| The memo said *"ship the check before designing the taxonomy"*; the draft designed a taxonomy and defended it nowhere | ✅ (memo §3.10 verbatim) | **Accepted in part.** `.git/config` reverts to presence/absence against a named-key list; the inversion survives only on the `hooks.*` and `tasks[]` closed sub-schemas. `RiskLevel` is kept and the deviation is now argued, not silent. §0.3 |
| The ~12-row git allowlist is off by an order of magnitude against git's ~700 keys; a bare `git clone` writes ~11, and `gh`/`git worktree`/`git-lfs` add more | ✅ | **Accepted** — this is what forced the rescope above. New regression test T5b |
| Draft claimed `.vscode/settings.json` has a "small closed schema", contradicted by the draft's own §1.2(c) | ✅ | **Accepted.** Dropped from v1 (as was `.envrc`) |
| `-o/--output` is not global; `args_conflicts_with_subcommands = true` (`src/main.rs:690`) makes `caro -o json preflight .` a parse error | ✅ | **Accepted.** Verb-local `--format`; the draft's whole "reuse `-o`" argument rested on a false premise |
| Test T7's `strace` assertion is unimplementable on macOS CI | ✅ | **Accepted.** Portable PATH-shim sentinel |
| `src/context/**` is in zero workflow path filters, so security-rule code would skip `safety-validation.yml` and mutation testing | ✅ | **Accepted.** Two workflow rows added |
| D6 claimed the digest covers files that were never inspected | ✅ | **Accepted.** Omissions are now digest inputs; guarantee restated at its true strength |
| No `## What breaks at 100 real users` section (Gate 3 requires the title) | ✅ | **Accepted.** §3.4, including the admission that the failure mode is currently undetectable by the project, and a 20-repo measurement as a PR precondition |
| The risk table D3 asks to be argued with was never printed | ✅ | **Accepted.** §D9 now has a `RiskLevel` column, 18 rows, each with its code path |
| "One new file" is really 700–1200 lines against `clippy.toml`'s complexity threshold | ✅ | **Accepted.** Five-file module + an INI-corpus gate that can trigger a `gix-config` spike |
| ADR-069 never mentions the ADR-059 moratorium that ADR-068 engaged with | ✅ | **Accepted.** Moratorium paragraph added; ADR-069 does *not* claim a carve-out |
| "Preflight is artifact scanning; the memo says stay out of that lane" | partial | **Recorded, not accepted.** Argued boundary: distributed artifacts vs. the local working directory. But the "nothing covers the union" claim is downgraded from moat to snapshot. §0.5 |
| No named consumer exists | ✅ | **Accepted as a deferral argument**, now in §3.2 |

## Key Decisions (reviewable)

- **The allowlist inversion (ADR-069 §D3) is the load-bearing bet — and review cut it in half.**
  The principle is sound: Jeff King's 2017 objection on the git list ("high risk of missing an
  option") was vindicated by git 2.54.0 adding `hook.<name>.command` in April 2026 — a shell
  one-liner needing no file and no exec bit, absent from every published key list including
  advisories written weeks earlier — and by VS Code's 124 scattered `restricted: true` flags
  with `chat.permissions.default` still unflagged at 1.139.0. **The application was wrong.**
  Enumerate-goodness over `.git/config`'s ~700-key namespace with a ~12-row allowlist is a noise
  generator, not a control; a vanilla `git clone` writes ~11 keys before a developer touches
  anything. Final shape: inversion on the two **closed sub-schemas** (`hooks.*` objects in
  `.claude/settings.json`, `tasks[]` objects in `.vscode/tasks.json`, ~14 allowlist rows),
  named-key blocklist on the git surface. **The ADR now says plainly that on the git surface it
  ships the blocklist Jeff King said would miss keys, and that it will miss keys.** That
  concession is the paragraph to argue with.
- **No new exit code.** Inherits the `2`-means-finding convention from
  ADR-065/066/067/068. Exit `0` is reserved for "inspected everything **and**
  found nothing"; an incomplete inspection collapses into `2` because it is
  fail-closed by definition, and the caller distinguishes via
  `completeness.complete` in the payload. This deliberately avoids the gitleaks
  contract, where exit 1 conflates "found leaks" with "the tool errored".
- **Stateless, and the constraint did not need relaxing.** Unlike the 2026-08-13
  run (budget ledger), which relaxed "no state" to "no *resident* state", this
  feature needs nothing durable. `state_digest` — sha256 over the ordered
  `(relative path, content sha256)` list of every inspected source **plus every
  omission**, concatenated with the rule-table version — hands the caller direnv's
  content-addressed trust property while Caro keeps no state directory, no lock file
  and no daemon. Scope of the guarantee, corrected in review: it covers the v1 source
  set, not "any file that appears".
- **Determinism as a contract property.** The payload carries **no timestamp, no
  absolute path, no hostname, no duration**. That is what makes the 16 integration
  tests able to assert byte-exact JSON — which no test in the tree does today
  (`tests/e2e_cli_tests.rs:193` only asserts `serde_json::from_str::<Value>().is_ok()`).
- **Placed in `src/context/`, not `src/safety/`.** `src/safety/` owns
  command-string validation; merging a directory scanner into it would blur the
  exact distinction the ADR argues for. `src/context/directory.rs` already owns
  directory inspection and is the right shape — with the wrong failure policy
  (`DirectoryContext::scan` returns `Self::default()` on a non-directory, i.e.
  "couldn't look" reads as "nothing there").
- **Never invoke git (§D2).** `.git/config` is parsed as INI by Caro; no
  `git config --get`, no subprocess, no `git2`/`gix` dependency. CVE-2025-41390
  (TruffleHog RCE via the config it was scanning) is the reason, and it is solved
  structurally rather than by sandboxing the scan.

---

## Codebase Facts Verified This Run

- **`--json` does not exist anywhere in `src/`, and `-o/--output` is not usable from a
  subcommand.** `-o` is a plain field on `struct Cli` (`src/main.rs:751-753`) with **no
  `global = true`**, and `#[command(args_conflicts_with_subcommands = true)]` at `:690` makes
  `caro -o json <subcommand>` a clap parse error. It is read at exactly one site (`:880-882`)
  and no dispatch arm consults it. **ADR-058/059 assume a `--json` flag that does not exist;
  ADR-069's first draft assumed `-o` was global, which it is not.** Every future ADR proposing
  a JSON-emitting verb needs to know this: today a subcommand must mint its own format flag, or
  someone must make `-o` global CLI-wide.
- **No `ExitCode` enum exists.** One constant, `EXIT_CODE_EDIT: i32 = 201`
  (`src/main.rs:938`); everything else is a literal `process::exit(0 | 1)` —
  34 and 32 occurrences respectively. Docs have allocated **0–20** across ~25
  ADRs with two competing proposals for where the enum lives (ADR-059:
  `src/safety/assessment.rs`; ADR-066: `CaroExitCode` in `src/main.rs`).
  Unresolved, and ADR-069 deliberately does not resolve it.
- **`caro assess` does not exist** — the variant is commented out at
  `src/main.rs:422-434` and `:3040-3047`, "disabled in v1.1.0-beta.1".
  Separately, `src/assessment/` is **hardware** assessment (cpu/gpu/memory),
  which collides by name with ADR-058's `caro.assessment.v1`. Worth a naming
  decision before either ships.
- **`caro.assessment.v1`, `caro.eval.v1`, `caro.bench.v1`, `caro.decompose.v1`,
  `HeadlessEnvelope`, `PathClass`: zero string hits in `src/` and `tests/`.**
  Every ADR from 024 upward is paper. ADR-069 is written to build on the tree
  that exists.
- **`canonicalize` appears zero times in `src/`.** No `walkdir`, `globset`,
  `git2`, `gix`, `ini` or JSONC dependency. A directory-walking verb owns its own
  traversal-safety story from scratch.
- **No per-project config.** One file at the XDG path; no ancestor walk, no
  `./.caro/config.toml`. This makes ADR-069's §D7 (preflight never reads its own
  config from the audited directory) free today and expensive later — hence the
  invariant plus regression test T9 rather than a comment.
- **`DangerPattern` still has no stable id** (`src/safety/mod.rs:334`); 67
  literals report free-text `description` strings. `dogma::CompiledPattern.id`
  (`src/dogma/compiler.rs:37`) remains the only stable-id precedent, and it is
  CVE-shaped. ADR-069 mints its own `PF-*` namespace.
- **`SuggestedRouting` (`src/models/mod.rs:189`) still lacks `JsonSchema` and
  `Ord`** — the same two-line prerequisite ADR-058 needs. Whichever ADR lands
  first pays for it.
- **`schema_version` is `u32`** in the only place it ships (`src/caroml/lock.rs:26`),
  while ADR-024 specifies `String`. ADR-069 carries both a `contract: String`
  and a `schema_version: u32` rather than picking a side silently.
- **Version drift, still unfixed:** `DANGEROUS_PATTERNS` is **67**, not the "52+"
  in module docs and CLAUDE.md (first noted 2026-08-13). `CLAUDE.md` says MSRV
  1.83; `Cargo.toml` says 1.85.
- **Next ADR number: 069** (highest on disk ADR-068, 2026-09-09). `docs/adr/README.md`
  index remains stale and two historical duplicate numbers (004, 015) remain —
  unchanged since the 2026-08-13 report flagged them.

---

## Suggested Follow-ups (not done — require human/PR review)

1. ★ **Harden `src/agent/mod.rs:790` (`get_command_info`).** ~20 lines: reject tokens containing
   `/`, resolve the rest against an explicit allowlist or `PATH`, `env_clear()` with a minimal
   env, `current_dir` to a neutral directory. Ungated under `validation-discipline.md` (bug fix
   in a thing Caro already has). **This should land before ADR-069 is implemented**, and arguably
   before anything else in this list. See "The Actual Defect" above.
2. **Open a feature branch + PR landing ADR-069.** Nothing was committed; see
   Artifacts below.
3. **Fifteen consecutive `Proposed` ADRs (055–069) are untracked, unindexed and unbeaded.**
   `grep -E 'ADR-05[5-9]|ADR-06[0-9]'` over the beads backlog returns nothing; all fifteen are
   `??` in `git status`; `docs/adr/README.md` indexes through ADR-015. The ADR-059 moratorium
   (2026-08-26 strategy memo: *"the gap between 24 proposed governance ADRs and zero shipped
   modules"*) has never been lifted and ten ADRs have been written under it. **Before ADR-070
   exists, someone should either commit and bead 055–068 or pause this scheduled task.** A
   proposal that cannot be found by `bd ready`, `git log`, or the ADR index is not a proposal.
4. **Feed the two memo corrections back into the Hermes scan process.** The
   pattern in both is the one the 2026-09-09 skeptic's note already named:
   trusting a secondary aggregator's framing over the primary source. A CVE that
   appears in a recommendation should be read from its advisory.
5. **Empirically check the `filter.*.clean` claim** before its rationale string
   ships. The "fires from plain `git status`" reading is derived from
   `ce_compare_data()` → `index_fd()` → `convert_to_git()`, not from docs and not
   from a PoC. Three lines of shell settles it; if it needs `git add`/`git diff`
   the key drops High → Moderate and nothing structural changes.
6. **Decide the `assess` name collision** (hardware assessment in
   `src/assessment/` vs ADR-058's decision contract) before either lands.
7. **`docs/adr/README.md` index refresh + duplicate-number cleanup** per
   `.claude/rules/adr-numbering.md`. Third consecutive run to flag this.
8. **The memo's 3.12 (hook adapter) is coupled to this ADR, not independent.**
   If the hook adapter is scheduled first, §D7 of ADR-069 is the part that has to
   come with it — a gate whose configuration the gated repository can write is
   Pillar's stated failure mode, and it is also CVE-2026-33068.
9. **Make the Gate 4 self-review permanent in this task.** It cost one subagent turn and it
   rescoped the deliverable, corrected the CLI surface, killed two unimplementable tests, and
   found follow-up #1. A scheduled task drafting feature specs with no human present is the
   exact confirmation-bias case `.claude/agents/devils-advocate.md` was written for; running it
   should not be optional. Consider adding it to the SKILL.md as a Phase 4.
10. **Fill the SKILL.md `[FEATURE NAME]` placeholder** or bless the "pick from the
    latest Hermes scan" convention in the task file. Twelfth consecutive run
    raising this.

---

## Artifacts

- `docs/adr/ADR-069-ambient-configuration-preflight.md` (new, uncommitted)
- This report (new, uncommitted)

*Neither file was committed — `.claude/rules/git-workflow.md` forbids main-branch
commits, and the working tree's git state is currently broken for at least one
worktree (`.git/worktrees/006-replace-ascii-morph` is a dangling registration,
so `git status` fails at the repo root). Landing these requires a feature branch
+ PR, and probably a `git worktree prune` first.*
