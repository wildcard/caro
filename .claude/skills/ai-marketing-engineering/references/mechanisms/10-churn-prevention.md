# Mechanism 10: Active Churn Prevention

## Overview

Detect customer frustration in real-time (support, chat, product signals) and intervene before churn.

## The Problem

Traditional churn management:
1. React after customer cancels
2. Exit surveys tell you what happened, not what to do
3. Support tickets treated as isolated incidents
4. No connection between sentiment and retention

**Result**: Preventable churn, reactive save attempts, lost revenue.

## The Solution

Build an **active churn prevention system**:
1. Monitor multiple signal sources (support, product, payment)
2. Detect frustration in real-time
3. Score churn risk continuously
4. Trigger proactive interventions
5. Escalate high-value accounts to humans

## Signal Sources

### 1. Support Tickets
```yaml
signals:
  sentiment:
    - negative: < -0.3 (scale -1 to 1)
    - very_negative: < -0.5
  keywords:
    high_risk:
      - "cancel", "cancellation", "refund"
      - "frustrated", "angry", "unacceptable"
      - "switching to", "alternative", "competitor"
      - "waste of money", "overpriced"
    medium_risk:
      - "disappointed", "unhappy", "issue"
      - "not working", "broken", "bug"
  urgency:
    - 5: "Immediate escalation"
    - 4: "Same-day response"
    - 3: "Within 24 hours"
```

### 2. Product Usage
```yaml
signals:
  activity_decline:
    - trigger: "> 50% drop in weekly usage"
    - trigger: "No login for 14+ days"
    - trigger: "Feature usage dropped to zero"
  engagement_pattern:
    - trigger: "Stopped using core feature"
    - trigger: "Only using export/download (data extraction)"
    - trigger: "Admin hasn't logged in for 30 days"
```

### 3. Payment Signals
```yaml
signals:
  payment_issues:
    - trigger: "Failed payment"
    - trigger: "2+ dunning attempts"
    - trigger: "Downgrade request"
  billing_behavior:
    - trigger: "Switched to monthly from annual"
    - trigger: "Removed payment method"
    - trigger: "Requested invoice change"
```

### 4. Communication Engagement
```yaml
signals:
  email_engagement:
    - trigger: "No email opens for 30+ days"
    - trigger: "Unsubscribed from all lists"
  nps_csat:
    - trigger: "NPS < 6 (detractor)"
    - trigger: "CSAT 1-2 stars"
    - trigger: "Negative survey comment"
```

## Risk Scoring Model

```yaml
risk_score:
  range: 1-100
  thresholds:
    critical: 80+   # Immediate intervention
    high: 60-79     # Priority attention
    medium: 40-59   # Monitor closely
    low: 0-39       # Normal status

  weights:
    support_sentiment: 25
    usage_decline: 30
    payment_issues: 20
    engagement_drop: 15
    nps_score: 10
```

### Score Calculation Example
```python
def calculate_risk_score(user):
    score = 0

    # Support signals (max 25 points)
    if latest_ticket_sentiment < -0.5:
        score += 25
    elif latest_ticket_sentiment < -0.3:
        score += 15

    # Usage signals (max 30 points)
    if usage_decline > 0.5:
        score += 30
    elif usage_decline > 0.25:
        score += 15

    # Payment signals (max 20 points)
    if failed_payments > 0:
        score += 20
    elif downgrade_requested:
        score += 15

    # Engagement signals (max 15 points)
    if days_since_email_open > 30:
        score += 15
    elif days_since_email_open > 14:
        score += 8

    # NPS signals (max 10 points)
    if nps_score <= 6:
        score += 10

    return min(score, 100)
```

## Intervention Playbooks

### Tier 1: Automated Responses

```yaml
playbook_automated:
  trigger: "risk_score 40-59"
  interventions:
    - type: "email"
      template: "check_in_helpful"
      timing: "immediate"
      content: |
        Hi {{name}},

        I noticed you might be running into some challenges.
        I wanted to personally check in - is there anything
        we can help with?

        Here are some resources that might help:
        - [Relevant help article]
        - [Video tutorial]

        Just reply to this email if you'd like to chat.

    - type: "in_app"
      template: "support_prompt"
      timing: "next_login"
      content: "Need help? Chat with us now."
```

### Tier 2: Proactive Outreach

```yaml
playbook_proactive:
  trigger: "risk_score 60-79"
  interventions:
    - type: "email"
      template: "proactive_call"
      from: "customer_success_manager"
      content: |
        Hi {{name}},

        I'm reaching out because I want to make sure you're
        getting value from {{product}}. I noticed
        {{specific_observation}} and wanted to help.

        Would you have 15 minutes for a quick call this week?

        [Calendar Link]

    - type: "phone"
      timing: "if no email response in 48h"
      script: "proactive_save_call"
```

### Tier 3: Critical Intervention

