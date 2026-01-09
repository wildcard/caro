# Caro: Analytics & Metrics Framework

**Version**: 1.0
**Last Updated**: 2026-01-08
**Status**: Complete

---

## Purpose

This document defines the comprehensive analytics and metrics framework for Caro, establishing what we measure, why we measure it, and how we use data to drive product decisions while respecting user privacy.

**Audience**: Product Manager, Engineering Team, Community Manager, Data Analysts

---

## Guiding Principles

### 1. Privacy First
- ❌ **Never collect**: User queries, generated commands, file paths, environment variables
- ✅ **Always anonymize**: Strip all PII before storage
- ✅ **Opt-in only**: No tracking without explicit user consent
- ✅ **User control**: Easy opt-out, data export, data deletion

### 2. Actionable Over Comprehensive
- Measure metrics that inform product decisions
- Avoid vanity metrics that don't drive action
- Focus on user outcomes, not just activity

### 3. Transparent
- Publish what we track in privacy policy
- Show users their own data
- Explain how data improves the product

---

## Metrics Hierarchy

```
North Star Metric: Weekly Active Users with High Satisfaction (4+ stars)
    ↓
Primary Metrics: Growth, Engagement, Quality, Satisfaction
    ↓
Secondary Metrics: Technical performance, community health
    ↓
Supporting Metrics: Detailed breakdowns and cohorts
```

---

## North Star Metric

### Definition
**Weekly Active Users (WAU) with High Satisfaction (4+/5 stars)**

**Why This Metric?**
- Combines growth (WAU) with quality (satisfaction)
- Aligns with mission: help users be productive AND happy
- Prevents optimizing for growth at expense of quality
- Single number to rally team around

**Target**:
| Timeframe | Target WAU with 4+ Satisfaction |
|-----------|----------------------------------|
| Launch (Jan 2026) | 500 users |
| Q1 2026 End | 2,000 users |
| Q2 2026 End | 10,000 users |
| Q3 2026 End | 30,000 users |
| Q4 2026 End | 80,000 users |
| 2027 End | 200,000 users |

---

## Primary Metrics

### 1. Growth Metrics

#### A. User Acquisition
**Total Registered Users**
- Definition: Cumulative count of users who installed Caro
- Source: GitHub release downloads, crates.io install count
- Target Growth: 10% week-over-week for first 6 months

**Weekly New Users**
- Definition: New users who installed in the past 7 days
- Segmentation: By platform (macOS, Linux), by source (HN, GitHub, Reddit, organic)
- Target: 200/week (Month 1) → 5,000/week (Month 12)

#### B. User Activation
**Time to First Successful Command**
- Definition: Time from install to first successful command execution
- Target: <60 seconds for 80% of users
- Measurement: Log timestamp of install, timestamp of first success
- Why It Matters: Fast activation = higher retention

**Activation Rate (7-Day)**
- Definition: % of new users who execute ≥3 successful commands within 7 days
- Target: 60%+ activation rate
- Segmentation: By persona (novice vs expert), by platform

#### C. User Retention
**Day 1 Retention**
- Definition: % of users who return and use Caro the day after install
- Target: 40%+
- Why It Matters: Early retention predicts long-term retention

**Week 1 Retention**
- Definition: % of users still active 7 days after install
- Target: 30%+
- Cohort Analysis: Track retention by install week

**Week 4 Retention**
- Definition: % of users still active 28 days after install
- Target: 20%+
- Gold Standard: Users at week 4 are likely long-term users

**Monthly Active Users (MAU)**
- Definition: Unique users who executed ≥1 command in past 30 days
- Target: 50% of total registered users
- Trend: Should grow steadily, not flat or declining

---

### 2. Engagement Metrics

#### A. Command Generation Volume
**Commands Generated per User per Week**
- Definition: Average # of commands generated per active user per week
- Target: 10+ commands/user/week (power users: 50+)
- Segmentation: By user cohort (new vs returning), by persona

**Commands Executed per User per Week**
- Definition: Average # of commands actually executed (not just generated)
- Target: 7+ executions/user/week
- Why It Matters: Generation without execution = product not useful

**Execution Rate**
- Definition: % of generated commands that are executed
- Target: 70%+ execution rate
- Formula: (Commands Executed / Commands Generated) × 100
- Low execution rate → Users don't trust generated commands

#### B. Feature Adoption
**Backend Usage Distribution**
- Static Matcher: % of queries handled by static matcher
- Embedded Backend: % of queries handled by embedded LLM
- MLX Backend (v1.2+): % of queries handled by MLX
- Target: 60% static, 35% embedded, 5% MLX (cold start optimization)

