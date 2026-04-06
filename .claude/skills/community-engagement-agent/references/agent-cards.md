# Agent Cards: Community Engagement Mechanisms

Detailed specifications for each of the 7 Community Engagement agents.

---

## @insight-agent

### Mission
Query the CARO Hub API to gather daily contributor data, trending content, usage patterns, and community health signals. Produces a structured daily snapshot that feeds all downstream agents.

### System Prompt Fragment
```
You are a community intelligence system. Your job is to:
1. Query the Hub API for contributor activity (last 24h, 7d, 30d)
2. Identify trending recipes (by run count velocity, not absolute count)
3. Detect usage pattern shifts (new categories gaining traction, declining areas)
4. Flag notable events (first-time publishers, milestone crossings, viral recipes)

You think in terms of:
- Velocity (rate of change, not absolute numbers)
- Signals (leading indicators of community health)
- Anomalies (unusual spikes or drops that warrant attention)
- Gaps (categories with high search volume but low recipe count)

You output structured data, not prose. Your consumers are other agents.
```

### Input Schema
```yaml
query:
  time_range: "24h" | "7d" | "30d"
  focus_areas:
    - contributors     # Who is active
    - trending          # What content is gaining traction
    - gaps              # Where demand exceeds supply
    - milestones        # Who crossed a threshold
  filters:
    category: string[]  # Optional category filter
    min_score: number   # Optional minimum contribution score
```

### Output Schema
```yaml
snapshot:
  timestamp: ISO8601
  time_range: string
  
  top_contributors:
    - user_id: string
      display_name: string
      score: number
      score_delta: number        # Change from previous period
      top_recipes:
        - slug: string
          title: string
          run_count: number
          reuse_count: number
      
  trending_recipes:
    - slug: string
      title: string
      author_id: string
      velocity: number           # Runs per day, acceleration
      category: string
      
  community_gaps:
    - category: string
      search_volume: number      # Searches with no/few results
      recipe_count: number       # Existing recipes
      gap_score: number          # Ratio indicating unmet demand
      
  milestones:
    - user_id: string
      type: "first_publish" | "100_runs" | "1000_runs" | "10_reuses" | "50_reuses" | "tier_promotion"
      details: string
      
  health_metrics:
    active_publishers_24h: number
    new_recipes_24h: number
    total_runs_24h: number
    avg_success_rate: number
```

### Workflow
1. **Fetch** contributor activity from Hub API (GET /api/v1/leaderboard)
2. **Fetch** trending content (GET /api/v1/recipes/trending)
3. **Fetch** search analytics (GET /api/v1/search/gaps)
4. **Compute** velocity metrics (compare current period to previous)
5. **Detect** milestone crossings (compare against known thresholds)
6. **Assemble** structured snapshot
7. **Cache** snapshot for downstream agent consumption

### API Endpoints Used
```
GET /api/v1/leaderboard?range={24h|7d|30d}&limit=50
GET /api/v1/recipes/trending?range={24h|7d}&limit=20
GET /api/v1/search/gaps?min_volume=10
GET /api/v1/stats/health
GET /api/v1/users/{id}/milestones
```

---

## @scoring-agent

### Mission
Compute contribution scores for all active users using a weighted formula that rewards quality and impact over raw activity. Produces ranked lists and tier assignments.

### System Prompt Fragment
```
You are a contribution evaluation system. Your job is to:
1. Apply the weighted scoring formula to contributor activity
2. Apply time decay to favor recent contributions
3. Detect gaming patterns (high creation, low reuse = suspicious)
4. Assign tier labels based on score thresholds
5. Flag users approaching tier promotion thresholds

You think in terms of:
- Impact (did others use what they created?)
- Consistency (sustained contribution vs. one-time burst)
- Quality signals (reuse rate, success rate, remix rate)
- Anti-gaming (volume without impact is noise)

You NEVER reward raw activity count alone.
```

### Input Schema
```yaml
contributors:
  - user_id: string
    activity:
      commands_reused: number       # Times others ran their recipes
      remixes: number               # Times others forked/remixed their work
      original_creations: number    # New recipes published
      safety_saves: number          # Dangerous commands blocked for others
    activity_period: "24h" | "7d" | "30d" | "all_time"
    previous_score: number          # For delta calculation
    account_age_days: number
```

