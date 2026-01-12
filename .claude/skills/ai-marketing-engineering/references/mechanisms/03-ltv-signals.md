# Mechanism 3: LTV Signal Hunting

## Overview

Find correlations in raw data that humans miss—the non-obvious patterns that predict lifetime value (LTV).

## The Problem

Marketing teams typically:
1. Look at obvious metrics (signup date, plan type)
2. Miss counterintuitive patterns
3. Treat all users the same
4. React to churn instead of predicting it

**Result**: Missed opportunities, wasted spend on low-LTV users, preventable churn.

## The Solution

Build a **signal hunting system**:
1. Collect broad behavioral and demographic data
2. Analyze for non-obvious correlations with LTV
3. Identify subpopulation effects (works for A but not B)
4. Find timing effects (week 1 behavior predicts month 6)
5. Suggest validation experiments

## What Makes a "Signal" Valuable?

| Attribute | Description |
|-----------|-------------|
| Counterintuitive | Not obvious ("Tuesday signups have 2x LTV") |
| Actionable | Can be used to change behavior |
| Statistically significant | > 95% confidence |
| Sufficiently sized | Large enough segment to matter |
| Causal hypothesis | Plausible explanation exists |

## Types of Patterns

### 1. Counterintuitive Correlations
```
Finding: Users who complete onboarding slowly (>1 week) have 30% higher LTV
Why: They're more deliberate, evaluate thoroughly before committing
Action: Stop rushing onboarding; add more discovery features
```

### 2. Subpopulation Effects
```
Finding: Mobile signups convert at 2x for B2C, 0.5x for B2B
Why: B2C users are casual; B2B users prefer desktop for work tools
Action: Segment mobile vs desktop in acquisition strategy
```

### 3. Timing Effects
```
Finding: Users who invite a teammate in week 1 have 4x retention at month 6
Why: Social accountability and shared investment
Action: Push team invites early in onboarding
```

### 4. Interaction Effects
```
Finding: Night signups + tutorial completion = highest LTV
Why: Serious users who invest personal time to learn properly
Action: Target night-owl professional segments
```

## System Architecture

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│  Data Sources   │────▶│  Feature        │────▶│  Correlation    │
│  (events, CRM)  │     │  Engineering    │     │  Analysis       │
└─────────────────┘     └─────────────────┘     └─────────────────┘
                                                        │
                                                        ▼
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│  Validation     │◀────│  Pattern        │◀────│  Signal         │
│  Experiments    │     │  Ranking        │     │  Detection      │
└─────────────────┘     └─────────────────┘     └─────────────────┘
```

## Data Requirements

### Behavioral Data
- Signup timestamp, device, source
- Feature usage (which, when, how often)
- Session patterns (frequency, duration, time of day)
- Content consumption (what, how much)
- Communication engagement (email opens, support tickets)

### Outcome Data
- Revenue/LTV at 30, 60, 90, 180, 365 days
- Churn date (if applicable)
- Upgrade/downgrade events
- NPS/CSAT scores

### Demographic Data
- Company size, industry (B2B)
- Role/seniority
- Geography
- Acquisition channel

## Analysis Methodology

### Step 1: Feature Engineering
```python
# Convert raw events to analyzable features
features = [
    "days_to_first_action",
    "features_used_week_1",
    "session_frequency_week_1",
    "time_of_day_signup",  # bucketed
    "device_type",
    "referral_source",
    "team_invites_week_1",
    # ...100+ features
]
```

### Step 2: Correlation Screening
```python
# Calculate correlation with LTV for each feature
for feature in features:
    r_squared = calculate_correlation(feature, ltv)
    p_value = statistical_significance(feature, ltv)
    if p_value < 0.05:
        candidates.append(feature, r_squared, p_value)
```

### Step 3: Surprise Ranking
```python
# Rank by how counterintuitive the finding is
for candidate in candidates:
    obvious_score = rate_obviousness(candidate)  # 1-5
    surprise_level = 6 - obvious_score  # Invert
    candidate.surprise_level = surprise_level
```

### Step 4: Subpopulation Analysis
```python
# Check if effect holds across segments
for candidate in top_candidates:
    for segment in segments:
        segment_effect = calculate_effect(candidate, segment)
        if segment_effect != overall_effect:
            findings.append(f"{candidate} varies by {segment}")
