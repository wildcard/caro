# Mechanism 2: Adaptive Budget Management

## Overview

Replace manual budget management with rule-based adaptive allocation that responds to campaign performance in real-time.

## The Problem

Traditional budget management:
1. Set monthly budgets manually
2. Check performance weekly
3. Adjust based on gut feeling
4. Miss opportunities, waste spend on losers

**Result**: Slow reaction, suboptimal allocation, human bottleneck.

## The Solution

Build an **adaptive budget controller**:
1. Define clear performance rules (CPL thresholds, ROAS targets)
2. Monitor campaigns continuously
3. Auto-reallocate based on rules
4. Alert humans for anomalies
5. Maintain total budget constraints

## Core Principles

1. **Money follows performance**: Lower CPL = more budget
2. **Concentration limits**: No single campaign > 40% of total
3. **Minimum viable tests**: New campaigns get $X to prove themselves
4. **Pause thresholds**: Pause campaigns exceeding CPL for 48 hours
5. **Human oversight**: Anomalies trigger alerts, not auto-fixes

## System Architecture

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│  Campaign Data  │────▶│  Rule Engine    │────▶│  Recommendations│
│  (performance)  │     │  (thresholds)   │     │  (changes)      │
└─────────────────┘     └─────────────────┘     └─────────────────┘
                                │
                                ▼
                        ┌─────────────────┐
                        │  Alert System   │
                        │  (anomalies)    │
                        └─────────────────┘
```

## Rule Types

### Threshold Rules
```yaml
rules:
  - name: "scale_winners"
    trigger: "cpl < target * 0.8"
    action: "increase_budget_15%"
    cap: "40% of total"

  - name: "reduce_losers"
    trigger: "cpl > target * 1.3"
    action: "decrease_budget_20%"
    floor: "$100 minimum"

  - name: "boost_roas"
    trigger: "roas > 4"
    action: "increase_budget_10%"
    cap: "30% of total"
```

### Constraint Rules
```yaml
constraints:
  max_single_campaign: 40%
  min_campaign_budget: $100
  new_campaign_test_budget: $500
  pause_threshold_hours: 48
  total_budget_change: 0  # zero-sum
```

### Alert Rules
```yaml
alerts:
  - type: "anomaly"
    trigger: "cpl_change > 50% in 24h"
    action: "alert_human"

  - type: "opportunity"
    trigger: "roas > 6 for 3 days"
    action: "suggest_scale"

  - type: "risk"
    trigger: "spend_pace > 150%"
    action: "alert_overspend"
```

## Implementation Steps

### Step 1: Data Collection
Collect from advertising platforms:
- Campaign ID and name
- Current daily budget
- 7-day rolling metrics: spend, leads, conversions, CPL, ROAS

### Step 2: Rule Definition
```yaml
target_cpl: $50
target_roas: 2.5
observation_period: 7  # days
confidence_threshold: 95%
```

### Step 3: Recommendation Engine
```python
# Pseudocode
for campaign in campaigns:
    if campaign.cpl < target_cpl * 0.8:
        if campaign.budget < total_budget * 0.4:
            recommend("increase", 15%)
    elif campaign.cpl > target_cpl * 1.3:
        if campaign.budget > min_budget:
            recommend("decrease", 20%)
    else:
        recommend("hold", 0%)
```

### Step 4: Zero-Sum Balancing
```yaml
# Before balancing
recommendations:
  - campaign_1: +$150
  - campaign_2: -$100
  - campaign_3: +$50
  - campaign_4: -$100
  # Net: $0 ✓

# If not zero-sum, scale proportionally
```

### Step 5: Execution & Logging
```yaml
change_log:
  - timestamp: "2024-01-15T10:00:00Z"
    campaign_id: "camp_123"
    old_budget: 1000
    new_budget: 1150
    change_pct: 15%
    rule_fired: "scale_winners"
    rationale: "CPL $32 < target $50 * 0.8"
