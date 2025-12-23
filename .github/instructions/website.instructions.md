---
applyTo: "website/**/*.astro,website/**/*.ts,website/**/*.tsx,website/**/*.css"
---

# Website Code Review Instructions

The caro website uses Astro framework with TypeScript. Apply these guidelines for frontend code.

## Framework Guidelines

### Astro Component Structure
```astro
---
// Frontmatter: TypeScript/JavaScript imports and logic
import { getCollection } from 'astro:content';
import BaseLayout from '../layouts/BaseLayout.astro';

interface Props {
  title: string;
  description?: string;
}

const { title, description = 'Default description' } = Astro.props;
---

<!-- Template: HTML with Astro expressions -->
<BaseLayout title={title}>
  <main>
    <h1>{title}</h1>
    {description && <p>{description}</p>}
  </main>
</BaseLayout>

<style>
  /* Scoped styles */
  h1 { color: var(--primary); }
</style>

<script>
  // Client-side JavaScript
  document.addEventListener('DOMContentLoaded', () => {
    // DOM manipulation
  });
</script>
```

## Common Issues from Past Reviews

### Timezone Identifiers
Use only valid IANA timezone identifiers:
```typescript
// BAD: Non-existent timezone
if (timezone === 'Asia/Tel_Aviv') // This timezone doesn't exist

// GOOD: Valid IANA timezone
if (timezone === 'Asia/Jerusalem')
```

### CSS Scoping in Astro
Scoped styles don't work with external class selectors:
```astro
<!-- BAD: .dark class is on <html>, outside component scope -->
<style>
  .dark h1 { color: white; }  /* Won't work */
</style>

<!-- GOOD: Use :global() for external classes -->
<style>
  :global(.dark) h1 { color: white; }
</style>

<!-- BETTER: Use CSS custom properties -->
<style>
  h1 { color: var(--text-color); }
</style>
```

### DOM Timing Issues
```astro
<!-- BAD: DOM doesn't exist when inline script runs -->
<script>
  positionElement(); // Element not yet in DOM
</script>

<!-- GOOD: Wait for DOM to be ready -->
<script>
  document.addEventListener('DOMContentLoaded', () => {
    positionElement();
  });
</script>

<!-- ALSO GOOD: Use defer or place script at end -->
<script defer>
  positionElement();
</script>
```

### Magic Numbers in CSS
```css
/* BAD: Magic number creates fragile coupling */
.content {
  padding-top: 51px; /* Assumes banner height */
}

/* GOOD: Use CSS custom properties */
:root {
  --banner-height: 51px;
}
.content {
  padding-top: var(--banner-height);
}
```

## Cross-Browser Compatibility

### Gradient Text Support
```css
/* BAD: Webkit-only gradient text */
.gradient-text {
  background: linear-gradient(...);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}

/* GOOD: Include fallback for Firefox */
.gradient-text {
  background: linear-gradient(...);
  -webkit-background-clip: text;
  background-clip: text;
  -webkit-text-fill-color: transparent;
  color: transparent; /* Firefox fallback */
}
```

## Date and Time Handling

### Expiry Date Validation
```typescript
// BAD: Hardcoded date that may already be past
const bannerExpiry = new Date('2025-01-10');

// GOOD: Validate at build time or use relative dates
const bannerExpiry = new Date();
bannerExpiry.setDate(bannerExpiry.getDate() + 30); // 30 days from now

// ALSO GOOD: Check if date is in the future
const isExpired = new Date(expiryDate) < new Date();
if (isExpired) {
  console.warn(`Banner expiry date ${expiryDate} is in the past`);
}
```

## Accessibility Requirements

### Required Practices
- All images must have `alt` attributes
- Interactive elements need keyboard support
- Use semantic HTML (`<nav>`, `<main>`, `<article>`)
- Ensure sufficient color contrast
- Provide visible focus indicators

```astro
<!-- BAD: Missing accessibility -->
<div onclick="handleClick()">Click me</div>

<!-- GOOD: Accessible interactive element -->
<button
  onclick="handleClick()"
  aria-label="Perform action"
  class="focus:ring-2"
>
  Click me
</button>
```

## Performance Guidelines

### Image Optimization
```astro
<!-- Use Astro's built-in Image component -->
---
import { Image } from 'astro:assets';
import heroImage from '../assets/hero.png';
---
<Image src={heroImage} alt="Hero" loading="lazy" />
```

### Script Loading
- Use `defer` for non-critical scripts
- Inline small critical JavaScript
- Avoid blocking the main thread

## TypeScript Standards

### Required Patterns
```typescript
// Define interfaces for props and data
interface BlogPost {
  title: string;
  date: Date;
  author?: string;
}

// Use strict type checking
function formatDate(date: Date): string {
  return date.toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'long',
    day: 'numeric',
  });
}
```

## Testing Checklist

Before approving website changes:
- [ ] Works in latest Chrome, Firefox, Safari
- [ ] Responsive design functions on mobile
- [ ] No console errors or warnings
- [ ] Accessibility audit passes (Lighthouse)
- [ ] Links are valid (no 404s)
- [ ] Build succeeds: `npm run build`
