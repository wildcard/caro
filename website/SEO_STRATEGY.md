# Caro.sh SEO & Social Media Optimization Strategy

## Executive Summary

This document outlines a comprehensive strategy for implementing industry-standard best practices for SEO, Open Graph social sharing, and LLM exposure for the Caro.sh website.

**Current State**: The website has minimal SEO implementation with only basic `<title>` and `<meta name="description">` tags. Critical gaps include no Open Graph tags, no Twitter Cards, no structured data, and no LLM-friendly content exposure.

---

## 1. Open Graph Protocol Implementation

### 1.1 Required Tags (All Pages)

```html
<!-- Primary Open Graph Tags -->
<meta property="og:type" content="website" />
<meta property="og:url" content="https://caro.sh/" />
<meta property="og:title" content="Caro - Your Loyal Shell Companion" />
<meta property="og:description" content="Caro is a companion agent that helps you with POSIX shell commands. Available as an MCP for Claude and as a dedicated Skill." />
<meta property="og:image" content="https://caro.sh/og-image.png" />
<meta property="og:image:width" content="1200" />
<meta property="og:image:height" content="630" />
<meta property="og:image:alt" content="Caro - Terminal companion for natural language shell commands" />
<meta property="og:site_name" content="Caro" />
<meta property="og:locale" content="en_US" />
```

### 1.2 Blog Post Specific Tags

```html
<meta property="og:type" content="article" />
<meta property="article:published_time" content="2024-01-15T00:00:00Z" />
<meta property="article:author" content="Caro Team" />
<meta property="article:section" content="Technology" />
<meta property="article:tag" content="terminal,shell,AI,developer-tools" />
```

### 1.3 Social Image Requirements

| Platform | Dimensions | Aspect Ratio | Max File Size |
|----------|------------|--------------|---------------|
| **Facebook/LinkedIn** | 1200 x 630 px | 1.91:1 | 8 MB |
| **Twitter Summary Large** | 1200 x 628 px | 1.91:1 | 5 MB |
| **Twitter Summary** | 144 x 144 px | 1:1 | 5 MB |
| **Discord** | 1200 x 630 px | 1.91:1 | 8 MB |
| **Slack** | 1200 x 630 px | 1.91:1 | 8 MB |

**Recommended Approach**: Create a primary OG image at 1200x630 and reuse across platforms.

---

## 2. Twitter (X) Card Implementation

### 2.1 Summary Large Image Card (Recommended)

```html
<meta name="twitter:card" content="summary_large_image" />
<meta name="twitter:site" content="@CaroDaShellShib" />
<meta name="twitter:creator" content="@CaroDaShellShib" />
<meta name="twitter:title" content="Caro - Your Loyal Shell Companion" />
<meta name="twitter:description" content="Convert natural language to POSIX shell commands with AI. Available as Claude MCP or dedicated Skill." />
<meta name="twitter:image" content="https://caro.sh/twitter-card.png" />
<meta name="twitter:image:alt" content="Caro terminal companion demo" />
```

### 2.2 Player Card (Optional - For Demo Videos)

For pages with video content (Asciinema demos):

```html
<meta name="twitter:card" content="player" />
<meta name="twitter:player" content="https://asciinema.org/a/FOxFkOjVHmYSB0S2I2xaZsaX0/embed" />
<meta name="twitter:player:width" content="640" />
<meta name="twitter:player:height" content="480" />
```

---

## 3. Search Engine Optimization (SEO)

### 3.1 Essential Meta Tags

```html
<!-- Primary SEO Tags -->
<meta charset="UTF-8" />
<meta name="viewport" content="width=device-width, initial-scale=1.0" />
<title>Caro - Your Loyal Shell Companion | AI Terminal Assistant</title>
<meta name="description" content="Caro is a companion agent that helps you with POSIX shell commands. Convert natural language to terminal commands with AI. Available as Claude MCP or dedicated Skill." />

<!-- Canonical URL (prevents duplicate content) -->
<link rel="canonical" href="https://caro.sh/" />

<!-- Robots Directives -->
<meta name="robots" content="index, follow, max-image-preview:large, max-snippet:-1, max-video-preview:-1" />

<!-- Author & Publisher -->
<meta name="author" content="Caro Team" />

<!-- Theme & Mobile -->
<meta name="theme-color" content="#1a1a2e" />
<meta name="color-scheme" content="dark light" />
```

### 3.2 robots.txt Configuration

Create `/public/robots.txt`:

