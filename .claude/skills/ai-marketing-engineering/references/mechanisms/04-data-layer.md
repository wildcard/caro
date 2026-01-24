# Mechanism 4: Contextual Data Layer

## Overview

Build a conversational interface layer on top of marketing data so AI agents (and humans) can query it naturally.

## The Problem

Marketing data is siloed:
1. GA4 for web analytics
2. Mixpanel for product events
3. Hubspot for CRM
4. Stripe for revenue
5. BigQuery for everything else

**Result**: AI agents can't easily access data to make decisions.

## The Solution

Build a **contextual data layer**:
1. Unified semantic layer across all sources
2. Natural language → structured query translation
3. Permission controls (who can ask what)
4. Caching for performance
5. Metadata for AI interpretation

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     Natural Language Interface                   │
│                 "What's our CPL by channel this week?"           │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                        Semantic Layer                            │
│     Concepts: "CPL", "channel", "week" → SQL/API mappings       │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                        Data Sources                              │
│   ┌──────┐  ┌──────────┐  ┌────────┐  ┌──────┐  ┌──────────┐   │
│   │ GA4  │  │ Mixpanel │  │Hubspot │  │Stripe│  │ BigQuery │   │
│   └──────┘  └──────────┘  └────────┘  └──────┘  └──────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

## Semantic Layer Design

### Concept Mapping
```yaml
concepts:
  - name: "CPL"
    aliases: ["cost per lead", "lead cost", "acquisition cost"]
    calculation: "total_spend / total_leads"
    sources:
      spend: "meta_ads.spend + google_ads.spend"
      leads: "hubspot.new_contacts"
    unit: "USD"
    default_aggregation: "average"

  - name: "channel"
    aliases: ["source", "acquisition channel", "marketing channel"]
    values:
      - "meta" → "meta_ads.campaign_source"
      - "google" → "google_ads.campaign_source"
      - "organic" → "ga4.source = 'organic'"
      - "referral" → "ga4.medium = 'referral'"

  - name: "week"
    aliases: ["this week", "weekly", "past 7 days"]
    calculation: "date >= current_date - 7"
    calendar: "ISO_WEEK"
```

### Query Patterns
```yaml
query_patterns:
  - natural_language: "What's our CPL by channel this week?"
    structured_query: |
      SELECT channel, SUM(spend) / COUNT(leads) as cpl
      FROM marketing_data
      WHERE date >= current_date - 7
      GROUP BY channel
    response_format: "table"

  - natural_language: "How many leads came from Meta yesterday?"
    structured_query: |
      SELECT COUNT(*) as leads
      FROM hubspot_contacts
      WHERE source = 'meta' AND created_date = current_date - 1
    response_format: "single_value"

  - natural_language: "Show me ROAS trend for the past month"
    structured_query: |
      SELECT date, SUM(revenue) / SUM(spend) as roas
      FROM marketing_data
      WHERE date >= current_date - 30
      GROUP BY date
      ORDER BY date
    response_format: "time_series_chart"
```

## Implementation Steps

### Step 1: Data Inventory
```yaml
data_sources:
  - name: "Meta Ads"
    type: "api"
    refresh_rate: "hourly"
    tables:
      - campaigns
      - ad_sets
      - ads
      - insights
    key_metrics: ["spend", "impressions", "clicks", "conversions"]

  - name: "Hubspot"
    type: "api"
    refresh_rate: "15min"
    tables:
      - contacts
      - deals
      - activities
    key_metrics: ["new_contacts", "deal_value", "lifecycle_stage"]

  - name: "BigQuery"
    type: "database"
    refresh_rate: "real-time"
    tables:
      - events
      - users
      - transactions
    key_metrics: ["revenue", "ltv", "retention"]
```

### Step 2: Permission Model
```yaml
permissions:
  - role: "marketing_manager"
    allowed_sources: ["meta_ads", "google_ads", "hubspot"]
    allowed_concepts: ["spend", "leads", "cpl", "channel"]
    denied_concepts: ["revenue", "ltv", "individual_user_data"]

  - role: "executive"
    allowed_sources: ["all"]
    allowed_concepts: ["all_aggregated"]
    denied_concepts: ["individual_user_data"]

  - role: "ai_agent"
    allowed_sources: ["meta_ads", "google_ads", "hubspot"]
    allowed_concepts: ["spend", "leads", "cpl", "roas"]
    rate_limit: "100 queries/hour"
```

