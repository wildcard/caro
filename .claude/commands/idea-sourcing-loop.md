# Idea Sourcing Loop

Automated idea sourcing from relevant sources in our domain (CLI tools, AI, Rust, developer productivity). Extracts actionable ideas for product features, content creation, and marketing opportunities.

## Usage

```
/idea-sourcing-loop [options]
```

## Options

- `--sources <list>` - Specific sources to check
- `--dry-run` - Don't create discussions or modify backlog
- `--limit <n>` - Max ideas to process
- `--verbose` - Show detailed analysis

## Sources

### Reddit

```yaml
subreddits:
  - r/commandline     # CLI discussions
  - r/rust            # Rust ecosystem
  - r/LocalLLaMA      # Local AI trends
  - r/programming     # General dev
```

### Hacker News

- Top stories with CLI/AI keywords
- "Show HN" posts for new tools

### GitHub

- Trending Rust/CLI repositories
- Issues on competitor repos
- New releases from key projects

### Dev.to

- Articles tagged: cli, rust, devtools

### Twitter/X

- Developer tool announcements
- CLI productivity tips

## Process

### 1. Fetch Content

```
For each source:
  ├── Fetch latest items (RSS, API)
  ├── Filter by keywords
  ├── Apply minimum score threshold
  └── Store raw items
```

### 2. Filter & Score

```python
def score_item(item):
    score = 0

    # Keyword matches (0-40)
    score += keyword_matches * 10

    # Engagement (0-30)
    score += normalized_engagement * 30

    # Recency (0-20)
    if hours_old < 6: score += 20
    elif hours_old < 24: score += 15

    # Source weight (0-10)
    score += source_weights[item.source]

    return score
```

### 3. Analyze with LLM

For high-scoring items, classify:

```yaml
types:
  product_feature:   # Feature idea for Caro
  content_idea:      # Blog/tutorial inspiration
  marketing_opportunity:  # Promotional angle
  competitive_intel: # Competitor movement
  noise:             # Not actionable
```

### 4. Store Ideas

Add to `.claude/automation/queues/ideas_backlog.yaml`:

```yaml
- id: "idea-2026-01-11-001"
  title: "Add fish shell support"
  source: "reddit-commandline"
  source_url: "https://reddit.com/..."

  classification:
    type: "product_feature"
    priority: "high"

  analysis:
    summary: "Users requesting fish shell integration"
    pain_point: "No native fish support"
    opportunity: "Fish has ~10% market share"

  status: "new"
  tags: ["shell", "fish", "user-request"]
```

### 5. Take Actions

**Product Ideas (High Priority):**
- Create GitHub Discussion for community input
- Link to relevant roadmap milestone

**Content Ideas:**
- Add to content calendar
- Tag for upcoming themes

**Competitive Intel:**
- Log to competitive tracking
- Alert if major (new release)

## Example Session

```
> /idea-sourcing-loop

Idea Sourcing Loop
══════════════════

Fetching from sources...
  ✓ Reddit (4 subreddits): 127 items
  ✓ Hacker News: 45 items
  ✓ GitHub Trending: 23 items
  ✓ Dev.to: 34 items
  ✓ Twitter: 56 items

  Total fetched: 285 items

Filtering...
  After keyword filter: 67 items
  After score filter (>40): 42 items
  After dedup: 38 items

Analyzing top items...
  [1/20] "Fish shell support request" → product_feature (high)
  [2/20] "Tutorial: Building CLI tools" → content_idea (medium)
  [3/20] "GitHub CLI v3 released" → competitive_intel (medium)
  ...

Classification Summary:
  product_feature: 5
  content_idea: 8
  marketing_opportunity: 2
  competitive_intel: 3
  noise: 2

Actions Taken:
  ✓ Created GitHub Discussion #45: "Fish shell support"
  ✓ Added 3 items to content calendar
  ✓ Updated competitive tracking

Ideas Backlog:
  New this run: 15
  Total pending: 47

Daily Summary available at:
  .claude/automation/state/idea_sourcing/2026-01-11.yaml
```

## Backlog Management

### View Backlog

```
/idea-sourcing-loop backlog [--status new|reviewing|approved]
```

### Review Ideas

```
/idea-sourcing-loop review <idea-id>
```

Actions:
- `approve` - Move to approved, create issue if product
- `reject` - Archive with reason
- `defer` - Keep for later review

### Promote to Roadmap

```
/idea-sourcing-loop promote <idea-id> --milestone v1.2.0
```

## Configuration

### Source Configuration

```yaml
# .claude/automation/config/idea_sources.yaml
sources:
  reddit:
    enabled: true
    subreddits:
      - name: "commandline"
        keywords: ["cli", "terminal"]
        min_score: 10
    fetch_limit: 50

  hackernews:
    enabled: true
    keywords: ["cli", "terminal", "rust cli"]
    min_score: 50
```

### Analysis Configuration

```yaml
# .claude/automation/config/idea_sourcing.yaml
idea_sourcing:
  enabled: true
  schedule: "0 8 * * *"

  limits:
    max_items_per_source: 50
    max_ideas_per_run: 20
    relevance_threshold: 40

  actions:
    create_discussions: true
    min_priority_for_discussion: "high"
```

## Output Files

```
.claude/automation/
├── queues/
│   └── ideas_backlog.yaml    # Idea backlog
├── state/
│   └── idea_sourcing/
│       ├── 2026-01-11.yaml   # Daily run report
│       └── ...
└── config/
    ├── idea_sources.yaml     # Source configuration
    └── idea_sourcing.yaml    # Loop configuration
```

## Related

- [IDEA_SOURCING_DRS.md](../.claude/automation/specs/IDEA_SOURCING_DRS.md)
- `/social-queue` - Content ideas feed into social
- `/caro.roadmap` - Product ideas feed into roadmap