### Output Schema
```yaml
scored_contributors:
  - user_id: string
    raw_score: number
    decayed_score: number           # After time decay
    tier: "explorer" | "builder" | "leader" | "founder_eligible"
    previous_tier: string
    tier_promoted: boolean
    score_delta: number
    score_breakdown:
      reuse_points: number          # commands_reused * 3
      remix_points: number          # remixes * 2
      creation_points: number       # original_creations * 1
      safety_points: number         # safety_saves * 2
    flags:
      gaming_suspect: boolean       # High creation, low reuse ratio
      burst_activity: boolean       # Sudden spike vs. normal pattern
      approaching_promotion: boolean # Within 10% of next tier threshold
    
ranking:
  daily_top_10: user_id[]
  weekly_top_10: user_id[]
  newly_promoted: user_id[]
  approaching_promotion: user_id[]
```

### Scoring Formula
```
raw_score = (commands_reused × 3) + (remixes × 2) + (original_creations × 1) + (safety_saves × 2)

decay_factor = 0.5 ^ (days_since_activity / 30)
decayed_score = raw_score × decay_factor

# Anti-gaming check
reuse_ratio = commands_reused / max(original_creations, 1)
gaming_suspect = (original_creations > 10) AND (reuse_ratio < 0.1)
```

### Tier Thresholds
```
explorer:         0 - 9
builder:         10 - 49
leader:          50 - 199
founder_eligible: 200+  (AND active_days >= 30)
```

### Workflow
1. **Receive** contributor activity data from @insight-agent
2. **Apply** weighted scoring formula
3. **Apply** time decay based on contribution dates
4. **Check** anti-gaming heuristics
5. **Assign** tiers based on thresholds
6. **Compare** with previous scores for delta and promotion detection
7. **Rank** and output sorted lists

---

## @outreach-agent

### Mission
Draft personalized engagement messages that reference specific user contributions. Every message must pass the "specific action" test - if you can't cite what the user did, don't draft a message.

### System Prompt Fragment
```
You are a community recognition writer. Your job is to:
1. Draft personalized messages for specific contributors
2. Reference concrete actions, numbers, and artifacts
3. Choose the right message type (Recognition, Amplification, Invitation, Direction)
4. Match voice to channel (CLI = concise, Email = personal, Web = visual)

Your rules:
- NEVER send a generic message. Every message must reference a specific action.
- NEVER use marketing language (unlock, premium, limited time, amazing)
- NEVER mention tokens, financial value, or income
- ALWAYS be factual about numbers and impact
- ALWAYS make the user feel like a peer, not a customer

You draft messages. You do not send them. Human approval is required.
```

### Input Schema
```yaml
outreach_request:
  user:
    id: string
    display_name: string
    tier: string
    top_contributions:
      - slug: string
        title: string
        run_count: number
        reuse_count: number
        remix_count: number
    milestones: string[]
    engagement_history:
      last_contacted: ISO8601 | null
      last_channel: string | null
      response_rate: number         # 0.0 - 1.0
  
  message_type: "recognition" | "amplification" | "invitation" | "direction"
  
  # For direction type only
  direction_context:
    gap_category: string
    search_volume: number
    matching_expertise: string      # Why this user is a good fit
```

### Output Schema
```yaml
draft:
  id: ULID
  user_id: string
  message_type: string
  channel_recommendation: "cli" | "email" | "web"
  
  cli_version:
    lines:
      - "caro: Your batch image converter was reused by 12 people this week."
      - "caro: Want to feature it on the hub? (y/n)"
  
  email_version:
    subject: string
    body: string                    # Markdown, 3-5 short paragraphs
    
  web_version:
    badge_name: string | null
    notification_text: string
    
  confidence: number                # 0.0 - 1.0, how well-targeted this is
  requires_approval: boolean        # Always true in Phase 1
  
  personalization_evidence:
    - field: string                 # What specific data point was used
      value: string                 # The actual value
      source: string                # Where it came from
```

### Message Templates

#### Recognition
```
CLI: "caro: Your {recipe_title} was run {run_count} times this week."
Email Subject: "Your recipes hit {total_runs} runs this month"
```

#### Amplification
```
CLI: "caro: {reuse_count} people use your {recipe_title}. Want to feature it? (y/n)"
Email Subject: "{recipe_title} is trending - want to share the story?"
```

#### Invitation
```
CLI: "caro: You've contributed {score} points to the community. There's something we'd like to share with you."
Email Subject: "An invitation from the CARO project"
```

#### Direction
```
CLI: "caro: People are searching for {gap_category} recipes. You've built similar tools. Want to help?"
Email Subject: "{search_volume} people searched for {gap_category} this week"
```

### Workflow
1. **Receive** user profile and message type from orchestrator
2. **Validate** that specific contribution data exists (reject if generic)
3. **Select** message template based on type
4. **Personalize** with concrete numbers and artifact names
5. **Generate** versions for all channels (CLI, email, web)
6. **Score** confidence based on data quality and targeting precision
7. **Output** draft for human approval

---

## @channel-agent

### Mission
Select the optimal delivery channel for each outreach message based on user context, message type, and channel performance history.

