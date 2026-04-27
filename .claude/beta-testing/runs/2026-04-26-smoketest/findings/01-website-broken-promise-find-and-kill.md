### [WEBSITE/BROKEN_PROMISE] Landing-page query "find and kill the runaway process eating CPU" generates a half-pipeline that never kills anything

- **Severity**: P0
- **Query**: `find and kill the runaway process eating CPU`
- **Generated command**: `ps aux | sort -nrk 3,3`
- **Advertised command** (per `website/src/data/gtm-use-cases.ts:9`): `ps aux | sort -nrk 3,3 | head -1 | awk '{print $2}' | xargs kill`
- **Expected**: caro emits the full pipeline that picks the top CPU process (`head -1`), extracts the PID (`awk '{print $2}'`), and terminates it (`xargs kill`). The user typed "find **and kill**" — both verbs must be honoured.
- **Actual**: caro stops after the sort. Three pipeline stages dropped, including the verb the user actually cared about.
- **Environment**: caro 1.3.0 (crates.io), macOS 26.3, zsh, cwd: `/Users/kobik-private/workspace/caro/.claude/worktrees/determined-lumiere-fe18c3`
- **Time-to-completion**: 1281 ms (run 1) / 1253 ms (run 2)

#### Reproduction

```bash
caro --dry-run "find and kill the runaway process eating CPU"
# Output:
# Command:
#   ps aux | sort -nrk 3,3
```

#### Retry triplet

| Step | Query | Generated | Outcome |
| ---- | ----- | --------- | ------- |
| (a) verbatim | `find and kill the runaway process eating CPU` | `ps aux \| sort -nrk 3,3` | FAIL |
| (b) +specific | *skipped — website query already explicit* | — | — |
| (c) website verbatim | `find and kill the runaway process eating CPU` | `ps aux \| sort -nrk 3,3` | FAIL — **website-promise broken** |

Determinism: ✓ identical across runs (a) and (c). Not flaky — consistently wrong.

#### Suspected root cause

The query contains two distinct verbs (`find` and `kill`) joined by `and`. The static matcher
or embedded backend appears to have matched on the `find … process … CPU` half and emitted a
"top CPU offender" pipeline, but truncated before the `head -1 | awk | xargs kill` tail that
makes the command actually *do* what the user asked for.

Likely failure modes to investigate:

1. **Static-matcher pattern** in `src/backends/static_matcher.rs` for "top CPU process"
   returns a *display* pipeline (`ps | sort`) instead of an *action* pipeline. Pattern
   probably keys on "process eating CPU" without weighting the `kill` verb.
2. **Token-budget truncation** in the embedded backend — the prompt template might cap
   output at a length that lops off the tail of multi-stage pipelines. Check the prompt
   in `src/prompts/command_templates.rs` and the max_tokens setting.
3. **Multi-verb intent splitting** is not implemented — caro picks one verb (`find`) and
   ignores the other (`kill`). This is the same family as `BUG/INTENT_DROPPED`.

Note that this is **not** the static_matcher.rs:685 fallback bug from the plan's
pre-confirmed backlog — that one returns generic `ls -la`. This is a different, equally
serious failure mode: the matcher fires on a *correct* pattern but the pattern itself is
incomplete relative to what the website advertises.

#### Stale work that should have prevented this

- **[PR #567](https://github.com/wildcard/caro/pull/567)** — "Improve UX for long-running
  commands" — open since 2026-01-19 (97 days as of 2026-04-26). If this had landed with
  better intent classification, multi-verb queries like "find and kill" would not silently
  drop half the pipeline.
- **[Issue #449](https://github.com/wildcard/caro/issues/449)** — "🎯 [EPIC] Exploration
  Agent: Complete Integration & Rollout" — open since 2026-01-14. The exploration agent
  is the right place to *ask* the user "do you want to display the top process or kill it?"
  when the query has two verbs. Currently caro guesses (wrongly).
- The four pre-confirmed findings in the routine's plan (static_matcher fallback,
  no-streaming, no-clarification, undermatch) all touch this — particularly
  `UX/NO_CLARIFICATION` from `src/agent/mod.rs`. Had the agent loop asked "I see two
  actions — find and kill — which do you want?" the user would have gotten a correct
  command.

#### Suggested fix direction

Three orthogonal patches, in order of cost:

1. **Cheapest** — fix the specific advertised pattern. Whatever pattern the matcher hit
   for this query, add the `head -1 | awk '{print $2}' | xargs kill` tail when the verb
   `kill` (or `terminate`, `stop`, `end`) appears in the query. Update test fixtures
   accordingly. **Note**: do this only as a stop-gap; it doesn't fix the class.
2. **Medium** — multi-verb detection in the prompt. Add a system-prompt rule: "if the
   query contains two action verbs joined by 'and', the generated pipeline must end in a
   command that performs the *second* verb." Test with "find and delete", "search and
   archive", "list and compress".
3. **Right fix** — clarifying questions (Issue #449). When the agent loop detects
   ambiguous intent ("find and kill" with confidence < 0.85 on the action-verb), it
   should ask the user instead of guessing. This is the architectural answer and the
   one the user explicitly called out as "should have landed".

Until any of these lands, the website needs to either ship a fix or change the
advertised example to one that caro can actually produce.
