# Mechanism 6: Dynamic Real-time Quiz

## Overview

Replace static forms with dynamic quizzes that adapt questions based on user context and responses.

## The Problem

Traditional lead forms:
1. Same questions for everyone
2. Long, intimidating forms hurt conversion
3. Poor qualification (bad fit leads get through)
4. No personalization in follow-up

**Result**: Low completion rates, poor lead quality, generic experiences.

## The Solution

Build **adaptive qualification flows**:
1. Every question earns its place (no fluff)
2. Answers change subsequent questions
3. Detect urgency/pain signals and adjust path
4. Build trust through personalization
5. Clear handoff criteria (self-serve vs sales vs nurture)

## Core Principles

1. **Progressive disclosure**: Start easy, get specific
2. **Branching logic**: Different paths for different segments
3. **Signal detection**: Urgency, budget, timeline indicators
4. **Personalized outcomes**: Different CTAs based on answers
5. **Skip option always available**: Respect user time

## Architecture

```
┌─────────────────┐
│   Entry Point   │
│   (Question 1)  │
└────────┬────────┘
         │
    ┌────┴────┐
    ▼         ▼
┌───────┐ ┌───────┐
│Path A │ │Path B │
│(SMB)  │ │(Ent)  │
└───┬───┘ └───┬───┘
    │         │
    ▼         ▼
┌───────┐ ┌───────┐
│Q2a    │ │Q2b    │
└───┬───┘ └───┬───┘
    │         │
    ▼         ▼
  ...       ...
    │         │
    ▼         ▼
┌───────────────────┐
│  Exit Points      │
│ (Self-serve/Sales)│
└───────────────────┘
```

## Question Design

### Question Types

| Type | Use Case | Example |
|------|----------|---------|
| Single choice | Quick segmentation | "What's your team size?" |
| Multi-choice | Feature interest | "What problems are you solving?" |
| Scale | Urgency/priority | "How urgent is this need?" |
| Free text | Qualification depth | "Describe your current workflow" |
| Slider | Budget/timeline | "What's your budget range?" |

### Good Question Criteria
- [ ] Answers change the path
- [ ] Provides qualification signal
- [ ] User sees value in answering
- [ ] Not invasive (email/phone at end only)
- [ ] Can be skipped without breaking flow

## Segmentation Logic

### Example: SaaS Product

**Question 1: Team size**
```yaml
question: "How many people will use this tool?"
options:
  - "Just me" → path: solo
  - "2-10" → path: small_team
  - "11-50" → path: mid_market
  - "50+" → path: enterprise
```

**Question 2a (solo path):**
```yaml
question: "What's your main goal?"
options:
  - "Personal productivity" → handoff: self_serve_trial
  - "Freelance business" → handoff: self_serve_trial
  - "Evaluating for team" → path: evaluator_flow
```

**Question 2b (enterprise path):**
```yaml
question: "What's your timeline for implementation?"
options:
  - "ASAP (< 1 month)" → signal: urgent, handoff: sales_priority
  - "1-3 months" → handoff: sales_standard
  - "Just exploring" → handoff: nurture
```

## Signal Detection

### Urgency Signals
```yaml
signals:
  high_urgency:
    - keyword: "ASAP", "urgent", "immediately"
    - timeline: "< 1 month"
    - follow_up_preference: "call me today"

  high_intent:
    - current_solution: "switching from [competitor]"
    - budget: "> $10k"
    - decision_maker: "yes"

  low_fit:
    - team_size: "just exploring"
    - budget: "no budget"
    - timeline: "next year"
```

### Signal-Based Routing
```yaml
routing_rules:
  - if: high_urgency AND high_intent
    then: sales_priority_queue

  - if: high_intent AND NOT high_urgency
    then: sales_standard_queue

  - if: low_fit
    then: nurture_drip

  - default: self_serve_trial
```

## Personalization Rules

### During Quiz
```yaml
personalization:
  - trigger: "team_size = enterprise"
    adjustment: "Show enterprise-focused messaging"

  - trigger: "urgency = high"
    adjustment: "Add 'fast implementation' messaging"

  - trigger: "industry = healthcare"
    adjustment: "Add HIPAA compliance mention"
```

### At Exit Points
```yaml
exit_personalization:
  - segment: "SMB + high_urgency"
    cta: "Start your 14-day trial now"
    messaging: "You can be up and running in 10 minutes"

  - segment: "Enterprise + decision_maker"
    cta: "Schedule a demo with our enterprise team"
    messaging: "We'll show you [specific features] for large teams"

  - segment: "Low_fit"
    cta: "Download our free guide"
    messaging: "Not ready yet? Learn more about [topic]"
```

## Implementation

