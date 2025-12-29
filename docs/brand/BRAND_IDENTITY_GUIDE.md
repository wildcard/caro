# Caro Blog Brand Identity Guide

**Inspired by Daytona's Dotfiles Insider Aesthetic**

A comprehensive brand identity system for a modern developer-focused tech blog that embodies technical sophistication, clarity, and developer-first design principles.

---

## Table of Contents

1. [Brand Philosophy](#brand-philosophy)
2. [Color System](#color-system)
3. [Typography](#typography)
4. [Spacing & Layout](#spacing--layout)
5. [Component Patterns](#component-patterns)
6. [Imagery Guidelines](#imagery-guidelines)
7. [Motion & Interaction](#motion--interaction)
8. [Code Examples](#code-examples)
9. [Accessibility](#accessibility)
10. [Brand Voice](#brand-voice)

---

## Brand Philosophy

### Core Principles

**1. Technical Authenticity**
The design speaks the language of developers. Monospace typography, terminal-inspired color schemes, and precise spacing create an environment that feels like home for technical audiences.

**2. Dark-First Design**
Dark mode is the default, not an afterthought. This respects developer preferences and reduces eye strain during extended reading sessions.

**3. Functional Minimalism**
Every design element serves a purpose. We eliminate decoration in favor of clarity, letting content breathe and information hierarchy guide the reader.

**4. Respectful Interaction**
Subtle animations and hover states provide feedback without distraction. The interface responds to the user, never demanding attention.

### Brand Personality

| Trait | Expression |
|-------|------------|
| **Precise** | Grid-based layouts, consistent spacing, mathematical proportions |
| **Intelligent** | High information density done right, smart typography choices |
| **Approachable** | Warm accents, inviting CTAs, conversational content |
| **Trustworthy** | Solid contrast ratios, reliable patterns, professional polish |

---

## Color System

### Primary Palette

```css
:root {
  /* ═══════════════════════════════════════════════════════════════
     BACKGROUND COLORS
     Deep, rich blacks that create depth without being harsh
     ═══════════════════════════════════════════════════════════════ */

  --bg-primary: #0a0a0a;        /* Main background - deepest black */
  --bg-secondary: #111111;      /* Card backgrounds, elevated surfaces */
  --bg-tertiary: #161616;       /* Subtle differentiation, code blocks */
  --bg-elevated: #1a1a1a;       /* Modals, dropdowns, popovers */
  --bg-hover: #252525;          /* Hover states for interactive elements */

  /* ═══════════════════════════════════════════════════════════════
     TEXT COLORS
     High contrast for accessibility, subtle grays for hierarchy
     ═══════════════════════════════════════════════════════════════ */

  --text-primary: #ffffff;      /* Headlines, primary content */
  --text-secondary: #a0a0a0;    /* Metadata, descriptions, captions */
  --text-tertiary: #666666;     /* Timestamps, subtle labels */
  --text-muted: #404040;        /* Disabled states, placeholders */

  /* ═══════════════════════════════════════════════════════════════
     ACCENT COLORS
     Vibrant highlights that pop against dark backgrounds
     ═══════════════════════════════════════════════════════════════ */

  --accent-primary: #00d991;    /* Primary CTA, success states */
  --accent-primary-hover: #00ffaa;
  --accent-secondary: #0080ff;  /* Links, secondary actions */
  --accent-secondary-hover: #59acff;

  /* ═══════════════════════════════════════════════════════════════
     SEMANTIC COLORS
     Consistent meaning across the interface
     ═══════════════════════════════════════════════════════════════ */

  --color-success: #00d991;
  --color-warning: #ffb800;
  --color-error: #ff4d4d;
  --color-info: #0080ff;

  /* ═══════════════════════════════════════════════════════════════
     BORDER COLORS
     Subtle divisions that don't compete with content
     ═══════════════════════════════════════════════════════════════ */

  --border-subtle: #1f1f1f;
  --border-default: #2a2a2a;
  --border-strong: #333333;
  --border-accent: var(--accent-primary);
}
```

### Light Mode Override

```css
[data-theme="light"] {
  --bg-primary: #ffffff;
  --bg-secondary: #f8f9fa;
  --bg-tertiary: #f1f3f4;
  --bg-elevated: #ffffff;
  --bg-hover: #e8eaed;

  --text-primary: #1a1a1a;
  --text-secondary: #5f6368;
  --text-tertiary: #80868b;
  --text-muted: #bdc1c6;

  --border-subtle: #e8eaed;
  --border-default: #dadce0;
  --border-strong: #c4c7c9;
}
```

### Color Usage Guidelines

| Use Case | Color Variable | Example |
|----------|---------------|---------|
| Page backgrounds | `--bg-primary` | Main content area |
| Cards & containers | `--bg-secondary` | Article cards, sidebars |
| Code blocks | `--bg-tertiary` | Inline and block code |
| Headlines | `--text-primary` | H1, H2, article titles |
| Body text | `--text-primary` | Paragraphs, lists |
| Metadata | `--text-secondary` | Dates, authors, read time |
| Primary buttons | `--accent-primary` | Subscribe, Read More |
| Links | `--accent-secondary` | Inline links, navigation |

---

## Typography

### Font Stack

```css
:root {
  /* Primary display font - technical, precise, developer-native */
  --font-display: 'Berkeley Mono', 'JetBrains Mono', 'Fira Code',
                  'SF Mono', 'Monaco', monospace;

  /* Body font - clean, readable, modern */
  --font-body: 'Inter', 'SF Pro Text', -apple-system,
               BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;

  /* Code font - matches display for consistency */
  --font-code: var(--font-display);
}
```

### Type Scale

Based on a 1.25 ratio (Major Third) for harmonious proportions:

```css
:root {
  /* ═══════════════════════════════════════════════════════════════
     FONT SIZES
     Modular scale: 1.25 ratio (Major Third)
     Base: 16px
     ═══════════════════════════════════════════════════════════════ */

  --text-xs: 0.64rem;     /* 10.24px - Legal, fine print */
  --text-sm: 0.8rem;      /* 12.8px  - Captions, metadata */
  --text-base: 1rem;      /* 16px    - Body text */
  --text-lg: 1.25rem;     /* 20px    - Lead paragraphs */
  --text-xl: 1.563rem;    /* 25px    - H4, card titles */
  --text-2xl: 1.953rem;   /* 31.25px - H3 */
  --text-3xl: 2.441rem;   /* 39px    - H2 */
  --text-4xl: 3.052rem;   /* 48.8px  - H1 */
  --text-5xl: 3.815rem;   /* 61px    - Hero headlines */

  /* ═══════════════════════════════════════════════════════════════
     LINE HEIGHTS
     Optimized for each size bracket
     ═══════════════════════════════════════════════════════════════ */

  --leading-none: 1;
  --leading-tight: 1.15;    /* Headlines */
  --leading-snug: 1.35;     /* Subheadings */
  --leading-normal: 1.5;    /* Short paragraphs */
  --leading-relaxed: 1.65;  /* Long-form content */
  --leading-loose: 1.85;    /* Maximum readability */

  /* ═══════════════════════════════════════════════════════════════
     FONT WEIGHTS
     ═══════════════════════════════════════════════════════════════ */

  --font-normal: 400;
  --font-medium: 500;
  --font-semibold: 600;
  --font-bold: 700;
  --font-black: 900;

  /* ═══════════════════════════════════════════════════════════════
     LETTER SPACING
     Tighter for headlines, looser for small text
     ═══════════════════════════════════════════════════════════════ */

  --tracking-tighter: -0.04em;
  --tracking-tight: -0.02em;
  --tracking-normal: 0;
  --tracking-wide: 0.02em;
  --tracking-wider: 0.05em;
  --tracking-widest: 0.1em;
}
```

### Typography Patterns

```css
/* Hero Headlines */
.hero-title {
  font-family: var(--font-display);
  font-size: var(--text-5xl);
  font-weight: var(--font-bold);
  line-height: var(--leading-tight);
  letter-spacing: var(--tracking-tighter);
  color: var(--text-primary);
}

/* Article Titles */
.article-title {
  font-family: var(--font-display);
  font-size: var(--text-3xl);
  font-weight: var(--font-semibold);
  line-height: var(--leading-snug);
  letter-spacing: var(--tracking-tight);
  color: var(--text-primary);
}

/* Card Titles */
.card-title {
  font-family: var(--font-display);
  font-size: var(--text-xl);
  font-weight: var(--font-medium);
  line-height: var(--leading-snug);
  color: var(--text-primary);
}

/* Body Text */
.body-text {
  font-family: var(--font-body);
  font-size: var(--text-base);
  font-weight: var(--font-normal);
  line-height: var(--leading-relaxed);
  color: var(--text-primary);
}

/* Metadata */
.metadata {
  font-family: var(--font-display);
  font-size: var(--text-sm);
  font-weight: var(--font-medium);
  line-height: var(--leading-normal);
  letter-spacing: var(--tracking-wider);
  text-transform: uppercase;
  color: var(--text-secondary);
}

/* Code */
.code-inline {
  font-family: var(--font-code);
  font-size: 0.9em;
  padding: 0.125em 0.375em;
  background: var(--bg-tertiary);
  border-radius: 4px;
}
```

---

## Spacing & Layout

### Spacing Scale

```css
:root {
  /* ═══════════════════════════════════════════════════════════════
     SPACING SCALE
     Based on 4px base unit for precise alignment
     ═══════════════════════════════════════════════════════════════ */

  --space-px: 1px;
  --space-0: 0;
  --space-0.5: 0.125rem;   /* 2px */
  --space-1: 0.25rem;      /* 4px */
  --space-1.5: 0.375rem;   /* 6px */
  --space-2: 0.5rem;       /* 8px */
  --space-2.5: 0.625rem;   /* 10px */
  --space-3: 0.75rem;      /* 12px */
  --space-3.5: 0.875rem;   /* 14px */
  --space-4: 1rem;         /* 16px */
  --space-5: 1.25rem;      /* 20px */
  --space-6: 1.5rem;       /* 24px */
  --space-7: 1.75rem;      /* 28px */
  --space-8: 2rem;         /* 32px */
  --space-9: 2.25rem;      /* 36px */
  --space-10: 2.5rem;      /* 40px */
  --space-11: 2.75rem;     /* 44px */
  --space-12: 3rem;        /* 48px */
  --space-14: 3.5rem;      /* 56px */
  --space-16: 4rem;        /* 64px */
  --space-20: 5rem;        /* 80px */
  --space-24: 6rem;        /* 96px */
  --space-28: 7rem;        /* 112px */
  --space-32: 8rem;        /* 128px */
  --space-36: 9rem;        /* 144px */
  --space-40: 10rem;       /* 160px */
}
```

### Container Widths

```css
:root {
  /* Content containers */
  --container-xs: 320px;    /* Mobile small */
  --container-sm: 640px;    /* Mobile large */
  --container-md: 768px;    /* Tablet */
  --container-lg: 1024px;   /* Desktop small */
  --container-xl: 1248px;   /* Desktop large - primary content width */
  --container-2xl: 1440px;  /* Ultrawide */

  /* Article content - optimized for reading */
  --prose-width: 720px;     /* Ideal line length: 65-75 characters */
  --prose-wide: 900px;      /* Code blocks, images */
}
```

### Grid System

```css
/* Base Layout Container */
.container {
  width: 100%;
  max-width: var(--container-xl);
  margin-inline: auto;
  padding-inline: var(--space-6);
}

@media (min-width: 768px) {
  .container {
    padding-inline: var(--space-10);
  }
}

/* Article Grid - 2 Column */
.article-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: var(--space-10);
}

@media (max-width: 1024px) {
  .article-grid {
    grid-template-columns: 1fr;
  }
}

/* Feature Grid - 3 Column */
.feature-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: var(--space-8);
}

@media (max-width: 1024px) {
  .feature-grid {
    grid-template-columns: repeat(2, 1fr);
  }
}

@media (max-width: 650px) {
  .feature-grid {
    grid-template-columns: 1fr;
  }
}
```

### Section Spacing

```css
/* Consistent vertical rhythm between major sections */
.section {
  padding-block: var(--space-20);
}

.section-lg {
  padding-block: var(--space-28);
}

/* Card internal padding */
.card {
  padding: var(--space-10);
}

.card-sm {
  padding: var(--space-6);
}

.card-lg {
  padding: var(--space-12);
}
```

---

## Component Patterns

### Article Card

```html
<article class="article-card">
  <div class="article-card__image">
    <img src="..." alt="..." />
    <div class="article-card__image-backdrop"></div>
  </div>
  <div class="article-card__content">
    <div class="article-card__meta">
      <span class="article-card__date">DEC 15 2025</span>
      <span class="article-card__divider">·</span>
      <span class="article-card__author">ALEX CHEN</span>
    </div>
    <h2 class="article-card__title">
      Building CLI Tools That Developers Actually Want to Use
    </h2>
    <p class="article-card__excerpt">
      A deep dive into the principles of developer experience...
    </p>
    <a href="#" class="article-card__link">
      Read more <span aria-hidden="true">↗</span>
    </a>
  </div>
</article>
```

```css
.article-card {
  display: grid;
  grid-template-columns: 380px 1fr;
  gap: var(--space-8);
  padding: var(--space-10);
  border-top: 1px solid var(--border-subtle);
  transition: background-color 0.6s ease;
}

.article-card:hover {
  background-color: var(--bg-hover);
}

.article-card__image {
  position: relative;
  aspect-ratio: 2 / 1;
  overflow: hidden;
  border-radius: 8px;
  transition: transform 0.6s ease;
}

.article-card:hover .article-card__image {
  transform: translateY(-10px);
}

.article-card__image-backdrop {
  position: absolute;
  inset: 10px;
  z-index: -1;
  background: var(--accent-primary);
  opacity: 0;
  transition: opacity 0.6s ease;
}

.article-card:hover .article-card__image-backdrop {
  opacity: 0.3;
}

.article-card__image img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.article-card__meta {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  margin-bottom: var(--space-4);
  font-family: var(--font-display);
  font-size: var(--text-sm);
  letter-spacing: var(--tracking-wider);
  text-transform: uppercase;
  color: var(--text-secondary);
}

.article-card__divider {
  color: var(--text-tertiary);
}

.article-card__title {
  font-family: var(--font-display);
  font-size: var(--text-2xl);
  font-weight: var(--font-semibold);
  line-height: var(--leading-snug);
  color: var(--text-primary);
  margin-bottom: var(--space-4);
}

.article-card__excerpt {
  font-family: var(--font-body);
  font-size: var(--text-base);
  line-height: var(--leading-relaxed);
  color: var(--text-secondary);
  margin-bottom: var(--space-6);
}

.article-card__link {
  font-family: var(--font-display);
  font-size: var(--text-sm);
  font-weight: var(--font-medium);
  letter-spacing: var(--tracking-wide);
  text-transform: uppercase;
  color: var(--text-secondary);
  text-decoration: none;
  transition: color 0.3s ease;
}

.article-card__link:hover {
  color: var(--accent-primary);
}
```

### Newsletter Signup

```html
<section class="newsletter">
  <div class="newsletter__content">
    <h2 class="newsletter__title">Stay in the loop</h2>
    <p class="newsletter__description">
      Weekly insights on developer tools, CLI design, and building great software.
    </p>
  </div>
  <form class="newsletter__form">
    <input
      type="email"
      class="newsletter__input"
      placeholder="you@company.com"
      required
    />
    <button type="submit" class="newsletter__button">
      Subscribe
    </button>
  </form>
</section>
```

```css
.newsletter {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-10);
  padding: var(--space-12);
  background: var(--bg-secondary);
  border: 1px solid var(--border-default);
  border-radius: 12px;
}

.newsletter__title {
  font-family: var(--font-display);
  font-size: var(--text-2xl);
  font-weight: var(--font-semibold);
  color: var(--text-primary);
  margin-bottom: var(--space-2);
}

.newsletter__description {
  font-family: var(--font-body);
  font-size: var(--text-base);
  color: var(--text-secondary);
}

.newsletter__form {
  display: flex;
  gap: var(--space-3);
}

.newsletter__input {
  min-width: 280px;
  padding: var(--space-3) var(--space-4);
  font-family: var(--font-body);
  font-size: var(--text-base);
  color: var(--text-primary);
  background: var(--bg-tertiary);
  border: 1px solid var(--border-default);
  border-radius: 8px;
  outline: none;
  transition: border-color 0.3s ease, box-shadow 0.3s ease;
}

.newsletter__input:focus {
  border-color: var(--accent-primary);
  box-shadow: 0 0 0 3px rgba(0, 217, 145, 0.15);
}

.newsletter__input::placeholder {
  color: var(--text-tertiary);
}

.newsletter__button {
  padding: var(--space-3) var(--space-6);
  font-family: var(--font-display);
  font-size: var(--text-sm);
  font-weight: var(--font-semibold);
  letter-spacing: var(--tracking-wide);
  text-transform: uppercase;
  color: var(--bg-primary);
  background: var(--accent-primary);
  border: none;
  border-radius: 8px;
  cursor: pointer;
  transition: background-color 0.3s ease, transform 0.15s ease;
}

.newsletter__button:hover {
  background: var(--accent-primary-hover);
}

.newsletter__button:active {
  transform: scale(0.98);
}
```

### Tag/Category Pills

```html
<div class="tag-list">
  <a href="#" class="tag tag--primary">CLI Tools</a>
  <a href="#" class="tag">Developer Experience</a>
  <a href="#" class="tag">Rust</a>
  <a href="#" class="tag">Open Source</a>
</div>
```

```css
.tag-list {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-2);
}

.tag {
  display: inline-flex;
  align-items: center;
  padding: var(--space-1.5) var(--space-3);
  font-family: var(--font-display);
  font-size: var(--text-xs);
  font-weight: var(--font-medium);
  letter-spacing: var(--tracking-wider);
  text-transform: uppercase;
  text-decoration: none;
  color: var(--text-secondary);
  background: var(--bg-tertiary);
  border: 1px solid var(--border-subtle);
  border-radius: 100px;
  transition: all 0.3s ease;
}

.tag:hover {
  color: var(--accent-primary);
  border-color: var(--accent-primary);
  background: rgba(0, 217, 145, 0.1);
}

.tag--primary {
  color: var(--bg-primary);
  background: var(--accent-primary);
  border-color: var(--accent-primary);
}

.tag--primary:hover {
  background: var(--accent-primary-hover);
  border-color: var(--accent-primary-hover);
}
```

### Code Block

```html
<div class="code-block">
  <div class="code-block__header">
    <span class="code-block__language">rust</span>
    <button class="code-block__copy">Copy</button>
  </div>
  <pre class="code-block__content"><code>fn main() {
    println!("Hello, world!");
}</code></pre>
</div>
```

```css
.code-block {
  border-radius: 12px;
  overflow: hidden;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-subtle);
}

.code-block__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-3) var(--space-4);
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-subtle);
}

.code-block__language {
  font-family: var(--font-display);
  font-size: var(--text-xs);
  font-weight: var(--font-medium);
  letter-spacing: var(--tracking-wider);
  text-transform: uppercase;
  color: var(--text-secondary);
}

.code-block__copy {
  padding: var(--space-1) var(--space-2);
  font-family: var(--font-display);
  font-size: var(--text-xs);
  font-weight: var(--font-medium);
  color: var(--text-tertiary);
  background: transparent;
  border: 1px solid var(--border-default);
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.3s ease;
}

.code-block__copy:hover {
  color: var(--text-primary);
  border-color: var(--text-primary);
}

.code-block__content {
  padding: var(--space-6);
  margin: 0;
  font-family: var(--font-code);
  font-size: var(--text-sm);
  line-height: var(--leading-relaxed);
  color: var(--text-primary);
  overflow-x: auto;
}

.code-block__content code {
  font-family: inherit;
}
```

---

## Imagery Guidelines

### Featured Images

**Aspect Ratios**
- Article cards: `2:1` (landscape, wide view)
- Guide cards: `5:8` (portrait, tall)
- Hero images: `16:9` or `21:9` (cinematic)
- Thumbnails: `1:1` (square)

**Visual Style**
- Dark, moody backgrounds that complement the dark UI
- Subtle gradients and abstract patterns
- Developer-focused imagery: terminals, code, workspace setups
- Avoid stock photography with posed people
- Prefer illustrations, diagrams, or abstract visuals

**Image Treatment**
```css
.featured-image {
  filter: brightness(0.9) contrast(1.05);
  transition: filter 0.6s ease;
}

.featured-image:hover {
  filter: brightness(1) contrast(1);
}
```

### Icons

**Style Guidelines**
- Line icons preferred over filled
- 1.5px stroke weight for consistency
- 24px base size with 20px and 16px variants
- Rounded line caps and joins
- Match `--text-secondary` color by default

**Recommended Icon Sets**
- [Lucide](https://lucide.dev) - Primary choice
- [Phosphor](https://phosphoricons.com) - Alternative
- [Heroicons](https://heroicons.com) - Outline variant

---

## Motion & Interaction

### Timing Functions

```css
:root {
  /* Standard easing curves */
  --ease-in: cubic-bezier(0.4, 0, 1, 1);
  --ease-out: cubic-bezier(0, 0, 0.2, 1);
  --ease-in-out: cubic-bezier(0.4, 0, 0.2, 1);

  /* Spring-like curves for playful interactions */
  --ease-spring: cubic-bezier(0.34, 1.56, 0.64, 1);

  /* Duration scale */
  --duration-fast: 150ms;
  --duration-base: 300ms;
  --duration-slow: 500ms;
  --duration-slower: 700ms;
}
```

### Hover Patterns

```css
/* Subtle lift effect for cards */
.card {
  transition:
    transform var(--duration-slow) var(--ease-out),
    box-shadow var(--duration-slow) var(--ease-out);
}

.card:hover {
  transform: translateY(-4px);
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.3);
}

/* Image lift with accent backdrop (Dotfiles Insider signature) */
.card__image {
  position: relative;
  transition: transform var(--duration-slow) var(--ease-out);
}

.card__image::after {
  content: '';
  position: absolute;
  inset: 8px;
  z-index: -1;
  background: var(--accent-primary);
  opacity: 0;
  transition: opacity var(--duration-slow) var(--ease-out);
}

.card:hover .card__image {
  transform: translateY(-10px);
}

.card:hover .card__image::after {
  opacity: 0.25;
}

/* Link color transitions */
a {
  transition: color var(--duration-fast) var(--ease-out);
}

/* Button press effect */
button:active {
  transform: scale(0.98);
  transition: transform var(--duration-fast) var(--ease-in);
}
```

### Focus States

```css
/* Accessible focus ring */
:focus-visible {
  outline: 2px solid var(--accent-primary);
  outline-offset: 2px;
}

/* Input focus */
input:focus,
textarea:focus {
  border-color: var(--accent-primary);
  box-shadow: 0 0 0 3px rgba(0, 217, 145, 0.15);
}
```

---

## Code Examples

### Complete Page Template

```html
<!DOCTYPE html>
<html lang="en" data-theme="dark">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Caro Blog - Developer Insights</title>
  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&display=swap" rel="stylesheet">
  <link href="https://fonts.cdnfonts.com/css/berkeley-mono" rel="stylesheet">
  <link rel="stylesheet" href="styles.css">
</head>
<body>
  <header class="header">
    <div class="container header__container">
      <a href="/" class="header__logo">
        <span class="header__logo-icon">⌘</span>
        <span class="header__logo-text">Caro</span>
      </a>
      <nav class="header__nav">
        <a href="/articles" class="header__link">Articles</a>
        <a href="/guides" class="header__link">Guides</a>
        <a href="/changelog" class="header__link">Changelog</a>
      </nav>
      <button class="header__theme-toggle" aria-label="Toggle theme">
        <svg><!-- moon/sun icon --></svg>
      </button>
    </div>
  </header>

  <main>
    <section class="hero section-lg">
      <div class="container">
        <h1 class="hero-title">Developer Insights</h1>
        <p class="hero-subtitle">
          Weekly deep dives into CLI tools, developer experience,
          and building software that developers love.
        </p>
      </div>
    </section>

    <section class="articles section">
      <div class="container">
        <div class="article-grid">
          <!-- Article cards here -->
        </div>
      </div>
    </section>

    <section class="newsletter-section section">
      <div class="container">
        <!-- Newsletter component here -->
      </div>
    </section>
  </main>

  <footer class="footer">
    <div class="container footer__container">
      <p class="footer__copyright">
        &copy; 2025 Caro. Open source under MIT license.
      </p>
      <div class="footer__links">
        <a href="https://github.com/wildcard/caro">GitHub</a>
        <a href="/rss.xml">RSS</a>
      </div>
    </div>
  </footer>
</body>
</html>
```

### CSS Reset & Base Styles

```css
/* Modern CSS Reset */
*, *::before, *::after {
  box-sizing: border-box;
}

* {
  margin: 0;
}

html {
  font-size: 16px;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

body {
  font-family: var(--font-body);
  font-size: var(--text-base);
  line-height: var(--leading-relaxed);
  color: var(--text-primary);
  background: var(--bg-primary);
  min-height: 100vh;
}

img, picture, video, canvas, svg {
  display: block;
  max-width: 100%;
}

input, button, textarea, select {
  font: inherit;
}

p, h1, h2, h3, h4, h5, h6 {
  overflow-wrap: break-word;
}

/* Selection styling */
::selection {
  background: var(--accent-primary);
  color: var(--bg-primary);
}

/* Scrollbar styling (Webkit) */
::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}

::-webkit-scrollbar-track {
  background: var(--bg-secondary);
}

::-webkit-scrollbar-thumb {
  background: var(--border-strong);
  border-radius: 4px;
}

::-webkit-scrollbar-thumb:hover {
  background: var(--text-tertiary);
}
```

---

## Accessibility

### Contrast Requirements

All text meets WCAG 2.1 AA standards:

| Text Type | Foreground | Background | Ratio |
|-----------|------------|------------|-------|
| Body text | `#ffffff` | `#0a0a0a` | 21:1 |
| Secondary text | `#a0a0a0` | `#0a0a0a` | 9.6:1 |
| Tertiary text | `#666666` | `#0a0a0a` | 4.8:1 |
| Accent on dark | `#00d991` | `#0a0a0a` | 9.2:1 |

### Keyboard Navigation

- All interactive elements are focusable
- Focus order follows logical reading order
- Skip links provided for main content
- Focus trapping in modals and dropdowns

### Screen Reader Support

```html
<!-- Decorative icons are hidden -->
<span aria-hidden="true">↗</span>

<!-- Informative icons have labels -->
<button aria-label="Toggle dark mode">
  <svg aria-hidden="true"><!-- icon --></svg>
</button>

<!-- Live regions for dynamic content -->
<div aria-live="polite" aria-atomic="true">
  <!-- Toast notifications appear here -->
</div>
```

### Reduced Motion

```css
@media (prefers-reduced-motion: reduce) {
  *,
  *::before,
  *::after {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
  }
}
```

---

## Brand Voice

### Writing Guidelines

**Tone**: Technical but approachable. We assume our readers are smart developers who appreciate clarity over cleverness.

**Do**
- Use active voice
- Keep sentences concise
- Include code examples
- Explain the "why" not just the "how"
- Use second person ("you") to speak directly to readers

**Don't**
- Use jargon without explanation
- Write walls of text without structure
- Assume knowledge of niche tools
- Use clickbait headlines
- Over-promise capabilities

### Example Headlines

**Good**
- "Building CLI Tools with Rust: A Practical Guide"
- "Why We Switched from SQLite to PostgreSQL"
- "Understanding Shell Command Safety"

**Bad**
- "You Won't Believe These 10 Rust Tips!"
- "The Ultimate, Comprehensive, Complete Guide to Everything CLI"
- "Revolutionary New Approach Disrupts Industry"

### Metadata Formatting

- **Dates**: `DEC 15 2025` (uppercase, abbreviated month)
- **Authors**: `ALEX CHEN` (uppercase, full name)
- **Reading time**: `8 MIN READ` (uppercase, abbreviated)
- **Categories**: `CLI TOOLS` (uppercase, with space)

---

## Quick Reference

### Color Tokens
```
Primary BG:     #0a0a0a
Card BG:        #111111
Code BG:        #161616
Primary Text:   #ffffff
Secondary Text: #a0a0a0
Accent Green:   #00d991
Accent Blue:    #0080ff
```

### Font Tokens
```
Display:  Berkeley Mono, monospace
Body:     Inter, sans-serif
Code:     Berkeley Mono, monospace
```

### Spacing Tokens
```
xs:   4px
sm:   8px
md:   16px
lg:   24px
xl:   40px
2xl:  64px
3xl:  96px
```

### Breakpoints
```
Mobile:  < 650px
Tablet:  650px - 1024px
Desktop: > 1024px
Wide:    > 1440px
```

---

## Resources

- [Daytona Brand Hub](https://www.daytona.io/company/brand) - Original inspiration
- [Dotfiles Insider](https://www.daytona.io/dotfiles) - Design reference
- [Inter Font](https://rsms.me/inter/) - Body typography
- [Berkeley Mono](https://berkeleygraphics.com/typefaces/berkeley-mono/) - Display typography
- [Lucide Icons](https://lucide.dev) - Icon system

---

*This brand identity guide was created for the Caro project, inspired by Daytona's Dotfiles Insider aesthetic. The design system prioritizes developer experience, accessibility, and technical authenticity.*