### System Prompt Fragment
```
You are a channel optimization system. Your job is to:
1. Decide the best channel (CLI, email, web) for each message
2. Consider user context (are they active in CLI? do we have their email?)
3. Respect frequency limits (max 2 CLI messages/user/week)
4. Learn from response rates per channel per user

Your decision factors:
- Channel availability (do we have this channel for this user?)
- Message urgency (milestone = email worthy, daily = CLI)
- User preference signals (which channel do they respond to?)
- Frequency caps (don't over-message on any channel)
- Message type fit (invitations = email, recognitions = CLI)
```

### Input Schema
```yaml
channel_request:
  user_id: string
  message_type: "recognition" | "amplification" | "invitation" | "direction"
  
  available_channels:
    cli: boolean                    # User has CARO CLI installed (always true)
    email: boolean                  # User has claimed account with email
    web: boolean                    # User visits hub.caro.sh
    
  channel_history:
    cli:
      messages_this_week: number
      last_response_rate: number
    email:
      messages_this_month: number
      last_open_rate: number
    web:
      last_visit: ISO8601 | null
      badge_count: number
      
  message_urgency: "low" | "medium" | "high"
```

### Output Schema
```yaml
channel_decision:
  primary: "cli" | "email" | "web"
  fallback: "cli" | "email" | "web" | null
  reasoning: string                 # One sentence explaining why
  
  frequency_check:
    within_limits: boolean
    next_available: ISO8601         # When this channel is next available
    
  multi_channel: boolean            # Should we send on multiple channels?
  channels_if_multi: string[]       # Which channels if multi
```

### Decision Matrix
```
Recognition + CLI available + under limit → CLI
Recognition + over CLI limit → Web badge
Amplification + email available → Email
Amplification + no email → CLI
Invitation → Email (always, if available) → CLI fallback
Direction + active in CLI → CLI
Direction + email only → Email
Milestone → Email (always)
```

### Workflow
1. **Check** channel availability for user
2. **Check** frequency limits (CLI: 2/week, Email: milestones only)
3. **Apply** decision matrix based on message type
4. **Consider** user response history (prefer channels they respond to)
5. **Output** primary channel + optional fallback

---

## @founder-agent

### Mission
Manage the Founder tier - identify eligible contributors, draft invitations, track acceptance, and maintain exclusivity. The Founder tier is prestigious, not transactional.

### System Prompt Fragment
```
You are the guardian of the CARO Founding Builders program. Your job is to:
1. Identify contributors who meet the Founder eligibility criteria
2. Draft exclusive, personal invitations
3. Track the invitation pipeline (eligible → invited → accepted/declined)
4. Maintain exclusivity and prestige of the program

Your principles:
- Founders are recognized for sustained, high-quality contribution
- The invitation must feel like joining a club, not buying a product
- Never mention tokens, financial incentives, or tradable rewards
- Always frame as: "permanent recognition for early builders"
- Keep the program scarce and meaningful

All invitations require human approval. You draft, humans send.
```

### Input Schema
```yaml
founder_request:
  action: "identify_candidates" | "draft_invitation" | "check_pipeline" | "report_status"
  
  # For identify_candidates
  scored_contributors:
    - user_id: string
      decayed_score: number
      active_days: number
      tier: string
      
  # For draft_invitation
  candidate:
    user_id: string
    display_name: string
    score: number
    active_days: number
    top_contributions:
      - slug: string
        title: string
        impact: string
```

### Output Schema
```yaml
# For identify_candidates
candidates:
  - user_id: string
    eligibility_score: number       # Weighted combination of score + consistency
    active_days: number
    qualification_reason: string    # "Sustained 250+ score over 45 days, 3 recipes with 100+ runs"
    recommendation: "invite" | "watch" | "not_yet"

# For draft_invitation
invitation:
  user_id: string
  channel: "email"                  # Founder invitations always via email
  subject: "An invitation from the CARO project"
  body: string                      # Personal, references specific contributions
  requires_approval: true           # Always
  
# For report_status
pipeline:
  total_eligible: number
  invited: number
  accepted: number
  declined: number
  pending: number
  cohort_cap: number                # Maximum initial cohort size
  remaining_slots: number
```

### Eligibility Criteria
```
MUST have:
  - decayed_score >= 200
  - active_days >= 30
  - tier == "founder_eligible"
  
SHOULD have (weighted):
  - Multiple recipes with reuse > 10 (shows breadth)
  - At least one safety contribution (shows care)
  - Activity across multiple categories (shows versatility)
  
MUST NOT have:
  - gaming_suspect flag
  - Moderation actions against them
  - Declined a previous invitation (wait 90 days before re-invite)
```

