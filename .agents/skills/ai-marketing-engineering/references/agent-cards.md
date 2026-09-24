# Agent Cards: Marketing Engineering Mechanisms

Detailed specifications for each of the 10 AI Marketing Engineering agents.

---

## @creative-agent

### Mission
Transform ad creative production from "one designer, one banner" to "evolutionary creative machine that generates, tests, and evolves hundreds of variations."

### System Prompt Fragment
```
You are a creative evolution system. Your job is to:
1. Generate systematic variations of ad creatives
2. Identify winning patterns from performance data
3. Clone winners with slight modifications
4. Kill underperformers quickly

You think in terms of:
- Variation axes (headline, visual, CTA, color, format)
- Mutation rates (how much to change between generations)
- Fitness functions (CTR, CVR, ROAS)
- Selection pressure (budget allocation to winners)
```

### Input Schema
```yaml
creative_brief:
  objective: string  # awareness | consideration | conversion
  platform: string   # meta | google | tiktok | linkedin
  base_assets:
    images: url[]
    copy_variants: string[]
    ctas: string[]
  constraints:
    brand_guidelines: url
    forbidden_claims: string[]
  performance_data: # optional
    existing_creatives:
      - id: string
        metrics: {impressions, clicks, conversions}
```

### Output Schema
```yaml
variation_batch:
  - id: string
    parent_id: string | null
    changes:
      - axis: string
        original: string
        new: string
    predicted_impact: string
    test_priority: 1-5
evolution_plan:
  generation_size: number
  mutation_rate: percentage
  selection_criteria: string[]
  review_cadence: string
```

### Workflow
1. **Decompose** base creative into variation axes
2. **Generate** N variations per axis (combinatorial expansion)
3. **Prioritize** by predicted impact and test cost
4. **Launch** in small test cohorts
5. **Analyze** performance after statistical significance
6. **Evolve** winners, kill losers
7. **Repeat** until performance plateau

---

## @budget-agent

### Mission
Replace manual budget management with rule-based adaptive allocation that responds to performance in real-time.

### System Prompt Fragment
```
You are an adaptive budget controller. Your rules:
1. Money follows performance (lower CPL = more budget)
2. Never let a single campaign consume > 40% of total
3. New campaigns get minimum viable test budget
4. Pause campaigns that exceed threshold for 48 hours
5. Alert humans for anomalies, don't auto-fix everything
```

### Input Schema
```yaml
campaigns:
  - id: string
    name: string
    current_budget: number
    metrics:
      spend: number
      leads: number
      cpl: number
      conversions: number
      roas: number
rules:
  - trigger: string  # e.g., "cpl < target * 0.8"
    action: string   # e.g., "increase_budget_10pct"
    cap: number
total_budget: number
constraints:
  min_campaign_budget: number
  max_single_campaign_pct: percentage
```

### Output Schema
```yaml
recommendations:
  - campaign_id: string
    current_budget: number
    recommended_budget: number
    change_pct: percentage
    rationale: string
    confidence: high | medium | low
alerts:
  - type: anomaly | opportunity | risk
    campaign_id: string
    message: string
    suggested_action: string
projected_impact:
  total_leads_change: percentage
  total_cpl_change: percentage
```

---

## @signals-agent

### Mission
Find correlations in raw data that humans miss—the non-obvious patterns that predict LTV.

### System Prompt Fragment
```
You are a pattern hunter. You look for:
1. Counterintuitive correlations (not obvious ones)
2. Subpopulation effects (works for segment A but not B)
3. Timing effects (behavior in week 1 predicts month 6)
4. Interaction effects (X alone doesn't matter, X + Y does)

Always distinguish correlation from causation.
Always report confidence intervals.
Always suggest validation experiments.
```

### Input Schema
```yaml
dataset:
  source: string
  rows: number
  columns:
    - name: string
      type: categorical | numeric | datetime
      description: string
outcome_variable: string
segment_by: string[] | null
time_window:
  start: date
  end: date
```

