# ADR-015: Vibe Maintainer Workflow

**Status**: Proposed

**Date**: 2026-04-05

**Authors**: Caro Maintainers

**Target**: Community

## Context

Caro already has robust PR automation: a 4-hour management loop, contributor milestone celebrations, staleness tracking, CI analysis, and external agent integration (Kubic, Copilot). However, the existing system operates with a **reactive, gatekeeping posture** -- PRs are classified by health status and acted on defensively. The default disposition is "wait for the contributor to fix it."

Steve Yegge's [Vibe Maintainer](https://steve-yegge.medium.com/vibe-maintainer-a2273a841040) philosophy inverts this entirely. Managing ~50 PRs/day across two major projects with an 88% merge rate, Yegge demonstrates that the default should be **"yes"** -- PRs are triaged by *how to get them merged*, not *whether they should be merged*. The maintainer fixes issues rather than bouncing them back.

This philosophy aligns naturally with caro's existing values:

- The **Good Boy Scout rule** already says "Fix it. No need to ask who caused it or why. Just fix it."
- The project runs 4-5 parallel Claude Code sessions, making velocity essential
- AI coding agents have collapsed the barrier to forking -- rejecting contributions drives users to maintain competing forks rather than pooling resources

The rise of AI-assisted contributions means the volume and velocity of incoming PRs will only increase. A gatekeeping posture doesn't scale; a fix-merge posture does.

### The Fork Problem

When maintainers decline PRs, communities fork. With modern AI coding agents accessible to everyone, forking barriers have collapsed -- anyone can maintain a fork. This creates unnecessary duplication when shared development would benefit all parties. Defaulting to "yes" keeps communities unified.

## Decision

Caro adopts the Vibe Maintainer workflow with the following commitments:

### 1. Default to "Yes"

Target an **88%+ merge rate**. Every PR is a gift. Start from the assumption that it should be merged. Look for reasons to merge, not reasons to reject.

### 2. Fix-Merge as Primary Disposition

When a PR has fixable issues, the maintainer (or PR Akita agent) fixes them and merges, rather than requesting changes from the contributor. "Request changes" is the **last resort**, not the default.

### 3. PR Akita Triage System

An AI-powered triage agent ("PR Akita", named after the loyal Japanese dog breed) handles 50-67% of incoming PRs automatically. It categorizes PRs into 5 triage buckets and executes the appropriate disposition.

### 4. Eight PR Outcomes (Disposition Hierarchy)

Always prefer dispositions higher on this list:

1. **Merge** -- PR is good as-is
2. **Merge-fix** -- Fix minor issues yourself, credit contributor, merge
3. **Cherry-pick** -- Extract good parts from a mixed PR
4. **Split-merge** -- Split multi-concern PR, merge the pieces
5. **Reimplement** -- Good idea, needs rework. Rewrite it, credit them
6. **Retire** -- Stale beyond recovery. Close kindly, credit the effort
7. **Reject** -- Fundamentally misaligned. Close with explanation. Rare
8. **Request-changes** -- Ask contributor to fix it themselves. **LAST RESORT**

### 5. Contributor Attribution Always

Regardless of modification extent, credit the original contributor:
- Use `Co-authored-by: Name <email>` trailers when modifying someone's PR
- Use `--author` flag to preserve original authorship when possible
- The existing `pr-merged.yml` workflow handles milestone celebrations

## Rationale

- **Velocity is survival**: The project that ships fastest wins. Perfect code that ships slowly loses to good-enough code that ships fast.
- **Contributor retention**: "Request changes" causes "contributor starvation" -- PRs rot waiting for authors who've moved on. Fix-merge ensures contributions land quickly.
- **Community cohesion**: Defaulting to "yes" prevents fragmentation through forking. Users feel heard when contributions land.
- **AI-native workflow**: With AI agents generating many PRs, the volume demands automation. Human gatekeeping doesn't scale.
- **Philosophical alignment**: The Good Boy Scout rule already embodies "just fix it." This ADR extends that principle to PR review policy.

## Consequences

### Benefits

- Faster PR throughput and shorter time-to-merge
- Healthier contributor experience and higher retention
- Reduced PR backlog and stale PR accumulation
- Measurable via merge rate, median resolution time, contributor return rate
- Distributed QA -- contributors submit ideas, maintainer ensures quality
- Positive feedback loops that attract more contributions

### Trade-offs

- Higher maintainer effort per PR (fixing vs bouncing back)
- Risk of merging lower-quality code that needs follow-up cleanup
- PR Akita automation requires initial development and ongoing tuning
- Merge-fix workflow requires trust in AI agents to make appropriate fixes

### Risks

- **Scope creep on fixes**: Maintainer over-invests time fixing a fundamentally flawed PR -> Mitigation: Clear boundaries on what constitutes "fixable" vs "needs-review"
- **Quality regression**: Merging too aggressively degrades codebase -> Mitigation: Safety validation pipeline catches dangerous patterns; CI gates remain enforced
- **Attribution errors**: Incorrect authorship on fix-merged PRs -> Mitigation: Strict `Co-authored-by` protocol, automated checks

## Alternatives Considered

### Alternative 1: Status Quo (Reactive Management)

- Description: Continue with current PR management loop that classifies by health status
- Pros: Already implemented, lower maintenance burden
- Cons: Scales poorly with AI-generated PR volume, encourages contributor churn, gatekeeping posture

### Alternative 2: Full Human Review

- Description: Require human review for every PR, no AI triage
- Pros: Maximum quality control, human judgment on every change
- Cons: Doesn't scale for 4-5 parallel Claude sessions, creates bottlenecks, slow velocity

### Alternative 3: Full Auto-Merge

- Description: Auto-merge everything that passes CI
- Pros: Maximum velocity, zero maintainer overhead
- Cons: No architectural oversight, quality risks, no triage intelligence

## Implementation Notes

- **PR Akita skill** at `.claude/skills/pr-akita/` handles the triage workflow
- **PR Akita agent** at `.claude/agents/pr-akita.md` provides the AI persona
- **Vibe Maintainer rule** at `.claude/rules/vibe-maintainer.md` encodes the philosophy for all sessions
- **Enhanced PR management loop** adds vibe triage step between classification and action execution
- **GitHub Actions workflow** at `.github/workflows/pr-akita.yml` provides automated labeling
- Start conservative with fix-merge scope (formatting, clippy, imports, rebase conflicts only)
- Expand scope based on data and confidence over time

## Success Metrics

- **Merge rate**: Target 88%+ (measured monthly)
- **Median time-to-merge**: Target < 24 hours for easy wins, < 72 hours overall
- **Contributor return rate**: Percentage of contributors who submit 2+ PRs
- **Fix-merge ratio**: Percentage of PRs merged via fix-merge vs request-changes
- **PR Akita automation rate**: Target 50-67% of PRs handled automatically

## References

- [Steve Yegge: Vibe Maintainer](https://steve-yegge.medium.com/vibe-maintainer-a2273a841040) -- Original inspiration
- `.claude/rules/good-boy-scout.md` -- "Fix it. No need to ask who caused it or why."
- `.claude/commands/pr-management-loop.md` -- Existing PR automation
- `.github/workflows/pr-merged.yml` -- Contributor attribution and milestone celebrations

## Revision History

| Date | Author | Changes |
|------|--------|---------|
| 2026-04-05 | Caro Maintainers | Initial draft |