```

## Dashboard Metrics

| Metric | Description | Target |
|--------|-------------|--------|
| Reallocation frequency | Changes per week | 3-5 |
| Budget efficiency | Spend on winners vs losers | > 70% on winners |
| Rule trigger rate | % of rules that fire | 20-40% |
| Alert volume | Anomaly alerts per week | < 5 |
| Total CPL trend | Week-over-week | Decreasing |

## Safety Mechanisms

### Hard Limits
- Maximum single-day change: 25%
- Minimum observation period: 48 hours
- Human approval for changes > $500
- Emergency pause if CPL > 3x target

### Soft Limits
- Suggest review if 5+ campaigns paused
- Flag if total budget utilization < 80%
- Alert if winning campaign approaching cap

## Anomaly Detection

| Anomaly Type | Signal | Action |
|--------------|--------|--------|
| Spend spike | > 150% daily pace | Alert + investigate |
| Performance crash | CTR drop > 50% | Pause + alert |
| Fraud indicator | CTR > 10% with low CVR | Pause + review |
| Platform issue | 0 impressions | Alert + check status |

## Example Scenario

### Input
```yaml
total_budget: $10,000
target_cpl: $50
campaigns:
  - id: "camp_a"
    budget: 2000
    cpl: 35
  - id: "camp_b"
    budget: 2500
    cpl: 45
  - id: "camp_c"
    budget: 3000
    cpl: 75
  - id: "camp_d"
    budget: 2500
    cpl: 52
```

### Output
```yaml
recommendations:
  - campaign_id: "camp_a"
    current_budget: 2000
    recommended_budget: 2300  # +15%
    change_pct: 15%
    rationale: "CPL $35 < target $40 (0.8 × $50)"
    rule_fired: "scale_winners"
    confidence: "high"

  - campaign_id: "camp_c"
    current_budget: 3000
    recommended_budget: 2400  # -20%
    change_pct: -20%
    rationale: "CPL $75 > target $65 (1.3 × $50)"
    rule_fired: "reduce_losers"
    confidence: "high"

  - campaign_id: "camp_b"
    current_budget: 2500
    recommended_budget: 2750  # +10%
    change_pct: 10%
    rationale: "Absorbing budget from losers"
    rule_fired: "rebalance"
    confidence: "medium"

  - campaign_id: "camp_d"
    current_budget: 2500
    recommended_budget: 2550  # +2%
    change_pct: 2%
    rationale: "Absorbing budget from losers"
    rule_fired: "rebalance"
    confidence: "low"

summary:
  total_before: 10000
  total_after: 10000
  net_change: 0
  projected_cpl_improvement: "-8%"
```

## Tools & Platforms

| Function | Tool Options |
|----------|-------------|
| Data collection | Meta Marketing API, Google Ads API |
| Rule engine | Custom Python, Segment |
| Alerting | Slack, PagerDuty, custom webhooks |
| Dashboard | Looker, Metabase, custom |
| Execution | Platform APIs, custom scripts |

## B2C vs B2B Considerations

| Aspect | B2C | B2B |
|--------|-----|-----|
| Reallocation frequency | Daily | Weekly |
| Minimum test period | 48 hours | 1-2 weeks |
| Budget granularity | $50 minimum | $500 minimum |
| Human approval threshold | $500 | $2,000 |

## Common Pitfalls

1. **Over-optimization**: Chasing short-term CPL at expense of LTV
2. **Insufficient data**: Making decisions before statistical significance
3. **Ignoring seasonality**: Weekend/holiday patterns affect performance
4. **Single metric focus**: CPL optimization may hurt brand awareness
5. **Over-automation**: Some decisions need human judgment

## Remember

> "The goal isn't to remove humans from budget decisions. It's to free humans from routine reallocation so they can focus on strategy."

Adaptive budget management is a **co-pilot**, not an **autopilot**.
