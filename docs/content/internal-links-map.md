# Caro Internal Links Map

> **Last Updated:** January 2026
> **Owner:** DevRel Team

This document defines our internal linking strategy for blog content. Use this as a reference when adding internal links to articles.

---

## Core Product Pages

These are our highest-priority link targets. Include at least one in every blog post.

| Page | URL | When to Link | Anchor Text Examples |
|------|-----|--------------|---------------------|
| Homepage | `caro.sh` | General mentions | "Caro", "learn more about Caro" |
| Installation | `caro.sh/install` | Getting started context | "install Caro", "get started", "download" |
| Documentation | `caro.sh/docs` | Technical deep-dive | "documentation", "full docs", "learn more" |
| AI Safety | `caro.sh/ai-command-safety` | Safety discussions | "AI command safety", "safety validation" |
| Features | `caro.sh/features` | Feature mentions | "features", "capabilities", "what Caro can do" |
| GitHub | `github.com/wildcard/caro` | Open source context | "GitHub", "source code", "contribute" |

---

## Documentation Pages

Link to these for technical depth and how-to guidance.

| Page | URL | When to Link | Anchor Text Examples |
|------|-----|--------------|---------------------|
| Quick Start | `/docs/quickstart` | Beginner content | "quick start guide", "getting started" |
| Configuration | `/docs/configuration` | Setup/config topics | "configuration options", "customize Caro" |
| Safety Patterns | `/docs/safety` | Safety discussions | "safety patterns", "command validation" |
| MCP Integration | `/docs/mcp` | Claude/MCP context | "MCP integration", "Claude Desktop" |
| Backends | `/docs/backends` | Inference topics | "inference backends", "MLX", "model support" |

---

## Blog Post Categories

### Safety & Security

| Post | URL | Topics | Priority |
|------|-----|--------|----------|
| (Planned) Command Safety Guide | `/blog/command-safety` | Safety, validation | HIGH |
| (Planned) 50 Dangerous Commands | `/blog/dangerous-commands` | Safety, warnings | HIGH |
| (Planned) Preventing Shell Disasters | `/blog/prevent-shell-disasters` | Safety, best practices | HIGH |

### Privacy & Local-First

| Post | URL | Topics | Priority |
|------|-----|--------|----------|
| (Planned) Why Local-First Matters | `/blog/local-first-ai` | Privacy, offline | HIGH |
| (Planned) Air-Gapped AI Tools | `/blog/air-gapped-ai` | Privacy, enterprise | MEDIUM |

### Tutorials & How-To

| Post | URL | Topics | Priority |
|------|-----|--------|----------|
| (Planned) Getting Started Guide | `/blog/getting-started` | Installation, basics | HIGH |
| (Planned) MLX Setup Guide | `/blog/mlx-setup` | Apple Silicon, models | MEDIUM |

### Technical Deep Dives

| Post | URL | Topics | Priority |
|------|-----|--------|----------|
| (Planned) How Caro Works | `/blog/how-caro-works` | Architecture, design | MEDIUM |
| (Planned) Building CLIs in Rust | `/blog/rust-cli` | Rust, development | LOW |

---

## Conversion Pages

Link to these when discussing pricing, enterprise, or commercial use.

| Page | URL | When to Link | Anchor Text Examples |
|------|-----|--------------|---------------------|
| Enterprise | `/enterprise` | Business/team context | "enterprise features", "team edition" |
| Pricing | `/pricing` | Commercial context | "pricing", "plans" |
| Contact | `/contact` | Support/sales context | "contact us", "get in touch" |

---

## Anchor Text Guidelines

### Best Practices

1. **Be Descriptive**: Anchor text should describe the destination
   - Good: "Learn about [command safety validation](link)"
   - Bad: "[Click here](link) to learn more"

2. **Use Keywords Naturally**: Include relevant keywords without forcing
   - Good: "Our [AI command line tool](link) validates commands before execution"
   - Bad: "Our [AI command line tool AI CLI tool safe commands](link)"

3. **Vary Anchor Text**: Don't use identical anchor text for every link to the same page
   - Good: "install Caro", "get started with Caro", "download Caro"
   - Bad: Always using "install Caro"

4. **Match User Intent**: Anchor should match what users expect to find
   - Good: "[Quick start guide](/docs/quickstart)" → leads to getting started content
   - Bad: "[Quick start guide](/docs/advanced-config)" → misleading

### Avoid

- Generic anchors: "click here", "read more", "this page"
- Over-optimized anchors: Exact match keywords stuffed unnaturally
- Broken links: Always verify links work before publishing
- Too many links: 1 link per 100-150 words is ideal

---

## Linking Strategy by Content Type

### Tutorial/How-To Posts

| Link Count | Targets |
|------------|---------|
| 3-5 internal | Documentation pages, related tutorials |
| 2-3 external | Official docs for technologies mentioned |

**Pattern:**
- Link to docs when mentioning configuration options
- Link to related tutorials at end ("You might also like...")
- Link to homepage/features in intro or CTA

### Deep Dive / Technical Posts

| Link Count | Targets |
|------------|---------|
| 5-7 internal | Mix of docs, blog posts, product pages |
| 3-4 external | Technical specifications, research papers |

**Pattern:**
- Heavy linking to documentation
- Cross-link related deep dives
- Link to GitHub for code references

### Comparison / Review Posts

| Link Count | Targets |
|------------|---------|
| 4-6 internal | Features page, relevant blog posts |
| 3-5 external | Competitor pages (for fair comparison), reviews |

**Pattern:**
- Link to features page when discussing capabilities
- Link to documentation for technical claims
- Link to installation at end (CTA)

### News / Announcement Posts

| Link Count | Targets |
|------------|---------|
| 2-4 internal | Product pages, documentation for new features |
| 1-2 external | Press coverage, related announcements |

**Pattern:**
- Link to feature documentation
- Link to changelog or release notes
- Link to installation/getting started

---

## Link Placement Strategy

### Introduction
- 1-2 links maximum
- Link to product page or main relevant doc
- Don't overwhelm before content starts

### Body Sections
- 1-2 links per major section (H2)
- Place naturally within context
- Prefer links that add value, not just SEO

### Conclusion
- 1-2 links including CTA
- Link to installation/getting started
- Link to related content ("Read next...")

---

## Internal Link Audit Checklist

Before publishing, verify:

- [ ] Minimum 3 internal links present
- [ ] At least 1 link to core product page
- [ ] No broken links (test all)
- [ ] Anchor text is descriptive
- [ ] Links are distributed throughout content
- [ ] No excessive linking (>10 in 2000 words)
- [ ] All links open in same tab (internal)
- [ ] External links have `rel="noopener"`

---

## Link Maintenance

### Monthly Tasks

1. Run link checker on all blog posts
2. Update broken links
3. Add links to new content from older posts
4. Review analytics for underlinked high-value pages

### When Publishing New Content

1. Link TO new content FROM 3-5 related existing posts
2. Link FROM new content TO relevant existing content
3. Update this document with new link targets

---

## High-Priority Link Gaps

Pages that need more inbound internal links:

| Page | Current Inbound Links | Target | Action Needed |
|------|----------------------|--------|---------------|
| /ai-command-safety | (New) | 10+ | Link from all safety content |
| /docs | (Update) | 20+ | Link from all tutorials |
| /enterprise | (New) | 5+ | Link from business/team content |

---

## Notes

- Update this document when adding new pages
- Review quarterly to align with content strategy
- Prioritize linking to pages that drive conversions
- Use analytics to identify high-value pages needing more links
