# Mechanism 2: Contribution Scoring

## Purpose
Compute weighted contribution scores that reward quality and impact over raw activity. Produces ranked lists and tier assignments.

## Scoring Formula

```
raw_score = (commands_reused × 3) + (remixes × 2) + (original_creations × 1) + (safety_saves × 2)
```

### Weight Rationale

| Signal | Weight | Why |
|--------|--------|-----|
| Commands reused | 3 | Strongest signal: others found it useful enough to run |
| Remixes | 2 | Shows ecosystem participation, building on others' work |
| Original creations | 1 | Base contribution, but quantity alone isn't valuable |
| Safety saves | 2 | Protecting others from dangerous commands is high-value community service |

## Time Decay

Recent activity is weighted higher than old activity:

```
decay_factor = 0.5 ^ (days_since_activity / half_life_days)

# With half_life = 30 days:
#   Today:   decay = 1.0
#   15 days: decay = 0.71
#   30 days: decay = 0.50
#   60 days: decay = 0.25
#   90 days: decay = 0.125

decayed_score = sum(raw_score_per_day × decay_factor_for_that_day)
```

### Why Decay Matters
- Rewards sustained engagement over one-time bursts
- Prevents inactive accounts from permanently holding top positions
- Aligns with the goal of identifying currently active contributors

## Tier Thresholds

| Tier | Score Range | Description | Engagement Actions |
|------|-------------|-------------|-------------------|
| Explorer | 0-9 | New contributor | Welcome message, first-use guidance |
| Builder | 10-49 | Active contributor | Recognition messages, amplification offers |
| Leader | 50-199 | Significant contributor | Featured placement, direction recruiting |
| Founder-eligible | 200+ | Sustained high-value (30+ active days) | Founder tier invitation |

### Promotion Detection
Users approaching a tier boundary (within 10%) are flagged as `approaching_promotion`. This enables proactive "you're close" messaging to encourage continued contribution.

## Anti-Gaming Measures

### Reuse Ratio Check
```
reuse_ratio = commands_reused / max(original_creations, 1)

gaming_suspect = (original_creations > 10) AND (reuse_ratio < 0.1)
# Someone created many recipes but nobody uses them → suspicious
```

### Burst Detection
```
daily_creation_avg = total_creations / active_days
burst_detected = (creations_today > daily_creation_avg × 5)
# Sudden spike in creation without corresponding reuse
```

### Self-Reuse Exclusion
A user's own runs of their recipes do NOT count toward `commands_reused`. Only runs by other users count.

### Consequences of Gaming Flags
- `gaming_suspect` users are excluded from outreach targeting
- Their content is deprioritized in trending (but not removed)
- Human review is triggered if they approach Founder eligibility
- Flag is cleared if reuse ratio improves over 14 days

## Score Computation Frequency

| Computation | Frequency | Purpose |
|-------------|-----------|---------|
| Daily scores | Every 24h | Identify daily risers for outreach |
| Weekly rollup | Every 7d | Weekly leaderboard, trend detection |
| Monthly rollup | Every 30d | Tier promotions, Founder eligibility |
| Real-time milestone | On event | Immediate notification on milestone crossing |

## Edge Cases

- **New users** (< 7 days): Score normally but don't flag as gaming if low reuse (too early)
- **Returning users** (inactive > 60 days, now active): Reset burst detection baseline
- **Deleted recipes**: Subtract from score retroactively
- **Reported/flagged recipes**: Freeze score contribution pending moderation
