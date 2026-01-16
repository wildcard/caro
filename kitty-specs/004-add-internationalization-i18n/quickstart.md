# Quickstart: i18n Development Guide

**Feature**: Website Internationalization with 15 Languages
**Audience**: Developers implementing or maintaining the i18n system
**Last Updated**: 2025-12-28

## Prerequisites

- Astro 4.0+ installed (`package.json` confirms 4.16.17)
- Node.js 20+
- Basic understanding of Astro components
- Familiarity with JSON structure

## Local Development Setup

### 1. Verify i18n Infrastructure

Check that the translation infrastructure exists:

```bash
cd website/src/i18n

# Expected structure:
# i18n/
# ├── config.ts           # Translation utilities
# ├── index.ts            # Export all translations
# └── locales/
#     ├── en/             # English (source)
#     │   ├── common.json
#     │   ├── navigation.json
#     │   ├── hero.json
#     │   └── ...
#     ├── es/             # Spanish
#     ├── fr/             # French
#     └── ...
```

### 2. Using Translation Functions

Import and use the `t()` function in Astro components:

```astro
---
// Component frontmatter
import { t, type Locale } from '../i18n/config';

interface Props {
  lang?: Locale;
}

const { lang = 'en' } = Astro.props;
---

<!-- Use translations in template -->
<nav>
  <a href="/#features">{t(lang, 'navigation.links.features')}</a>
  <a href="/#download">{t(lang, 'navigation.links.getStarted')}</a>
</nav>
```

### 3. Adding New Strings

To add a new translatable string:

**Step 1**: Add to English source file

```json
// website/src/i18n/locales/en/navigation.json
{
  "brand": "Caro",
  "links": {
    "features": "Features",
    "newLink": "New Page"  // ← Add here
  }
}
```

**Step 2**: Use in component

```astro
<a href="/new">{t(lang, 'navigation.links.newLink')}</a>
```

**Step 3**: Push to main branch

```bash
git add website/src/i18n/locales/en/navigation.json
git commit -m "feat(i18n): Add new navigation link"
git push origin main
```

**Step 4**: Wait for automation

- GitHub Action detects change
- Translates to all 14 non-English locales
- Creates PR within ~10 minutes
- Review 2-3 sample translations
- Merge PR

### 4. Creating Localized Pages

Use Astro's `getStaticPaths()` for locale-based routes:

```astro
---
// website/src/pages/[lang]/index.astro
import Layout from '../../layouts/Layout.astro';
import { languages, type Locale } from '../../i18n/config';
import Hero from '../../components/Hero.astro';

export function getStaticPaths() {
  return Object.keys(languages)
    .filter(lang => lang !== 'en')  // Skip English (default route)
    .map(lang => ({ params: { lang } }));
}

const { lang } = Astro.params as { lang: Locale };
---

<Layout title="Caro" lang={lang}>
  <Hero lang={lang} />
  <!-- Other components with lang prop -->
</Layout>
```

This generates:
- `/es/` (Spanish)
- `/fr/` (French)
- `/he/` (Hebrew)
- etc.

### 5. Testing RTL Languages Locally

Test RTL rendering for Hebrew, Arabic, or Urdu:

```bash
npm run dev
# Visit http://localhost:4321/he/  (Hebrew)
# Visit http://localhost:4321/ar/  (Arabic)
# Visit http://localhost:4321/ur/  (Urdu)
```

**What to check**:
- Text flows right-to-left
- Navigation items reversed
- `<html dir="rtl">` attribute present
- Font renders correctly (Noto fonts loaded)
- No layout breaks

## Translation File Organization

### File Structure

```
locales/
├── en/                    # English (source of truth)
│   ├── common.json       # Shared UI (buttons, labels, status)
│   ├── navigation.json   # Nav bar and footer
│   ├── hero.json         # Hero section
│   ├── features.json     # Features section
│   ├── download.json     # Download section
│   ├── faq.json          # FAQ content
│   ├── landing.json      # Landing page copy
│   └── compare.json      # Comparison pages
├── es/                    # Spanish (auto-generated)
│   └── [same files]
├── fr/                    # French (auto-generated)
│   └── [same files]
└── ...
```

### Key Naming Conventions

Use hierarchical dot-notation keys:

```json
{
  "section": {
    "subsection": {
      "key": "Value"
    }
  }
}
```

**Examples**:
- `navigation.links.features` → "Features"
- `hero.cta.getStarted` → "Get Started"
- `common.buttons.copy` → "Copy"
- `features.safety.title` → "Safety First"

### Placeholder Support

Use `{variable}` for dynamic content:

```json
{
  "downloads": "{count} downloads",
  "greeting": "Hello, {name}!"
}
```

Translation automation preserves these placeholders.

## Component Refactoring Checklist

When refactoring a component for i18n:

- [ ] Add `lang?: Locale` to Props interface
- [ ] Import `t` function from `../i18n/config`
- [ ] Extract all hardcoded strings to English JSON file
- [ ] Replace strings with `t(lang, 'key.path')` calls
- [ ] Pass `lang` prop to all child components
- [ ] Test with English (`/`)
- [ ] Test with Spanish (`/es/`)
- [ ] Test with Hebrew (`/he/`) for RTL

