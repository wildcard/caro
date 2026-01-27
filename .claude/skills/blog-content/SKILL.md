# Blog Content Workflow Skill

A comprehensive skill for creating SEO-optimized blog content following a structured lifecycle from idea to publication.

## Invocation

```
/blog-content [action] [topic]
```

**Actions:**
- `research` - Create research brief for a new topic
- `write` - Generate SEO-optimized article from brief
- `optimize` - Final SEO audit before publishing
- `analyze` - Evaluate existing content for improvements
- `review` - Performance review and prioritization

## Workflow Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                    BLOG CONTENT LIFECYCLE                          │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  IDEA → RESEARCH → WRITE → OPTIMIZE → PUBLISH → MONITOR → REWRITE │
│    │        │         │         │          │         │         │   │
│    ▼        ▼         ▼         ▼          ▼         ▼         ▼   │
│ topics/  research/  drafts/  drafts/   published/  metrics   rewrites/
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

## Pre-Execution Context Loading

Before any content operation, load these context files:

1. **Brand Voice**: `docs/brand/BRAND_IDENTITY_GUIDE.md`
   - Voice pillars and personality
   - Tone guidelines by content type
   - Terminology preferences

2. **SEO Guidelines**: `docs/content/seo-guidelines.md`
   - Keyword density requirements (1-2%)
   - Meta element specifications
   - Link strategy minimums

3. **Target Keywords**: `docs/content/target-keywords.md`
   - Primary keyword clusters
   - Topic cluster mapping
   - Priority rankings

4. **Internal Links Map**: `docs/content/internal-links-map.md`
   - Core product pages
   - High-traffic blog posts
   - Anchor text guidelines

5. **Writing Examples**: `docs/content/writing-examples.md`
   - Exemplary blog posts
   - Style patterns to follow

---

## Action: Research (`/blog-content research [topic]`)

### Purpose
Create comprehensive research brief before writing.

### Steps

1. **Keyword Analysis**
   - Identify primary keyword for topic
   - Find related keywords and questions
   - Assess search volume and difficulty
   - Classify search intent (informational/commercial/transactional)

2. **Competitive Analysis**
   - Analyze top 10 SERP results for primary keyword
   - Note content length of competitors
   - Identify common themes and gaps
   - Find unique angle opportunities

3. **Content Brief Creation**
   Generate brief including:
   - Primary and secondary keywords
   - Recommended word count (based on competitors + 10%)
   - Suggested H1 and H2 structure
   - Key points to cover
   - Internal linking opportunities
   - External sources to cite
   - Unique angle/differentiator

### Output

Save to: `content/research/brief-[topic-slug]-[YYYY-MM-DD].md`

```markdown
# Research Brief: [Topic]

**Date:** [YYYY-MM-DD]
**Author:** [Name]
**Status:** Ready for Writing

## SEO Foundation

| Metric | Value |
|--------|-------|
| Primary Keyword | [keyword] |
| Search Volume | [volume] |
| Difficulty | [low/medium/high] |
| Search Intent | [informational/commercial/transactional] |
| Target Word Count | [X,XXX] |

## Secondary Keywords
- [keyword 1]
- [keyword 2]
- [keyword 3]

## Competitive Landscape

| Competitor | Word Count | Key Strengths | Gaps |
|------------|------------|---------------|------|
| [URL 1] | X,XXX | ... | ... |
| [URL 2] | X,XXX | ... | ... |
| [URL 3] | X,XXX | ... | ... |

## Our Unique Angle
[What makes our take different and valuable]

## Recommended Structure

### H1: [Proposed Title]

#### H2: [Section 1]
- Key points to cover

#### H2: [Section 2]
- Key points to cover

[etc.]

## Internal Links to Include
- [Page 1] - anchor text suggestion
- [Page 2] - anchor text suggestion

## External Sources
- [Source 1] - for [topic/claim]
- [Source 2] - for [topic/claim]

## Visual Assets Needed
- [ ] Hero image
- [ ] Diagrams/screenshots
- [ ] Code examples
```

---

## Action: Write (`/blog-content write [topic]`)

### Purpose
Generate SEO-optimized long-form article from research brief.

### Prerequisites
- Research brief must exist in `content/research/`
- All context files loaded

### Content Requirements

#### Structure
- **H1 (Title)**: Include primary keyword, <60 chars, compelling value
- **Introduction**: 150-200 words, primary keyword in first 100 words, hook + problem + promise
- **Body**: 1800-2500+ words, 4-7 H2 sections, logical flow
- **Conclusion**: 150-200 words, summary + next steps + CTA

#### SEO Checklist
- [ ] Primary keyword in H1
- [ ] Primary keyword in first 100 words
- [ ] Primary keyword in 2+ H2 headings
- [ ] Primary keyword in conclusion
- [ ] Keyword density: 1-2%
- [ ] 3-5+ internal links with descriptive anchors
- [ ] 2-3 external authority links
- [ ] Meta title: 50-60 characters
- [ ] Meta description: 150-160 characters