### Output Schema
```yaml
findings:
  - pattern: string
    strength: r_squared | odds_ratio
    confidence: percentage
    sample_size: number
    surprising_level: 1-5  # 5 = very counterintuitive
    segment: string | "all"
    actionable_insight: string
    validation_experiment:
      hypothesis: string
      test_design: string
      success_metric: string
```

---

## @data-layer-agent

### Mission
Build a conversational interface layer on top of marketing data so AI agents can query it naturally.

### System Prompt Fragment
```
You design data interfaces for AI consumers, not human dashboards.
Your outputs should be:
1. Query-friendly (natural language → SQL/API)
2. Contextual (include metadata AI needs to interpret)
3. Fresh (define refresh cadence)
4. Permissioned (who can ask what)
```

### Input Schema
```yaml
data_sources:
  - name: string
    type: database | api | dashboard
    schema_url: string
    refresh_rate: string
use_cases:
  - query_example: string
    expected_answer_type: string
permissions:
  - role: string
    allowed_sources: string[]
```

### Output Schema
```yaml
interface_design:
  query_patterns:
    - natural_language: string
      structured_query: string
      response_format: string
  semantic_layer:
    - concept: string
      maps_to: string
      aggregation_default: string
  caching_strategy: string
  access_control: object
implementation_steps:
  - step: string
    effort: hours
    dependencies: string[]
```

---

## @aeo-agent

### Mission
Shift from SEO (Google ranking) to AEO (Answer Engine Optimization)—become the source that LLMs cite.

### System Prompt Fragment
```
You optimize for AI answer engines, not just Google.
Your strategy:
1. Become the authoritative source in communities (Reddit, forums)
2. Create content structured for LLM consumption (clear, factual, citable)
3. Monitor LLM responses for your brand/competitors
4. Value-first engagement (never spam)
```

### Input Schema
```yaml
target_queries:
  - query: string
    intent: informational | transactional | navigational
    current_llm_response: string | null
content_inventory:
  - url: string
    topic: string
    last_updated: date
community_presence:
  - platform: string  # reddit | quora | stackoverflow
    subreddits: string[]
    current_karma: number
```

### Output Schema
```yaml
aeo_strategy:
  content_gaps:
    - query: string
      recommended_content: string
      citation_optimization: string[]
  community_plan:
    - platform: string
      target_communities: string[]
      engagement_approach: string  # value-first, no spam
      content_calendar: object
  monitoring:
    - llm: string  # chatgpt | perplexity | claude
      queries_to_track: string[]
      alert_triggers: string[]
```

---

## @quiz-agent

### Mission
Replace static forms with dynamic quizzes that adapt questions based on user context and responses.

### System Prompt Fragment
```
You design adaptive qualification flows.
Principles:
1. Every question must earn its place (no fluff)
2. Answers change subsequent questions
3. Detect urgency/pain signals and adjust path
4. Build trust through personalization
5. Clear handoff criteria (self-serve vs sales vs not-a-fit)
```

### Input Schema
```yaml
qualification_criteria:
  - segment: string
    criteria: string[]
    handoff: self_serve | sales | nurture | disqualify
product_context:
  value_props: string[]
  common_objections: string[]
  pain_points: string[]
user_signals:
  - signal: string  # e.g., "urgent language"
    indicator: string
    response: string
```

### Output Schema
```yaml
quiz_flow:
  entry_point: question_id
  questions:
    - id: string
      text: string
      type: single_choice | multi_choice | free_text | scale
      options: string[] | null
      branching:
        - condition: string
          next_question: string
  exit_points:
    - id: string
      segment: string
      handoff: string
      messaging: string
personalization_rules:
  - trigger: string
    adjustment: string
```

---

## @activation-agent

### Mission
Detect user friction in real-time and trigger targeted interventions to prevent drop-off.

### System Prompt Fragment
```
You monitor user behavior for friction signals.
Your job:
1. Define "stuck" moments (time on page, repeat actions, etc.)
2. Design interventions (tooltip, email, chat prompt)
3. A/B test intervention effectiveness
4. Measure impact on activation metrics
```

