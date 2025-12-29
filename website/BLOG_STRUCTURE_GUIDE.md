# Blog Post Structure Enhancement & Improvement Guide

> A comprehensive guide for creating effective, engaging, and consistent blog posts for the Caro website.

---

## Table of Contents

1. [Current State Analysis](#current-state-analysis)
2. [Content Architecture](#content-architecture)
3. [Visual Hierarchy & Typography](#visual-hierarchy--typography)
4. [Engagement Elements](#engagement-elements)
5. [SEO & Discoverability](#seo--discoverability)
6. [Post Templates](#post-templates)
7. [Style Consistency Checklist](#style-consistency-checklist)
8. [Implementation Recommendations](#implementation-recommendations)

---

## Current State Analysis

### What's Working Well

1. **Consistent Layout**: All posts use the `BlogPost.astro` layout component
2. **Clean Typography**: Well-designed responsive typography with good line heights
3. **Visual Branding**: Consistent use of the #ff8c42 (orange) brand color
4. **Highlight Boxes**: Effective use of `.highlight` class for callouts
5. **Dark Mode Support**: Theme-aware styling throughout

### Areas for Improvement

| Area | Current State | Recommended Enhancement |
|------|---------------|------------------------|
| Post Categories | None | Add category/tag system |
| Author Attribution | Missing | Add author info section |
| Table of Contents | None | Auto-generate for long posts |
| Social Sharing | Missing | Add share buttons |
| Related Posts | None | Add related content section |
| Reading Progress | None | Add progress indicator |
| Post Series | No linking | Add series navigation |
| Code Highlighting | Basic styling | Integrate Shiki/Prism |
| Images | Basic `<img>` | Add figure captions, lazy loading |
| Estimated Dates | Static | Add "Updated on" metadata |

---

## Content Architecture

### The Ideal Post Structure

```
┌─────────────────────────────────────────────────┐
│  META (Title, Date, Read Time, Category, Tags)  │
├─────────────────────────────────────────────────┤
│  HOOK (Opening highlight box - the "why care")  │
├─────────────────────────────────────────────────┤
│  TABLE OF CONTENTS (for posts > 5 min read)     │
├─────────────────────────────────────────────────┤
│  INTRODUCTION (Context + Promise)               │
├─────────────────────────────────────────────────┤
│  MAIN CONTENT                                   │
│  ├── Section 1 (H2) + Subsections (H3)         │
│  ├── Section 2 (H2) + Subsections (H3)         │
│  └── Section N (H2) + Subsections (H3)         │
├─────────────────────────────────────────────────┤
│  CONCLUSION (Summary + Next Steps)              │
├─────────────────────────────────────────────────┤
│  CALL TO ACTION (What readers should do)        │
├─────────────────────────────────────────────────┤
│  FOOTER (Author, Tags, Social, Related Posts)   │
└─────────────────────────────────────────────────┘
```

### Section Breakdown

#### 1. Hook (Opening Highlight)

**Purpose**: Capture attention and establish relevance within 5 seconds.

**Current Pattern** (Good):
```html
<div class="highlight">
  <p>
    Security isn't an afterthought at Caro—it's foundational to everything
    we build. As a CLI tool that generates and executes shell commands,
    we take our responsibility to protect users seriously.
  </p>
</div>
```

**Enhanced Pattern** (Better):
```html
<div class="highlight hook">
  <p class="hook-headline">
    <strong>TL;DR:</strong> Security isn't an afterthought at Caro—it's foundational.
  </p>
  <p class="hook-subtext">
    In this post, we'll walk through our security posture, the GitHub security
    features we use, and the practices that make Caro a security-conscious project.
  </p>
</div>
```

#### 2. Introduction

**Purpose**: Set context and make a promise about what readers will learn.

**Formula**:
1. **Context**: Why is this topic relevant now?
2. **Problem**: What challenge does this address?
3. **Promise**: What will readers gain by reading?

**Example**:
```html
<p>
  Most AI-powered developer tools fall into one of two camps: DIY Everything
  or Remote Black Boxes. Both have their place, but for developers who want
  local AI without the expertise tax, neither is quite right.
</p>
<p>
  <strong>In this post, you'll learn:</strong>
</p>
<ul>
  <li>Why "batteries included" matters for AI tooling</li>
  <li>How Qwen 2.5 Coder powers Caro's local inference</li>
  <li>The engineering that makes small models punch above their weight</li>
</ul>
```

#### 3. Main Content Sections

**Heading Hierarchy**:
- `<h2>`: Major sections (usually 3-6 per post)
- `<h3>`: Subsections within major sections
- `<h4>`: Rarely used; consider restructuring if needed

**Content Rhythm**:
Every 2-3 paragraphs, include ONE of:
- A code example
- A highlight box
- A blockquote
- A list (bulleted or numbered)
- An image or diagram

#### 4. Conclusion

**Purpose**: Summarize key points and transition to action.

**Pattern**:
```html
<h2>Key Takeaways</h2>

<ul>
  <li><strong>Point 1</strong>: Brief summary</li>
  <li><strong>Point 2</strong>: Brief summary</li>
  <li><strong>Point 3</strong>: Brief summary</li>
</ul>

<div class="highlight">
  <p>
    Final thought that reinforces the main message and leads into the CTA.
  </p>
</div>
```

#### 5. Call to Action (CTA)

**Purpose**: Give readers a clear next step.

**Pattern**:
```html
<h2>Try It Yourself</h2>

<pre><code># One-line installation
bash <(curl --proto '=https' --tlsv1.2 -sSfL https://setup.caro.sh)

# First command
caro "show me disk usage by directory, sorted"</code></pre>

<p>
  That's it. No API keys to configure. No models to download manually.
  Just Caro, ready to help.
</p>
```

---

## Visual Hierarchy & Typography

### Current Typography Scale

| Element | Desktop | Mobile | Line Height |
|---------|---------|--------|-------------|
| Post Title | 48px | 32px | 1.2 |
| H2 | 32px | 26px | 1.3 |
| H3 | 24px | 20px | 1.4 |
| Body | 18px | 16px | 1.8 |
| Code | 14px | 14px | 1.5 |
| Blockquote | 20px | 18px | 1.7 |

### Spacing Guidelines

| Element | Margin Top | Margin Bottom |
|---------|------------|---------------|
| H2 | 50px | 20px |
| H3 | 40px | 16px |
| Paragraph | 0 | 24px |
| Code Block | 30px | 30px |
| Highlight Box | 30px | 30px |
| List | 0 | 24px |
| Blockquote | 30px | 30px |
| HR | 50px | 50px |

### Recommended Enhancements

#### 1. Add Section Anchors

Enable deep linking to sections:

```astro
<!-- In BlogPost.astro, add auto-generated IDs -->
<script>
  document.querySelectorAll('h2, h3').forEach(heading => {
    const id = heading.textContent.toLowerCase().replace(/\s+/g, '-').replace(/[^\w-]/g, '');
    heading.id = id;
    heading.innerHTML += `<a href="#${id}" class="anchor-link">#</a>`;
  });
</script>

<style>
  .anchor-link {
    opacity: 0;
    margin-left: 8px;
    color: var(--color-text-secondary);
    transition: opacity 0.2s;
  }
  h2:hover .anchor-link,
  h3:hover .anchor-link {
    opacity: 1;
  }
</style>
```

#### 2. Add Table of Contents Component

```astro
<!-- components/TableOfContents.astro -->
---
interface Props {
  headings: { depth: number; text: string; slug: string }[];
}
const { headings } = Astro.props;
---

<nav class="toc" aria-label="Table of Contents">
  <h2 class="toc-title">In This Post</h2>
  <ul class="toc-list">
    {headings
      .filter(h => h.depth <= 3)
      .map(h => (
        <li class={`toc-item toc-depth-${h.depth}`}>
          <a href={`#${h.slug}`}>{h.text}</a>
        </li>
      ))}
  </ul>
</nav>

<style>
  .toc {
    background: var(--color-bg-tertiary);
    border-radius: 8px;
    padding: 24px;
    margin: 30px 0;
  }
  .toc-title {
    font-size: 16px;
    text-transform: uppercase;
    letter-spacing: 1px;
    color: var(--color-text-secondary);
    margin-bottom: 16px;
  }
  .toc-list {
    list-style: none;
    padding: 0;
    margin: 0;
  }
  .toc-item {
    margin-bottom: 8px;
  }
  .toc-depth-3 {
    padding-left: 20px;
  }
  .toc-item a {
    color: var(--color-text);
    text-decoration: none;
    border-bottom: 1px solid transparent;
    transition: color 0.2s, border-color 0.2s;
  }
  .toc-item a:hover {
    color: #ff8c42;
    border-bottom-color: #ff8c42;
  }
</style>
```

#### 3. Reading Progress Indicator

```astro
<!-- components/ReadingProgress.astro -->
<div class="reading-progress" id="reading-progress"></div>

<script>
  const progressBar = document.getElementById('reading-progress');
  const article = document.querySelector('.blog-post-content');

  window.addEventListener('scroll', () => {
    const articleTop = article.offsetTop;
    const articleHeight = article.offsetHeight;
    const windowHeight = window.innerHeight;
    const scrollTop = window.scrollY;

    const progress = Math.min(
      100,
      Math.max(0, ((scrollTop - articleTop + windowHeight) / articleHeight) * 100)
    );

    progressBar.style.width = `${progress}%`;
  });
</script>

<style>
  .reading-progress {
    position: fixed;
    top: 0;
    left: 0;
    height: 3px;
    background: linear-gradient(90deg, #ff8c42, #ff6b35);
    width: 0%;
    z-index: 1000;
    transition: width 0.1s ease-out;
  }
</style>
```

---

## Engagement Elements

### 1. Highlight Boxes (Callouts)

**Current Implementation** (Single Style):
```html
<div class="highlight">
  <p>Key insight or important information.</p>
</div>
```

**Enhanced Implementation** (Multiple Variants):

```html
<!-- Info callout -->
<div class="highlight highlight-info">
  <span class="highlight-icon">ℹ️</span>
  <div class="highlight-content">
    <p><strong>Note:</strong> Additional context or helpful information.</p>
  </div>
</div>

<!-- Warning callout -->
<div class="highlight highlight-warning">
  <span class="highlight-icon">⚠️</span>
  <div class="highlight-content">
    <p><strong>Caution:</strong> Important warning or consideration.</p>
  </div>
</div>

<!-- Success callout -->
<div class="highlight highlight-success">
  <span class="highlight-icon">✅</span>
  <div class="highlight-content">
    <p><strong>Best Practice:</strong> Recommended approach.</p>
  </div>
</div>

<!-- Tip callout -->
<div class="highlight highlight-tip">
  <span class="highlight-icon">💡</span>
  <div class="highlight-content">
    <p><strong>Pro Tip:</strong> Expert insight or shortcut.</p>
  </div>
</div>
```

**CSS for Variants**:
```css
.highlight {
  display: flex;
  gap: 16px;
  background: linear-gradient(135deg, var(--color-bg-tertiary) 0%, var(--color-bg) 100%);
  border-left: 4px solid #ff8c42;
  padding: 20px 24px;
  margin: 30px 0;
  border-radius: 8px;
}

.highlight-icon {
  font-size: 24px;
  flex-shrink: 0;
}

.highlight-content p {
  margin: 0;
}

.highlight-warning {
  border-left-color: #f59e0b;
  background: linear-gradient(135deg, rgba(245, 158, 11, 0.1) 0%, var(--color-bg) 100%);
}

.highlight-success {
  border-left-color: #10b981;
  background: linear-gradient(135deg, rgba(16, 185, 129, 0.1) 0%, var(--color-bg) 100%);
}

.highlight-info {
  border-left-color: #3b82f6;
  background: linear-gradient(135deg, rgba(59, 130, 246, 0.1) 0%, var(--color-bg) 100%);
}

.highlight-tip {
  border-left-color: #8b5cf6;
  background: linear-gradient(135deg, rgba(139, 92, 246, 0.1) 0%, var(--color-bg) 100%);
}
```

### 2. Code Blocks

**Current**: Basic `<pre><code>` with dark background

**Enhanced**: Add language labels and copy buttons

```astro
<!-- components/CodeBlock.astro -->
---
interface Props {
  language?: string;
  filename?: string;
}
const { language = 'shell', filename } = Astro.props;
---

<div class="code-block">
  <div class="code-header">
    {filename && <span class="code-filename">{filename}</span>}
    <span class="code-language">{language}</span>
    <button class="code-copy" data-copy>Copy</button>
  </div>
  <pre><code class={`language-${language}`}><slot /></code></pre>
</div>

<script>
  document.querySelectorAll('[data-copy]').forEach(button => {
    button.addEventListener('click', async () => {
      const code = button.closest('.code-block').querySelector('code').textContent;
      await navigator.clipboard.writeText(code);
      button.textContent = 'Copied!';
      setTimeout(() => button.textContent = 'Copy', 2000);
    });
  });
</script>

<style>
  .code-block {
    margin: 30px 0;
    border-radius: 8px;
    overflow: hidden;
    background: #1e1e1e;
  }
  .code-header {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 16px;
    background: #2d2d2d;
    border-bottom: 1px solid #3d3d3d;
    font-size: 12px;
  }
  .code-filename {
    color: #e0e0e0;
    font-family: monospace;
  }
  .code-language {
    color: #888;
    text-transform: uppercase;
    margin-left: auto;
  }
  .code-copy {
    background: transparent;
    border: 1px solid #555;
    color: #888;
    padding: 4px 12px;
    border-radius: 4px;
    cursor: pointer;
    transition: all 0.2s;
  }
  .code-copy:hover {
    background: #3d3d3d;
    color: #fff;
  }
  .code-block pre {
    margin: 0;
    padding: 20px;
  }
</style>
```

### 3. Blockquotes

**Current**: Standard styled blockquote

**Enhanced**: Add attribution support

```html
<figure class="quote">
  <blockquote>
    After the S3 deletion incident, we made Caro mandatory for all
    production access. No regrets.
  </blockquote>
  <figcaption>
    <cite>— Platform Lead, Series B Startup</cite>
  </figcaption>
</figure>
```

```css
.quote {
  margin: 30px 0;
}
.quote blockquote {
  background: var(--color-bg-tertiary);
  border-left: 4px solid #ff8c42;
  padding: 20px 30px;
  border-radius: 4px;
  font-style: italic;
  font-size: 20px;
  line-height: 1.7;
  color: var(--color-text);
  margin: 0;
}
.quote figcaption {
  padding: 12px 30px 0;
  color: var(--color-text-secondary);
  font-size: 14px;
}
.quote cite {
  font-style: normal;
}
```

### 4. Image Figures

**Enhanced**: Add proper figure with caption and lazy loading

```html
<figure class="figure">
  <img
    src="/images/blog/example.png"
    alt="Descriptive alt text for accessibility"
    loading="lazy"
    width="800"
    height="450"
  />
  <figcaption>
    Figure 1: Description of what the image shows
  </figcaption>
</figure>
```

```css
.figure {
  margin: 40px 0;
}
.figure img {
  width: 100%;
  height: auto;
  border-radius: 8px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.1);
}
.figure figcaption {
  margin-top: 12px;
  text-align: center;
  color: var(--color-text-secondary);
  font-size: 14px;
  font-style: italic;
}
```

---

## SEO & Discoverability

### Meta Tags Enhancement

**Current**: Basic title and description

**Enhanced** (add to `BlogPost.astro`):

```astro
---
interface Props {
  title: string;
  description: string;
  date: string;
  readTime: string;
  author?: string;
  image?: string;
  tags?: string[];
  category?: string;
}

const {
  title,
  description,
  date,
  readTime,
  author = "Caro Team",
  image = "/og-default.png",
  tags = [],
  category = "Engineering"
} = Astro.props;

const canonicalUrl = new URL(Astro.url.pathname, Astro.site);
---

<head>
  <!-- Primary Meta Tags -->
  <title>{title} - Caro Blog</title>
  <meta name="title" content={`${title} - Caro Blog`} />
  <meta name="description" content={description} />
  <link rel="canonical" href={canonicalUrl} />

  <!-- Open Graph / Facebook -->
  <meta property="og:type" content="article" />
  <meta property="og:url" content={canonicalUrl} />
  <meta property="og:title" content={title} />
  <meta property="og:description" content={description} />
  <meta property="og:image" content={image} />
  <meta property="article:published_time" content={date} />
  <meta property="article:author" content={author} />
  <meta property="article:section" content={category} />
  {tags.map(tag => (
    <meta property="article:tag" content={tag} />
  ))}

  <!-- Twitter -->
  <meta property="twitter:card" content="summary_large_image" />
  <meta property="twitter:url" content={canonicalUrl} />
  <meta property="twitter:title" content={title} />
  <meta property="twitter:description" content={description} />
  <meta property="twitter:image" content={image} />

  <!-- JSON-LD Structured Data -->
  <script type="application/ld+json" set:html={JSON.stringify({
    "@context": "https://schema.org",
    "@type": "BlogPosting",
    "headline": title,
    "description": description,
    "image": image,
    "datePublished": date,
    "author": {
      "@type": "Organization",
      "name": author
    },
    "publisher": {
      "@type": "Organization",
      "name": "Caro",
      "logo": {
        "@type": "ImageObject",
        "url": "/logo.png"
      }
    }
  })} />
</head>
```

### URL Structure Recommendations

**Current**: `/blog/announcing-caro`

**Best Practice**: Keep current structure, but ensure:
- URLs are lowercase
- Use hyphens, not underscores
- Keep URLs concise (< 60 characters)
- Include primary keyword

### Internal Linking Strategy

1. **Cross-reference related posts**: Link to relevant posts within content
2. **Link to documentation**: Reference GitHub, SECURITY.md, etc.
3. **Anchor text optimization**: Use descriptive anchor text, not "click here"

**Example**:
```html
<!-- Good -->
<p>
  For more details, see our <a href="/blog/security-practices">security practices guide</a>.
</p>

<!-- Bad -->
<p>
  For more details, <a href="/blog/security-practices">click here</a>.
</p>
```

---

## Post Templates

### Template 1: Announcement Post

Use for: Product launches, feature releases, major updates

```html
<BlogPost
  title="Announcing [Feature Name]: [Benefit Statement]"
  description="Brief description for search engines and social sharing"
  date="YYYY-MM-DD"
  readTime="X min read"
  category="Announcements"
  tags={["release", "feature-name"]}
>

<div class="highlight">
  <p>
    <strong>TL;DR:</strong> One-sentence summary of the announcement.
  </p>
</div>

<h2>What's New</h2>
<p>Describe the feature/update in user-focused terms.</p>

<h2>Why We Built This</h2>
<p>Context on the problem this solves.</p>

<h2>Key Features</h2>
<ul>
  <li><strong>Feature 1</strong>: Description</li>
  <li><strong>Feature 2</strong>: Description</li>
  <li><strong>Feature 3</strong>: Description</li>
</ul>

<h2>Getting Started</h2>
<pre><code># Installation or usage command</code></pre>
<p>Step-by-step guide.</p>

<h2>What's Next</h2>
<p>Roadmap preview and call for feedback.</p>

<h2>Thank You</h2>
<p>Acknowledge contributors, community, etc.</p>

<hr />
<p style="text-align: center;">
  <em>Tagline | Built with Rust | Open Source</em>
</p>

</BlogPost>
```

### Template 2: Technical Deep Dive

Use for: Architecture explanations, implementation details, how-it-works posts

```html
<BlogPost
  title="[Technical Topic]: [Outcome/Insight]"
  description="Deep dive into how Caro implements X"
  date="YYYY-MM-DD"
  readTime="X min read"
  category="Engineering"
  tags={["technical", "architecture"]}
>

<div class="highlight">
  <p>
    <strong>What you'll learn:</strong> Brief summary of technical insights.
  </p>
</div>

<h2>The Problem</h2>
<p>What challenge are we solving?</p>

<h2>Our Approach</h2>
<p>High-level solution overview.</p>

<h3>Component 1</h3>
<p>Technical details with code examples.</p>
<pre><code>// Example code</code></pre>

<h3>Component 2</h3>
<p>More technical details.</p>

<h2>Trade-offs</h2>
<p>What we considered and why we chose this approach.</p>

<h2>Results</h2>
<p>Performance metrics, improvements, outcomes.</p>

<h2>Lessons Learned</h2>
<ul>
  <li><strong>Lesson 1</strong>: Explanation</li>
  <li><strong>Lesson 2</strong>: Explanation</li>
</ul>

<h2>Resources</h2>
<ul>
  <li><a href="#">Related documentation</a></li>
  <li><a href="#">Source code</a></li>
  <li><a href="#">Further reading</a></li>
</ul>

<hr />
<p style="text-align: center;">
  <em>Built with Rust | Safety First | Open Source</em>
</p>

</BlogPost>
```

### Template 3: Story/Philosophy Post

Use for: Origin stories, philosophy discussions, community updates

```html
<BlogPost
  title="[Compelling Question or Statement]"
  description="The story/philosophy behind X"
  date="YYYY-MM-DD"
  readTime="X min read"
  category="Philosophy"
  tags={["story", "community"]}
>

<p>
  Opening narrative hook that draws readers in.
</p>

<h2>The Beginning</h2>
<p>Set the scene, introduce characters/context.</p>

<blockquote>
  Memorable quote that captures the essence.
</blockquote>

<h2>The Journey</h2>
<p>Tell the story with vivid details.</p>

<div class="highlight">
  <p>
    Key insight or turning point.
  </p>
</div>

<h2>The Lessons</h2>
<p>What we/readers can take away.</p>

<h2>Looking Forward</h2>
<p>How this shapes the future.</p>

<hr />
<p style="font-style: italic; color: #7f8c8d;">
  If you enjoyed this story, [call to action].
</p>

</BlogPost>
```

### Template 4: Security/Best Practices Post

Use for: Security advisories, best practices guides, compliance information

```html
<BlogPost
  title="[Security Topic]: [Clear Benefit/Outcome]"
  description="How Caro implements X for security"
  date="YYYY-MM-DD"
  readTime="X min read"
  category="Security"
  tags={["security", "best-practices"]}
>

<div class="highlight highlight-warning">
  <p>
    <strong>Security Notice:</strong> Important context or urgency.
  </p>
</div>

<h2>Why This Matters</h2>
<p>Stakes and impact of the security topic.</p>

<h2>The Threat Model</h2>
<p>What we're protecting against.</p>

<h2>Our Security Controls</h2>

<h3>Control 1: [Name]</h3>
<p>Description and implementation.</p>

<h3>Control 2: [Name]</h3>
<p>Description and implementation.</p>

<h2>Best Practices for Users</h2>
<ol>
  <li><strong>Practice 1</strong>: Guidance</li>
  <li><strong>Practice 2</strong>: Guidance</li>
  <li><strong>Practice 3</strong>: Guidance</li>
</ol>

<h2>Reporting Vulnerabilities</h2>
<p>How to responsibly disclose issues.</p>

<h2>Resources</h2>
<ul>
  <li><a href="#">SECURITY.md</a></li>
  <li><a href="#">Security advisories</a></li>
</ul>

<hr />
<p style="text-align: center;">
  <em>Built with Rust | Safety First | Open Source</em>
</p>

</BlogPost>
```

---

## Style Consistency Checklist

Use this checklist before publishing any blog post:

### Content Quality

- [ ] **Hook**: Opening highlight box captures attention and states value
- [ ] **Promise**: Introduction tells readers what they'll learn
- [ ] **Structure**: Logical flow with clear section progression
- [ ] **Rhythm**: Visual variety every 2-3 paragraphs
- [ ] **Conclusion**: Summary reinforces key points
- [ ] **CTA**: Clear next step for readers

### Formatting

- [ ] **Heading hierarchy**: H2 for sections, H3 for subsections only
- [ ] **List formatting**: Consistent use of bullets vs. numbers
- [ ] **Code blocks**: Proper syntax, reasonable length
- [ ] **Links**: Descriptive anchor text, opens external in new tab
- [ ] **Images**: Alt text, captions, lazy loading

### Metadata

- [ ] **Title**: < 60 characters, includes primary keyword
- [ ] **Description**: < 160 characters, compelling summary
- [ ] **Date**: Accurate publication date
- [ ] **Read time**: Calculated correctly (avg 200 words/min)
- [ ] **Category/Tags**: Appropriate categorization

### Technical

- [ ] **No broken links**: All internal/external links work
- [ ] **No console errors**: Page loads cleanly
- [ ] **Mobile responsive**: Content readable on mobile
- [ ] **Dark mode**: All elements render correctly in dark mode

### Brand Consistency

- [ ] **Tone**: Friendly, technical, trustworthy
- [ ] **Voice**: Active, direct, clear
- [ ] **Terminology**: Consistent product/feature names
- [ ] **Footer**: Standard signature line

---

## Implementation Recommendations

### Priority 1: Quick Wins (Implement Now)

1. **Add author attribution** to BlogPost layout
2. **Standardize highlight box usage** across all posts
3. **Add "Updated" date support** for revised content
4. **Implement copy button** for code blocks

### Priority 2: Medium-Term (Next Sprint)

1. **Build TableOfContents component** for long posts
2. **Add reading progress indicator**
3. **Implement callout variants** (info, warning, tip, success)
4. **Add social sharing buttons**

### Priority 3: Long-Term (Future)

1. **Category/tag system** with filtered views
2. **Related posts** recommendation engine
3. **Full syntax highlighting** with Shiki/Prism
4. **RSS feed** generation
5. **Newsletter integration**
6. **Search functionality**

---

## Appendix: Component Reference

### Available Classes

| Class | Purpose | Usage |
|-------|---------|-------|
| `.highlight` | Callout box | Important information, key insights |
| `.highlight-info` | Info callout | Notes, additional context |
| `.highlight-warning` | Warning callout | Cautions, important considerations |
| `.highlight-success` | Success callout | Best practices, recommended approaches |
| `.highlight-tip` | Tip callout | Pro tips, expert insights |

### HTML Elements

| Element | Style Applied | Notes |
|---------|---------------|-------|
| `<h2>` | 32px, bold | Major sections |
| `<h3>` | 24px, semi-bold | Subsections |
| `<p>` | 18px, 1.8 line-height | Body text |
| `<ul>/<ol>` | 18px, indented | Lists |
| `<blockquote>` | Italic, bordered | Quotes |
| `<pre><code>` | Dark theme, monospace | Code blocks |
| `<code>` | Inline, orange | Inline code |
| `<a>` | Orange, underline on hover | Links |

---

*Last updated: December 2025*
*Maintained by: Caro Website Team*