```

## Output Schema

```yaml
findings:
  - pattern: "Users who use feature X within 72 hours have 2.5x LTV"
    strength: "r² = 0.42"
    confidence: "98%"
    sample_size: 8500
    surprising_level: 4  # 5 = very counterintuitive
    segment: "all"
    actionable_insight: "Promote feature X in onboarding email"
    validation_experiment:
      hypothesis: "Prompting feature X use will increase LTV"
      test_design: "A/B test: prompt vs no prompt in day 3 email"
      success_metric: "30-day retention +10%"
      sample_size_needed: 2000
      duration: "4 weeks"
```

## Example Findings Report

```markdown
# LTV Signal Analysis - Q1 2024

## Top 5 Findings (Ranked by Surprise Level)

### 1. Slow Onboarders Have Higher LTV (Surprise: 5/5)
- **Pattern**: Users completing onboarding in 7+ days have 40% higher LTV
- **Strength**: r² = 0.35, p < 0.001, n = 12,500
- **Hypothesis**: Deliberate evaluation → higher commitment
- **Action**: Stop rushing; add "explore at your pace" messaging
- **Experiment**: Remove "complete now" prompts for 50% of signups

### 2. Night Owl Premium (Surprise: 4/5)
- **Pattern**: Signups between 9pm-2am have 2x conversion to paid
- **Strength**: OR = 2.1, p < 0.01, n = 3,200
- **Hypothesis**: Personal time investment = serious intent
- **Action**: Increase ad spend in evening hours
- **Experiment**: Shift 20% of ad budget to 8pm-12am

### 3. Support Ticket Paradox (Surprise: 4/5)
- **Pattern**: Users who file 1-2 tickets in month 1 have higher retention
- **Strength**: r² = 0.28, p < 0.001, n = 6,800
- **Hypothesis**: Engagement signal; investment in making it work
- **Action**: Make support more accessible in onboarding
- **Experiment**: Add "need help?" prompt at friction points

### 4. Mobile-First B2C (Surprise: 3/5)
- **Pattern**: Mobile signups convert 2x for B2C, 0.5x for B2B
- **Strength**: Segment effect, p < 0.001
- **Segment**: B2C (3,400) vs B2B (2,100)
- **Action**: Segment mobile advertising by vertical
- **Experiment**: Create B2C-specific mobile landing pages

### 5. Referral Source Quality (Surprise: 2/5)
- **Pattern**: Organic search has 1.5x LTV vs paid social
- **Strength**: r² = 0.22, p < 0.001, n = 15,000
- **Hypothesis**: Intent-driven discovery → better fit
- **Action**: Invest more in SEO/AEO
- **Experiment**: Track LTV by channel, adjust CAC targets
```

## Tools & Platforms

| Function | Tool Options |
|----------|-------------|
| Data warehouse | BigQuery, Snowflake, Redshift |
| Feature store | Feast, Tecton, custom |
| Analysis | Python (pandas, scipy), R, SQL |
| Visualization | Looker, Metabase, Hex |
| Experimentation | Statsig, Eppo, custom |

## Quality Gates

### Before Acting on Findings
- [ ] Statistical significance > 95%
- [ ] Sample size > 500 for main effect
- [ ] Correlation vs causation distinguished
- [ ] Business logic sanity check
- [ ] Validation experiment designed

### Red Flags
- Effect only in small subpopulation (< 100)
- No plausible causal mechanism
- Contradicts known customer behavior
- Data collection issues (selection bias)

## Common Pitfalls

1. **Overfitting**: Finding patterns that don't generalize
2. **Survivorship bias**: Ignoring churned users
3. **Correlation ≠ causation**: Acting without validation
4. **Data leakage**: Using future data to predict past
5. **Simpson's paradox**: Aggregate trend hides segment effects

## B2C vs B2B Considerations

| Aspect | B2C | B2B |
|--------|-----|-----|
| LTV timeline | 30-90 days | 6-12 months |
| Sample size | Large (10k+) | Smaller (100s) |
| Segmentation | Demographics | Company attributes |
| Timing signals | Daily patterns | Deal stage patterns |

## Remember

> "The non-obvious patterns are where competitive advantage lives. Anyone can see that 'more usage = better retention.' The winners find that 'Tuesday signups who use mobile have 3x LTV.'"

Signal hunting is a continuous process, not a one-time analysis.
