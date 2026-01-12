# Mechanism 8: Personalized Video at Scale

## Overview

Create personalized video content at scale with lip-sync name/company mentions for high-touch outreach without the time cost.

## The Problem

Traditional video outreach:
1. Record one generic video, send to many
2. Or record individual videos, doesn't scale
3. "Hey [FIRST_NAME]" text personalization feels fake
4. Low response rates on cold outreach

**Result**: Either scale OR personalization, not both.

## The Solution

Build a **personalized video production system**:
1. Record base video with placeholder mentions
2. AI generates personalized name/company audio with lip-sync
3. Quality assurance at scale
4. Automated delivery pipeline
5. Response tracking and optimization

## Technology Stack

| Component | Technology | Purpose |
|-----------|------------|---------|
| Lip-sync | HeyGen, Synthesia, D-ID | Video + audio personalization |
| Voice cloning | ElevenLabs, WellSaid | Natural pronunciation |
| Video hosting | Vidyard, Loom, custom | Tracking and delivery |
| CRM integration | Hubspot, Salesforce | Recipient data, tracking |

## Quality Gates

### Must-Have Quality Standards

| Standard | Criteria | Validation |
|----------|----------|------------|
| Name pronunciation | > 95% accuracy | Native speaker review |
| Lip-sync quality | No uncanny valley | Human QA sample |
| Natural timing | Pauses feel natural | Timing analysis |
| Brand voice | Matches presenter's style | Brand team approval |
| Consent | Presenter approved likeness use | Legal sign-off |

### Red Flags to Watch
- Name sounds robotic or mispronounced
- Lip movement doesn't match audio
- Abrupt transitions between personalized and base content
- Unnatural pauses or speeding
- Presenter looks "off" during personalized sections

## Implementation

### Step 1: Base Video Production

```yaml
base_video:
  duration: "60-90 seconds"
  structure:
    - intro: "Generic greeting, no names"
    - personalization_point_1: "Hey [NAME]" # Placeholder
    - value_prop: "Specific to their situation"
    - personalization_point_2: "[COMPANY] specific mention"
    - cta: "Generic call to action"

  recording_tips:
    - Look slightly off-center for natural feel
    - Pause clearly at personalization points
    - Speak at consistent pace
    - Good lighting and audio quality
    - Multiple takes for options
```

### Step 2: Recipient Data Preparation

```yaml
recipients:
  - id: "rec_001"
    name: "Sarah Johnson"
    name_pronunciation: "SAIR-uh JOHN-son"  # phonetic
    company: "Acme Corp"
    company_pronunciation: "AK-mee"
    role: "VP of Engineering"
    pain_point: "scaling engineering team"
    custom_field_1: "recently raised Series B"

  validation:
    - All names have pronunciation guide
    - Company names verified
    - No special characters that break synthesis
    - Duplicate check completed
```

### Step 3: Video Generation Pipeline

```yaml
pipeline:
  step_1:
    name: "Audio synthesis"
    tool: "ElevenLabs/HeyGen"
    input: "name, company, custom fields"
    output: "personalized audio clips"

  step_2:
    name: "Lip-sync generation"
    tool: "HeyGen/Synthesia"
    input: "base video + audio clips"
    output: "personalized video"

  step_3:
    name: "Quality assurance"
    process: "Human review of sample"
    sample_rate: "10% for first batch, 5% ongoing"

  step_4:
    name: "Upload and hosting"
    tool: "Vidyard/Loom"
    output: "unique URLs per recipient"
```

### Step 4: Quality Assurance Checklist

```yaml
qa_checklist:
  audio:
    - [ ] Name pronounced correctly
    - [ ] Company pronounced correctly
    - [ ] Natural speech rhythm
    - [ ] No audio artifacts

  video:
    - [ ] Lip-sync matches audio
    - [ ] No uncanny valley effect
    - [ ] Transitions are smooth
    - [ ] Lighting/color consistent

  content:
    - [ ] Personalization feels natural
    - [ ] Message makes sense for recipient
    - [ ] CTA is clear
    - [ ] No placeholder text visible
```

### Step 5: Delivery Automation

