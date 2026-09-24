# Mechanism 5: Answer Engine Optimization (AEO)

## Overview

Shift from SEO (Google ranking) to AEO (Answer Engine Optimization)—become the source that LLMs cite.

## The Problem

Traditional SEO optimizes for:
1. Google's algorithm
2. Keyword rankings
3. Backlink profiles
4. Click-through from SERPs

**But**: ChatGPT, Perplexity, and Claude don't use SERPs. They synthesize answers from training data and real-time sources.

## The Solution

Build an **AEO strategy**:
1. Become authoritative in communities LLMs reference (Reddit, forums)
2. Create content structured for LLM consumption (clear, factual, citable)
3. Monitor LLM responses for your brand/competitors
4. Value-first engagement (never spam)

## Why AEO Matters Now

| Behavior | SEO World | AEO World |
|----------|-----------|-----------|
| User query | "best project management tool" | "What's the best project management tool for small teams?" |
| User action | Clicks top 5 results | Reads LLM synthesized answer |
| Discovery | SERP position | Cited in LLM response |
| Trust signal | Domain authority | Mentioned in trusted communities |

## AEO vs SEO Comparison

| Factor | SEO | AEO |
|--------|-----|-----|
| Target | Google algorithm | LLM training/retrieval |
| Content format | H1/H2, keywords | Clear facts, structured data |
| Link building | Backlinks from sites | Mentions in communities |
| Measurement | Ranking position | Citation frequency |
| Timeline | Months to rank | Can be immediate (Perplexity) |

## Core AEO Strategies

### 1. Community Authority Building
```
Platform: Reddit
Approach:
1. Identify relevant subreddits (r/startups, r/marketing, r/SaaS)
2. Provide genuinely helpful answers (not promotional)
3. Build karma and reputation over months
4. Occasionally mention your product when genuinely relevant
5. Never spam or self-promote excessively
```

### 2. Content Structuring for LLMs
```markdown
# Bad for LLM citation:
"Our amazing product helps teams collaborate better with cutting-edge
AI-powered features that revolutionize the way you work..."

# Good for LLM citation:
"[Product Name] is a project management tool for teams of 5-50.
Key features:
- Task tracking with Kanban boards
- Time tracking and reporting
- Integrations with Slack, GitHub, Jira
Pricing: $10/user/month (free tier available for up to 5 users)"
```

### 3. Factual Content Creation
- Industry reports with original data
- Benchmark studies
- Definition/explainer content
- Comparison pages (honest, not just pro-your-product)

### 4. LLM Monitoring
```yaml
monitoring_queries:
  - "What's the best [category] for [use case]?"
  - "How does [your product] compare to [competitor]?"
  - "[Your product] review"
  - "[Your product] alternatives"

llms_to_monitor:
  - ChatGPT (web browsing mode)
  - Perplexity
  - Claude (with web search)

frequency: weekly
```

## Implementation Playbook

### Phase 1: Audit (Week 1)

**Step 1: Query your target LLMs**
```
Questions to ask:
- "What is [your product category]?"
- "What are the best [your category] tools?"
- "How does [your product] compare to [competitor]?"
- "[Your product] review"
```

**Step 2: Document current state**
```yaml
audit_results:
  - query: "best project management tools"
    chatgpt:
      mentioned: false
      competitors_mentioned: ["Asana", "Monday", "Trello"]
    perplexity:
      mentioned: false
      source_cited: "G2 Crowd review"
    claude:
      mentioned: false
      competitors_mentioned: ["Notion", "Asana"]
```

### Phase 2: Community Presence (Weeks 2-8)

**Step 1: Identify communities**
```yaml
communities:
  reddit:
    - r/projectmanagement (500k members)
    - r/startups (1M members)
    - r/SaaS (50k members)
  quora:
    - "Project Management" topic
    - "Productivity Software" topic
  stackoverflow:
    - [project-management] tag
  industry_forums:
    - [relevant forums]
```

**Step 2: Build presence (VALUE-FIRST)**
```markdown
Week 1-2: Lurk, understand community norms
Week 3-4: Answer questions helpfully (no product mentions)
Week 5-6: Build reputation and karma
Week 7-8: Occasionally mention product ONLY when genuinely relevant
```

