# Mechanism 7: Direction Recruiting

## Purpose
Identify categories where user demand exceeds recipe supply, find contributors with matching expertise, and recruit them to fill the gap.

## Gap Detection

### Data Source
Search analytics from the Hub API: queries that return no/few results.

### Gap Score Formula
```
gap_score = search_volume_7d / max(recipe_count, 1)

# Interpretation:
#   gap_score > 10 → Critical: high demand, almost no supply
#   gap_score > 5  → Moderate: meaningful demand, sparse supply
#   gap_score > 2  → Nice-to-have: some demand, limited supply
#   gap_score < 2  → Covered: supply meets demand
```

### Example Gaps
| Category | Searches (7d) | Recipes | Gap Score | Priority |
|----------|---------------|---------|-----------|----------|
| PDF tools | 230 | 3 | 76.7 | Critical |
| Audio processing | 180 | 8 | 22.5 | Critical |
| CSV/data transforms | 120 | 12 | 10.0 | Critical |
| Docker cleanup | 90 | 15 | 6.0 | Moderate |
| Git workflows | 200 | 45 | 4.4 | Nice-to-have |

## Contributor Matching

### Expertise Signals
How to determine if a contributor can fill a gap:

1. **Direct match**: They've published recipes in the same category
2. **Tool match**: They use the same underlying tools (e.g., FFmpeg user → audio processing)
3. **Adjacent match**: They've built similar complexity recipes in another category
4. **Tag match**: Their recipe tags overlap with the gap's search terms

### Relevance Score
```
relevance = (direct_match × 3) + (tool_match × 2) + (adjacent_match × 1) + (tag_match × 0.5)
```

### Match Quality Threshold
Only recruit if `relevance >= 3`. Below that, the connection is too weak to feel personal.

## Recruitment Message Design

### Key Principles
1. **Show real demand**: Use actual search numbers, not vague "people want this"
2. **Reference their work**: Connect to what they've already built
3. **Frame as helping users**: "People need this" not "we need content"
4. **Suggest, don't demand**: "Would you be interested?" not "Please create"

### Template Structure
```
CLI:
  caro: {volume} people searched for "{gap}" this week.
  caro: You built {related_recipe} - would you create something similar? (y/n)

Email:
  Subject: {volume} people searched for {gap} this week
  
  You built {related_recipe}, which has been run {run_count} times.
  
  This week, {volume} people searched for {gap_description} but
  found very few results. Based on your experience with {tool},
  you'd be a great person to help.
  
  If you're interested, here are the most common searches:
  - "{query_1}" ({count_1} searches)
  - "{query_2}" ({count_2} searches)
  - "{query_3}" ({count_3} searches)
  
  No pressure - just thought you'd want to know people are looking
  for exactly the kind of thing you're good at.
```

## Follow-Through Tracking

| Metric | Measurement | Target |
|--------|-------------|--------|
| Recruitment response rate | % of recruited users who respond | > 25% |
| Conversion rate | % who publish a relevant recipe within 14d | > 15% |
| Gap closure rate | % of critical gaps reduced to moderate within 30d | > 50% |
| Recipe quality | Avg reuse rate of recruited recipes | > community average |

### Follow-Up Rules
- If no response after 7 days: no follow-up (respect silence)
- If user responds positively but doesn't publish in 14d: one gentle reminder
- If user publishes: send Recognition message thanking them
- Track whether recruited recipes actually get used (quality signal)

## Gap Prioritization

When multiple gaps exist, prioritize by:

1. **Gap score** (highest demand-supply ratio first)
2. **Ease of filling** (gaps where matching contributors exist)
3. **Category importance** (consumer categories > niche developer categories)
4. **Trend direction** (growing search volume > declining)