### Step 1: Define Segments
```yaml
segments:
  - name: "SMB Self-Serve"
    criteria: ["team_size < 10", "budget < $1k", "no urgency"]
    handoff: "self_serve_trial"

  - name: "Mid-Market Sales"
    criteria: ["team_size 10-50", "budget $1k-10k"]
    handoff: "sales_demo"

  - name: "Enterprise Priority"
    criteria: ["team_size > 50", "urgency high", "budget > $10k"]
    handoff: "enterprise_ae"

  - name: "Not Ready"
    criteria: ["timeline > 6 months", "just exploring"]
    handoff: "nurture_sequence"
```

### Step 2: Design Question Flow
```yaml
flow:
  q1:
    text: "What best describes your role?"
    type: "single_choice"
    options:
      - value: "founder"
        next: "q2_founder"
      - value: "manager"
        next: "q2_manager"
      - value: "individual"
        next: "q2_individual"
    skip: "q3"

  q2_founder:
    text: "How many employees at your company?"
    type: "single_choice"
    options:
      - value: "1-10"
        segment_signal: "smb"
        next: "q3"
      - value: "11-50"
        segment_signal: "mid_market"
        next: "q3"
      - value: "50+"
        segment_signal: "enterprise"
        next: "q3_enterprise"
```

### Step 3: Build Exit Points
```yaml
exit_points:
  - id: "self_serve_trial"
    segment: "smb"
    components:
      - headline: "Start your free trial"
      - subhead: "No credit card required"
      - form_fields: ["email"]
      - cta: "Get Started"
    redirect: "/trial"

  - id: "sales_demo"
    segment: "mid_market"
    components:
      - headline: "Let's find the right plan for your team"
      - subhead: "Book a 15-minute call"
      - form_fields: ["email", "phone", "company"]
      - cta: "Schedule Demo"
    calendar_integration: true
```

### Step 4: Analytics Setup
```yaml
tracking:
  events:
    - "quiz_started"
    - "question_answered" (with question_id, answer)
    - "question_skipped"
    - "path_changed"
    - "quiz_completed" (with segment, score)
    - "exit_cta_clicked"

  metrics:
    - completion_rate_by_entry_source
    - drop_off_by_question
    - segment_distribution
    - conversion_by_segment
```

## Quality Checklist

### Flow Quality
- [ ] No dead-end paths (every path reaches an exit)
- [ ] Skip option on every question
- [ ] Mobile-friendly (thumb-scrollable)
- [ ] Progress indicator
- [ ] Back button works

### Question Quality
- [ ] Each question has a purpose
- [ ] Language is conversational, not corporate
- [ ] Options are mutually exclusive
- [ ] No leading or biased questions
- [ ] No jargon without context

### Exit Quality
- [ ] Clear value proposition at each exit
- [ ] Personalized based on quiz answers
- [ ] Minimum required fields
- [ ] Trust signals (reviews, logos, guarantees)

## Tools & Platforms

| Function | Tool Options |
|----------|-------------|
| Quiz builder | Typeform, Outgrow, custom |
| Logic engine | Custom, Zapier |
| CRM integration | Hubspot, Salesforce |
| Analytics | Mixpanel, Amplitude, custom |
| Calendar booking | Calendly, Chili Piper |

## Example Quiz Flow

```markdown
## Startup Qualification Quiz

### Q1: What's your role?
- Founder/CEO → Q2a
- Product/Engineering → Q2b
- Marketing/Growth → Q2b
- Other → Q2b

### Q2a (Founder): How many employees?
- Solo/1-5 → Q3 (SMB path)
- 6-20 → Q3 (Growth path)
- 20+ → Q3 (Scale path)

### Q3: What's your biggest challenge?
- [Multi-select list of pain points]
- Used to personalize messaging at exit

### Q4: How soon do you need a solution?
- This week (URGENT) → Sales Priority Exit
- This month → Standard Sales Exit
- Just exploring → Self-Serve Exit

### Exit: Sales Priority
"Based on your answers, it looks like [product] could help you
[solve specific pain point]. Let's get you on a call with someone
who specializes in [their situation] today."

[Calendar Widget]
```

## Common Pitfalls

1. **Too many questions**: >7 questions hurts completion
2. **No personalization at exit**: Generic landing page defeats purpose
3. **Fake personalization**: User notices when it's just their name inserted
4. **Missing skip option**: Forced answers = fake data
5. **Over-engineering**: Start simple, add complexity based on data

## B2C vs B2B Considerations

| Aspect | B2C | B2B |
|--------|-----|-----|
| Quiz length | 3-5 questions | 5-8 questions |
| Data collected | Preferences, goals | Company info, timeline, budget |
| Exit points | Free trial, purchase | Demo, call, nurture |
| Personalization | Product recommendations | Solution matching |

## Remember

> "Every question should earn its place. If the answer doesn't change what you show or how you qualify, delete the question."

The best quiz feels like a conversation, not an interrogation.
