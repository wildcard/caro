# Idea Sourcing Loop - Design Requirements Specification

> **Document Type**: DRS
> **Version**: 1.0.0
> **Status**: Active
> **Parent**: [AUTOMATED_DEV_FLOW_DRS.md](./AUTOMATED_DEV_FLOW_DRS.md)
> **Pack**: Content

---

## 1. Overview

The Idea Sourcing Loop automatically monitors relevant sources in our domain (CLI tools, AI, Rust, developer productivity) and extracts actionable ideas for product features, content creation, and marketing opportunities.

### 1.1 Objectives

1. **Front-of-Mind Awareness**: Stay current with industry trends
2. **Competitive Intelligence**: Track competitor movements
3. **Community Insights**: Capture user needs and pain points
4. **Content Inspiration**: Generate ideas for blog posts, tutorials, social

---

## 2. System Design

### 2.1 Component Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     IDEA SOURCING LOOP                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  SOURCES                                                         │
│  ───────                                                         │
│  ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐         │
│  │Reddit│ │  HN  │ │GitHub│ │Dev.to│ │ X/TW │ │ RSS  │         │
│  └──┬───┘ └──┬───┘ └──┬───┘ └──┬───┘ └──┬───┘ └──┬───┘         │
│     │        │        │        │        │        │              │
│     └────────┴────────┴────────┴────────┴────────┘              │
│                          │                                       │
│                          ▼                                       │
│                  ┌───────────────┐                               │
│                  │    Fetcher    │                               │
│                  │   (Parallel)  │                               │
│                  └───────┬───────┘                               │
│                          │                                       │
│                          ▼                                       │
│                  ┌───────────────┐                               │
│                  │   Relevance   │                               │
│                  │    Filter     │                               │
│                  └───────┬───────┘                               │
│                          │                                       │
│                          ▼                                       │
│                  ┌───────────────┐                               │
│                  │    Analyzer   │                               │
│                  │  (LLM-based)  │                               │
│                  └───────┬───────┘                               │
│                          │                                       │
│              ┌───────────┴───────────┐                           │
│              ▼                       ▼                           │
│       ┌───────────┐           ┌───────────┐                      │
│       │  Product  │           │  Content  │                      │
│       │   Ideas   │           │   Ideas   │                      │
│       └─────┬─────┘           └─────┬─────┘                      │
│             │                       │                            │
│             ▼                       ▼                            │
│       ┌───────────┐           ┌───────────┐                      │
│       │  Backlog  │           │  Content  │                      │
│       │  + Issue  │           │  Calendar │                      │
│       └───────────┘           └───────────┘                      │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### 2.2 Source Configuration

```yaml
# .claude/automation/config/idea_sources.yaml
sources:
  reddit:
    enabled: true
    subreddits:
      - name: "commandline"
        keywords: ["cli", "terminal", "shell", "bash", "zsh"]
        min_score: 10
      - name: "rust"
        keywords: ["cli", "clap", "terminal", "tui"]
        min_score: 20
      - name: "LocalLLaMA"
        keywords: ["local", "inference", "mlx", "ollama", "llama.cpp"]
        min_score: 15
      - name: "programming"
        keywords: ["cli tool", "command line", "developer tools"]
        min_score: 50
    fetch_limit: 50
    fetch_type: "hot"  # hot, new, top

  hackernews:
    enabled: true
    keywords:
      - "cli"
      - "terminal"
      - "command line"
      - "rust cli"
      - "ai assistant"
      - "local llm"
    min_score: 50
    fetch_limit: 30

  github:
    enabled: true
    trending:
      languages: ["rust", "go", "python"]
      spoken_language: "en"
    topics:
      - "cli"
      - "terminal"
      - "developer-tools"
      - "rust-cli"
    stars_threshold: 100

  devto:
    enabled: true
    tags:
      - "cli"
      - "terminal"
      - "rust"
      - "productivity"
      - "devtools"
    min_reactions: 20

  twitter:
    enabled: true
    lists:
      - "cli-developers"
      - "rust-community"
    accounts:
      - "@rustlang"
      - "@ThePrimeagen"
    keywords:
      - "cli tool"
      - "terminal productivity"
    min_engagement: 100

  competitors:
    enabled: true
    repos:
      - "github/gh-cli"
      - "junegunn/fzf"
      - "sharkdp/bat"
      - "BurntSushi/ripgrep"
      - "ajeetdsouza/zoxide"
    check_for:
      - releases
      - issues
      - discussions
```

