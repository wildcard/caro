# Caro SEO Guidelines

> **Last Updated:** January 2026
> **Owner:** DevRel Team

This document defines SEO standards for all Caro blog content. Follow these guidelines to maximize organic search visibility while maintaining content quality.

---

## Keyword Strategy

### Density Requirements

| Keyword Type | Target Density | Acceptable Range |
|--------------|----------------|------------------|
| Primary | 1.5% | 1.0-2.0% |
| Secondary | 0.75% | 0.5-1.0% |
| Related/LSI | Natural | No forced placement |

**Warning Thresholds:**
- Below 0.5%: Under-optimized
- Above 3.0%: Keyword stuffing risk

### Placement Requirements

Primary keyword MUST appear in:
- [ ] H1 (exact or close match)
- [ ] First 100 words of introduction
- [ ] At least 2 H2 headings (natural integration)
- [ ] Conclusion paragraph
- [ ] Meta title
- [ ] Meta description
- [ ] URL slug

Secondary keywords should appear in:
- [ ] At least 1 H2 or H3 heading
- [ ] Body paragraphs (naturally distributed)
- [ ] Image alt text where relevant

---

## Content Structure

### Heading Hierarchy

```
H1: Main Title (only ONE per page)
  └── H2: Major Section 1
        └── H3: Subsection 1.1
        └── H3: Subsection 1.2
  └── H2: Major Section 2
        └── H3: Subsection 2.1
  └── H2: Major Section 3
  └── H2: Conclusion
```

**Rules:**
- Single H1 per article (the title)
- 4-7 H2 sections for comprehensive coverage
- Use H3 for subsections within H2s
- Never skip levels (H1 → H3 is invalid)
- Headings should be descriptive, not clever

### Section Structure

Each major section (H2) should:
- Start with a brief intro sentence
- Contain 200-400 words
- Include at least one of: code example, list, or visual
- Flow logically to the next section

### Paragraph Standards

| Metric | Target |
|--------|--------|
| Sentences per paragraph | 2-4 |
| Words per paragraph | 50-100 |
| Paragraphs per section | 3-6 |

**Avoid:**
- Walls of text (>5 sentences)
- Single-sentence paragraphs (except for emphasis)
- Overly long sentences (>25 words)

---

## Meta Elements

### Title Tag

| Requirement | Specification |
|-------------|---------------|
| Length | 50-60 characters (absolute max: 65) |
| Primary keyword | Include near the beginning |
| Branding | Add "| Caro" at end if space allows |
| Style | Compelling, clear value proposition |

**Examples:**
- "AI Command Line Safety: A Complete Guide | Caro" (52 chars)
- "How to Prevent rm -rf Disasters with AI" (41 chars)
- "Local-First AI for Shell Commands: Why It Matters" (51 chars)

**Avoid:**
- Clickbait ("You Won't Believe...")
- All caps
- Keyword stuffing
- Duplicate titles across pages

### Meta Description

| Requirement | Specification |
|-------------|---------------|
| Length | 150-160 characters (absolute max: 165) |
| Primary keyword | Include naturally |
| CTA | Include action word or value proposition |
| Uniqueness | Each page needs unique description |

**Examples:**
- "Learn how Caro's AI-powered CLI keeps your commands safe. Validate before execution, prevent accidents, and work faster with local AI. Free and open source." (156 chars)
- "Tired of Googling shell commands? Caro converts natural language to safe, platform-aware commands. Works offline. Install in 10 seconds." (138 chars)

**Avoid:**
- Truncated sentences
- Duplicate descriptions
- Missing call-to-action
- Pure keyword lists

---

## Link Strategy

### Internal Links

| Metric | Minimum | Target |
|--------|---------|--------|
| Per article | 3 | 5-7 |
| Unique destinations | 3 | 5+ |

**Rules:**
- Use descriptive anchor text (not "click here")
- Link to relevant product pages when appropriate
- Link to related blog posts
- Distribute links throughout content (not all at end)
- Vary anchor text for same destination

**Priority destinations:**
1. Core product pages (installation, features, safety)
2. High-traffic blog posts
3. Documentation pages
4. Conversion pages (enterprise, pricing)

### External Links

| Metric | Minimum | Target |
|--------|---------|--------|
| Per article | 2 | 3-4 |
| Authority sites | 100% | 100% |

**Rules:**
- Link to authoritative sources only
- Support claims with citations
- Use `rel="noopener"` for security
- Avoid linking to competitors
- Prefer evergreen sources (not dated news)

**Good sources:**
- Official documentation
- Academic research
- Industry standards (OWASP, NIST)
- Reputable tech publications

---

## Readability

### Target Metrics

| Metric | Target | Acceptable Range |
|--------|--------|------------------|
| Flesch Reading Ease | 60-70 | 50-80 |
| Flesch-Kincaid Grade | 8th-10th | 7th-12th |
| Avg sentence length | 15-18 words | 12-22 |
| Passive voice | <15% | <20% |

