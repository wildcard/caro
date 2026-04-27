# /caro.frustrated-qa — Daily Frustrated-Beta QA Routine

Daily 5 AM autonomous QA routine that drives the `caro-frustrated-beta`
sub-agent against the commands the website advertises, classifies the
paper-cuts a real power user would hit, scans GitHub for stale work that
should have prevented the pain, and turns every finding into a concrete
GitHub issue with a fix direction.

This is the **harsher younger sibling** of the existing 9 AM
`/qa-automation-loop` (which runs the multi-profile audit). The 9 AM run
asks "does it work for the typical user?". The 5 AM run asks "does it work
for the user who has 30 seconds?" — and that user is much harder to keep.

## Usage

```
/caro.frustrated-qa [--dry-run] [--limit N] [--queries q1,q2,...] [--no-community]
```

| Flag             | Default | Effect                                                              |
| ---------------- | ------- | ------------------------------------------------------------------- |
| `--dry-run`      | `false` | Capture findings, draft issues, **do not** `gh issue create`        |
| `--limit N`      | all     | Run only the first N advertised queries (smoke testing)             |
| `--queries q1,..`| —       | Override the website-extracted query list with explicit queries     |
| `--no-community` | `false` | Skip the GitHub issues/discussions frustration sweep                |

## Schedule

Wired into `.claude/automation/config/schedule.yaml` as
`technical.frustrated_qa_loop` at `0 5 * * *` (daily 5 AM, project tz
`America/Los_Angeles`). The 9 AM `qa_automation_loop` continues to run
unchanged.

## Process

### 0. Preflight

```bash
# Confirm we're at the worktree root and caro is built
test -f Cargo.toml || { echo "must run from caro repo root"; exit 2; }
cargo build --release --quiet 2>/dev/null || cargo build --quiet
CARO_BIN=$(command -v caro || echo "./target/release/caro" )
$CARO_BIN --version
gh repo view --json nameWithOwner -q .nameWithOwner   # confirm gh auth + repo
```

If any of those fail, abort and write a single
`.claude/beta-testing/runs/<date>/PREFLIGHT_FAILED.md` with the error so the
human reader knows the routine couldn't even start. Do **not** silently skip.

### 1. Source-of-truth refresh

Read **live** from the website source — do not cache:

- `website/src/data/gtm-use-cases.ts` — extract every advertised query
  (the strings users see in marketing copy).
- `website/src/components/InteractiveTerminal.astro` — extract the demo
  prompt chips (the queries the landing page literally tells users to try).

Deduplicate, normalise (trim, lowercase preserved as-typed, strip outer
quotes), and pass the resulting list to the persona agent. **Log the list
into `runs/<date>/queries.txt`** so the human reviewer knows exactly what
was tested.

### 2. Spawn the persona agent

Use the Agent tool with `subagent_type: caro-frustrated-beta` and pass the
query list plus the run directory path. The agent owns generation, capture,
classification, retry triplet, stale-work scan, and bug-report drafting per
its system prompt. The orchestrator's job here is supervisory only.

### 3. Read back findings

The persona writes to:

- `runs/<date>/summary.md`
- `runs/<date>/findings/<NN>-<class>-<slug>.md`
- `runs/<date>/proposed-issues/<NN>-<class>-<slug>.md`

Read every `proposed-issues/*.md` file. Each is a complete issue body in the
shape defined in the agent's "Bug Report Shape" section.

### 4. Dedup against existing issues

Before filing anything, search:

```bash
gh issue list --state all --search "<finding-title>" --limit 5 --json number,title,state,url
```

If an open issue with the same symptom + same query exists → **do not file
again**. Append a comment on the existing issue using the canonical agent
comment template (`~/.claude/rules/pr-comment-structure.md`):

```
`[agent]`

**Agent:** Claude Code (`<model-id>`) — caro-frustrated-beta

---

Reproduced again on <date> with caro <version>. Adding latest evidence.

<excerpt of the new finding>

---
```

Otherwise, mark it as ready to file.

### 5. File new issues

For each ready-to-file finding:

```bash
gh issue create \
  --title "[QA/<CLASS>] <one-line-title>" \
  --label "qa-routine,frustrated-beta,<class-slug>,<priority>" \
  --body-file "$RUN_DIR/proposed-issues/<NN>-<class>-<slug>.md"
```

Required labels (create them once if missing — `gh label create` is
idempotent enough with `|| true`):

| Label                | Color   | Purpose                                          |
| -------------------- | ------- | ------------------------------------------------ |
| `qa-routine`         | `1d76db`| Filed by an automated QA routine                 |
| `frustrated-beta`    | `D93F0B`| Specifically the 5 AM frustrated-user routine    |
| `bug-fallback-overmatch` | `B60205` | The `ls -la` family                          |
| `bug-undermatch`     | `B60205`| Generated command too narrow                     |
| `bug-intent-dropped` | `B60205`| Query keyword ignored                            |
| `ux-no-streaming`    | `D93F0B`| Buffered output, user waits                      |
| `ux-no-clarification`| `D93F0B`| Should have asked instead of guessing            |
| `website-broken-promise` | `B60205` | Verbatim landing-page query fails           |
| `safety-missed-danger` | `B60205` | Dangerous command not flagged                  |
| `P0` `P1` `P2` `P3`  | various | Priority                                         |

If `--dry-run`, **skip this step** and instead leave the proposed-issues
files in place for the human reviewer.

### 6. Stale-work cross-link

For each newly-filed issue that the persona's report cites a stale PR for,
post a comment on the **stale PR** linking the new issue:

```
`[agent]`

**Agent:** Claude Code (`<model-id>`) — caro-frustrated-beta orchestrator

---

This PR was flagged by today's frustrated-beta QA run as expected to prevent
the pain reported in #<new-issue>. Filing for visibility — what's blocking
it from landing?

---

<details>
<summary>Prompt used to generate this comment</summary>

```
`/caro.frustrated-qa` — daily 5 AM stale-work cross-link
```

</details>
```

This is the leverage step. Stale PRs that should have shipped get a daily
nudge until they do or someone closes them on the record.

### 7. Community frustration sweep

Skipped if `--no-community`. Otherwise:

```bash
# Open user-feedback issues
gh issue list --state open --label "user-feedback,beta-feedback,community" --limit 20 \
  --json number,title,body,createdAt,url

# Recent discussions (if discussions enabled)
gh api repos/{owner}/{repo}/discussions --paginate -q '.[] | select(.created_at > "<7-days-ago>") | {number, title, body, html_url}'
```

Filter for frustration keywords: `frustrat`, `confus`, `doesn't work`,
`gave up`, `unclear`, `why does it`, `expected`, `weird`, `bug`, `broken`.