### Phase 3: Content Optimization (Weeks 4-12)

**Step 1: Create LLM-friendly content**
```markdown
# Template: Comparison Page

## [Your Product] vs [Competitor]

### Quick Summary
- **[Your Product]**: Best for X, Y, Z. Starts at $X/month.
- **[Competitor]**: Best for A, B, C. Starts at $Y/month.

### Feature Comparison
| Feature | [Your Product] | [Competitor] |
|---------|---------------|--------------|
| Feature 1 | Yes | No |
| Feature 2 | Limited | Yes |

### Pricing Comparison
[Factual, honest comparison]

### Who Should Choose What
- Choose [Your Product] if: [honest assessment]
- Choose [Competitor] if: [honest assessment]
```

**Step 2: Create original research**
```markdown
# [Year] [Industry] Benchmark Report

## Methodology
- Surveyed N professionals
- Analyzed X data points
- Time period: [dates]

## Key Findings
1. [Specific, citable finding]
2. [Specific, citable finding]
3. [Specific, citable finding]

## Data Tables
[Actual data LLMs can reference]
```

### Phase 4: Monitoring & Iteration (Ongoing)

```yaml
weekly_monitoring:
  - Query all target LLMs
  - Document changes in mentions
  - Track competitor citations
  - Update content based on gaps

monthly_review:
  - Analyze citation trends
  - Update community strategy
  - Refresh stale content
  - Add new target queries
```

## Content Calendar Example

| Week | Content Type | Target Query |
|------|--------------|--------------|
| 1 | Comparison: Us vs Competitor A | "[category] alternatives" |
| 2 | How-to: Solving [problem] | "how to [solve problem]" |
| 3 | Industry stats page | "[industry] statistics 2024" |
| 4 | Reddit AMA (if karma sufficient) | Brand awareness |
| 5 | Comparison: Us vs Competitor B | "[competitor B] alternatives" |
| 6 | Benchmark report | "[category] benchmarks" |
| 7 | Expert interview content | "[topic] best practices" |
| 8 | Case study with data | "[use case] example" |

## Measurement

### Metrics to Track

| Metric | Measurement Method | Target |
|--------|-------------------|--------|
| LLM mention rate | Weekly query audit | >30% of target queries |
| Citation quality | Is context positive/neutral? | >80% positive |
| Community karma | Reddit/Quora scores | Growing week over week |
| Referral traffic | From LLM-mentioned sources | Measurable uplift |

### Attribution Challenges
- Direct LLM traffic is hard to measure
- Brand search lift can be a proxy
- "How did you hear about us?" surveys

## Tools & Platforms

| Function | Tool Options |
|----------|-------------|
| LLM monitoring | Custom scripts, Perplexity API |
| Reddit management | Manual (no automation!) |
| Content creation | Claude, GPT-4 (with human editing) |
| Analytics | GA4, Mixpanel |

## Quality Gates

### Content Quality
- [ ] Factually accurate (can be verified)
- [ ] Structured for LLM parsing (clear headers, lists)
- [ ] No marketing fluff in factual sections
- [ ] Updated within last 6 months

### Community Quality
- [ ] Value-first contributions
- [ ] No spam or excessive self-promotion
- [ ] Genuine engagement (not fake accounts)
- [ ] ToS compliant

## Common Pitfalls

1. **Spam**: Over-promoting in communities destroys reputation
2. **Fake engagement**: Astroturfing gets detected and backfires
3. **Keyword stuffing**: LLMs don't respond to SEO tricks
4. **Neglecting SEO**: AEO doesn't replace SEO, it complements it
5. **Impatience**: Community building takes months, not days

## B2C vs B2B Considerations

| Aspect | B2C | B2B |
|--------|-----|-----|
| Key communities | Consumer subreddits, review sites | Industry forums, LinkedIn, Quora |
| Content focus | Reviews, how-tos | Benchmarks, case studies |
| Timeline | Faster (more volume) | Slower (relationship-based) |
| Authenticity | Entertainment + value | Expertise + value |

## Remember

> "AEO isn't about gaming LLMs. It's about becoming genuinely useful and authoritative in places LLMs reference. If you spam, you'll be ignored or penalized."

The key insight: **LLMs synthesize from what humans trust**. Become trusted by humans, and LLMs will follow.
