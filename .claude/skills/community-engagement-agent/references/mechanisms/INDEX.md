# Community Engagement Mechanisms

Deep-dive documentation for each of the 7 engagement mechanisms.

| # | Mechanism | File | Purpose |
|---|-----------|------|---------|
| 1 | Insight Gathering | [01-insight-gathering.md](01-insight-gathering.md) | Query Hub API for contributor data, trending content, usage patterns |
| 2 | Contribution Scoring | [02-contribution-scoring.md](02-contribution-scoring.md) | Weighted formula with anti-gaming, time decay, tier thresholds |
| 3 | Personalized Outreach | [03-personalized-outreach.md](03-personalized-outreach.md) | Message templates, personalization variables, anti-patterns |
| 4 | Channel Selection | [04-channel-selection.md](04-channel-selection.md) | Decision matrix for CLI vs. email vs. web delivery |
| 5 | Founder Curation | [05-founder-curation.md](05-founder-curation.md) | Eligibility criteria, invitation flow, cohort management |
| 6 | Feedback Tracking | [06-feedback-tracking.md](06-feedback-tracking.md) | Response rate tracking, A/B testing, strategy adjustment |
| 7 | Direction Recruiting | [07-direction-recruiting.md](07-direction-recruiting.md) | Gap detection, contributor matching, recruitment messages |

## How Mechanisms Relate

```
[1] Insight Gathering
  ├──→ [2] Contribution Scoring ──→ [3] Personalized Outreach ──→ [4] Channel Selection
  │         └──→ [5] Founder Curation ──→ [3]
  └──→ [7] Direction Recruiting ──→ [3] ──→ [4]

[6] Feedback Tracking (reads from all, adjusts strategy)
```