```yaml
delivery:
  channel: "email"
  subject_template: "{{name}}, quick video for you"
  body_template: |
    Hi {{name}},

    I recorded a short video specifically for you about
    how {{company}} could {{value_prop}}.

    [VIDEO THUMBNAIL]

    Let me know what you think!

    {{sender_name}}

  personalization:
    - thumbnail: personalized frame from video
    - video_url: unique tracking URL

  schedule:
    - batch_size: 50
    - send_time: "9am recipient timezone"
    - days: "Tue-Thu"
```

## Metrics & Tracking

| Metric | Description | Benchmark |
|--------|-------------|-----------|
| Open rate | Email opens | 35-50% |
| Video play rate | Plays / opens | 40-60% |
| Watch rate | % of video watched | 60-80% |
| Reply rate | Replies / sends | 5-15% |
| Meeting booked | Conversions | 2-5% |

## Use Cases

### Cold Outreach (Enterprise Sales)
```
Volume: 200 videos/week
Personalization: Name, company, specific pain point
Goal: Meeting booked
Expected lift: 3-5x vs text email
```

### Customer Success (Onboarding)
```
Volume: 50 videos/week (new customers)
Personalization: Name, company, product use case
Goal: Faster activation
Expected lift: 20% faster time-to-value
```

### Renewal/Expansion
```
Volume: 100 videos/month
Personalization: Name, usage stats, upsell opportunity
Goal: Expansion revenue
Expected lift: 15% increase in upgrade rate
```

## Cost Structure

| Component | Cost Model | Example |
|-----------|------------|---------|
| Video platform | Per video | $0.50-2.00/video |
| Voice synthesis | Per minute | $0.10-0.30/minute |
| Hosting | Per view | $0.01-0.05/view |
| Human QA | Per hour | $25-50/hour |

### Example ROI Calculation
```
Campaign: 200 enterprise prospects
Cost per video: $2.00
Total cost: $400
Meeting booked rate: 4%
Meetings: 8
Meeting value: $500 (based on deal size)
Total value: $4,000
ROI: 10x
```

## Two Approaches

### High-Touch (Premium)
```yaml
approach: "high_touch"
qa_rate: "100% human review"
personalization_depth: "Name + company + specific insight"
volume: "50 videos/week"
cost: "$5/video"
use_case: "Enterprise, high ACV"
```

### Scaled (Volume)
```yaml
approach: "scaled"
qa_rate: "5% human review"
personalization_depth: "Name + company only"
volume: "500 videos/week"
cost: "$1/video"
use_case: "Mid-market, lower ACV"
```

## Common Pitfalls

1. **Uncanny valley**: Personalization that feels creepy
2. **Name mispronunciation**: Worse than no personalization
3. **No consent**: Using presenter likeness without permission
4. **Over-personalization**: Including details that feel stalker-ish
5. **Bad audio quality**: Robotic or unclear speech

## Legal & Ethical Considerations

### Required
- [ ] Presenter consent for AI likeness use
- [ ] Recipient knows video is personalized (not deceptive)
- [ ] Opt-out mechanism in email
- [ ] Data handling compliance (GDPR if applicable)

### Best Practices
- Disclose personalization method if asked
- Don't pretend it's a live recording
- Respect unsubscribes immediately
- Don't use for manipulative purposes

## Tools & Platforms

| Function | Tool Options |
|----------|-------------|
| Video personalization | HeyGen, Synthesia, D-ID |
| Voice cloning | ElevenLabs, WellSaid Labs |
| Video hosting | Vidyard, Wistia, Loom |
| Email delivery | Hubspot, Outreach, Salesloft |
| CRM | Salesforce, Hubspot |

## Example Script Template

```markdown
## Base Video Script (60 seconds)

[0:00-0:05]
Hi there! [PAUSE FOR NAME INSERT]

[0:05-0:15]
I noticed [PAUSE FOR COMPANY INSERT] is [specific situation based on research].

[0:15-0:35]
I work with companies like yours to [value prop]. Recently, we helped
[similar company] achieve [specific result].

[0:35-0:50]
I put together a few ideas specific to your situation that I'd love to
share with you.

[0:50-0:60]
If you're open to a quick chat, just reply to this email or grab time
on my calendar below. Looking forward to connecting!
```

## Remember

> "The goal isn't to fool people into thinking you recorded a video just for them. It's to show you put effort into personalization while scaling your outreach."

Done right, personalized video feels thoughtful, not creepy.
