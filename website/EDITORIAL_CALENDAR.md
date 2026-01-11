# Caro Blog Editorial Calendar

## Publishing Cadence

**Frequency**: Bi-weekly (every 2 weeks)
**Day**: Tuesdays
**Time**: 10:00 AM PST / 1:00 PM EST / 6:00 PM UTC

### Why Bi-weekly on Tuesdays?

- **Bi-weekly**: Sustainable pace for a FOSS team. Quality over quantity.
- **Tuesdays**: Avoids Monday chaos and Friday drop-off. Peak developer engagement.
- **Consistent timing**: Builds audience expectation and habit.

---

## Q1 2026 Schedule

| Publish Date | Post Title | Status | Category |
|--------------|------------|--------|----------|
| Jan 21, 2026 | The AI Command Line Safety Paradox | DRAFT | Technical |
| Feb 4, 2026 | Zero Cloud Dependencies: Building Offline-First AI | DRAFT | Technical |
| Feb 18, 2026 | Platform-Aware AI: Teaching LLMs BSD vs GNU | DRAFT | Technical |
| Mar 4, 2026 | 52 Regex Patterns That Could Save Your Server | DRAFT | Technical |
| Mar 18, 2026 | Privacy-First Telemetry | DRAFT | Philosophy |

---

## Draft Workflow

### How Drafts Work

```
1. Create post in /website/src/pages/blog/[slug].astro
2. Add to allPosts array in /website/src/pages/blog/index.astro
3. Set draft: true and badge: "DRAFT"
4. Post is visible in:
   - Local dev (npm run dev)
   - Vercel preview deployments (PR previews)
5. Post is hidden in:
   - Production builds (caro.sh)
```

### Publishing a Draft

```bash
# 1. Edit the post entry in index.astro
draft: false,        # Change from true
badge: "NEW",        # Change from "DRAFT" (or null for older posts)

# 2. Update the date to actual publish date
date: "January 21, 2026",

# 3. Commit and push to main
git add website/src/pages/blog/
git commit -m "Publish: The AI Command Line Safety Paradox"
git push origin main
```

---

## Content Pipeline

### Stages

1. **Idea** → Captured in `/docs/blog-post-ideas.md`
2. **Outline** → Author creates structure
3. **Draft** → Full post written, `draft: true`
4. **Review** → Team reviews on Vercel preview
5. **Scheduled** → Date assigned in calendar
6. **Published** → `draft: false`, merged to main

### Lead Time

- **2 weeks before publish**: Draft complete
- **1 week before publish**: Review complete
- **Day of publish**: Change `draft: false`, merge

---

## Growth Tactics

### Distribution Channels (Priority Order)

1. **Hacker News** - Technical posts, early morning PST
2. **Reddit** - r/rust, r/commandline, r/devops, r/linux
3. **Twitter/X** - Thread summary with key insights
4. **Dev.to** - Cross-post (canonical to caro.sh/blog)
5. **LinkedIn** - Enterprise-focused content

### Per-Post Checklist

```
[ ] Post published to caro.sh/blog
[ ] Twitter thread drafted (5-7 tweets)
[ ] Reddit post prepared (match subreddit tone)
[ ] Hacker News title crafted (no clickbait)
[ ] Dev.to cross-post scheduled (24h delay)
[ ] LinkedIn post for enterprise angles
```

### Engagement Metrics

Track for each post:
- **HN rank** - Did it reach front page?
- **Reddit upvotes** - Community reception
- **GitHub stars** - Conversion to repo
- **Install script runs** - setup.caro.sh analytics
- **Referral traffic** - Google Analytics

---

## 2026 Roadmap

### Q1: Establish Authority (Current)
- Focus: Technical deep-dives
- Goal: Position Caro as the safety leader
- Posts: Safety, offline-first, platform-awareness

### Q2: Expand Reach
- Focus: Tutorials and how-tos
- Goal: Attract new users with practical content
- Topics: Shell scripting tips, workflow automation

### Q3: Community Building
- Focus: Contributor stories, community features
- Goal: Build sense of ownership in community
- Topics: Contributor spotlights, roadmap updates

### Q4: Enterprise & Scale
- Focus: Enterprise use cases, security deep-dives
- Goal: Attract enterprise adoption
- Topics: Compliance, air-gapped deployment, security audits

---

## Content Ideas Backlog

### Priority 1 (Q1-Q2)
- [ ] Hybrid Intelligence: Pattern Matching vs LLMs
- [ ] Spec-Driven Development at Scale
- [ ] Shipping Rust Binaries to 5 Platforms
- [ ] Hardware-Aware AI: Model Recommendations
- [ ] Building a Test Harness for Unreliable AI

### Priority 2 (Q2-Q3)
- [ ] BSD/GNU Security Culture in Open Source
- [ ] TDD for Safety-Critical Systems
- [ ] Writing Safe Rust: FFI and Async Patterns
- [ ] Configuration as Code Philosophy

### Priority 3 (Q3-Q4)
- [ ] Contributor Spotlight Series
- [ ] Caro in Enterprise: Case Studies
- [ ] Air-Gapped Deployment Guide
- [ ] Annual Roadmap Review

---

## Style Guide Summary

- **Tone**: Conversational, Reddit-friendly, not academic
- **Data**: Every stat needs a source or is from Caro metrics
- **Length**: 1,500-2,500 words (6-10 min read)
- **Code**: Include working examples users can try
- **CTA**: End with install command or link to docs

---

## Team Responsibilities

| Role | Responsibility |
|------|----------------|
| Author | Write draft, respond to review feedback |
| Reviewer | Technical accuracy, tone check |
| Publisher | Merge to main, coordinate distribution |
| Community | Monitor HN/Reddit, engage with comments |

---

*Last updated: January 11, 2026*
