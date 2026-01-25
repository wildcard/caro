# Mechanism 7: Behavior-driven Activation

## Overview

Detect user friction in real-time and trigger targeted interventions to prevent drop-off and accelerate time-to-value.

## The Problem

Traditional onboarding:
1. Same experience for everyone
2. No awareness of where users get stuck
3. Interventions (emails, tooltips) are time-based, not behavior-based
4. Drop-off happens silently

**Result**: Low activation rates, invisible friction, one-size-fits-none experience.

## The Solution

Build a **behavior-driven activation system**:
1. Define the "aha moment" precisely
2. Instrument friction signals (stuck, repeat, quit)
3. Design interventions for each friction point
4. A/B test intervention effectiveness
5. Measure impact on activation metrics

## Key Concepts

### Aha Moment
The point where a user first experiences your product's core value.

| Product Type | Example Aha Moment |
|--------------|-------------------|
| Project management | "User creates first project and invites 1 teammate" |
| Analytics | "User sees their first insight from real data" |
| Communication | "User sends first message and gets a reply" |
| E-commerce | "User completes first purchase" |

### Friction Signals
Behaviors that indicate a user is struggling:

| Signal | Definition | Example |
|--------|------------|---------|
| Time threshold | Spent > X seconds on step | 60s on setup wizard |
| Repeat action | Same action > N times | Clicked button 3x |
| Backtracking | Returns to previous step | Back to settings 2x |
| Rage click | Rapid repeated clicks | 5+ clicks in 2s |
| Idle | No action for > X seconds | 30s inactive on page |
| Quit | Left mid-flow | Closed during onboarding |

## System Architecture

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│  User Events    │────▶│  Friction       │────▶│  Intervention   │
│  (telemetry)    │     │  Detector       │     │  Trigger        │
└─────────────────┘     └─────────────────┘     └─────────────────┘
                                                        │
                        ┌───────────────────────────────┘
                        ▼
        ┌───────────────────────────────────────────────────────┐
        │                  Intervention Types                    │
        │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌────────┐ │
        │  │ In-app   │  │  Email   │  │   Chat   │  │ Human  │ │
        │  │ tooltip  │  │  nudge   │  │  prompt  │  │ outreach│ │
        │  └──────────┘  └──────────┘  └──────────┘  └────────┘ │
        └───────────────────────────────────────────────────────┘
```

## Friction Rules Definition

```yaml
friction_rules:
  - name: "stuck_on_setup"
    trigger:
      event: "setup_wizard_step_3"
      time_threshold: 45  # seconds
      repeat_threshold: null
    intervention:
      type: "tooltip"
      template_id: "setup_help_step3"
      delay: 5  # seconds after trigger
      personalization:
        step_name: "{{current_step}}"
    success_metric: "setup_completed"

  - name: "repeat_action_failure"
    trigger:
      event: "integration_connect_click"
      repeat_threshold: 3  # clicks without success
    intervention:
      type: "chat_prompt"
      template_id: "integration_help"
      delay: 0
    success_metric: "integration_connected"

  - name: "abandoned_onboarding"
    trigger:
      event: "onboarding_quit"
      conditions: "step < 3"
    intervention:
      type: "email"
      template_id: "comeback_onboarding"
      delay: 3600  # 1 hour
    success_metric: "onboarding_resumed"
```

## Intervention Templates

### In-App Tooltip
```yaml
template_id: "setup_help_step3"
type: "tooltip"
content:
  title: "Need help with {{step_name}}?"
  body: "Most users connect their {{integration_type}} here. Here's a quick guide."
  cta: "Show me how"
  dismiss: "I'm fine"
positioning: "anchor_to_element"
element_selector: "#setup-step-3-button"
```

### Email Nudge
```yaml
template_id: "comeback_onboarding"
type: "email"
content:
  subject: "You were so close! 🎯"
  body: |
    Hey {{first_name}},

    You started setting up {{product_name}} but didn't finish.
    You're just {{steps_remaining}} steps away from {{value_prop}}.

    Want to pick up where you left off?
  cta: "Continue Setup"
  cta_url: "{{resume_url}}"
send_conditions:
  - not_completed: "onboarding"
  - hours_since_quit: "> 1"
  - hours_since_quit: "< 24"
```

### Chat Prompt
```yaml
template_id: "integration_help"
type: "chat_prompt"
content:
  message: "Having trouble connecting your {{integration_name}}? I can walk you through it in 2 minutes."
  options:
    - "Yes, help me"
    - "No thanks, I'll figure it out"
routing:
  "Yes, help me": "integration_support_flow"
  "No thanks": "dismiss"