```txt
# Caro.sh Robots Configuration
User-agent: *
Allow: /

# Sitemap
Sitemap: https://caro.sh/sitemap-index.xml

# AI Crawlers - Welcome
User-agent: GPTBot
Allow: /

User-agent: Google-Extended
Allow: /

User-agent: CCBot
Allow: /

User-agent: anthropic-ai
Allow: /

User-agent: Claude-Web
Allow: /

User-agent: PerplexityBot
Allow: /

User-agent: Bytespider
Allow: /

# Disallow dev assets
User-agent: *
Disallow: /*.map$
Disallow: /_astro/
```

### 3.3 Sitemap Strategy

The `@astrojs/sitemap` integration is already configured. Ensure it generates:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
  <url>
    <loc>https://caro.sh/</loc>
    <lastmod>2024-01-15</lastmod>
    <changefreq>weekly</changefreq>
    <priority>1.0</priority>
  </url>
  <url>
    <loc>https://caro.sh/blog/why-caro</loc>
    <lastmod>2024-01-15</lastmod>
    <changefreq>monthly</changefreq>
    <priority>0.8</priority>
  </url>
</urlset>
```

---

## 4. Structured Data (JSON-LD Schema.org)

### 4.1 Organization Schema (Homepage)

```json
{
  "@context": "https://schema.org",
  "@type": "SoftwareApplication",
  "name": "Caro",
  "alternateName": "Caro.sh",
  "description": "A companion agent that helps you with POSIX shell commands using AI",
  "url": "https://caro.sh",
  "applicationCategory": "DeveloperApplication",
  "operatingSystem": "macOS, Linux",
  "offers": {
    "@type": "Offer",
    "price": "0",
    "priceCurrency": "USD"
  },
  "featureList": [
    "Natural language to shell command conversion",
    "Claude MCP integration",
    "POSIX compliant command generation",
    "Safety validation for dangerous commands"
  ],
  "screenshot": "https://caro.sh/og-image.png",
  "softwareVersion": "1.0",
  "aggregateRating": {
    "@type": "AggregateRating",
    "ratingValue": "5",
    "ratingCount": "1"
  }
}
```

### 4.2 WebSite Schema

```json
{
  "@context": "https://schema.org",
  "@type": "WebSite",
  "name": "Caro",
  "url": "https://caro.sh",
  "description": "Your loyal shell companion - AI-powered terminal assistant",
  "potentialAction": {
    "@type": "SearchAction",
    "target": "https://caro.sh/?q={search_term_string}",
    "query-input": "required name=search_term_string"
  }
}
```

### 4.3 Article Schema (Blog Posts)

```json
{
  "@context": "https://schema.org",
  "@type": "BlogPosting",
  "headline": "Why Caro? The Story Behind Your Terminal Companion",
  "description": "The story of how Caro came to be...",
  "image": "https://caro.sh/blog/why-caro-og.png",
  "datePublished": "2024-01-15",
  "dateModified": "2024-01-15",
  "author": {
    "@type": "Organization",
    "name": "Caro Team"
  },
  "publisher": {
    "@type": "Organization",
    "name": "Caro",
    "logo": {
      "@type": "ImageObject",
      "url": "https://caro.sh/caro-pixel.png"
    }
  },
  "mainEntityOfPage": {
    "@type": "WebPage",
    "@id": "https://caro.sh/blog/why-caro"
  }
}
```

---

## 5. LLM Exposure Strategy

### 5.1 llms.txt Implementation

Create `/public/llms.txt` - a emerging standard for AI agent-friendly content:

```txt
# Caro - AI Shell Companion

> Caro is a companion agent that converts natural language to POSIX shell commands.

## Overview

Caro helps developers and terminal users by:
- Converting plain English to shell commands
- Providing safety validation for dangerous operations
- Integrating with Claude as an MCP (Model Context Protocol)
- Available as a dedicated Skill for quick access

## Installation

```bash
# Via Homebrew
brew install wildcard/tap/caro

# Via npm (for Claude Desktop MCP)
npm install -g @anthropic/caro
```

## Usage Examples

Ask Caro in natural language:
- "find all Python files modified today"
- "compress this folder into a tarball"
- "show disk usage sorted by size"
- "kill the process using port 3000"

## Integration

### Claude MCP Mode
Add to Claude Desktop config:
```json
{
  "mcpServers": {
    "caro": {
      "command": "npx",
      "args": ["-y", "@anthropic/caro"]
    }
  }
}
```

### Skill Mode
Access directly via Claude Code with /caro

## Links

- Website: https://caro.sh
- GitHub: https://github.com/wildcard/caro
- Documentation: https://caro.sh/docs

## Contact