### Writing Style

**Do:**
- Use active voice
- Keep sentences concise
- Explain technical terms on first use
- Use examples and analogies
- Include code samples for technical topics
- Use bullet points for lists
- Break up complex ideas

**Don't:**
- Use jargon without explanation
- Write run-on sentences
- Use overly academic language
- Assume deep technical knowledge
- Use buzzwords without substance

### Technical Content

For technical blog posts:
- Include working code examples
- Test all commands before publishing
- Provide context for why, not just how
- Link to documentation for deeper dives
- Consider copy-paste usability

---

## Content Length

### Word Count Guidelines

| Content Type | Minimum | Target | Maximum |
|--------------|---------|--------|---------|
| Tutorial | 1,500 | 2,500 | 4,000 |
| How-To Guide | 1,200 | 2,000 | 3,000 |
| Comparison | 2,000 | 3,000 | 4,000 |
| Deep Dive | 2,500 | 3,500 | 5,000 |
| News/Announcement | 500 | 800 | 1,500 |
| Case Study | 1,500 | 2,000 | 3,000 |

### Competitive Benchmarking

Before writing, analyze top 10 SERP results for:
- Average word count
- Common sections/topics
- Content gaps to fill

**Target:** 10-20% more comprehensive than top competitor.

---

## Featured Snippets

### Optimization Strategies

**For Definition Queries:**
- Include a concise 40-60 word definition
- Start with "X is..." or "X refers to..."
- Place immediately after relevant H2

**For List Queries:**
- Use numbered or bulleted lists
- Keep items concise (1-2 sentences)
- Include 5-8 items
- Add a brief intro before the list

**For Table Queries:**
- Use markdown tables for comparisons
- Include clear headers
- Limit to 4-6 columns, 5-10 rows

**For How-To Queries:**
- Use numbered steps
- Start each step with an action verb
- Keep steps to 1-2 sentences
- Include 5-10 steps

---

## Image Optimization

### Requirements

| Element | Requirement |
|---------|-------------|
| Alt text | Descriptive, keyword-relevant |
| File name | Descriptive, hyphenated |
| Format | WebP preferred, PNG/JPG acceptable |
| Size | <200KB for standard, <500KB for hero |
| Dimensions | Appropriate for container |

### Alt Text Guidelines

**Good:** "Terminal screenshot showing Caro blocking a dangerous rm -rf command"
**Bad:** "image1.png" or "screenshot"

---

## URL Structure

### Best Practices

- Use lowercase letters
- Separate words with hyphens (not underscores)
- Include primary keyword
- Keep under 60 characters
- Remove stop words (a, the, and, or)
- Avoid dates in URLs (for evergreen content)

**Examples:**
- `/blog/ai-command-line-safety` (Good)
- `/blog/how-to-use-ai-for-safe-cli-commands-2026` (Too long, has date)
- `/blog/post123` (Bad - not descriptive)

---

## Quality Checklist

Before publishing, verify:

### Critical
- [ ] Primary keyword in H1, intro, and conclusion
- [ ] Meta title 50-60 characters with primary keyword
- [ ] Meta description 150-160 characters with CTA
- [ ] Minimum 3 internal links
- [ ] No broken links
- [ ] Proper heading hierarchy (single H1)

### Important
- [ ] Word count meets target for content type
- [ ] Keyword density 1-2%
- [ ] Readability at 8th-10th grade level
- [ ] 2+ external authority links
- [ ] All images have alt text
- [ ] Code examples tested and working

### Recommended
- [ ] Featured snippet optimization where applicable
- [ ] Related posts suggestions
- [ ] Clear CTA in conclusion
- [ ] Social sharing meta tags (OG, Twitter)

---

## Common SEO Mistakes to Avoid

1. **Keyword Stuffing**: Forcing keywords unnaturally
2. **Thin Content**: Under 1,000 words without good reason
3. **Duplicate Content**: Copy-pasting from other sources
4. **Missing Meta Elements**: No title or description
5. **Broken Links**: Links that 404
6. **Heading Abuse**: Using H2-H6 for styling
7. **Ignoring Mobile**: Content that doesn't render well on mobile
8. **No Internal Links**: Orphaned pages
9. **Outdated Content**: Statistics or examples from >2 years ago
10. **Generic Titles**: "Blog Post #5" or "Untitled"

---

## Tools & Resources

### Recommended Tools
- **Readability**: Hemingway Editor, Readable.com
- **SEO Analysis**: Screaming Frog, Ahrefs, SEMrush
- **Keyword Research**: Google Keyword Planner, Ahrefs
- **Link Checking**: Check My Links (Chrome extension)

### Reference Documents
- `docs/brand/BRAND_IDENTITY_GUIDE.md` - Voice and tone
- `docs/content/target-keywords.md` - Keyword strategy
- `docs/content/internal-links-map.md` - Link targets
- `docs/content/writing-examples.md` - Style reference