**Example** (Navigation.astro refactoring):

```diff
---
+ import { t, type Locale } from '../i18n/config';
+
+ interface Props {
+   lang?: Locale;
+ }
+
+ const { lang = 'en' } = Astro.props;
---

<nav>
-  <a href="/">🐕 Caro</a>
+  <a href="/">🐕 {t(lang, 'navigation.brand')}</a>
  <ul>
-    <li><a href="/#features">Features</a></li>
+    <li><a href="/#features">{t(lang, 'navigation.links.features')}</a></li>
-    <li><a href="/compare">Compare</a></li>
+    <li><a href="/compare">{t(lang, 'navigation.links.compare')}</a></li>
  </ul>
</nav>
```

## RTL Styling Guidelines

### Use CSS Logical Properties

Replace directional properties with logical equivalents:

```css
/* ❌ Physical properties (don't use) */
.container {
  margin-left: 20px;
  padding-right: 10px;
  border-left: 1px solid;
  text-align: left;
}

/* ✅ Logical properties (use these) */
.container {
  margin-inline-start: 20px;
  padding-inline-end: 10px;
  border-inline-start: 1px solid;
  text-align: start;
}
```

### RTL-Specific Overrides

For elements that need explicit RTL handling:

```css
/* Automatic with logical properties */
.button {
  margin-inline-end: 10px;  /* Becomes margin-left in RTL */
}

/* Manual override when needed */
[dir="rtl"] .icon-arrow {
  transform: scaleX(-1);  /* Flip arrow direction */
}

[dir="rtl"] .nav-links {
  flex-direction: row-reverse;  /* Reverse menu order */
}
```

## GitHub Action Workflow

### Manual Trigger

Force retranslation of all content:

```bash
# Via GitHub UI
# Actions → Translate Website Content → Run workflow
# Set "force_retranslate" to true
```

### Spot-Check Review Process

When reviewing translation PRs:

1. **Select 2-3 strings per language** (random sampling)
2. **Check for**:
   - Contextual accuracy
   - Placeholder preservation (`{count}` unchanged)
   - Cultural appropriateness
   - Technical term accuracy (POSIX, shell, CLI)
3. **For RTL languages**: Preview `/he/`, `/ar/`, `/ur/` on Vercel preview deploy
4. **Approve if** 95%+ sample passes
5. **Request changes if** significant issues found

### Troubleshooting

**Problem**: GitHub Action fails with "OpenAI API key not found"
**Solution**: Verify `OPENAI_API_KEY` secret is set in repository settings

**Problem**: Translation quality is poor for technical terms
**Solution**: Update system prompt in `.github/scripts/translate.js` to emphasize technical accuracy

**Problem**: RTL layout broken after merge
**Solution**: Check for physical CSS properties (margin-left, padding-right) - convert to logical properties

**Problem**: New locale not generating routes
**Solution**: Verify locale added to `astro.config.mjs` locales array

## Performance Optimization

### Lazy Loading

Translation files are lazy-loaded per route:

```javascript
// Astro automatically code-splits by route
// /es/ loads only Spanish translations
// /fr/ loads only French translations
// No need for manual optimization
```

### Build Time

Expect ~50ms overhead per locale during build:

```bash
npm run build
# Building 15 locales × 5 pages = 75 total pages
# Expected build time: ~10-15 seconds
```

## Common Patterns

### Language Switcher Component

```astro
---
import { getAllLocales, type Locale } from '../i18n/config';

const allLocales = getAllLocales();
const currentLang = Astro.params.lang || 'en';
---

<select id="lang-switcher">
  {allLocales.map(locale => (
    <option value={locale.code} selected={locale.code === currentLang}>
      {locale.nativeName}
    </option>
  ))}
</select>

<script>
  document.getElementById('lang-switcher')?.addEventListener('change', (e) => {
    const newLang = (e.target as HTMLSelectElement).value;
    const path = window.location.pathname.replace(/^\/[a-z]{2}/, '');
    window.location.href = newLang === 'en' ? path : `/${newLang}${path}`;
  });
</script>
```

### Fallback Handling

```typescript
// config.ts automatically falls back to English
export function t(lang: Locale, key: string): string {
  const keys = key.split('.');
  let value: any = translations[lang];

  for (const k of keys) {
    value = value?.[k];
  }

  // Fallback to English if not found
  if (value === undefined) {
    value = translations.en;
    for (const k of keys) {
      value = value?.[k];
    }
  }

  return value ?? key;  // Return key if still not found
}
```

## Next Steps

1. Read the full plan at `kitty-specs/004-add-internationalization-i18n/plan.md`
2. Review data model at `data-model.md`
3. Check implementation tasks at `tasks.md` (generated by `/spec-kitty.tasks`)
4. Start with Phase 1: Infrastructure (Astro config, Layout.astro RTL)