### Step 3: Caching Strategy
```yaml
caching:
  real_time:
    - current_spend
    - live_campaign_status
  hourly:
    - cpl_by_channel
    - daily_performance
  daily:
    - ltv_cohorts
    - monthly_trends
  weekly:
    - segment_analysis
    - attribution_models
```

### Step 4: Query Interface
```python
# Example Python interface
from marketing_data import query

# Natural language query
result = query("What's our CPL by channel this week?")
# Returns: {"meta": 45.20, "google": 52.10, "organic": 12.50}

# Structured query
result = query({
    "metric": "cpl",
    "dimensions": ["channel"],
    "filters": {"date_range": "last_7_days"}
})
```

## Use Cases for AI Agents

### Budget Agent
```
Query: "Which campaigns have CPL below target?"
Response: [{"campaign_id": "abc", "cpl": 32, "target": 50}, ...]
Action: Recommend budget increase for those campaigns
```

### Signals Agent
```
Query: "What's the correlation between first_action_time and ltv?"
Response: {"correlation": 0.42, "p_value": 0.001}
Action: Add to LTV signal findings
```

### Churn Agent
```
Query: "How many support tickets had negative sentiment today?"
Response: {"count": 15, "critical": 3}
Action: Prioritize intervention queue
```

## Metadata for AI Interpretation

```yaml
metric_metadata:
  cpl:
    description: "Cost to acquire one lead"
    good_direction: "lower is better"
    benchmark: "$30-50 for B2B SaaS"
    related_metrics: ["roas", "conversion_rate"]
    caveats: "Varies significantly by channel and segment"

  roas:
    description: "Revenue generated per dollar spent"
    good_direction: "higher is better"
    benchmark: "2-4x for healthy campaigns"
    related_metrics: ["cpl", "ltv"]
    caveats: "Attribution model affects accuracy"
```

## Tools & Platforms

| Function | Tool Options |
|----------|-------------|
| Semantic layer | dbt, LookML, Cube.js |
| Query interface | Text-to-SQL (custom), Metabase |
| Caching | Redis, Memcached, BigQuery BI Engine |
| Permissions | Custom RBAC, dbt permissions |
| API layer | FastAPI, GraphQL |

## Example Implementation

### Minimum Viable Data Layer (2 weeks)

**Week 1: Core Setup**
- Connect top 3 data sources (Meta, Hubspot, BigQuery)
- Define 10 core concepts (spend, leads, CPL, ROAS, etc.)
- Build basic query parser

**Week 2: AI Integration**
- Add metadata for AI interpretation
- Create query patterns for each agent
- Implement caching for common queries

### Comprehensive Data Layer (6 weeks)

**Weeks 1-2**: Full data source integration
**Weeks 3-4**: Complete semantic layer with 50+ concepts
**Weeks 5-6**: Permission system, advanced caching, monitoring

## Quality Gates

### Data Quality
- [ ] Source freshness monitored
- [ ] Null/missing data handling
- [ ] Cross-source reconciliation

### Query Quality
- [ ] Natural language parsing accuracy > 90%
- [ ] Query response time < 2 seconds
- [ ] Result format matches expected schema

### Security
- [ ] Permission model tested
- [ ] PII data masked appropriately
- [ ] Audit logging enabled

## Common Pitfalls

1. **Over-engineering**: Building for 50 sources when you need 3
2. **Stale data**: Queries returning outdated results
3. **Ambiguous semantics**: "Revenue" means different things
4. **Permission gaps**: AI accessing restricted data
5. **Performance issues**: Slow queries blocking agents

## Remember

> "The goal isn't a perfect data warehouse. It's an interface that lets AI agents (and humans) ask questions and get reliable answers in seconds."

Start with the queries you actually need, not the queries you might need.