```yaml
playbook_critical:
  trigger: "risk_score 80+"
  interventions:
    - type: "immediate_escalation"
      owner: "customer_success_lead"
      sla: "4 hours"

    - type: "executive_outreach"
      trigger: "high_value_account"
      from: "vp_customer_success"

    - type: "compensation_offer"
      authority: "manager_approval"
      options:
        - "Free month"
        - "Feature unlock"
        - "Training session"
        - "Discount (up to 30%)"
```

## Response Templates

### Empathetic Check-In
```
Subject: Everything okay, {{name}}?

Hi {{name}},

I noticed {{specific_signal}} and wanted to personally reach out.

At {{company}}, we're committed to your success, and if something
isn't working for you, I want to fix it.

Would you be open to a quick 10-minute call? I'm confident we can
help with whatever's going on.

[Book a call] or just reply to this email.

{{sender_name}}
Customer Success
```

### Support Follow-Up
```
Subject: Following up on your recent ticket

Hi {{name}},

I saw you recently reached out about {{issue}}. I wanted to
personally make sure that was resolved to your satisfaction.

If there's anything else bothering you, I'm here to help.
Sometimes small frustrations add up, and I'd rather address
them now than have them affect your experience.

How can I help?

{{sender_name}}
```

### Compensation Offer
```
Subject: We want to make this right

Hi {{name}},

I understand you've been frustrated with {{issue}}, and I
take that seriously.

I'd like to offer you {{compensation}} as a gesture of our
commitment to your success. No strings attached.

More importantly, I want to make sure the underlying issue
is fully resolved. Can we schedule a call to discuss?

{{sender_name}}
```

## Dashboard Metrics

| Metric | Description | Target |
|--------|-------------|--------|
| Churn rate | Monthly cancellations | < 5% |
| Risk detection rate | At-risk identified / actual churners | > 80% |
| Intervention success | Saves / interventions | > 30% |
| Time to intervene | Risk trigger → first contact | < 24 hours |
| Save value | Revenue retained from saves | > $X/month |

## Escalation Matrix

| Risk Score | Account Value | Owner | SLA |
|------------|---------------|-------|-----|
| 80+ | High ($10k+) | VP CS | 4 hours |
| 80+ | Medium | CS Lead | 8 hours |
| 80+ | Low | CS Rep | 24 hours |
| 60-79 | High | CS Manager | 24 hours |
| 60-79 | Medium | CS Rep | 48 hours |
| 60-79 | Low | Automated | Immediate |
| 40-59 | Any | Automated | Immediate |

## Implementation Steps

### Step 1: Signal Integration
```yaml
data_sources:
  - support: "Zendesk/Intercom API"
  - product: "Mixpanel/Amplitude events"
  - payment: "Stripe/Recurly webhooks"
  - email: "Customer.io/Iterable events"
  - nps: "Delighted/Wootric API"
```

### Step 2: Scoring Engine
```yaml
scoring_engine:
  frequency: "real-time (webhook-driven)"
  storage: "user risk score in CRM"
  history: "score changes logged"
```

### Step 3: Intervention Triggers
```yaml
trigger_system:
  type: "event-driven"
  tools: "Segment + Customer.io / custom"
  owner_assignment: "round-robin + account value"
```

### Step 4: Tracking & Optimization
```yaml
tracking:
  - intervention_sent
  - intervention_opened
  - response_received
  - outcome (saved / churned / downgraded)
  - time_to_resolution
```

## Quality Gates

### Before Intervention
- [ ] Risk score validated against actual churn
- [ ] Templates reviewed for empathy and clarity
- [ ] Escalation paths defined
- [ ] Compensation authority documented

### Ethical Considerations
- [ ] No manipulation of genuine grievances
- [ ] Honest about product limitations
- [ ] Respect customer decision to leave
- [ ] Don't make promises you can't keep

## Tools & Platforms

| Function | Tool Options |
|----------|-------------|
| Sentiment analysis | AWS Comprehend, custom NLP |
| Risk scoring | Custom model, Gainsight |
| Intervention automation | Customer.io, Intercom |
| CS workflow | Gainsight, ChurnZero, Vitally |
| Communication | Intercom, Front |

## Common Pitfalls

1. **Over-automation**: Some situations need humans
2. **Ignoring root cause**: Saving without fixing the problem
3. **Desperate messaging**: "Please don't leave!" doesn't work
4. **No follow-through**: Promising changes, not delivering
5. **One-size-fits-all**: Different segments need different approaches

## B2C vs B2B Considerations

| Aspect | B2C | B2B |
|--------|-----|-----|
| Signal sources | App usage, reviews | Support, stakeholder engagement |
| Intervention style | Automated, product-driven | Human, relationship-driven |
| Compensation | Discounts, free months | Custom solutions, training |
| Timeline | Fast (days) | Slow (weeks) |

## Remember

> "Churn prevention isn't about begging customers to stay. It's about fixing problems before they become reasons to leave."

The best churn prevention is a product and experience so good that customers never want to leave.
