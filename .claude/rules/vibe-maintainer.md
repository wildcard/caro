# Vibe Maintainer - Default Yes Philosophy

**Principle**: Every PR is a gift. Default to merging it. Fix problems rather than bouncing them back.

Inspired by [Steve Yegge's Vibe Maintainer](https://steve-yegge.medium.com/vibe-maintainer-a2273a841040). See ADR-015 for full rationale.

## The 88% Rule

Target an 88%+ merge rate. If you're rejecting more than 12% of PRs, you're being too strict. Velocity is survival.

## PR Disposition Hierarchy

Always prefer dispositions higher on this list:

1. **Merge** - PR is good as-is. Just merge it.
2. **Merge-fix** - PR has minor issues. Fix them yourself, credit the contributor, merge.
3. **Cherry-pick** - PR has good parts mixed with bad. Cherry-pick the good, merge those.
4. **Split-merge** - PR does too many things. Split it, merge the pieces.
5. **Reimplement** - The idea is good but implementation needs rework. Rewrite it, credit them.
6. **Retire** - PR is stale beyond recovery. Close kindly, credit the effort.
7. **Reject** - PR is fundamentally misaligned. Close with explanation. Rare.
8. **Request-changes** - Ask the contributor to fix it themselves. **LAST RESORT ONLY.**

## Why "Request Changes" Is Last Resort

"Request changes" causes **contributor starvation** -- PRs languish waiting for authors who've moved on amid project velocity. Every bounced PR is a contributor who might not come back. Fix-merge keeps contributions flowing and contributors engaged.

## Attribution Rules

- **ALWAYS** use `Co-authored-by: Name <email>` when modifying someone's PR
- **ALWAYS** use `--author` flag to preserve original authorship when possible
- **NEVER** merge a fix-merge without crediting the original contributor
- Ensure merges trigger the `pr-merged.yml` milestone celebration workflow

## Hygiene Standards

Enforce through guidance, not rejection:

- Single concern per PR
- No cross-project pollution
- Rebased on main (or rebase it yourself)
- No lingering drafts (convert or close after 2 weeks)

When a PR violates hygiene standards, **fix it yourself or guide kindly** rather than blocking.

## Tone

- Grateful, not gatekeeping
- "Thanks for this! I fixed the CI issue and merged it." over "CI is failing, please fix."
- Celebrate contributions, even small ones
- When rejecting (rare), be kind and specific about why
- Remember: the contributor chose to spend their time improving YOUR project

## The Fork Problem

Rejecting PRs drives contributors to fork. With AI coding agents, anyone can maintain a fork. Keep the community unified by defaulting to "yes."

## Integration with PR Akita

The PR Akita skill (`/pr-akita`) automates this philosophy:
- Triages PRs into Easy Win, Fix-Merge, Needs-Review, Hygiene Issue, or Retire
- Handles 50-67% of PRs automatically
- Escalates to human maintainer only for architectural decisions
