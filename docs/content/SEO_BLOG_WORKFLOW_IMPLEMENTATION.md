# SEO Blog Content Workflow Implementation Plan

> **Status:** Implementation Plan
> **Based on:** Analysis of [SEO Machine](https://github.com/TheCraigHewitt/seomachine)
> **Created:** January 2026

---

## Executive Summary

This document outlines improvements to Caro's blog content workflow based on learnings from the SEO Machine project—a Claude Code workspace designed for long-form, SEO-optimized blog content creation. The goal is to adopt proven patterns for structured content lifecycle management, SEO optimization, and automated quality analysis.

---

## Part 1: Current State Analysis

### What Caro Already Has (Strong Foundation)

| Asset | Location | Status |
|-------|----------|--------|
| Brand Identity Guide | `docs/brand/BRAND_IDENTITY_GUIDE.md` | Comprehensive |
| Social Media Guide | `docs/devrel/SOCIAL_MEDIA_GUIDE.md` | Comprehensive |
| Landing Page Strategy | `docs/marketing/LANDING_PAGE_EXPANSION_STRATEGY.md` | Comprehensive |
| Technical Writer Agent | `.claude/agents/technical-writer.md` | Basic |

### What's Missing (Opportunities)

| Gap | SEO Machine Has | Caro Needs |
|-----|-----------------|------------|
| Content lifecycle commands | `/research`, `/write`, `/optimize`, `/rewrite` | Blog-specific skills |
| SEO analysis agents | Content Analyzer, SEO Optimizer, Meta Creator | SEO-focused agents |
| Content directories | `topics/`, `research/`, `drafts/`, `published/` | Structured content folders |
| Writing examples | `context/writing-examples.md` | Sample blog posts |
| SEO guidelines | `context/seo-guidelines.md` | SEO standards document |
| Target keywords | `context/target-keywords.md` | Keyword strategy document |
| Internal links map | `context/internal-links-map.md` | Link strategy document |
| Analytics integration | GA4, Search Console, DataForSEO | Performance tracking |

---

## Part 2: SEO Machine Workflow Analysis

### Content Lifecycle Flow

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   TOPICS    │────▶│  RESEARCH   │────▶│   DRAFTS    │────▶│  PUBLISHED  │
│             │     │             │     │             │     │             │
│ Ideas queue │     │ SEO briefs  │     │ WIP content │     │ Final posts │
└─────────────┘     └─────────────┘     └─────────────┘     └─────────────┘
                                                                   │
                                                                   ▼
                                                            ┌─────────────┐
                                                            │  REWRITES   │
                                                            │             │
                                                            │Updated posts│
                                                            └─────────────┘
```

### Command Workflow

| Command | Purpose | Auto-Triggers |
|---------|---------|---------------|
| `/research [topic]` | Keyword analysis, competitor review, content brief | None |
| `/write [topic]` | Generate 2000-3000+ word SEO-optimized article | Content Analyzer, SEO Optimizer, Meta Creator, Internal Linker, Keyword Mapper |
| `/optimize [file]` | Final SEO audit before publishing | All analysis agents |
| `/rewrite [topic]` | Update existing content based on performance | Analysis agents |
| `/analyze-existing [URL/file]` | Evaluate content health score (0-100) | None |
| `/performance-review` | Data-driven content prioritization | Performance agent |

### Agent Responsibilities

| Agent | Function |
|-------|----------|
| **Content Analyzer** | 5-module analysis: search intent, keyword density, content length, readability, SEO quality (0-100 score) |
| **SEO Optimizer** | On-page optimization: keyword placement, structure, links, meta elements, featured snippets |
| **Meta Creator** | Generates 5 title/description variations with SERP previews |
| **Internal Linker** | Recommends 3-5 strategic internal links with exact placements |
| **Keyword Mapper** | Analyzes keyword distribution, density, natural integration |
| **Editor** | Assesses "humanity score" and injects personality into content |
| **Performance Agent** | Integrates GA4, Search Console, DataForSEO for data-driven decisions |

### Context Files Structure

```
context/
├── brand-voice.md          # Voice pillars, tone guidelines, terminology
├── writing-examples.md     # 3-5 exemplary articles demonstrating style
├── style-guide.md          # Writing conventions, formatting rules
├── seo-guidelines.md       # SEO standards, keyword requirements
├── target-keywords.md      # Keyword research, topic clusters
├── internal-links-map.md   # Key pages for strategic linking
├── competitor-analysis.md  # Competitive intelligence
└── features.md             # Product feature documentation
```

---

## Part 3: Implementation Plan

### Phase 1: Foundation (Week 1-2)

#### 1.1 Create Content Directory Structure

```
content/
├── topics/                 # Content ideas and backlog
│   └── .gitkeep
├── research/              # Research briefs
│   └── .gitkeep
├── drafts/                # Work-in-progress articles
│   └── .gitkeep
├── published/             # Finalized content
│   └── .gitkeep
└── rewrites/              # Content update queue
    └── .gitkeep
```

#### 1.2 Create SEO Context Files

**File 1: `docs/content/seo-guidelines.md`**

Content should include:
- Target keyword density (1-2%)
- Title/meta description requirements
- Heading hierarchy standards (H1, H2, H3)
- Internal linking minimums (3-5 per article)
- External citation requirements
- Readability targets (8th-10th grade)
- Content length benchmarks (2000-3000+ words)
- Featured snippet optimization

**File 2: `docs/content/target-keywords.md`**

Content should include:
- Primary keyword clusters
- Search volume and difficulty data
- Topic clusters and pillar pages
- Long-tail keyword opportunities
- Competitor keyword gaps

**File 3: `docs/content/internal-links-map.md`**

Content should include:
- Core product pages
- Documentation pages
- High-traffic blog posts
- Conversion pages
- Anchor text guidelines

**File 4: `docs/content/writing-examples.md`**

Content should include:
- 3-5 exemplary blog posts
- Analysis of what makes each effective
- Style notes and patterns to follow

### Phase 2: Skills Implementation (Week 3-4)

#### 2.1 Blog Research Skill

Create `.claude/skills/blog-research/SKILL.md`:

```yaml
name: blog-research
description: Perform keyword research, competitive analysis, and create content briefs
invocation: /blog-research [topic]
```

**Capabilities:**
- Keyword research and search volume analysis
- Top 10 SERP competitor analysis
- Content gap identification
- Unique angle development
- Research brief generation

**Output:** `research/brief-[topic-slug]-[YYYY-MM-DD].md`

#### 2.2 Blog Write Skill

Create `.claude/skills/blog-write/SKILL.md`:

```yaml
name: blog-write
description: Generate SEO-optimized long-form blog articles
invocation: /blog-write [topic]
```

**Pre-write checks:**
- Load brand voice context
- Load writing examples
- Load SEO guidelines
- Load target keywords
- Check for existing research brief

**Content requirements:**
- 2000-3000+ words
- Primary keyword in H1, first 100 words, 2+ H2s
- 1-2% keyword density
- 3-5 internal links
- 2-3 external authority links
- Meta title (50-60 chars) and description (150-160 chars)
- 8th-10th grade readability

**Auto-triggers after write:**
1. SEO Analyzer agent
2. Meta Creator agent
3. Internal Linker agent
4. Keyword Mapper agent

**Output:** `drafts/[topic-slug]-[YYYY-MM-DD].md`

#### 2.3 Blog Optimize Skill

Create `.claude/skills/blog-optimize/SKILL.md`:

```yaml
name: blog-optimize
description: Final SEO audit and publishing readiness check
invocation: /blog-optimize [file]
```

**Analysis includes:**
- Keyword placement verification
- Meta element validation
- Link health check
- Readability score
- SEO quality score (0-100)
- Publishing readiness assessment

**Output:** Optimization recommendations with priority ranking

#### 2.4 Blog Analyze Skill

Create `.claude/skills/blog-analyze/SKILL.md`:

```yaml
name: blog-analyze
description: Evaluate existing content for improvement opportunities
invocation: /blog-analyze [URL or file]
```

**Analysis modules:**
- Search intent classification
- Keyword density and distribution
- Content length vs. competitors
- Readability metrics (Flesch scores)
- SEO quality rating (0-100)

**Output:** Content health score with quick wins and strategic improvements

### Phase 3: Agent Implementation (Week 5-6)

#### 3.1 SEO Analyzer Agent

Create `.claude/agents/seo-analyzer.md`:

**Responsibilities:**
- Keyword optimization analysis (density, placement, stuffing risk)
- Content structure evaluation (heading hierarchy, paragraph length)
- Link strategy assessment (internal, external, anchor text)
- Technical SEO checks (meta elements, URL structure)
- Readability scoring (Flesch, grade level, sentence complexity)
- Featured snippet opportunity identification

**Scoring criteria:**
- Content quality: 0-100
- Keyword optimization: 0-100
- Structure: 0-100
- Links: 0-100
- Readability: 0-100
- Overall SEO score: 0-100

#### 3.2 Meta Creator Agent

Create `.claude/agents/meta-creator.md`:

**Responsibilities:**
- Generate 5 title variations (50-60 chars)
- Generate 5 description variations (150-160 chars)
- SERP preview mockups
- Click-through rate optimization tips
- Power word suggestions

#### 3.3 Content Performance Agent

Create `.claude/agents/content-performance.md`:

**Responsibilities:**
- Analyze content performance metrics
- Identify quick wins (rankings 11-20)
- Prioritize content updates
- Track keyword position changes
- Recommend content refresh opportunities

### Phase 4: Integration & Automation (Week 7-8)

#### 4.1 Workflow Automation

- Auto-trigger agents after content generation
- Automatic file organization (research → drafts → published)
- Content scrubber for Unicode cleanup
- Performance tracking integration

#### 4.2 Quality Gates

Before publishing, content must:
- Score 80+ on SEO quality rating
- Have 3+ internal links
- Pass readability check (8th-10th grade)
- Have optimized meta elements
- Be reviewed by editor agent for "humanity score"

---

## Part 4: Context File Templates

### 4.1 SEO Guidelines Template

```markdown
# Caro SEO Guidelines

## Keyword Strategy

### Density Requirements
- Primary keyword: 1.0-2.0%
- Secondary keywords: 0.5-1.0% each
- Avoid keyword stuffing (>3%)

### Placement Requirements
- Primary keyword in:
  - H1 (exact match)
  - First 100 words
  - At least 2 H2 headings
  - Conclusion
  - Meta title
  - Meta description

## Content Structure

### Heading Hierarchy
- Single H1 (article title)
- 4-7 H2 sections
- H3 for subsections within H2s
- No skipping levels (H1 → H3)

### Paragraph Standards
- 2-4 sentences per paragraph
- No walls of text
- Use bullet points for lists
- Include relevant code examples

## Meta Elements

### Title Tag
- Length: 50-60 characters
- Include primary keyword
- Compelling value proposition
- Brand at end (if space): "| Caro"

### Meta Description
- Length: 150-160 characters
- Include primary keyword naturally
- Clear value proposition
- Call to action

## Link Strategy

### Internal Links
- Minimum: 3-5 per article
- Use descriptive anchor text
- Link to relevant product pages
- Link to related blog posts

### External Links
- Minimum: 2-3 per article
- Link to authoritative sources
- Use rel="noopener" for external links
- Avoid linking to competitors

## Readability

### Target Metrics
- Flesch Reading Ease: 60-70
- Grade Level: 8th-10th
- Average sentence length: <20 words
- Passive voice: <20%

### Writing Style
- Active voice preferred
- Short sentences for clarity
- Explain technical terms
- Use examples and analogies
```

### 4.2 Target Keywords Template

```markdown
# Caro Target Keywords

## Primary Keyword Clusters

### Cluster 1: AI CLI Tools
| Keyword | Volume | Difficulty | Priority |
|---------|--------|------------|----------|
| AI command line tool | 500 | Medium | High |
| natural language CLI | 200 | Low | High |
| AI shell commands | 300 | Medium | High |

### Cluster 2: Command Safety
| Keyword | Volume | Difficulty | Priority |
|---------|--------|------------|----------|
| safe shell commands | 150 | Low | High |
| command line safety | 100 | Low | Medium |
| prevent rm rf | 80 | Low | High |

### Cluster 3: Local AI
| Keyword | Volume | Difficulty | Priority |
|---------|--------|------------|----------|
| local AI tools | 400 | Medium | High |
| offline AI CLI | 100 | Low | High |
| privacy-first AI | 200 | Low | High |

## Topic Clusters

### Pillar: AI-Powered CLI Tools
- What is an AI command line tool?
- How AI generates shell commands
- Comparing AI CLI tools
- Safety considerations for AI commands

### Pillar: Command Line Safety
- Understanding dangerous shell commands
- How to prevent accidental file deletion
- Best practices for production commands
- Command validation and safety patterns

### Pillar: Local-First Development
- Benefits of local AI processing
- Privacy-focused development tools
- Air-gapped environment tools
- Self-hosted AI solutions
```

### 4.3 Internal Links Map Template

```markdown
# Caro Internal Links Map

## Core Product Pages

| Page | URL | Anchor Text Options |
|------|-----|---------------------|
| Homepage | caro.sh | Caro, shell companion, CLI tool |
| Documentation | caro.sh/docs | documentation, getting started, learn more |
| Installation | caro.sh/install | install Caro, get started, download |
| Safety | caro.sh/ai-command-safety | command safety, safety validation |

## Key Blog Posts

| Post | URL | Topics |
|------|-----|--------|
| Understanding Command Safety | /blog/command-safety | safety, validation, dangerous commands |
| Getting Started with Caro | /blog/getting-started | installation, setup, first commands |
| Local AI Benefits | /blog/local-first-ai | privacy, offline, local processing |

## Conversion Pages

| Page | When to Link |
|------|--------------|
| Enterprise | Enterprise/security context |
| GitHub | Open source/contribution context |
| Pricing | Feature comparison context |

## Anchor Text Guidelines

- Use descriptive, keyword-rich anchor text
- Avoid "click here" or "learn more"
- Vary anchor text for same destination
- Match anchor to target page content
```

---

## Part 5: Success Metrics

### Content Quality Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| SEO Score | 80+ | Blog analyze skill |
| Readability | 8th-10th grade | Flesch-Kincaid |
| Word Count | 2000-3000+ | Word count tool |
| Internal Links | 3-5+ | Manual count |
| Keyword Density | 1-2% | Keyword analyzer |

### Content Performance Metrics

| Metric | Target | Source |
|--------|--------|--------|
| Organic Traffic | +50% MoM | Google Analytics |
| Average Position | Top 10 | Search Console |
| Click-Through Rate | 3%+ | Search Console |
| Time on Page | 3+ minutes | Google Analytics |
| Bounce Rate | <60% | Google Analytics |

### Workflow Efficiency Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Research to Draft | <2 hours | Time tracking |
| Draft to Published | <1 week | Git history |
| Content Updates | Monthly | Publishing cadence |

---

## Part 6: Quick Start Checklist

### Immediate Actions (This Week)

- [ ] Create `content/` directory structure
- [ ] Create `docs/content/seo-guidelines.md`
- [ ] Create `docs/content/target-keywords.md`
- [ ] Create `docs/content/internal-links-map.md`

### Short-Term Actions (Next 2 Weeks)

- [ ] Create `/blog-research` skill
- [ ] Create `/blog-write` skill
- [ ] Create `/blog-optimize` skill
- [ ] Create SEO Analyzer agent

### Medium-Term Actions (Next Month)

- [ ] Create Meta Creator agent
- [ ] Create Content Performance agent
- [ ] Implement auto-trigger workflow
- [ ] Create writing examples document

---

## Appendix: SEO Machine Key Patterns

### Pattern 1: Context-Driven Generation

SEO Machine loads 5 context files before every write:
1. Brand voice
2. Writing examples
3. Style guide
4. SEO guidelines
5. Target keywords

This ensures consistent output quality.

### Pattern 2: Auto-Triggered Agents

After content generation, 5 agents automatically run:
1. Content Analyzer (comprehensive scoring)
2. SEO Optimizer (optimization suggestions)
3. Meta Creator (title/description options)
4. Internal Linker (link placements)
5. Keyword Mapper (density analysis)

This provides immediate feedback without manual intervention.

### Pattern 3: File Organization

Content automatically saves to appropriate directories:
- Research → `research/brief-[topic]-[date].md`
- Drafts → `drafts/[topic]-[date].md`
- Published → `published/[topic].md`

This maintains clear content lifecycle tracking.

### Pattern 4: Performance-Driven Prioritization

The Performance agent identifies:
- Quick wins (rankings 11-20)
- Declining content
- High-impression/low-click content
- Content gaps vs competitors

This ensures data-driven content decisions.

---

*Document created based on analysis of [SEO Machine](https://github.com/TheCraigHewitt/seomachine). Adapt patterns to Caro's specific needs and existing infrastructure.*