**Safety Mode Distribution**
- Strict: % of users using strict safety mode
- Normal (default): % of users using normal mode
- Permissive: % of users using permissive mode
- Target: 10% strict, 80% normal, 10% permissive

**Command History Usage (v1.2+)**
- % of users who enable command history
- Target: 40%+ enable within first month
- Avg commands searched in history per week

**Plugin Adoption (v1.3+)**
- % of users with ≥1 plugin installed
- Target: 30%+ adopt plugins within 3 months of v1.3 release
- Most popular plugins (by install count)

#### C. Depth of Engagement
**Query Complexity Distribution**
- Simple (1-3 words): % of queries
- Medium (4-7 words): % of queries
- Complex (8+ words): % of queries
- Trend: Complexity increasing → Users trust Caro more

**Multi-Step Query Usage**
- % of users who refine queries (use agent loop refinement)
- Avg refinements per complex query
- Target: 20%+ of complex queries use refinement

---

### 3. Quality Metrics

#### A. Command Success Rate
**First-Attempt Success Rate**
- Definition: % of generated commands that work on first try (no validation errors, no execution errors)
- Target: 85%+ (already achieved 86.2% in beta)
- Segmentation: By backend, by query complexity, by platform

**Post-Repair Success Rate**
- Definition: % of commands that work after agent loop repair
- Target: 95%+ (repair should fix most validation errors)
- Improvement: (Post-Repair - First-Attempt) = agent loop value

#### B. Safety Validation Accuracy
**True Positive Rate (Dangerous Correctly Blocked)**
- Definition: % of actually dangerous commands that are blocked
- Target: 100% for P0 patterns (critical safety)
- Measurement: Manual audit of blocked commands

**False Positive Rate (Safe Incorrectly Blocked)**
- Definition: % of safe commands that are incorrectly blocked
- Target: <5% false positive rate
- User Feedback: "Was this safety check helpful?" survey

**Override Rate**
- Definition: % of safety warnings that users override
- Target: <10% override rate (high override = warnings not trusted)
- Segmentation: By severity level (critical vs high vs medium)

#### C. Performance Quality
**Latency P50/P95/P99**
- P50 (median): Target <100ms for static matcher
- P95: Target <500ms for embedded backend
- P99: Target <1000ms for embedded backend
- Measurement: Log generation start → end time

**Model Inference Time**
- SmolLM: Target <300ms on consumer hardware
- Qwen: Target <600ms on consumer hardware
- MLX (v1.2+): Target <30ms on Apple Silicon

**Error Rate**
- Definition: % of queries that result in errors (generation fails, crash, etc.)
- Target: <2% error rate
- Breakdown: By error type (validation, generation, platform, etc.)

---

### 4. Satisfaction Metrics

#### A. Net Promoter Score (NPS)
**Overall NPS**
- Survey: "How likely are you to recommend Caro? (0-10)"
- Target: 40+ (6 months), 60+ (12 months)
- Calculation: % Promoters (9-10) - % Detractors (0-6)

**NPS by Cohort**
- New Users (Week 1): Measures initial impression
- Power Users (Month 3+): Measures long-term satisfaction
- By Persona: DevOps, Data Scientist, SysAdmin, etc.

#### B. User Satisfaction (CSAT)
**In-App Satisfaction**
- Survey: "How's Caro working for you? (1-5 stars)"
- Trigger: After 10 successful commands
- Target: 4.2+ average (6 months), 4.5+ (12 months)

**Feature-Specific Satisfaction**
- Safety Validation: "Was this safety check helpful?"
- Agent Loop Repair: "Did the repaired command work?"
- MLX Backend (v1.2+): "Is performance better with MLX?"

#### C. Support Satisfaction
**Response Time Satisfaction**
- Survey: "Were you satisfied with our response time? (Yes/No)"
- Target: 80%+ satisfaction
- Measurement: After support interaction (GitHub, Discord, email)

**Resolution Satisfaction**
- Survey: "Did we solve your problem? (Yes/No/Partially)"
- Target: 70%+ "Yes", <10% "No"

---

## Secondary Metrics

### 1. Technical Performance

**System Resource Usage**
- Memory: Avg/Max memory usage during inference
- CPU: Avg CPU utilization during inference
- Disk: Model storage size, history database size

**Platform Distribution**
- macOS (Intel): % of users
- macOS (ARM): % of users
- Linux (x86): % of users
- Linux (ARM): % of users

**Binary Download Size**
- Target: <10MB compressed for static binaries
- Trend: Monitor growth as features added