For questions or support, open an issue on GitHub.
```

### 5.2 AI-Friendly Content Guidelines

1. **Clear Headings**: Use semantic HTML (`h1`, `h2`, etc.) for easy parsing
2. **Code Blocks**: Wrap code in proper `<pre><code>` tags with language hints
3. **Structured Lists**: Use `<ul>`/`<ol>` for scannable content
4. **Alt Text**: Provide descriptive alt text for all images
5. **Machine-Readable Dates**: Use ISO 8601 format (`datetime` attribute)

### 5.3 Content Accessibility for AI

Add `<meta>` tags for AI assistants:

```html
<!-- AI Content Hints -->
<meta name="ai:purpose" content="developer tool, terminal assistant, shell commands" />
<meta name="ai:capabilities" content="natural language to shell, command safety, MCP integration" />
```

---

## 6. Social Image Design Specifications

### 6.1 Primary OG Image Design

**Filename**: `og-image.png`
**Dimensions**: 1200 x 630 pixels
**Format**: PNG (for quality) or WebP (for size)

**Design Elements**:
- Caro pixel character prominently displayed
- Tagline: "Your Loyal Shell Companion"
- Terminal-style background with subtle code hints
- Brand colors from existing dark theme
- Clear, readable text at small sizes

### 6.2 Twitter Card Image

**Filename**: `twitter-card.png`
**Dimensions**: 1200 x 628 pixels (Twitter's exact spec)

Same design as OG image, optimized for Twitter's rendering.

### 6.3 Blog Post Images

Each blog post should have:
- **Hero Image**: 1200 x 630 for social sharing
- **Inline Images**: Optimized for content width
- **Alt Text**: Descriptive, keyword-rich

### 6.4 Favicon Package

Extend current favicon to full package:

```html
<!-- Favicon Package -->
<link rel="icon" type="image/png" sizes="32x32" href="/favicon-32x32.png" />
<link rel="icon" type="image/png" sizes="16x16" href="/favicon-16x16.png" />
<link rel="apple-touch-icon" sizes="180x180" href="/apple-touch-icon.png" />
<link rel="manifest" href="/site.webmanifest" />
<link rel="mask-icon" href="/safari-pinned-tab.svg" color="#1a1a2e" />
<meta name="msapplication-TileColor" content="#1a1a2e" />
```

---

## 7. Implementation Plan

### Phase 1: Core Meta Tags (Priority: HIGH)

**Files to Create/Modify**:

1. **Create SEO Component** - `src/components/SEO.astro`
   - Centralized meta tag management
   - Props for title, description, image, type, url
   - Conditional rendering for page types

2. **Update Layout.astro**
   - Import and use SEO component
   - Add structured data injection point

3. **Create robots.txt** - `public/robots.txt`
   - Allow all crawlers
   - Include sitemap reference
   - AI crawler permissions

4. **Create llms.txt** - `public/llms.txt`
   - AI-friendly documentation
   - Usage examples
   - Integration guides

### Phase 2: Social Images (Priority: HIGH)

**Files to Create**:

1. `public/og-image.png` - Primary 1200x630 social image
2. `public/twitter-card.png` - Twitter-optimized card
3. `public/apple-touch-icon.png` - iOS home screen icon
4. `public/favicon-32x32.png` - Standard favicon
5. `public/site.webmanifest` - PWA manifest

### Phase 3: Structured Data (Priority: MEDIUM)

**Implementation**:

1. Add JSON-LD to Layout.astro for global schemas
2. Create blog-specific schema in BlogPost.astro
3. Validate with Google's Rich Results Test

### Phase 4: Blog SEO Enhancement (Priority: MEDIUM)

**Enhancements**:

1. Per-post OG images with title overlays
2. Article schema for each blog post
3. Author information
4. Reading time in structured data

### Phase 5: Performance & Monitoring (Priority: LOW)

**Tools**:

1. Google Search Console setup
2. Bing Webmaster Tools
3. Social media debuggers validation
4. Core Web Vitals monitoring

---

## 8. SEO Component Architecture

### Proposed SEO.astro Component

```astro
---
interface Props {
  title: string;
  description: string;
  image?: string;
  imageAlt?: string;
  type?: 'website' | 'article';
  publishedTime?: string;
  modifiedTime?: string;
  author?: string;
  tags?: string[];
  noindex?: boolean;
  canonical?: string;
}

const {
  title,
  description,
  image = '/og-image.png',
  imageAlt = 'Caro - Your Loyal Shell Companion',
  type = 'website',
  publishedTime,
  modifiedTime,
  author = 'Caro Team',
  tags = [],
  noindex = false,
  canonical,
} = Astro.props;

