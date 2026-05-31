# Verify Agent-Authored Artifacts Before Posting

**APPLIES TO**: Any agent (Claude Code, Hermes, ml-ds-engineer, qa-agent, …)
producing public-facing text — PR comments, GitHub Release bodies, Hermes
daily digests in `.hermes/digests/`, release-acceptance audit artifacts in
`.claude/releases/`, beta-test reports in `.claude/beta-testing/`.

Codified after porting warden's `hallucination_audit.py` shape to
`bin/verify-artifact` (PR #1187). The helper is a stdlib-only Python
grounding gate that exits non-zero when a draft contains a PR/issue
reference, URL, commit SHA, file path, or @mention not present in the
supplied evidence corpus.

## The rule

Before posting any agent-authored artifact that names concrete entities
(PR numbers, SHAs, URLs, file paths, user handles), pipe the draft and
its evidence corpus through `bin/verify-artifact`. **Block on exit
code 1.**

```bash
# Pre-PR-comment pattern
gh pr view <PR> --json title,body,commits \
  | bin/verify-artifact \
      --text-file draft.md \
      --evidence-file - \
      --allowed-pr <PR>
# exit 0 → proceed with `gh pr comment`
# exit 1 → fix the draft, re-verify, retry
```

This operationalizes [`~/.claude/rules/claim-verification.md`](../../.claude/rules/claim-verification.md)'s
"80%-false-claim" lesson as a deterministic check instead of a habit.

## Required for

| Artifact | Evidence corpus to supply |
|----------|---------------------------|
| `gh pr comment <PR>` body | `gh pr view <PR> --json title,body,commits,comments` |
| `gh pr review` body | same as above, plus the diff if claims reference code |
| Hermes daily digest (`.hermes/digests/<date>.md`) | concatenated `gh pr list` + `gh issue list` JSON for that day |
| Release-acceptance audit (`.claude/releases/v<VERSION>-acceptance.md`) | CHANGELOG entry + verifier command outputs |
| `gh release create/edit` body | CHANGELOG section being shipped |
| Beta-test report posted to GH | the test session transcript |

## When to flip `--strict-quotes`

Default behavior treats ungrounded quoted spans as `warn` (not `fail`),
because agents often paraphrase. Use `--strict-quotes` when the artifact
explicitly cites the evidence (e.g. an audit matrix's "verifier output"
column or a release note quoting the CHANGELOG). In those contexts
paraphrase is a bug.

## Recommended allowlists

| Flag | When to use |
|------|-------------|
| `--allowed-pr <N>` | The PR/issue being commented on. Always pass the canonical number. |
| `--allowed-mention <USER>` | The original author + reviewers known from evidence. |
| `--allowed-url <URL>` | Whitelisted docs URLs (caro.sh, docs.caro.sh) that won't appear in PR evidence. |

## Exit codes

| Code | Meaning | Caller action |
|------|---------|---------------|
| 0 | OK (warnings may be present) | Proceed |
| 1 | Grounding failure | Block; fix draft; retry |
| 2 | Usage error | Fix invocation |

## What this rule does NOT replace

- It does not check spelling, grammar, or tone.
- It does not check whether claims are *true*, only that referenced
  entities exist in the supplied evidence. Garbage evidence produces
  garbage "OK".
- It does not replace `~/.claude/rules/pr-comment-structure.md` —
  artifacts must still follow the canonical `\`[agent]\`` / identity /
  body / details template.

## See also

- `bin/verify-artifact --help` — full flag reference
- `scripts/tests/test_verify_artifact.py` — 21 test cases worth reading as
  examples of what passes and what fails
- `~/.claude/rules/claim-verification.md` — the global rule this
  operationalizes
- Upstream inspiration: [JithendraNara/warden](https://github.com/JithendraNara/warden)
  `runtime/hallucination_audit.py`