---

### 2. Community Health

**GitHub Metrics**
- Stars: Total and growth rate
- Forks: Total and fork rate
- Watchers: Total
- Contributors: Unique contributors per month
- Pull Requests: Opened, merged, time to merge
- Issues: Opened, closed, time to close, P0/P1/P2 distribution

**Discord Metrics**
- Total Members: Growth rate
- Active Members: % active in past 7 days
- Messages per Day: Engagement level
- Response Time: Median time to first response in #help

**Content Engagement**
- Blog Views: Page views per post
- Demo Video Views: YouTube/social video views
- Social Media Engagement: Likes, shares, comments on launch posts

---

### 3. Business Metrics (Freemium Model)

**Revenue Metrics (v1.2+)**
- MRR (Monthly Recurring Revenue): Target $5K (Q2 2026) → $50K (Q4 2026)
- ARPU (Average Revenue Per User): Target $3-5/month
- Conversion Rate (Free → Pro): Target 5-10%

**Tier Distribution**
- Free Tier: % of users
- Pro Tier ($5/mo): % of users
- Team Tier ($10/user/mo): % of users
- Enterprise ($20/user/mo): % of users

**Churn Rate**
- Monthly Churn: % of paying users who cancel per month
- Target: <5% monthly churn for Pro, <3% for Enterprise

---

## Data Collection Strategy

### What We Track (Opt-In Only)

```rust
pub struct TelemetryEvent {
    // Meta
    pub event_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub session_id: Uuid, // Random per session, not persistent

    // Event
    pub event_type: EventType,
    // command_generated, command_executed, command_failed,
    // validation_blocked, agent_loop_repair, safety_override

    // Context (no PII)
    pub backend: String, // static, embedded, mlx
    pub platform: String, // macos-arm64, linux-x86
    pub caro_version: String, // 1.1.0-beta

    // Performance
    pub latency_ms: u64,

    // Outcome
    pub success: bool,
    pub error_category: Option<String>,

    // Quality (if applicable)
    pub confidence_score: Option<f32>,
    pub validation_severity: Option<Severity>,

    // What we DON'T track:
    // ❌ query: String - NEVER tracked (privacy)
    // ❌ command: String - NEVER tracked (privacy)
    // ❌ user_id: String - NEVER tracked (anonymity)
    // ❌ file_paths: Vec<String> - NEVER tracked (privacy)
    // ❌ env_vars: HashMap<String, String> - NEVER tracked (security)
}
```

### What We DON'T Track (Privacy)

**Never Collected** (even with opt-in):
- ❌ User queries (e.g., "list files larger than 100MB")
- ❌ Generated commands (e.g., "find . -size +100M")
- ❌ User identifiers (email, username, IP address)
- ❌ File paths (e.g., /Users/alice/secrets.txt)
- ❌ Environment variables (e.g., API_KEY=secret)
- ❌ Command output
- ❌ Directory structure

**Why?**
- User queries/commands may contain sensitive information
- File paths may reveal private project names or structure
- Environment variables often contain secrets
- We don't need this data to improve the product

---

## Data Pipeline Architecture

### Collection → Storage → Analysis → Action

```
1. Collection (Client-Side)
   ↓
   User opts in via `caro config set telemetry enabled`
   ↓
   Events generated during usage
   ↓
   Events batched locally (every 100 events or 1 hour)

2. Transmission (Encrypted)
   ↓
   HTTPS POST to telemetry.caro-cli.dev
   ↓
   TLS 1.3, certificate pinning

3. Storage (Server-Side)
   ↓
   PostgreSQL (anonymized events only)
   ↓
   90-day rolling retention window

4. Analysis (Data Warehouse)
   ↓
   Daily aggregation jobs
   ↓
   Metrics dashboard (Grafana / Metabase)

5. Action (Product Team)
   ↓
   Weekly metrics review
   ↓
   Feature prioritization, bug triage, roadmap updates
```

---

## Dashboards

### 1. Growth Dashboard (Weekly Review)

**Panels**:
- WAU with High Satisfaction (North Star Metric)
- New Users This Week (by source, by platform)
- Retention Curves (Day 1, Week 1, Week 4)
- Activation Rate (7-day)

**Alerts**:
- WAU declining week-over-week
- Retention curves deteriorating
- Activation rate <50%

---

### 2. Engagement Dashboard (Weekly Review)

**Panels**:
- Commands Generated per User
- Execution Rate (commands executed / generated)
- Backend Usage Distribution (static, embedded, MLX)
- Query Complexity Distribution (simple, medium, complex)

