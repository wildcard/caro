# Mechanism 6: Feedback Tracking

## Purpose
Track engagement effectiveness to continuously improve outreach quality. Measure what works, what doesn't, and adjust strategy accordingly.

## Metrics Tracked

### Per-Message Metrics
| Metric | Description | Source |
|--------|-------------|--------|
| Delivered | Message successfully sent | Send confirmation |
| Opened | Email opened / CLI message displayed | Email tracking / CLI event |
| Responded | User took an action (replied, clicked, published) | Hub API / CLI event |
| Response time | Hours between send and response | Timestamps |
| Action taken | What the user did (published, clicked, replied, ignored) | Hub API |

### Aggregate Metrics
| Metric | Computation | Target |
|--------|-------------|--------|
| Response rate | responded / delivered | > 20% |
| Channel effectiveness | response_rate per channel | Varies |
| Message type effectiveness | response_rate per type | Varies |
| Optimal send time | response_rate by hour/day | Data-driven |

## A/B Testing Framework

### When to A/B Test
- Minimum sample size: 20 messages per variant
- Only test one variable at a time
- Run for minimum 7 days before evaluating

### Testable Variables
| Variable | Example Variants |
|----------|-----------------|
| Opening line | Specific stat vs. question vs. compliment |
| CTA phrasing | "Want to feature it?" vs. "Share with the community?" |
| Message length | 2-line vs. 4-line CLI messages |
| Tone | Factual vs. warm |
| Timing | Morning vs. evening send |

### Variant Assignment
```
variant = hash(user_id + test_name) % 2  # Deterministic, consistent per user
```

### Significance Testing
- Use simple proportion comparison
- Require p < 0.05 before declaring winner
- Minimum 50 observations per variant before testing significance

## Strategy Adjustment Rules

### Reduce Frequency When
- Channel response rate drops below 10% for 2 consecutive weeks
- User has ignored 3+ consecutive messages on a channel
- Overall engagement is declining week-over-week

### Change Channel When
- User responds on channel B but ignores channel A for 4+ messages
- Channel A is consistently below 10% response rate for this user

### Modify Template When
- A/B test shows clear winner (p < 0.05)
- Response rate for a message type drops below 15% for 2+ weeks
- Qualitative feedback indicates message tone is off

### Pause Outreach When
- User explicitly opts out or expresses annoyance
- All channels below 5% response rate for this user
- User hasn't been active on CARO for 30+ days (they may have churned)

## Reporting

### Weekly Summary
```
Period: [date range]
Messages sent: N
Response rate: X%
  - CLI: X%
  - Email: X%
  - Web: X%
Top performing message type: [type] at X%
Recommendations: [list of adjustments]
Active A/B tests: [list with current results]
```

### Monthly Review
- Trend analysis (improving, stable, declining)
- Channel mix optimization recommendations
- Template refresh recommendations
- Founder pipeline effectiveness
- Direction recruiting success rate
