# Blog Content Skill - Quick Reference

## Commands

| Command | Description |
|---------|-------------|
| `/blog-content research [topic]` | Create research brief |
| `/blog-content write [topic]` | Generate article from brief |
| `/blog-content optimize [file]` | Final SEO audit |
| `/blog-content analyze [URL/file]` | Evaluate existing content |
| `/blog-content review` | Performance prioritization |

## Workflow

```
1. IDEA → content/topics/ideas.md
2. /blog-content research "topic" → content/research/brief-*.md
3. /blog-content write "topic" → content/drafts/*.md
4. /blog-content optimize draft.md → SEO audit
5. PUBLISH → content/published/*.md
```

## Context Files to Load

Before any content operation, load:

1. `docs/brand/BRAND_IDENTITY_GUIDE.md` - Voice/tone
2. `docs/content/seo-guidelines.md` - SEO standards
3. `docs/content/target-keywords.md` - Keywords
4. `docs/content/internal-links-map.md` - Link targets
5. `docs/content/writing-examples.md` - Style reference

## Content Requirements

| Metric | Target |
|--------|--------|
| Word count | 2000-3000+ |
| Keyword density | 1-2% |
| Internal links | 3-5+ |
| External links | 2-3 |
| Readability | 8th-10th grade |
| SEO score | 80+ |

## SEO Checklist

- [ ] Primary keyword in H1
- [ ] Primary keyword in first 100 words
- [ ] Primary keyword in 2+ H2 headings
- [ ] Meta title: 50-60 chars
- [ ] Meta description: 150-160 chars
- [ ] 3-5 internal links
- [ ] Proper heading hierarchy

## File Locations

```
content/
├── topics/ideas.md      # Backlog
├── research/            # Briefs
├── drafts/              # WIP
├── published/           # Final
└── rewrites/            # Updates

docs/content/
├── seo-guidelines.md
├── target-keywords.md
├── internal-links-map.md
└── writing-examples.md
```