**Alerts**:
- Execution rate <60% (users don't trust generated commands)
- Static matcher usage <50% (not catching common queries)
- Engagement declining (commands/user dropping)

---

### 3. Quality Dashboard (Daily Review)

**Panels**:
- Success Rate (first-attempt, post-repair)
- Latency (P50, P95, P99)
- Error Rate (by error type)
- Safety Validation (true positives, false positives, override rate)

**Alerts**:
- Success rate <80%
- P95 latency >1000ms
- Error rate >5%
- False positive rate >10%

---

### 4. Satisfaction Dashboard (Monthly Review)

**Panels**:
- NPS Trend (monthly)
- CSAT Trend (monthly)
- Survey Response Rate
- Support Satisfaction

**Alerts**:
- NPS dropping (month-over-month)
- CSAT <4.0
- Support satisfaction <70%

---

## Analysis & Reporting

### Weekly Metrics Email

**Recipients**: Product Manager, Engineering Lead, Community Manager

**Content**:
```
Subject: Caro Weekly Metrics - Week of Jan 8, 2026

## North Star Metric
WAU with High Satisfaction: 1,247 (+15% WoW) ✅

## Growth
- New Users: 342 (+23% WoW)
- Week 1 Retention: 34% (+2pp)
- Week 4 Retention: 21% (+1pp)

## Engagement
- Commands/User/Week: 12.3 (+0.8)
- Execution Rate: 73% (-2pp) ⚠️
- Backend: 58% static, 37% embedded, 5% MLX

## Quality
- Success Rate: 87% (+1pp)
- P95 Latency: 420ms (-30ms)
- Error Rate: 1.8% (stable)

## Satisfaction
- NPS: 48 (+3)
- CSAT: 4.3 (stable)

## Action Items
1. Investigate execution rate drop (73% → goal 75%+)
2. Continue latency improvements (420ms → target 400ms)
3. Celebrate NPS improvement (+3 points!)
```

---

### Monthly Business Review

**Meeting**: First Monday of month, 60 minutes
**Attendees**: Full team + stakeholders

**Agenda**:
1. **Headline Metrics** (10 min): North Star, growth, retention, satisfaction
2. **Deep Dive** (20 min): One topic per month (e.g., "Why is retention improving?")
3. **User Feedback Themes** (15 min): Top insights from interviews, surveys
4. **Roadmap Alignment** (10 min): Are we building what metrics say we need?
5. **Action Items** (5 min): Owners, deadlines

---

## Privacy Controls

### User Controls

**View Telemetry Status**:
```bash
caro telemetry status
# Output: Telemetry: DISABLED (opt-in required)
```

**Opt-In**:
```bash
caro config set telemetry enabled
# Output: Telemetry enabled. View privacy policy: caro telemetry privacy
```

**Opt-Out**:
```bash
caro config set telemetry disabled
# Output: Telemetry disabled. No data will be sent.
```

**View What's Tracked**:
```bash
caro telemetry show
# Output: Event types, fields collected, retention policy
```

**Export My Data**:
```bash
caro telemetry export
# Output: Exports all telemetry events for this session_id to JSON file
```

**Delete My Data**:
```bash
caro telemetry delete
# Output: Requests deletion of all data for this session_id (90 days)
```

---

## Compliance

### GDPR Compliance
- ✅ **Lawful Basis**: Consent (opt-in)
- ✅ **Right to Access**: `caro telemetry export`
- ✅ **Right to Rectification**: N/A (anonymized data)
- ✅ **Right to Erasure**: `caro telemetry delete`
- ✅ **Right to Portability**: JSON export
- ✅ **Data Minimization**: Only essential data collected

### CCPA Compliance
- ✅ **Notice**: Privacy policy explains data practices
- ✅ **Opt-Out**: `caro config set telemetry disabled`
- ✅ **No Sale**: We never sell user data
- ✅ **Deletion**: `caro telemetry delete`

---

## Conclusion

**Metrics guide us, but don't define us.**

**Key Principles**:
1. **Privacy First**: Opt-in only, minimal data, user control
2. **Actionable Metrics**: Measure what matters, not vanity metrics
3. **Transparent**: Users know what we track and why
4. **Outcome-Focused**: Success = users productive AND happy
5. **Continuous Improvement**: Weekly review, monthly deep dives

**North Star Metric**: Weekly Active Users with High Satisfaction (4+ stars)

**By respecting privacy while collecting essential metrics, we can build a better product for users.**

---

**Last Updated**: 2026-01-08
**Next Review**: 2026-02-01 (post-launch analysis)
**Version**: 1.0
