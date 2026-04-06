# Mechanism 5: Founder Curation

## Purpose
Manage the Founding Builders program - identify eligible contributors, draft invitations, track acceptance, and maintain exclusivity.

## What Founding Builders Is

A permanent recognition group for early contributors who shaped the CARO project. It is:
- **Prestigious**: Earned through sustained, high-quality contribution
- **Permanent**: Once accepted, membership doesn't expire
- **Scarce**: Capped initial cohort (configurable, default 50)
- **Non-transactional**: Not a purchase, not an exchange, not a payment

## Eligibility Criteria

### Hard Requirements (must ALL be met)
- `decayed_score >= 200`
- `active_days >= 30` (contributed on at least 30 distinct days)
- `tier == "founder_eligible"` (from scoring agent)
- No `gaming_suspect` flag
- No active moderation actions

### Weighted Soft Criteria (improve ranking)
| Criterion | Weight | Description |
|-----------|--------|-------------|
| Recipe breadth | 0.3 | Multiple recipes with reuse > 10 |
| Safety contributions | 0.2 | At least one safety-related contribution |
| Category diversity | 0.2 | Activity across 3+ categories |
| Community helping | 0.15 | Responded to others' questions/issues |
| Consistency | 0.15 | Even activity distribution (not burst-heavy) |

### Disqualifying Factors
- Previously declined invitation (wait 90 days before re-invite)
- Content flagged by community moderation
- Detected gaming behavior

## Invitation Flow

```
Eligible ──→ Candidate Review ──→ Draft Invitation ──→ Human Approval ──→ Send ──→ Response
                                                                                     │
                                                                          ┌──────────┼──────────┐
                                                                          ▼          ▼          ▼
                                                                       Accept     Decline    No Response
                                                                          │          │          │
                                                                    Welcome flow  Log reason  Follow up
                                                                    + onboard     Wait 90d    after 14d
```

### Invitation Stages
1. **Identified**: Score qualifies, added to candidate pool
2. **Reviewed**: Human reviewed candidate profile
3. **Drafted**: Invitation email drafted by @outreach-agent
4. **Approved**: Human approved the invitation
5. **Sent**: Invitation delivered via email
6. **Accepted** / **Declined** / **Pending**: Response tracking

## Invitation Email Structure

```
Subject: An invitation from the CARO project

[Name],

[Specific reference to their most impactful contribution]

We're forming a small group of Founding Builders — the people
whose work shaped CARO in its earliest days. Your [specific recipe/
contribution] has been [specific impact: run count, reuse count].

Founding Builders get:
- Permanent recognition in the project
- Early access to new features before public release
- Direct influence on the product roadmap
- A seat at the table for the project's future direction

This isn't a program we're selling. It's recognition for people
who built something real when it mattered most.

If you're interested, [simple CTA - reply or click link].

— The CARO team
```

## Founder Benefits

| Benefit | Description | Available From |
|---------|-------------|---------------|
| Permanent badge | "Founding Builder" badge on profile and recipes | Acceptance |
| Early access | New features 2 weeks before public release | Acceptance |
| Roadmap influence | Quarterly feedback sessions, feature voting | Acceptance |
| Future recognition | First in line for future rewards programs | When announced |
| Direct channel | Private communication channel with team | Acceptance |

## Cohort Management

### Initial Cohort
- Maximum: 50 members (configurable)
- Invitations sent in batches of 5-10
- Pipeline tracked: eligible → invited → accepted
- Remaining slots visible to operators

### After Initial Cohort Fills
- New eligible users go to waitlist
- Waitlist reviewed quarterly
- Cohort may expand based on community growth
- "Founding" designation remains exclusive to initial group

## Metrics to Track

| Metric | Target | Description |
|--------|--------|-------------|
| Invitation acceptance rate | > 70% | High = good targeting |
| Time to accept | < 7 days | Fast = high engagement |
| Post-acceptance activity | Increased | Founders should stay active |
| Founder retention (30d) | > 90% | Founders should not churn |
