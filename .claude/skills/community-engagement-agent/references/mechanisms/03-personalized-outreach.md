# Mechanism 3: Personalized Outreach

## Purpose
Draft engagement messages that reference specific user contributions. Every message must pass the "specific action" test.

## The Specificity Rule

Before drafting any message, verify:
- Can you name a specific recipe/command the user created? If no, stop.
- Can you cite a number (runs, reuses, remixes)? If no, stop.
- Can you describe the impact on other users? If no, stop.

If all three are yes, proceed with drafting.

## Message Types

### 1. Recognition
**Trigger**: User's work crossed a meaningful threshold.

| Channel | Template |
|---------|----------|
| CLI | `caro: Your {title} was run {count} times this week.` |
| Email | Subject: `Your recipes hit {total} runs this month` |
| Web | Badge: `{milestone_name}` with description |

**Examples**:
```
CLI:
  caro: Your batch image converter was reused by 12 people.
  caro: That's more than most published recipes.

Email:
  Subject: Your recipes hit 100 runs this month
  
  Your FFmpeg recipes collectively hit 100 runs this month.
  The batch converter alone accounts for 47 of those.
  
  We're building something special here, and your work is
  a big part of why people keep coming back.

Web:
  Badge: "Century Club" - Recipe reached 100 runs
```

### 2. Amplification
**Trigger**: User has a high-quality recipe that deserves more visibility.

| Channel | Template |
|---------|----------|
| CLI | `caro: {count} people use your {title}. Want to feature it? (y/n)` |
| Email | Subject: `{title} is trending - want to share the story?` |
| Web | Featured placement on category page |

### 3. Invitation
**Trigger**: User meets Founder tier eligibility or special program criteria.

| Channel | Template |
|---------|----------|
| CLI | `caro: You've contributed {score} points. There's something we'd like to share.` |
| Email | Subject: `An invitation from the CARO project` |

**Email body structure** (invitation):
1. Open with specific contribution reference
2. Explain what Founding Builders is (one paragraph)
3. Why they qualify (reference their specific work)
4. What it means (permanent recognition, early access, roadmap influence)
5. Simple call to action (reply or click link)

### 4. Direction
**Trigger**: Community has an unmet need matching this user's expertise.

| Channel | Template |
|---------|----------|
| CLI | `caro: People are searching for {gap}. You've built similar tools. Want to help?` |
| Email | Subject: `{volume} people searched for {gap} this week` |

**Email body structure** (direction):
1. Show the demand (real numbers, real search queries)
2. Connect to their expertise (reference what they've already built)
3. Suggest what they could create
4. Frame as helping real people, not helping the platform

## Personalization Variables

| Variable | Source | Example |
|----------|--------|---------|
| `{title}` | Recipe title | "batch image converter" |
| `{slug}` | Recipe slug | "batch-image-converter" |
| `{count}` | Run count | "47" |
| `{reuse_count}` | Distinct users who ran it | "12" |
| `{remix_count}` | Forks of the recipe | "3" |
| `{score}` | Contribution score | "156" |
| `{tier}` | Current tier | "Leader" |
| `{gap}` | Category with demand | "PDF tools" |
| `{volume}` | Search volume | "230" |
| `{display_name}` | User's display name | "brave-ocean-tiger" |

## Anti-Patterns (Banned)

1. **Generic praise**: "Thanks for being part of our community!"
2. **Mass messaging**: Same template sent to >5 users with only name swapped
3. **Token hype**: Any mention of financial value, income, or tradable rewards
4. **Fake urgency**: "Limited spots!" "Act now!"
5. **Over-attribution**: "CARO wouldn't exist without you" (for a user with 3 recipes)
6. **Comparison marketing**: "Better than [competitor]"
7. **Empty amplification**: Featuring mediocre work to fill content gaps
8. **Frequency violation**: More than 2 CLI messages per user per week
9. **Stalker energy**: Referencing too many details about usage patterns

## Quality Checklist

Before a draft enters the approval queue:

- [ ] References at least one specific recipe or action
- [ ] Includes at least one real number
- [ ] Complies with persona-spec.md voice rules
- [ ] No banned words/phrases from persona-spec.md
- [ ] Within channel frequency limits
- [ ] User not contacted in last 3 days
- [ ] No non-public user data revealed