### Input Schema
```yaml
product_events:
  - event: string
    aha_moment: boolean
    typical_time: seconds
    drop_off_rate: percentage
intervention_templates:
  - id: string
    channel: in_app | email | sms | chat
    template: string
activation_metric: string
observation_window: hours
```

### Output Schema
```yaml
friction_rules:
  - trigger:
      event_pattern: string
      time_threshold: seconds
      repeat_threshold: number
    intervention:
      template_id: string
      personalization: object
      delay: seconds
    success_metric: string
ab_test_plan:
  - hypothesis: string
    control: string
    variant: string
    sample_size: number
    duration: days
```

---

## @video-agent

### Mission
Create personalized video content at scale with lip-sync name/company mentions.

### System Prompt Fragment
```
You orchestrate personalized video production.
Quality gates:
1. Name pronunciation accuracy
2. Lip-sync quality (no uncanny valley)
3. Natural timing (not robotic)
4. Brand voice consistency
5. Recipient consent verified
```

### Input Schema
```yaml
base_video:
  url: string
  personalization_points:
    - timestamp: seconds
      type: name | company | custom
      placeholder: string
recipients:
  - id: string
    name: string
    company: string
    custom_fields: object
delivery:
  channel: email | linkedin | crm
  subject_template: string
  body_template: string
```

### Output Schema
```yaml
production_plan:
  total_videos: number
  estimated_time: minutes
  quality_checks:
    - check: string
      sample_rate: percentage
delivery_schedule:
  - batch: number
    count: number
    send_time: datetime
tracking:
  - metric: string
    measurement: string
```

---

## @competitive-agent

### Mission
Mine competitor reviews for pain points and create targeted landing pages addressing each weakness.

### System Prompt Fragment
```
You extract competitive intelligence ethically.
Your approach:
1. Aggregate public review data (G2, Capterra, app stores)
2. Categorize pain points by theme
3. Map your product's strengths to their weaknesses
4. Create content that addresses specific frustrations
5. No false claims, only verifiable differentiators
```

### Input Schema
```yaml
competitors:
  - name: string
    review_sources:
      - platform: string
        url: string
your_product:
  strengths: string[]
  differentiators: string[]
content_templates:
  - type: landing_page | blog_post | comparison
    template_url: string
```

### Output Schema
```yaml
pain_point_analysis:
  - competitor: string
    pain_points:
      - category: string
        frequency: number
        severity: 1-5
        example_quotes: string[]
        your_solution: string
content_recommendations:
  - pain_point: string
    content_type: string
    headline: string
    key_messages: string[]
    seo_keywords: string[]
competitive_matrix:
  features:
    - feature: string
      you: string
      competitors: object
```

---

## @churn-agent

### Mission
Detect customer frustration in real-time (support, chat, product signals) and intervene before churn.

### System Prompt Fragment
```
You prevent churn through early intervention.
Your sensors:
1. Support ticket sentiment
2. Chat tone analysis
3. Product usage decline
4. Engagement drop-off
5. Payment issues

Your interventions:
1. Empathetic response scripts
2. Proactive outreach
3. Compensation offers
4. Escalation to humans
```

### Input Schema
```yaml
signals:
  support_tickets:
    - id: string
      text: string
      sentiment: number
      urgency: 1-5
  product_usage:
    - user_id: string
      activity_trend: increasing | stable | declining
      days_since_login: number
  payment:
    - user_id: string
      failed_payments: number
      dunning_stage: number
intervention_playbooks:
  - trigger: string
    action: string
    owner: team | automated
```

### Output Schema
```yaml
at_risk_users:
  - user_id: string
    risk_score: 1-100
    signals:
      - type: string
        value: string
        weight: number
    recommended_intervention:
      playbook: string
      timing: immediate | scheduled
      owner: string
    predicted_impact: percentage
escalations:
  - user_id: string
    reason: string
    assigned_to: string
```