```

## Implementation Steps

### Step 1: Define Aha Moment
```yaml
aha_moment:
  definition: "User creates first project AND invites 1 teammate"
  events:
    - "project_created"
    - "teammate_invited"
  time_window: "7 days from signup"
  current_rate: "15%"
  target_rate: "30%"
```

### Step 2: Map the Path to Aha
```yaml
path_to_aha:
  steps:
    - step: 1
      name: "Signup"
      drop_off: "5%"

    - step: 2
      name: "Email verified"
      drop_off: "20%"

    - step: 3
      name: "Profile completed"
      drop_off: "15%"

    - step: 4
      name: "First project created"
      drop_off: "25%"  # HIGH FRICTION

    - step: 5
      name: "Teammate invited"
      drop_off: "35%"  # HIGHEST FRICTION
```

### Step 3: Instrument Friction Detection
```javascript
// Example tracking code
trackEvent('step_viewed', {
  step_id: 'create_project',
  timestamp: Date.now()
});

// Friction detection
if (timeOnStep > 45000) {  // 45 seconds
  triggerFrictionEvent('stuck_on_step', {
    step_id: 'create_project',
    time_on_step: timeOnStep
  });
}
```

### Step 4: Design Interventions
```yaml
interventions:
  step_4_friction:  # First project created
    - condition: "time_on_step > 30s"
      intervention: "tooltip_project_template"
    - condition: "time_on_step > 60s"
      intervention: "chat_project_help"
    - condition: "quit_at_step_4"
      intervention: "email_project_nudge"

  step_5_friction:  # Teammate invited
    - condition: "project_created AND no_invite_after_10min"
      intervention: "email_invite_reminder"
    - condition: "invite_button_clicked > 2x"
      intervention: "tooltip_invite_help"
```

### Step 5: A/B Test Interventions
```yaml
ab_test:
  name: "tooltip_vs_chat_step4"
  hypothesis: "Chat prompts have higher conversion than tooltips"
  control: "tooltip_project_template"
  variant: "chat_project_help"
  metric: "project_created"
  sample_size: 2000
  duration: "14 days"
  significance_threshold: 0.95
```

## Dashboard Metrics

| Metric | Description | Target |
|--------|-------------|--------|
| Activation rate | Users reaching aha moment / signups | > 30% |
| Time to aha | Median time from signup to aha | < 24 hours |
| Friction detection rate | Friction events / total users | < 40% |
| Intervention trigger rate | Interventions fired / friction events | > 80% |
| Intervention success rate | Resolved / triggered | > 25% |
| Drop-off by step | Users lost at each step | Decreasing |

## Quality Gates

### Before Launch
- [ ] All paths lead to either aha or defined recovery
- [ ] Intervention frequency caps (no spam)
- [ ] Opt-out mechanism available
- [ ] Mobile experience tested

### After Launch
- [ ] Intervention not hurting conversion
- [ ] False positive rate acceptable (< 10%)
- [ ] User feedback positive
- [ ] No critical flow interruption

## Tools & Platforms

| Function | Tool Options |
|----------|-------------|
| Event tracking | Segment, Mixpanel, Amplitude |
| In-app messaging | Intercom, Pendo, Appcues |
| Email | Customer.io, Iterable |
| Chat | Intercom, Drift |
| A/B testing | Statsig, Eppo, LaunchDarkly |

## Example Intervention Flow

```markdown
## Project Creation Friction Flow

### Friction Signal
User has been on "Create Project" screen for > 45 seconds

### Intervention 1: Tooltip (immediate)
"Tip: Start with a template! Most teams begin with our 'Getting Started' template."
[Use Template] [Create Blank]

### If not resolved after 30 more seconds...

### Intervention 2: Chat Prompt
"Hey! Creating your first project? I can help you set one up in under a minute."
[Help me] [I'm fine]

### If user quits without creating project...

### Intervention 3: Email (1 hour later)
Subject: "Your first project is waiting 🚀"
Body: Personalized nudge with template suggestions

### If user returns and creates project...
Track: "intervention_success", source: "email_comeback"
```

## Common Pitfalls

1. **Over-intervention**: Too many popups annoy users
2. **Wrong timing**: Interrupting users who are fine
3. **Generic interventions**: Same message for everyone
4. **No measurement**: Can't tell if interventions work
5. **Ignoring mobile**: Different friction patterns on mobile

## B2C vs B2B Considerations

| Aspect | B2C | B2B |
|--------|-----|-----|
| Aha timeline | Minutes to hours | Days to weeks |
| Intervention channel | In-app, email | In-app, email, human outreach |
| Frequency caps | Lower tolerance | Higher tolerance |
| Personalization | Behavior-based | Company + behavior |

## Remember

> "Friction is invisible until you instrument it. Users don't complain—they just leave."

The goal isn't to remove all friction (some is necessary for learning). It's to identify **unnecessary** friction and help users through it.