const siteUrl = 'https://caro.sh';
const canonicalUrl = canonical || new URL(Astro.url.pathname, siteUrl).href;
const imageUrl = image.startsWith('http') ? image : new URL(image, siteUrl).href;
---

<!-- Primary Meta Tags -->
<title>{title}</title>
<meta name="title" content={title} />
<meta name="description" content={description} />
<meta name="author" content={author} />
<link rel="canonical" href={canonicalUrl} />

{noindex ? (
  <meta name="robots" content="noindex, nofollow" />
) : (
  <meta name="robots" content="index, follow, max-image-preview:large" />
)}

<!-- Open Graph / Facebook -->
<meta property="og:type" content={type} />
<meta property="og:url" content={canonicalUrl} />
<meta property="og:title" content={title} />
<meta property="og:description" content={description} />
<meta property="og:image" content={imageUrl} />
<meta property="og:image:alt" content={imageAlt} />
<meta property="og:site_name" content="Caro" />
<meta property="og:locale" content="en_US" />

{type === 'article' && publishedTime && (
  <>
    <meta property="article:published_time" content={publishedTime} />
    {modifiedTime && <meta property="article:modified_time" content={modifiedTime} />}
    <meta property="article:author" content={author} />
    {tags.map(tag => <meta property="article:tag" content={tag} />)}
  </>
)}

<!-- Twitter -->
<meta name="twitter:card" content="summary_large_image" />
<meta name="twitter:url" content={canonicalUrl} />
<meta name="twitter:title" content={title} />
<meta name="twitter:description" content={description} />
<meta name="twitter:image" content={imageUrl} />
<meta name="twitter:image:alt" content={imageAlt} />
```

---

## 9. Testing & Validation Checklist

### Pre-Launch Validation

- [ ] **Facebook Debugger**: https://developers.facebook.com/tools/debug/
- [ ] **Twitter Card Validator**: https://cards-dev.twitter.com/validator
- [ ] **LinkedIn Post Inspector**: https://www.linkedin.com/post-inspector/
- [ ] **Google Rich Results Test**: https://search.google.com/test/rich-results
- [ ] **Schema.org Validator**: https://validator.schema.org/
- [ ] **Google PageSpeed Insights**: https://pagespeed.web.dev/

### SEO Audit Tools

- [ ] Lighthouse SEO audit (Chrome DevTools)
- [ ] Screaming Frog crawl
- [ ] Ahrefs/SEMrush site audit

---

## 10. Success Metrics

### KPIs to Track

1. **Organic Search Traffic**: Monitor via Google Search Console
2. **Social Share Engagement**: Track click-through from social platforms
3. **Rich Snippet Appearance**: Monitor in GSC Performance report
4. **Core Web Vitals**: LCP, FID, CLS scores
5. **Indexation Rate**: Pages indexed vs. submitted

### Expected Outcomes

After implementation:
- ✅ Rich previews on all social platforms
- ✅ Higher click-through rates from search results
- ✅ Improved AI assistant awareness of Caro
- ✅ Better mobile home screen icon appearance
- ✅ Enhanced discoverability for developer audiences

---

## Appendix A: File Checklist

### New Files to Create

| File | Location | Purpose |
|------|----------|---------|
| `SEO.astro` | `src/components/` | Centralized meta tag component |
| `robots.txt` | `public/` | Search engine directives |
| `llms.txt` | `public/` | LLM-friendly documentation |
| `og-image.png` | `public/` | Primary social sharing image |
| `twitter-card.png` | `public/` | Twitter-optimized card |
| `apple-touch-icon.png` | `public/` | iOS home screen icon |
| `site.webmanifest` | `public/` | PWA manifest |

### Files to Modify

| File | Changes |
|------|---------|
| `Layout.astro` | Import SEO component, add JSON-LD |
| `BlogPost.astro` | Add article-specific SEO props |
| `index.astro` | Pass SEO props to layout |
| `why-caro.astro` | Add blog-specific meta data |

---

## Appendix B: Quick Reference

### Meta Tag Priority Order

1. **Title** - Most important for SEO
2. **Description** - Affects click-through rate
3. **Canonical URL** - Prevents duplicate content
4. **OG Image** - Critical for social sharing
5. **Structured Data** - Enables rich snippets

### Character Limits

| Element | Recommended | Maximum |
|---------|-------------|---------|
| Title | 50-60 chars | 70 chars |
| Description | 150-160 chars | 320 chars |
| OG Title | 60 chars | 90 chars |
| OG Description | 200 chars | 300 chars |

---

*Strategy Document Version 1.0 - Created for Caro.sh SEO Implementation*
