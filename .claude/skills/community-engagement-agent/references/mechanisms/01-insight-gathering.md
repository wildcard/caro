# Mechanism 1: Insight Gathering

## Purpose
Query the CARO Hub API to produce a structured daily snapshot of community activity. This snapshot feeds all downstream agents.

## API Endpoints

| Endpoint | Purpose | Frequency |
|----------|---------|-----------|
| `GET /api/v1/leaderboard` | Top contributors by score | Daily |
| `GET /api/v1/recipes/trending` | Recipes gaining traction | Daily |
| `GET /api/v1/search/gaps` | Searches with no/few results | Daily |
| `GET /api/v1/stats/health` | Overall community metrics | Daily |
| `GET /api/v1/users/{id}/milestones` | Individual milestone events | On-demand |
| `GET /api/v1/recipes/{slug}/stats` | Per-recipe run/reuse data | On-demand |

## Data Aggregation Patterns

### Daily Snapshot
- Top 50 contributors by 24h activity
- Top 20 trending recipes by velocity
- Top 10 community gaps by demand score
- Milestone events from last 24h
- Health metrics (active publishers, new recipes, total runs, success rate)

### Weekly Rollup
- Top 50 contributors by 7d activity
- Score deltas (who rose, who fell)
- New publishers (first-time recipe authors)
- Category distribution shifts

### Monthly Summary
- Tier promotions
- Founder tier candidate pipeline
- Category coverage map
- Engagement funnel metrics

## Trending Detection

Trending is measured by **velocity** (rate of change), not absolute count:

```
velocity = (runs_current_period - runs_previous_period) / runs_previous_period

# A recipe with 10 runs going to 50 is more trending than
# a recipe with 1000 runs going to 1050
```

### Velocity Thresholds
- **Hot**: velocity > 2.0 (more than tripled)
- **Rising**: velocity > 0.5 (grew by 50%+)
- **Stable**: velocity between -0.1 and 0.5
- **Declining**: velocity < -0.1

## Gap Detection

A "gap" is a category where search demand exceeds recipe supply:

```
gap_score = search_volume_7d / max(recipe_count, 1)

# Categories with gap_score > 10 are "critical gaps"
# Categories with gap_score > 5 are "moderate gaps"
```

## Output Caching

Snapshots are cached for 1 hour to avoid redundant API calls when multiple agents consume the same data within a daily run cycle.

## Error Handling

- API timeout: retry 3 times with exponential backoff (1s, 2s, 4s)
- API 404: skip endpoint, log warning, continue with partial data
- API 500: abort daily run, alert human operator
- Rate limit (429): respect Retry-After header, queue for next cycle