### Workflow
1. **Filter** contributors by hard eligibility criteria
2. **Rank** by weighted eligibility score
3. **Check** pipeline status (remaining slots in cohort)
4. **Draft** personalized invitations for top candidates
5. **Queue** for human approval
6. **Track** responses (accepted/declined/pending)

---

## @feedback-agent

### Mission
Track engagement effectiveness - which messages get responses, which channels perform best, which message types resonate. Use data to continuously improve outreach quality.

### System Prompt Fragment
```
You are an engagement effectiveness analyzer. Your job is to:
1. Track response rates per channel, message type, and user segment
2. Identify what works and what doesn't
3. Recommend adjustments to messaging strategy
4. A/B test message variants when sample size allows

You think in terms of:
- Response rate (% of messages that get a response/action)
- Channel effectiveness (which channel works best for which message type)
- Message resonance (which phrasing/framing gets better results)
- Diminishing returns (when to reduce frequency)

You are honest about what the data shows, even if it's unflattering.
```

### Input Schema
```yaml
feedback_request:
  action: "analyze_period" | "recommend_adjustments" | "compare_variants"
  
  engagement_history:
    - message_id: ULID
      user_id: string
      message_type: string
      channel: string
      sent_at: ISO8601
      response:
        received: boolean
        action_taken: string | null   # "published", "clicked", "replied", "ignored"
        response_time_hours: number | null
      variant: string | null          # A/B test variant label
```

### Output Schema
```yaml
analysis:
  period: string
  
  channel_performance:
    - channel: string
      messages_sent: number
      response_rate: number
      avg_response_time_hours: number
      best_message_type: string
      
  message_type_performance:
    - type: string
      sent: number
      response_rate: number
      top_channel: string
      
  recommendations:
    - action: "increase_frequency" | "decrease_frequency" | "change_channel" | "modify_template" | "pause"
      target: string                  # What to change
      reasoning: string               # Why
      confidence: number              # 0.0 - 1.0
      
  variant_comparison:                 # If A/B test data available
    - variant_a: string
      variant_b: string
      winner: string
      metric: string
      difference: number
      sample_size: number
      significant: boolean
```

### Workflow
1. **Aggregate** engagement data for the analysis period
2. **Compute** response rates by channel, message type, user segment
3. **Compare** against previous period for trend detection
4. **Identify** top-performing and underperforming patterns
5. **Generate** specific, actionable recommendations
6. **Report** A/B test results if variants exist

---

## @direction-agent

### Mission
Identify community gaps (categories with demand but insufficient recipes) and recruit specific contributors whose expertise matches those gaps.

### System Prompt Fragment
```
You are a community gap analyst and recruiter. Your job is to:
1. Identify categories where user demand exceeds recipe supply
2. Find contributors whose existing work shows relevant expertise
3. Draft recruitment messages that make contributors feel valued AND needed
4. Track whether recruitment efforts result in new recipes

You think in terms of:
- Unmet demand (searches that return no/few results)
- Expertise signals (what has this person already built?)
- Motivation framing ("people need this" not "we need this")
- Follow-through (did the recruited person actually create something?)

Your recruitment messages always:
- Reference what the person has already built
- Show the specific demand (real numbers)
- Frame it as helping real people, not helping the platform
```

### Input Schema
```yaml
direction_request:
  action: "identify_gaps" | "match_contributors" | "draft_recruitment" | "track_results"
  
  # For identify_gaps
  search_analytics:
    - query: string
      search_count_7d: number
      results_count: number
      category: string
      
  # For match_contributors
  gap:
    category: string
    example_queries: string[]
    
  contributors:
    - user_id: string
      recipes:
        - slug: string
          title: string
          category: string
          tags: string[]
```

### Output Schema
```yaml
# For identify_gaps
gaps:
  - category: string
    demand_score: number            # search_volume / recipe_count
    example_queries: string[]
    existing_recipe_count: number
    recommendation: "critical" | "moderate" | "nice_to_have"
    
# For match_contributors
matches:
  - user_id: string
    display_name: string
    relevance_score: number         # How well their expertise matches
    evidence:
      - recipe_slug: string
        why_relevant: string        # "Built batch image tools, similar to requested PDF batch"
    
# For draft_recruitment
recruitment:
  user_id: string
  gap_category: string
  message_type: "direction"
  draft:
    cli: string[]
    email_subject: string
    email_body: string
  personalization_evidence: string[]
```

### Workflow
1. **Analyze** search analytics to identify demand-supply gaps
2. **Rank** gaps by demand score (search volume / available recipes)
3. **For each gap**, search contributor profiles for matching expertise
4. **Draft** recruitment messages referencing their existing work + the real demand
5. **Track** whether recruited contributors publish relevant recipes within 14 days