For each match, hand the issue/discussion to the persona agent for triage
(see the "Community Frustration Sweep" section of the agent's prompt).
The agent produces either a repro-request draft or a derivative bug report
under `runs/<date>/community-replies/`. The orchestrator posts those
comments using the canonical comment template.

### 8. Daily summary

Write `runs/<date>/summary.md` (the persona drafts the body; the
orchestrator appends the filing outcomes):

```markdown
# Frustrated-Beta QA Run — <date>

caro <version> on <os>, shell <shell>.
Queries tested: <N> from gtm-use-cases.ts + InteractiveTerminal.astro.

## Findings by class

| Class | P0 | P1 | P2 | P3 | Filed | Updated existing |
| ----- | -- | -- | -- | -- | ----- | ---------------- |
| ...   | .. | .. | .. | .. | ..    | ..               |

## Issues filed today

- #<n> — [QA/<class>] <title>
- ...

## Issues updated today (existing reproductions)

- #<n> — appended new evidence
- ...

## Stale PRs cross-linked

- PR #<n> — <title> — open <days> days, expected to fix #<new-issue>

## Community sweep

- <issue/discussion> — <action: repro requested | derivative filed | tracked-in posted | no action>

## Loudest signal

<one paragraph the persona emitted on its closing line>
```

Append a single line to `.claude/beta-testing/runs/INDEX.md` for trend
tracking:

```
<date>  queries=<N>  findings=<P0/P1/P2>  filed=<F>  updated=<U>  community=<C>
```

### 9. Close the loop

If anything was filed or updated, the orchestrator must:

1. `git add .claude/beta-testing/runs/<date>/`
2. `git commit -m "chore(qa): frustrated-beta run <date>"` (on the
   automation branch — never on `main`, per `.claude/rules/git-workflow.md`)
3. `git push`

If nothing was filed (clean run), still commit the run directory so the
trend log captures the no-finding day.

## First-Run Pre-Confirmed Backlog

Exploration during plan-mode pre-confirmed four findings the very first
real-mode run will file. The persona should still go through full evidence
capture for each rather than treat them as hand-typed shortcuts:

1. **`[BUG/FALLBACK_OVERMATCH]` Static matcher returns `ls -la` for any
   "show … files" query.** Expected location: `src/backends/static_matcher.rs:685-693`,
   pattern 43 ("list files"). Loose regex shadows specific patterns at
   lines 680–682. **P1.**

2. **`[UX/NO_STREAMING]` Command execution buffers stdout instead of
   streaming.** Expected location: `src/main.rs:738-763`. Suggested fix:
   `BufReader` over `child.stdout`. **P1.**

3. **`[UX/NO_CLARIFICATION]` Agent loop guesses on under-specified queries
   instead of asking.** Expected location: `src/agent/mod.rs:1-156, 444`.
   `refine_command()` exists but no interactive ask-user flow. The
   stale-work scan should specifically look for any open PR that attempted
   to add this and stalled. **P1.**

4. **`[BUG/UNDERMATCH]` "find config files" → `find . -name "*.conf"` misses
   `.yml/.toml/.json/.ini/.env`.** Pattern needs glob alternation or a
   defer-to-embedded path. **P2.**

If on first real run any of those four don't reproduce (because someone
shipped a fix in the meantime), that is itself good news — log it in the
summary as "expected finding N did not reproduce; suspected fix in
<commit-sha>" and skip filing.

## Verification

| # | Step | How                                                                        |
| - | ---- | -------------------------------------------------------------------------- |
| 1 | Dry run | `/caro.frustrated-qa --dry-run --limit 1` — produces a run dir, **no** issues filed |
| 2 | Persona separation | Confirm spawned `caro-frustrated-beta` agent has Bash/Read/Grep/Glob/Write only |
| 3 | Schedule entry | `cat .claude/automation/config/schedule.yaml` shows `frustrated_qa_loop` at `0 5 * * *` |
| 4 | Real-run smoke | `/caro.frustrated-qa --limit 3` produces `runs/<date>/summary.md` and at least one filed issue |
| 5 | First-run backlog | After first real run, the four pre-confirmed findings are filed (or marked "did not reproduce") |
| 6 | Community sweep smoke | `runs/<date>/summary.md` shows non-empty community-sweep section even on quiet days ("0 hits") |

## Operational Notes

- **Never silent-skip a step.** Every step that does nothing must still log
  one line. A morning summary that says "community sweep: 0 hits" is far
  more valuable than one that omits the section entirely — the latter
  hides routine breakage.
- **Never edit `src/`.** This routine *files issues*. The dev team picks up
  fixes via the standard `bd ready` / PR workflow.
- **Never run on `main`.** Per `.claude/rules/git-workflow.md`. The cron
  driver is responsible for creating an automation branch like
  `automation/frustrated-qa-<date>` and pushing the run-dir commit there.
- **Always reuse, never reinvent.** If a finding matches an existing open
  issue, append evidence — do not file a duplicate. The trend value of
  "this has now been reproduced 14 days in a row" is the actual leverage.

## Related

- `.claude/agents/caro-frustrated-beta.md` — the persona sub-agent
- `.claude/skills/unbiased-beta-tester/SKILL.md` — the persona-framework root
- `.claude/skills/beta-test-cycles/SKILL.md` — pattern-refinement loop
- `.claude/commands/qa-automation-loop.md` — sister 9 AM multi-profile audit
- `.claude/memory/qa_agent_role.md` — persistent QA identity & escalation
- `~/.claude/rules/pr-comment-structure.md` — canonical comment shape
- `.claude/rules/git-workflow.md` — feature-branch enforcement