---

## 3. Execution Flow

### 3.1 Step-by-Step Process

```
1. FETCH (Parallel)
   │
   For each source:
   │
   ├── Reddit
   │   ├── Fetch hot posts from subreddits
   │   ├── Filter by keywords and score
   │   └── Extract: title, score, comments, url
   │
   ├── Hacker News
   │   ├── Fetch top stories
   │   ├── Filter by keywords and score
   │   └── Extract: title, score, comments, url
   │
   ├── GitHub Trending
   │   ├── Fetch trending repos
   │   ├── Filter by language/topic
   │   └── Extract: name, description, stars, language
   │
   └── ... (other sources)

2. FILTER & DEDUPLICATE
   │
   ├── Remove duplicates (by URL/title similarity)
   ├── Apply relevance scoring
   │   ├── Keyword match score
   │   ├── Engagement score
   │   └── Recency score
   └── Keep top N items per source

3. ANALYZE (LLM-based)
   │
   For each item:
   │
   ├── Classify type:
   │   ├── product_feature - Feature request or improvement
   │   ├── content_idea - Blog/tutorial inspiration
   │   ├── marketing_opportunity - Promotional angle
   │   ├── competitive_intel - Competitor movement
   │   └── noise - Not actionable
   │
   ├── Extract insights:
   │   ├── Summary (1-2 sentences)
   │   ├── User pain point (if any)
   │   ├── Opportunity (what we could do)
   │   └── Priority (high/medium/low)
   │
   └── Generate tags

4. STORE & ACTION
   │
   For product ideas:
   │   ├── Add to ideas_backlog.yaml
   │   ├── Create GitHub Discussion (if high priority)
   │   └── Link to roadmap (if applicable)
   │
   For content ideas:
   │   ├── Add to content_ideas.yaml
   │   └── Add to content calendar (if timely)
   │
   For competitive intel:
   │   ├── Add to competitive_intel.yaml
   │   └── Alert if major (new release, etc.)
```

### 3.2 Relevance Scoring Algorithm

```python
def calculate_relevance_score(item, config):
    score = 0

    # Keyword matching (0-40 points)
    keyword_matches = count_keyword_matches(item, config.keywords)
    score += min(keyword_matches * 10, 40)

    # Engagement score (0-30 points)
    normalized_engagement = normalize_engagement(item.score, item.source)
    score += normalized_engagement * 30

    # Recency score (0-20 points)
    hours_old = get_hours_since_posted(item)
    if hours_old < 6:
        score += 20
    elif hours_old < 24:
        score += 15
    elif hours_old < 48:
        score += 10
    elif hours_old < 168:  # 1 week
        score += 5

    # Source weight (0-10 points)
    score += config.source_weights.get(item.source, 5)

    return score
```

---

## 4. Output Artifacts

### 4.1 Ideas Backlog

```yaml
# .claude/automation/queues/ideas_backlog.yaml
metadata:
  last_updated: "2026-01-11T08:15:00Z"
  total_ideas: 47
  new_this_week: 12

ideas:
  - id: "idea-2026-01-11-001"
    title: "Add fish shell autosuggestions support"
    source: "reddit-commandline"
    source_url: "https://reddit.com/r/commandline/..."
    source_engagement: 156
    discovered: "2026-01-11T08:00:00Z"

    classification:
      type: "product_feature"
      priority: "high"
      effort: "medium"

    analysis:
      summary: |
        Users requesting fish shell integration with autosuggestions.
        Currently caro only supports bash/zsh completions.
      pain_point: "No native fish support limits adoption"
      opportunity: "Fish has ~10% shell market share, loyal users"
      related_roadmap: "v1.2.0 - Shell Expansion"

    status: "new"  # new, reviewing, approved, rejected, in_progress, done
    tags: ["shell", "fish", "completion", "user-request"]

    actions:
      - type: "github_discussion"
        id: "#123"
        created: "2026-01-11T08:05:00Z"

  - id: "idea-2026-01-11-002"
    title: "Tutorial: Building CLI tools with local LLMs"
    source: "devto"
    source_url: "https://dev.to/..."
    source_engagement: 89
    discovered: "2026-01-11T08:00:00Z"

    classification:
      type: "content_idea"
      priority: "medium"

    analysis:
      summary: |
        Popular article about local LLM tooling. Good opportunity
        for us to write about caro's architecture.
      opportunity: "Educational content positions us as thought leaders"

    status: "approved"
    tags: ["content", "tutorial", "llm", "architecture"]

    actions:
      - type: "content_calendar"
        scheduled: "2026-01-18"
```