#### Readability
- [ ] Grade level: 8th-10th
- [ ] Sentences: <20 words average
- [ ] Paragraphs: 2-4 sentences
- [ ] Passive voice: <20%
- [ ] Active voice preferred
- [ ] Code examples where relevant

### Post-Write Analysis

After generating content, automatically run these analyses:

1. **Keyword Analysis**
   - Density calculation for primary/secondary keywords
   - Placement verification (H1, H2s, intro, conclusion)
   - Natural integration assessment
   - Stuffing risk check

2. **Readability Analysis**
   - Flesch Reading Ease score
   - Flesch-Kincaid Grade Level
   - Sentence length distribution
   - Passive voice percentage

3. **SEO Quality Check**
   - Meta element validation
   - Heading hierarchy check
   - Link count and quality
   - Content length vs. target

4. **Meta Element Generation**
   Generate 3 options each for:
   - Title tags (50-60 chars)
   - Meta descriptions (150-160 chars)
   Include SERP preview mockups

5. **Internal Link Suggestions**
   - Recommend 3-5 internal links
   - Suggest exact placement in content
   - Provide anchor text options

### Output

Save to: `content/drafts/[topic-slug]-[YYYY-MM-DD].md`

```markdown
---
title: "[H1 Title]"
description: "[Meta description]"
keywords: ["primary", "secondary1", "secondary2"]
author: "[Name]"
date: [YYYY-MM-DD]
status: draft
research_brief: "research/brief-[topic-slug]-[date].md"
---

# [H1 Title]

[Article content...]

---

## SEO Analysis Summary

| Metric | Value | Target | Status |
|--------|-------|--------|--------|
| Word Count | X,XXX | 2,000+ | [Pass/Fail] |
| Primary Keyword Density | X.X% | 1-2% | [Pass/Fail] |
| Internal Links | X | 3-5+ | [Pass/Fail] |
| External Links | X | 2-3 | [Pass/Fail] |
| Readability Grade | Xth | 8-10th | [Pass/Fail] |
| SEO Score | XX/100 | 80+ | [Pass/Fail] |

## Meta Options

### Title Options
1. [Title option 1] (XX chars)
2. [Title option 2] (XX chars)
3. [Title option 3] (XX chars)

### Description Options
1. [Description 1] (XXX chars)
2. [Description 2] (XXX chars)
3. [Description 3] (XXX chars)

## Suggested Internal Links
1. "[Anchor text]" → [URL] - in [section]
2. "[Anchor text]" → [URL] - in [section]
3. "[Anchor text]" → [URL] - in [section]

## Recommendations
- [High priority fix 1]
- [Medium priority improvement 1]
- [Optional enhancement 1]
```

---

## Action: Optimize (`/blog-content optimize [file]`)

### Purpose
Final SEO audit and publishing readiness check.

### Analysis Performed

1. **SEO Quality Rating (0-100)**
   - Content quality score
   - Keyword optimization score
   - Structure score
   - Link quality score
   - Readability score
   - Overall composite score

2. **Critical Issues Check**
   - Missing primary keyword in H1
   - Missing meta elements
   - Broken links
   - Heading hierarchy errors
   - Extremely low/high keyword density

3. **Quick Wins Identification**
   - Easy improvements for immediate impact
   - Keyword placement adjustments
   - Link additions
   - Meta element tweaks

4. **Featured Snippet Optimization**
   - Identify opportunities (lists, tables, definitions)
   - Format content for snippet capture
   - Question-answer structure opportunities

### Publishing Readiness Checklist

```markdown
## Publishing Readiness Checklist

### Critical (Must Fix Before Publishing)
- [ ] SEO Score: 80+
- [ ] All meta elements present
- [ ] Primary keyword in H1, intro, conclusion
- [ ] Minimum 3 internal links
- [ ] No broken links
- [ ] Proper heading hierarchy

### Recommended (Should Fix)
- [ ] Keyword density within 1-2%
- [ ] Readability at 8th-10th grade
- [ ] 2+ external authority links
- [ ] All images have alt text
- [ ] Code examples tested

### Optional (Nice to Have)
- [ ] Featured snippet optimization
- [ ] Schema markup recommendations
- [ ] Social media preview images
```

### Output

Updates the draft file with optimization report and moves to `content/published/` when ready.

---

## Action: Analyze (`/blog-content analyze [URL or file]`)

### Purpose
Evaluate existing content for improvement opportunities.

### Analysis Modules

1. **Search Intent Classification**
   - Informational vs Commercial vs Transactional
   - Alignment with target intent
   - Recommendations for better alignment

2. **Keyword Analysis**
   - Current keyword density
   - Keyword distribution heatmap
   - Missing keyword opportunities
   - Competitor keyword comparison

