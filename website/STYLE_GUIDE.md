# Caro Blog Style Guide

**A comprehensive design system for the Caro tech blog.**

Inspired by Daytona's Dotfiles Insider aesthetic, this design system embodies technical authenticity, functional minimalism, and developer-first principles.

---

## Table of Contents

1. [Brand Guidelines](#brand-guidelines)
2. [Color System](#color-system)
3. [Typography](#typography)
4. [Spacing & Layout](#spacing--layout)
5. [Components](#components)
6. [Animation & Motion](#animation--motion)
7. [Accessibility](#accessibility)
8. [Usage Examples](#usage-examples)

---

## Brand Guidelines

### Brand Name

**Caro** - Always lowercase in logos and branding, capitalized at the start of sentences.

- Logo: `caro`
- In text: "Caro is a CLI tool..."
- Never: "CARO", "CaRo", or "Caro CLI Tool™"

### Tone of Voice

| Characteristic | Description |
|----------------|-------------|
| **Technical** | We speak the language of developers. Use precise terminology. |
| **Approachable** | Technical doesn't mean cold. Be helpful and human. |
| **Confident** | State facts clearly. Avoid hedging language. |
| **Concise** | Respect the reader's time. Get to the point. |

### Target Audience

- Professional software developers
- CLI enthusiasts and power users
- AI/ML practitioners interested in local inference
- Open source contributors
- Developers building internal tools

### Brand Personality

```
Precise → We value clarity and accuracy
Intelligent → We assume smart readers
Trustworthy → We earn confidence through quality
Approachable → We welcome newcomers
```

---

## Color System

### Primary Palette

| Token | Hex | Usage |
|-------|-----|-------|
| `--green-500` | `#00E500` | Primary accent, CTAs, success states |
| `--blue-500` | `#0080FF` | Links, secondary actions, info states |
| `--coral-500` | `#FF5733` | Warnings, highlights, attention |

### Background Colors (Dark Mode)

| Token | Hex | Usage |
|-------|-----|-------|
| `--bg-base` | `#000000` | Page background |
| `--bg-surface` | `#0a0a0a` | Elevated surfaces |
| `--bg-elevated` | `#0f0f0f` | Cards, modals |
| `--bg-subtle` | `#1a1a1a` | Subtle backgrounds |
| `--bg-hover` | `#292929` | Hover states |

### Text Colors

| Token | Hex | Usage |
|-------|-----|-------|
| `--text-primary` | `#ffffff` | Headlines, body text |
| `--text-secondary` | `#9a9a9a` | Descriptions, metadata |
| `--text-tertiary` | `#666666` | Timestamps, captions |
| `--text-muted` | `#404040` | Disabled, placeholders |

### Usage Rules

1. **Never use raw color values** - Always use CSS custom properties
2. **Maintain contrast ratios** - Primary text on dark: 21:1, Secondary: 9.6:1
3. **Use semantic tokens** - Prefer `--color-success` over `--green-500` for states
4. **Test in both themes** - Verify colors work in dark and light mode

---

## Typography

### Font Stack

```css
/* Display & Code */
--font-mono: 'JetBrains Mono', 'Fira Code', 'SF Mono', monospace;

/* Body text */
--font-sans: 'Inter', 'Helvetica Neue', sans-serif;
```

### Type Scale

| Token | Size | Usage |
|-------|------|-------|
| `--text-xs` | 12px | Labels, fine print |
| `--text-sm` | 14px | Metadata, captions |
| `--text-base` | 16px | Body text |
| `--text-md` | 18px | Lead paragraphs |
| `--text-lg` | 20px | Card titles |
| `--text-xl` | 24px | Section headers |
| `--text-2xl` | 32px | Page titles |
| `--text-3xl` | 48px | Hero headlines |
| `--text-4xl` | 64px | Display text |

### Heading Styles

```css
/* H1 - Page headlines */
font-family: var(--font-mono);
font-size: var(--text-4xl);
font-weight: 700;
line-height: 1.1;
letter-spacing: -0.03em;

/* H2 - Section headers */
font-family: var(--font-mono);
font-size: var(--text-3xl);
font-weight: 700;
line-height: 1.15;
letter-spacing: -0.02em;

/* H3 - Subsection headers */
font-family: var(--font-mono);
font-size: var(--text-2xl);
font-weight: 600;
line-height: 1.25;
```

### Body Text

```css
/* Standard paragraphs */
font-family: var(--font-sans);
font-size: var(--text-base);
line-height: 1.6;

/* Article prose */
font-family: var(--font-sans);
font-size: var(--text-md);
line-height: 1.75;
max-width: 65ch;
```

### Metadata Style

```css
/* Dates, authors, reading time */
font-family: var(--font-mono);
font-size: var(--text-xs);
font-weight: 500;
letter-spacing: 0.1em;
text-transform: uppercase;
color: var(--text-secondary);
```

**Format examples:**
- Dates: `DEC 15 2025`
- Authors: `ALEX CHEN`
- Reading time: `12 MIN READ`

---

## Spacing & Layout

### Spacing Scale

| Token | Value | Usage |
|-------|-------|-------|
| `--space-1` | 4px | Tight spacing |
| `--space-2` | 8px | Component internal |
| `--space-4` | 16px | Standard gaps |
| `--space-6` | 24px | Section padding |
| `--space-8` | 32px | Large gaps |
| `--space-12` | 48px | Section margins |
| `--space-16` | 64px | Major sections |
| `--space-24` | 96px | Hero padding |

### Container Widths

```css
--container-max: 1200px;  /* Main content */
--content-max: 720px;     /* Article prose */
--wide-max: 960px;        /* Wide content */
```

### Grid System

```css
/* 3-column article grid */
display: grid;
grid-template-columns: repeat(3, 1fr);
gap: var(--space-6);

/* Responsive breakpoints */
@media (max-width: 1024px) { grid-template-columns: repeat(2, 1fr); }
@media (max-width: 768px) { grid-template-columns: 1fr; }
```

### Border Radius

| Token | Value | Usage |
|-------|-------|-------|
| `--radius-sm` | 4px | Buttons, tags |
| `--radius-md` | 8px | Cards, inputs |
| `--radius-lg` | 12px | Modals, images |
| `--radius-full` | 9999px | Pills, avatars |

---

## Components

### Button

**Variants:**
- `primary` - Neon green background, dark text
- `secondary` - Green border, transparent background
- `ghost` - No background, subtle hover

**Sizes:** `sm` (32px), `md` (40px), `lg` (48px)

**States:**
```css
/* Hover - Primary */
background: var(--green-400);
box-shadow: 0 0 20px rgba(0, 229, 0, 0.4);

/* Active */
transform: scale(0.98);

/* Disabled */
opacity: 0.5;
cursor: not-allowed;
```

### Article Card

**Features:**
- Image with hover lift effect (+10px)
- Neon green accent line on hover
- Category tag and metadata
- Author byline with avatar

**Hover animation:**
```css
transition: transform 0.4s ease, box-shadow 0.4s ease;

&:hover {
  transform: translateY(-4px);
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.3);
}
```

### Category Tag

**Variants:**
- `default` - Outlined with subtle border
- `primary` - Filled with green
- `blue` - Blue accent color
- `coral` - Coral accent color

**Interactive state:**
```css
&:hover {
  color: var(--green-500);
  border-color: var(--green-500);
  background: rgba(0, 229, 0, 0.1);
}
```

### Code Block

**Features:**
- Terminal header with colored dots
- Language/filename label
- Copy button with feedback
- Optional line numbers
- Line highlighting

**Syntax highlighting colors:**
```css
--syntax-keyword: #FF5733;   /* Coral */
--syntax-string: #33ff33;    /* Green */
--syntax-number: #3399ff;    /* Blue */
--syntax-function: #0080FF;  /* Blue */
--syntax-comment: #6a737d;   /* Gray */
```

### Newsletter Signup

**Variants:**
- `default` - Stacked layout
- `compact` - Smaller padding
- `inline` - Side-by-side layout

**States:** idle → loading → success/error

---

## Animation & Motion

### Timing Functions

```css
--ease-out: cubic-bezier(0, 0, 0.2, 1);
--ease-in-out: cubic-bezier(0.4, 0, 0.2, 1);
```

### Duration Scale

| Token | Value | Usage |
|-------|-------|-------|
| `--duration-fast` | 150ms | Micro-interactions |
| `--duration-base` | 250ms | Standard transitions |
| `--duration-slow` | 400ms | Complex animations |
| `--duration-slower` | 600ms | Page transitions |

### Signature Animations

**Card Hover:**
```css
transform: translateY(-4px);
box-shadow: 0 20px 40px rgba(0, 0, 0, 0.3);
transition: all 0.4s ease;
```

**Image Lift (Dotfiles Insider style):**
```css
.image {
  transition: transform 0.6s ease;
}

.card:hover .image {
  transform: translateY(-10px);
}

/* Accent glow behind image */
.image::after {
  background: var(--green-500);
  opacity: 0 → 0.25;
}
```

**Link Color Transition:**
```css
transition: color 0.2s ease;
```

### Reduced Motion

```css
@media (prefers-reduced-motion: reduce) {
  * {
    animation-duration: 0.01ms !important;
    transition-duration: 0.01ms !important;
  }
}
```

---

## Accessibility

### Color Contrast

All combinations meet WCAG 2.1 AA standards:

| Combination | Ratio | Status |
|-------------|-------|--------|
| White on black | 21:1 | ✓ AAA |
| Secondary text on black | 9.6:1 | ✓ AAA |
| Green on black | 9.2:1 | ✓ AAA |
| Blue on black | 4.5:1 | ✓ AA |

### Focus States

```css
:focus-visible {
  outline: 2px solid var(--green-500);
  outline-offset: 2px;
}
```

### Screen Reader Support

- Use semantic HTML (`<article>`, `<nav>`, `<main>`)
- Provide `aria-label` for icon buttons
- Hide decorative elements with `aria-hidden="true"`
- Use `aria-live` for dynamic content updates

### Keyboard Navigation

- All interactive elements are focusable
- Focus order follows visual order
- Skip links for main content
- Escape closes modals

---

## Usage Examples

### Article Card Usage

```tsx
<ArticleCard
  title="Building CLI Tools with Rust"
  excerpt="A step-by-step guide..."
  slug="building-cli-rust"
  coverImage="/images/article.jpg"
  category="Rust"
  author={{ name: "Alex Chen", avatar: "/avatars/alex.jpg" }}
  publishedAt="2025-12-15"
  readingTime="12 min read"
/>
```

### Button Usage

```tsx
<Button variant="primary" size="lg">
  Subscribe to Newsletter
</Button>

<Button variant="secondary" leftIcon={<GitHubIcon />}>
  View on GitHub
</Button>
```

### Code Block Usage

```tsx
<CodeBlock
  language="rust"
  filename="src/main.rs"
  showLineNumbers
  highlightLines={[4, 5, 6]}
  code={`fn main() {
    println!("Hello, world!");
}`}
/>
```

### Newsletter Signup Usage

```tsx
<NewsletterSignup
  title="Stay in the loop"
  description="Weekly developer insights."
  variant="inline"
  onSubmit={handleSubscribe}
/>
```

---

## File Structure

```
website/
├── styles/
│   ├── colors.css      # Color system
│   ├── typography.css  # Font system
│   └── base.css        # Reset + utilities
├── components/
│   ├── Button.tsx
│   ├── ArticleCard.tsx
│   ├── CategoryTag.tsx
│   ├── AuthorByline.tsx
│   ├── CodeBlock.tsx
│   └── NewsletterSignup.tsx
├── layouts/
│   └── Layout.tsx      # Header + Footer
├── pages/
│   ├── HomePage.tsx
│   ├── ArticlePage.tsx
│   └── demos/          # Article variants
└── assets/
    └── icons/
        ├── logo.svg
        ├── favicon.svg
        └── social/     # Social icons
```

---

## Quick Reference

### Colors
```
Green:  #00E500
Blue:   #0080FF
Coral:  #FF5733
Black:  #000000
White:  #FFFFFF
```

### Fonts
```
Display: JetBrains Mono
Body:    Inter
```

### Breakpoints
```
Mobile:  < 768px
Tablet:  768px - 1024px
Desktop: > 1024px
```

### Transitions
```
Fast:   150ms
Base:   250ms
Slow:   400ms
```

---

*This style guide is a living document. Update it as the design system evolves.*