### 4.2 Daily Report

```yaml
# .claude/automation/state/idea_sourcing/{date}.yaml
run:
  id: "sourcing-2026-01-11"
  started: "2026-01-11T08:00:00Z"
  completed: "2026-01-11T08:12:34Z"

  sources_fetched:
    reddit: 127
    hackernews: 45
    github: 23
    devto: 34
    twitter: 56

  after_filtering: 42
  after_dedup: 38

  ideas_generated:
    product_feature: 5
    content_idea: 8
    marketing_opportunity: 2
    competitive_intel: 3
    noise: 20

  high_priority:
    - id: "idea-2026-01-11-001"
      title: "Add fish shell support"
      type: "product_feature"

  actions_taken:
    discussions_created: 1
    content_calendar_added: 3
    alerts_sent: 0
```

---

## 5. Configuration

### 5.1 Main Configuration

```yaml
# .claude/automation/config/idea_sourcing.yaml
idea_sourcing:
  enabled: true
  schedule: "0 8 * * *"  # Daily 8 AM

  limits:
    max_items_per_source: 50
    max_ideas_per_run: 20
    relevance_threshold: 40  # Minimum score to consider

  analysis:
    model: "claude-3-haiku"  # Use fast model for classification
    batch_size: 10

  actions:
    create_discussions: true
    min_priority_for_discussion: "high"
    add_to_content_calendar: true

  notifications:
    on_high_priority: true
    daily_digest: true
    digest_time: "09:00"

  retention:
    keep_raw_data: 7  # days
    keep_ideas: 90  # days
    archive_after: 30  # days
```

---

## 6. LLM Analysis Prompt

```markdown
## Idea Classification Task

Analyze the following item from {source} and classify it.

### Item
Title: {title}
URL: {url}
Score/Engagement: {score}
Content: {content}

### Our Context
We build Caro, a CLI tool that converts natural language to shell commands
using local LLMs. Key features: safety validation, offline-first, Rust-based.

### Classification
Classify this item into one of:
- product_feature: A feature request or improvement idea for Caro
- content_idea: Inspiration for a blog post, tutorial, or educational content
- marketing_opportunity: A promotional angle or community engagement opportunity
- competitive_intel: Information about a competitor or alternative tool
- noise: Not relevant or actionable

### Output Format (YAML)
```yaml
type: <classification>
priority: <high|medium|low>
summary: <1-2 sentence summary>
pain_point: <user pain point if applicable>
opportunity: <what we could do>
tags: [<relevant tags>]
```
```

---

## 7. Integration Points

### 7.1 Outputs to Other Systems

| Destination | Trigger | Data |
|-------------|---------|------|
| GitHub Discussions | High priority product idea | Idea details |
| Content Calendar | Content idea approved | Title, tags, date |
| Roadmap | Feature aligns with milestone | Link to discussion |
| Social Queue | Marketing opportunity | Draft post |
| Competitive Intel DB | Competitor update | Summary |

### 7.2 Input to Dev Loop

When a product idea is approved and prioritized:
1. Create GitHub Discussion for community input
2. If approved, create GitHub Issue with `feature` label
3. Issue enters Dev Loop via `/caro.roadmap next`

---

## 8. Related Documents

- [IDEA_SOURCING_TEST.md](../tests/IDEA_SOURCING_TEST.md) - Test cases
- [v1.1.0-content-marketing-strategy.md](../../releases/v1.1.0-content-marketing-strategy.md)
- [SOCIAL_QUEUE_DRS.md](./SOCIAL_QUEUE_DRS.md) - Social posting integration