3. **Content Length Comparison**
   - Word count vs. top 10 competitors
   - Section length analysis
   - Recommendations for expansion/contraction

4. **Readability Scoring**
   - Flesch Reading Ease
   - Flesch-Kincaid Grade Level
   - Sentence complexity
   - Passive voice percentage

5. **SEO Quality Rating**
   - Comprehensive 0-100 score
   - Category breakdowns
   - Specific improvement recommendations

### Output

```markdown
# Content Analysis: [Title/URL]

**Analyzed:** [Date]
**Overall Health Score:** XX/100

## Executive Summary
[2-3 sentence summary of content health and priority actions]

## Quick Wins (High Impact, Low Effort)
1. [Quick win 1]
2. [Quick win 2]

## Strategic Improvements (High Impact, Higher Effort)
1. [Strategic improvement 1]
2. [Strategic improvement 2]

## Detailed Scores

| Category | Score | Benchmark | Status |
|----------|-------|-----------|--------|
| Content Quality | XX/100 | 75+ | [icon] |
| Keyword Optimization | XX/100 | 80+ | [icon] |
| Technical SEO | XX/100 | 85+ | [icon] |
| Readability | XX/100 | 70+ | [icon] |
| Link Profile | XX/100 | 75+ | [icon] |

## Recommendations by Priority

### Critical
- ...

### High
- ...

### Medium
- ...

### Low
- ...
```

---

## Action: Review (`/blog-content review`)

### Purpose
Data-driven content prioritization and performance review.

### Analysis

1. **Quick Wins Identification**
   - Content ranking 11-20 (push to page 1)
   - High impressions, low CTR (meta optimization)
   - Declining rankings (refresh needed)

2. **Content Gaps**
   - Keywords with no content
   - Competitor topics we're missing
   - User questions not answered

3. **Performance Trends**
   - Traffic trends by post
   - Engagement metrics
   - Conversion attribution

### Output

```markdown
# Content Performance Review

**Period:** [Date Range]
**Generated:** [Date]

## Priority Actions

### Tier 1: Quick Wins (This Week)
| Content | Current Rank | Action | Expected Impact |
|---------|--------------|--------|-----------------|
| [Post 1] | #12 | Optimize meta + add section | Page 1 ranking |
| [Post 2] | #8 | Update stats + refresh intro | +20% CTR |

### Tier 2: Content Refresh (This Month)
| Content | Issue | Recommended Action |
|---------|-------|-------------------|
| [Post 3] | Outdated info | Full content update |
| [Post 4] | Thin content | Expand by 1000 words |

### Tier 3: New Content Opportunities
| Topic | Search Volume | Competition | Priority |
|-------|---------------|-------------|----------|
| [Topic 1] | X,XXX | Low | High |
| [Topic 2] | X,XXX | Medium | Medium |

## Content Health Overview

| Status | Count | Percentage |
|--------|-------|------------|
| Healthy (80+ score) | X | XX% |
| Needs Attention (60-79) | X | XX% |
| Critical (<60) | X | XX% |
```

---

## Best Practices

### Before Writing
1. Always run `/blog-content research` first
2. Review research brief thoroughly
3. Check for existing content on topic
4. Identify unique angle

### During Writing
1. Keep context files open for reference
2. Follow heading hierarchy strictly
3. Add internal links as you write
4. Check keyword density periodically

### After Writing
1. Run `/blog-content optimize` before publishing
2. Address all critical issues
3. Review all recommendations
4. Get human review for quality

### Content Maintenance
1. Run `/blog-content review` monthly
2. Update content based on performance data
3. Refresh outdated posts quarterly
4. Monitor ranking changes

---

## File Structure

```
content/
├── topics/                     # Content ideas backlog
│   └── ideas.md               # Running list of topics
├── research/                   # Research briefs
│   └── brief-[topic]-[date].md
├── drafts/                     # Work in progress
│   └── [topic]-[date].md
├── published/                  # Final versions
│   └── [topic].md
└── rewrites/                   # Content updates
    └── [topic]-v2-[date].md

docs/content/
├── seo-guidelines.md          # SEO standards
├── target-keywords.md         # Keyword strategy
├── internal-links-map.md      # Link strategy
└── writing-examples.md        # Style reference
```

---

## Integration with Existing Skills

This skill can be used alongside:

- `/caro.feature` - For technical feature blog posts
- `/spec-kitty.specify` - For specification-based content
- `/technical-writer` agent - For documentation-style content

---

## Metrics & Success Criteria

| Metric | Target | Measurement |
|--------|--------|-------------|
| SEO Score | 80+ | `/blog-content optimize` |
| Readability | 8th-10th grade | Flesch-Kincaid |
| Word Count | 2,000+ | Word count |
| Internal Links | 3-5+ | Link count |
| Organic Traffic | +10% MoM | Analytics |
| Avg Position | Top 10 | Search Console |
